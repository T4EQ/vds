use crate::{
    db::Database,
    manifest::{ManifestFile, Video},
};

use super::DownloadContext;

use std::collections::VecDeque;

use sha2::Digest;
use tokio::{io::AsyncWriteExt, task::JoinSet};
use tokio_stream::StreamExt;

/// Makes sure that all manifest videos are present in the database with their corresponding state.
/// Creates entries for missing videos.
pub async fn initialize_video_entries(
    database: &Database,
    new_manifest: &ManifestFile,
) -> anyhow::Result<()> {
    for video in new_manifest.sections.iter().flat_map(|s| s.content.iter()) {
        match database.find_video(video.id).await {
            Ok(_) => {}
            Err(crate::db::Error::Diesel(diesel::result::Error::NotFound)) => {
                database
                    .insert_video(video.id, &video.name, video.file_size)
                    .await?
            }
            Err(e) => return Err(e.into()),
        }
    }
    Ok(())
}

/// Iterates through the on-disk video entries, deleting video content that is not present in the current
/// manifest. This is a cleanup action that is deferred until the new manifest has been fully
/// adopted.
pub async fn remove_old_video_content(
    database: &Database,
    new_manifest: &ManifestFile,
) -> anyhow::Result<()> {
    let in_manifest = |id| {
        new_manifest
            .sections
            .iter()
            .flat_map(|s| s.content.iter())
            .any(|v| v.id == id)
    };

    for video in database.list_all_videos().await? {
        if !in_manifest(video.id) {
            database.delete_video(video.id).await?;
        }
    }

    Ok(())
}

/// An async task in charge of downloading the content listed in a manifest.
///
/// This task needs to be cancel-safe, because it might get cancelled by calling code if a newer
/// manifest is found.
/// For references on cancellation-safety: https://sunshowers.io/posts/cancelling-async-rust/
pub async fn download_manifest_task(
    ctx: DownloadContext,
    new_manifest: ManifestFile,
) -> anyhow::Result<()> {
    println!("Creating video entries in the db from manifest");
    initialize_video_entries(&ctx.db, &new_manifest).await?;

    // After the video entries for the current manifest have been populated, we are ready to
    // publish the manifest and make it visible to the HTTP clients.
    println!("Publishing the manifest for availability");
    ctx.db.publish_manifest(&new_manifest).await;

    // Mark older content for deletion
    println!("Deleting old video content");
    remove_old_video_content(&ctx.db, &new_manifest).await?;

    // Collect the content that we need to download
    let mut pending_downloads: VecDeque<Video> = VecDeque::new();
    for video in new_manifest.sections.iter().flat_map(|s| s.content.iter()) {
        let already_downloaded = ctx
            .db
            .find_video(video.id)
            .await
            .unwrap()
            .download_status
            .is_downloaded();
        if pending_downloads.iter().all(|v| video.id != v.id) && !already_downloaded {
            pending_downloads.push_back(video.clone());
        }
    }

    // Because we do not want to ovewhelm the network, we limit the number of concurrent downloads
    // we perform. This limit is configurable via the configuration file.
    let mut inprogress_videos = JoinSet::new();
    let mut backoff_list = VecDeque::new();

    loop {
        if inprogress_videos.is_empty() && backoff_list.is_empty() && pending_downloads.is_empty() {
            break;
        }

        // Try to start more downloads while we have some
        while inprogress_videos.len() < ctx.config.concurrent_downloads {
            let Some(current_video) = pending_downloads.pop_front() else {
                break;
            };

            println!("Starting download for {:?}", current_video.id);
            let job = download_job_task(ctx.clone(), current_video.clone());
            inprogress_videos.spawn(job);
        }

        // We have 2 situations to wait for here.
        //  1. A download finished, which opens up a new slot to start another download
        //  2. A failed video which was held has now completed the backoff duration and can be
        //     scheduled again.
        let first_backoff_video = async {
            let Some(wakeup_time) = backoff_list
                .iter()
                .next()
                .map(|(wakeup_time, _)| *wakeup_time)
            else {
                // Make sure this does not exit unless we actually need to do work
                std::future::pending().await
            };

            tokio::time::sleep_until(wakeup_time).await;

            // Actually finished waiting, so let's pop it off now (to be cancel-safe).
            let (_, video): (tokio::time::Instant, Video) = backoff_list.pop_front().expect("");
            video
        };

        tokio::select! {
            video = first_backoff_video => {
                println!("Video {} will reattempt download", video.id);
                pending_downloads.push_back(video);
            }

            Some(finished_video) = inprogress_videos.join_next() => {
                match finished_video? {
                    Ok(()) => { }
                    Err(DownloadJobError::ShouldRetry(video)) => {
                        println!("Video {} failed. Backing off for {:?}", video.id, ctx.config.initial_backoff);

                        // TODO: Adjust backoff exponentially
                        let wakeup_time = tokio::time::Instant::now() + ctx.config.initial_backoff;
                        backoff_list.push_back((wakeup_time, video));
                    }
                    Err(DownloadJobError::Unrecoverable(video)) => {
                        println!("Unrecoverable download error for video: {}", video.id);
                        anyhow::bail!("Unrecoverable error for video: {}", video.id);
                    }
                }
            }
        }
    }

    Ok(())
}

#[derive(Debug)]
enum DownloadJobError {
    ShouldRetry(crate::manifest::Video),
    Unrecoverable(crate::manifest::Video),
}

/// download job task
async fn download_job_task(
    ctx: DownloadContext,
    video: crate::manifest::Video,
) -> Result<(), DownloadJobError> {
    let mut stream = ctx.backend.fetch_resource(&video.uri);

    let target_filepath = ctx.config.content_path.join(format!("{}.mp4", video.id));
    let mut target_file = tokio::fs::File::create(&target_filepath)
        .await
        .map_err(|e| {
            println!("Error creating file: {:?}. Error: {}", target_filepath, e);
            DownloadJobError::ShouldRetry(video.clone())
        })?;

    let translate_error = |e: crate::db::Result<()>| {
        e.map_err(|e| {
            println!(
                "Error setting download status for file: {:?}. Error: {}",
                target_filepath, e
            );
            DownloadJobError::Unrecoverable(video.clone())
        })
    };

    let mut hasher = sha2::Sha256::new();

    let mut total_size = 0;
    while let Some(chunk) = stream.next().await {
        let chunk = match chunk {
            Ok(v) => v,
            Err(err) => {
                let error_msg = format!(
                    "Error fetching file with id: {}, name: {}. Error: {}",
                    video.id, video.name, err
                );
                println!("{error_msg}");

                translate_error(ctx.db.set_download_failed(video.id, &error_msg).await)?;

                return Err(DownloadJobError::ShouldRetry(video.clone()));
            }
        };

        hasher.update(&chunk[..]);
        target_file.write_all(&chunk[..]).await.map_err(|e| {
            println!("Error writing file: {:?}. Error: {}", target_filepath, e);
            DownloadJobError::ShouldRetry(video.clone())
        })?;
        total_size += chunk.len();

        translate_error(
            ctx.db
                .update_download_progress(video.id, total_size as u64)
                .await,
        )?;
    }

    let hash = hasher.finalize();
    let hash = hash.as_slice();
    let expected_hash = video.sha256.as_bytes();
    if hash != &expected_hash[..] {
        let hash: crate::manifest::Sha256 = hash.try_into().expect("Should have 32 bytes");
        let err_msg = &format!("Got hash: {hash}. Expected: {}", video.sha256);
        translate_error(ctx.db.set_download_failed(video.id, err_msg).await)?;
        println!("{}", err_msg);
        return Err(DownloadJobError::ShouldRetry(video.clone()));
    }

    translate_error(ctx.db.set_downloaded(video.id, &target_filepath).await)?;

    Ok(())
}

#[cfg(test)]
pub mod test {
    use std::{str::FromStr, sync::Arc, time::Duration};

    use crate::{
        cfg::{DbConfig, DownloaderConfig},
        downloader::backend::{self, Backend},
        manifest::{ManifestFile, Section, Version, Video},
    };

    use googletest::prelude::*;
    use http::Uri;

    use super::*;

    fn manifest_for_test() -> googletest::Result<ManifestFile> {
        Ok(ManifestFile {
            name: "manifest".to_string(),
            date: chrono::NaiveDate::from_str("2025-10-10").or_fail()?,
            version: Version {
                major: 2,
                minor: 0,
                revision: 0,
            },
            sections: vec![
                Section {
                    name: "".to_string(),
                    content: vec![
                        Video {
                            name: "Linear equations".to_string(),
                            id: uuid::Uuid::from_str("bf978778-1c5d-44b3-b2c1-1cc253563799")
                                .or_fail()?,
                            uri: "s3://bucket/linear-equations.mp4".parse().or_fail()?,
                            sha256:
                                "0b88b2dec2be5e2ef74022ef6a8023232e28374d67e917b76f9bb607e691f327"
                                    .try_into()
                                    .or_fail()?,
                            file_size: 123456,
                        },
                        Video {
                            name: "Quadratic equations".to_string(),
                            id: uuid::Uuid::from_str("5eb9e089-79cf-478d-9121-9ca3e7bb1d4a")
                                .or_fail()?,
                            uri: "s3://bucket/quadratic-equations.mp4".parse().or_fail()?,
                            sha256:
                                "8f9e3a4ae7d86c4abdf731a947fc90b607b82a0362da0b312e3b644defedb81f"
                                    .try_into()
                                    .or_fail()?,
                            file_size: 123457,
                        },
                    ],
                },
                Section {
                    name: "Integration".to_string(),
                    content: vec![
                        Video {
                            name: "Riemann sum".to_string(),
                            id: uuid::Uuid::from_str("eddb4450-a9ff-4a4b-ad81-2a8b78998405")
                                .or_fail()?,
                            uri: "s3://bucket/riemann-sum.mp4".parse().or_fail()?,
                            sha256:
                                "a6d3b80cd14f78b21ffbf5995bbda38ad8834459557782d245ed720134d36fc4"
                                    .try_into()
                                    .or_fail()?,
                            file_size: 123459,
                        },
                        Video {
                            name: "List of integrals".to_string(),
                            id: uuid::Uuid::from_str("f47e6cdc-1bcf-439a-9ea4-038dc7153648")
                                .or_fail()?,
                            uri: "s3://bucket/list-of-integrals.mp4".parse().or_fail()?,
                            sha256:
                                "98780990e94fb55d0b88ebcd78fe82f069eac547731a4b0822332d826c970aec"
                                    .try_into()
                                    .or_fail()?,
                            file_size: 123460,
                        },
                    ],
                },
            ],
        })
    }

    fn manifest_for_test2() -> googletest::Result<ManifestFile> {
        Ok(ManifestFile {
            name: "manifest 2".to_string(),
            date: chrono::NaiveDate::from_str("2025-10-11").or_fail()?,
            version: Version {
                major: 2,
                minor: 0,
                revision: 0,
            },
            sections: vec![Section {
                name: "Section with a name".to_string(),
                content: vec![
                    Video {
                        name: "Quadratic equations".to_string(),
                        id: uuid::Uuid::from_str("5eb9e089-79cf-478d-9121-9ca3e7bb1d4a")
                            .or_fail()?,
                        uri: "s3://bucket/quadratic-equations.mp4".parse().or_fail()?,
                        sha256: "8f9e3a4ae7d86c4abdf731a947fc90b607b82a0362da0b312e3b644defedb81f"
                            .try_into()
                            .or_fail()?,
                        file_size: 123457,
                    },
                    Video {
                        name: "Riemann sum".to_string(),
                        id: uuid::Uuid::from_str("eddb4450-a9ff-4a4b-ad81-2a8b78998405")
                            .or_fail()?,
                        uri: "s3://bucket/riemann-sum.mp4".parse().or_fail()?,
                        sha256: "a6d3b80cd14f78b21ffbf5995bbda38ad8834459557782d245ed720134d36fc4"
                            .try_into()
                            .or_fail()?,
                        file_size: 123459,
                    },
                ],
            }],
        })
    }

    struct TestContext {
        dummy_backend: Arc<DummyBackend>,
        download_ctx: DownloadContext,

        // We need to keep these to make sure the dirs are not removed from the fs
        _content_path: tempfile::TempDir,
        _runtime_path: tempfile::TempDir,
    }

    async fn create_context() -> TestContext {
        let content_path = tempfile::TempDir::new().unwrap();
        let downloader_config = Arc::new(DownloaderConfig {
            concurrent_downloads: 2,
            content_path: content_path.path().to_path_buf(),
            initial_backoff: Duration::from_millis(100),
            remote_server: "/Invalid".try_into().unwrap(),
            update_interval: Duration::from_secs(300),
        });

        let runtime_path = tempfile::TempDir::new().unwrap();
        let db_config = DbConfig {
            busy_timeout: Duration::from_secs(2),
            runtime_path: runtime_path.path().to_path_buf(),
        };

        let db = Arc::new(Database::open(db_config).await.unwrap());
        db.apply_pending_migrations().await.or_fail().unwrap();

        let dummy_backend = Arc::new(DummyBackend::default());

        let download_ctx = DownloadContext {
            config: downloader_config,
            backend: dummy_backend.clone(),
            db,
        };

        TestContext {
            dummy_backend,
            download_ctx,
            _content_path: content_path,
            _runtime_path: runtime_path,
        }
    }

    struct BackendFile {
        uri: Uri,
        content: Vec<u8>,
    }

    struct DummyBackend {
        files: tokio::sync::Mutex<Vec<BackendFile>>,
    }

    impl Default for DummyBackend {
        fn default() -> Self {
            Self {
                files: tokio::sync::Mutex::new(vec![]),
            }
        }
    }

    impl DummyBackend {
        async fn add_file(&self, file: BackendFile) {
            let mut files = self.files.lock().await;
            files.push(file);
        }
    }

    #[async_trait::async_trait]
    impl Backend for DummyBackend {
        fn fetch_resource<'a, 'b>(
            &'a self,
            uri: &'b http::Uri,
        ) -> std::pin::Pin<Box<dyn tokio_stream::Stream<Item = backend::ChunkResult> + Send + 'a>>
        where
            'b: 'a,
        {
            Box::pin(async_stream::stream! {
                let files = self.files.lock().await;
                let Some(file) = files.iter().find(|f| f.uri == *uri) else {
                    yield Err(crate::downloader::Error::IoError(std::io::Error::new(std::io::ErrorKind::NotFound, "")));
                    return;
                };

                yield Ok(file.content.clone());
            })
        }

        async fn fetch_manifest(&self) -> std::result::Result<Vec<u8>, crate::downloader::Error> {
            todo!()
        }
    }

    #[tokio::test]
    #[googletest::test]
    async fn test_initialize_video_entries() -> googletest::Result<()> {
        let ctx = create_context().await;
        let db = &ctx.download_ctx.db;

        let manifest = manifest_for_test()?;

        initialize_video_entries(db, &manifest).await.or_fail()?;

        for video in manifest.sections.iter().flat_map(|s| s.content.iter()) {
            let db_video = db.find_video(video.id).await.or_fail()?;
            expect_that!(
                db_video,
                eq(&crate::db::Video {
                    id: video.id,
                    name: video.name.clone(),
                    file_size: video.file_size,
                    download_status: crate::db::DownloadStatus::Pending,
                    view_count: 0,
                })
            );
        }

        Ok(())
    }

    #[tokio::test]
    #[googletest::test]
    async fn test_remove_old_video_content() -> googletest::Result<()> {
        let ctx = create_context().await;
        let db = &ctx.download_ctx.db;

        let manifest = manifest_for_test()?;
        let new_manifest = manifest_for_test2()?;

        initialize_video_entries(db, &manifest).await.or_fail()?;

        remove_old_video_content(db, &new_manifest)
            .await
            .or_fail()?;

        for video in manifest.sections.iter().flat_map(|s| s.content.iter()) {
            let db_video = db.find_video(video.id).await;

            let in_new_manifest = new_manifest
                .sections
                .iter()
                .flat_map(|s| s.content.iter())
                .any(|v| v.id == video.id);

            if in_new_manifest {
                expect_that!(
                    db_video,
                    ok(eq(&crate::db::Video {
                        id: video.id,
                        name: video.name.clone(),
                        file_size: video.file_size,
                        download_status: crate::db::DownloadStatus::Pending,
                        view_count: 0,
                    }))
                );
            } else {
                expect_that!(
                    db_video,
                    err(matches_pattern!(crate::db::Error::Diesel(
                        matches_pattern!(diesel::result::Error::NotFound)
                    )))
                );
            }
        }

        Ok(())
    }

    #[tokio::test]
    #[googletest::test]
    async fn test_download_job_task_recoverable_io_failure() -> googletest::Result<()> {
        let ctx = create_context().await;
        let id = uuid::Uuid::from_str("5eb9e089-79cf-478d-9121-9ca3e7bb1d4a").or_fail()?;

        initialize_video_entries(&ctx.download_ctx.db, &manifest_for_test().or_fail()?)
            .await
            .or_fail()?;

        let result = download_job_task(
            ctx.download_ctx.clone(),
            Video {
                name: "Quadratic equations".to_string(),
                id,
                uri: "s3://bucket/quadratic-equations.mp4".parse().or_fail()?,
                sha256: "8f9e3a4ae7d86c4abdf731a947fc90b607b82a0362da0b312e3b644defedb81f"
                    .try_into()
                    .or_fail()?,
                file_size: 123457,
            },
        )
        .await;

        assert_that!(
            result,
            err(matches_pattern!(DownloadJobError::ShouldRetry(
                matches_pattern!(Video { id: &id, .. })
            )))
        );

        // Check that file is available in the database
        let db_video = ctx.download_ctx.db.find_video(id).await.or_fail()?;
        expect_that!(
            db_video,
            matches_pattern!(crate::db::Video {
                id: &id,
                download_status: matches_pattern!(crate::db::DownloadStatus::Failed(eq(
                    "Error fetching file with id: 5eb9e089-79cf-478d-9121-9ca3e7bb1d4a, name: Quadratic equations. Error: I/O error reading from backend: "
                ))),
                ..
            })
        );

        Ok(())
    }

    #[tokio::test]
    #[googletest::test]
    async fn test_download_job_task_successful() -> googletest::Result<()> {
        let ctx = create_context().await;
        let name = "Quadratic equations".to_string();
        let id = uuid::Uuid::from_str("5eb9e089-79cf-478d-9121-9ca3e7bb1d4a").or_fail()?;
        let uri: Uri = "s3://bucket/quadratic-equations.mp4".parse().or_fail()?;

        ctx.dummy_backend
            .add_file(BackendFile {
                uri: uri.clone(),
                content: vec![1, 2, 3, 4],
            })
            .await;

        initialize_video_entries(&ctx.download_ctx.db, &manifest_for_test().or_fail()?)
            .await
            .or_fail()?;

        let result = download_job_task(
            ctx.download_ctx.clone(),
            Video {
                name: name.clone(),
                id,
                uri,
                sha256: "9f64a747e1b97f131fabb6b447296c9b6f0201e79fb3c5356e6c77e89b6a806a"
                    .try_into()
                    .or_fail()?,
                file_size: 4,
            },
        )
        .await;

        assert_that!(result, ok(anything()));

        // Check that file is available in the database
        let video_fs_path = ctx
            .download_ctx
            .config
            .content_path
            .join(format!("{id}.mp4"));
        let db_video = ctx.download_ctx.db.find_video(id).await.or_fail()?;
        expect_that!(
            db_video,
            matches_pattern!(crate::db::Video {
                id: &id,
                download_status: &crate::db::DownloadStatus::Downloaded(video_fs_path.clone()),
                ..
            })
        );

        // Check that file is available in the filesystem
        let data = tokio::fs::read(video_fs_path).await.or_fail()?;
        assert_that!(data, eq(&vec![1, 2, 3, 4]));

        Ok(())
    }

    #[tokio::test]
    #[googletest::test]
    async fn test_download_job_task_invalid_checksum() -> googletest::Result<()> {
        let ctx = create_context().await;
        let name = "Quadratic equations".to_string();
        let id = uuid::Uuid::from_str("5eb9e089-79cf-478d-9121-9ca3e7bb1d4a").or_fail()?;
        let uri: Uri = "s3://bucket/quadratic-equations.mp4".parse().or_fail()?;

        ctx.dummy_backend
            .add_file(BackendFile {
                uri: uri.clone(),
                content: vec![1, 2, 3, 5],
            })
            .await;

        initialize_video_entries(&ctx.download_ctx.db, &manifest_for_test().or_fail()?)
            .await
            .or_fail()?;

        let result = download_job_task(
            ctx.download_ctx.clone(),
            Video {
                name: name.clone(),
                id,
                uri,
                sha256: "9f64a747e1b97f131fabb6b447296c9b6f0201e79fb3c5356e6c77e89b6a806a"
                    .try_into()
                    .or_fail()?,
                file_size: 4,
            },
        )
        .await;

        assert_that!(
            result,
            err(matches_pattern!(DownloadJobError::ShouldRetry(
                matches_pattern!(Video { id: &id, .. })
            )))
        );

        // Check that file is available in the database
        let db_video = ctx.download_ctx.db.find_video(id).await.or_fail()?;
        expect_that!(
            db_video,
            matches_pattern!(crate::db::Video {
                id: &id,
                download_status: matches_pattern!(crate::db::DownloadStatus::Failed(eq(
                    "Got hash: 1571902abec0a45661de965dbe90cb0177b98c49fc58a5aabfa1edb6c678d972. Expected: 9f64a747e1b97f131fabb6b447296c9b6f0201e79fb3c5356e6c77e89b6a806a"
                ))),
                ..
            })
        );

        Ok(())
    }
}
