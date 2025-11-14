use std::path::PathBuf;

use crate::manifest::ManifestFile;

use super::Error;

#[async_trait::async_trait]
pub trait Backend {
    // FIXME: Return stream instead of Vec so that we can tell the progress
    async fn fetch_resource(&self, uri: &http::Uri) -> Result<Vec<u8>, Error>;

    /// Obtains the current manifest from the upstream
    async fn fetch_manifest(&self) -> Result<ManifestFile, Error>;
}

pub struct FileBackend {
    base_path: PathBuf,
}

impl FileBackend {
    pub fn new(base_path: &std::path::Path) -> Self {
        let base_path = base_path.to_path_buf();
        Self { base_path }
    }
}

#[async_trait::async_trait]
impl Backend for FileBackend {
    async fn fetch_resource(&self, uri: &http::Uri) -> Result<Vec<u8>, Error> {
        let path = uri.path();
        let path = self.base_path.join(path);
        Ok(tokio::fs::read(path).await?)
    }

    async fn fetch_manifest(&self) -> Result<ManifestFile, Error> {
        let manifest_path = self.base_path.join("manifest.json");
        let result = tokio::fs::read(manifest_path).await?;
        Ok(serde_json::from_slice(&result)?)
    }
}
