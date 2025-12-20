use std::{collections::VecDeque, path::PathBuf, sync::Arc, time::Duration};

use crate::manifest::{ManifestFile, Video};
use backend::FileBackend;

use anyhow::Context;
use tokio::{io::AsyncWriteExt, task::JoinSet};
use tokio_stream::StreamExt;

mod backend;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("I/O error reading from backend: {0}")]
    IoError(std::io::Error),
    #[error("Manifest is malformed: {0}")]
    MalformedManifest(serde_json::Error),
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::MalformedManifest(value)
    }
}

struct Downloader {
    current_manifest: Option<ManifestFile>,
    video_store: PathBuf,
}

impl Downloader {
    pub async fn update(
        &mut self,
        backend: Arc<dyn backend::Backend>,
        mut cancel_signal: tokio::sync::mpsc::Receiver<()>,
    ) -> anyhow::Result<()> {
        // Inspect new manifest file
        let manifest = backend
            .fetch_manifest()
            .await
            .context("Error getting current manifest")?;

        if !self
            .current_manifest
            .as_ref()
            .is_none_or(|v| *v != manifest && v.date.cmp(&manifest.date).is_gt())
        {
            return Ok(());
        }

        // TODO: Delete old videos

        let mut pending_downloads = manifest
            .sections
            .iter()
            .flat_map(|s| s.content.iter())
            .fold(VecDeque::new(), |mut pending: VecDeque<Video>, video| {
                // TODO: Check which videos are already available and remove them from the pending
                // downloads list.
                if pending.iter().any(|v| video.id == v.id) {
                    pending
                } else {
                    pending.push_back(video.clone());
                    pending
                }
            });

        let mut inprogress_videos = JoinSet::new();
        const CONCURRENT_DOWNLOADS: usize = 8;
        'outer: loop {
            while inprogress_videos.len() < CONCURRENT_DOWNLOADS {
                let Some(current_video) = pending_downloads.pop_front() else {
                    break 'outer;
                };

                let job = Self::download_job(
                    self.video_store.clone(),
                    backend.clone(),
                    current_video.clone(),
                );
                inprogress_videos.spawn(job);
            }

            tokio::select! {
                finished_video = inprogress_videos.join_next() => {
                    // TODO: fetch errors should retry
                    finished_video.expect("At least one video is present")??;
                    println!("Fetching completed" );
                }
                _ = cancel_signal.recv() => {
                    // TODO: Handle cancellation
                }
            }
        }

        // TODO: Poll in-progress downloads
        // while inprogress_videos {}

        // All fetched!
        Ok(())
    }

    async fn download_job(
        video_store: PathBuf,
        backend: Arc<dyn backend::Backend>,
        video: crate::manifest::Video,
    ) -> anyhow::Result<()> {
        let mut stream = backend.fetch_resource(&video.uri);

        let mut target_filename = video_store;
        target_filename.push(format!("{}.mp4", video.id));
        let mut target_file = tokio::fs::File::create(&target_filename).await?;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.inspect_err(|err| {
                println!(
                    "Error fetching file with id: {}, name: {}. Error: {}",
                    video.id, video.name, err
                );
            })?;

            target_file.write_all(&chunk[..]).await?;
        }

        Ok(())
    }
}

pub async fn downloader_task(base_path: &std::path::Path) -> anyhow::Result<()> {
    let mut downloader = Downloader {
        current_manifest: None,
        video_store: PathBuf::from(""),
    };

    let backend = Arc::new(FileBackend::new(base_path));

    loop {
        downloader.update(backend.clone()).await?;

        // Do this every second
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
