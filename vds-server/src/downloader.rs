use std::time::Duration;

use crate::manifest::ManifestFile;
use backend::FileBackend;

use anyhow::Context;

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
}

impl Downloader {
    pub async fn update(&mut self, backend: &dyn backend::Backend) -> anyhow::Result<()> {
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

        for video in manifest.sections.iter().flat_map(|s| s.content.iter()) {
            let video = backend
                .fetch_resource(&video.uri)
                .await
                .with_context(|| format!("Error reading video {}", video.uri))?;
            // TODO: handle the video
            let _ = video;
        }

        // All fetched!
        Ok(())
    }
}

pub async fn downloader_task(base_path: &std::path::Path) -> anyhow::Result<()> {
    let mut downloader = Downloader {
        current_manifest: None,
    };

    let backend = FileBackend::new(base_path);

    loop {
        downloader.update(&backend).await?;

        // Do this every second
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
