use std::path::PathBuf;
use std::pin::Pin;

use crate::downloader::Error;

use async_stream::stream;
use tokio::io::AsyncReadExt;
use tokio_stream::Stream;

pub type ChunkResult = Result<Vec<u8>, Error>;

#[async_trait::async_trait]
pub trait Backend: Sync + Send {
    /// Fetches a resource from the given URI. Returns a stream of data.
    fn fetch_resource<'a, 'b>(
        &'a self,
        uri: &'b http::Uri,
    ) -> Pin<Box<dyn Stream<Item = ChunkResult> + Send + 'a>>
    where
        'b: 'a;

    /// Obtains the current manifest from the upstream
    async fn fetch_manifest(&self) -> Result<Vec<u8>, Error>;
}

const DEFAULT_CHUNK_SIZE: usize = 1024;

pub struct FileBackend {
    base_path: PathBuf,
    chunk_size: usize,
}

impl FileBackend {
    pub fn new(base_path: &std::path::Path) -> Self {
        let base_path = base_path.to_path_buf();
        Self {
            base_path,
            chunk_size: DEFAULT_CHUNK_SIZE,
        }
    }
}

#[async_trait::async_trait]
impl Backend for FileBackend {
    fn fetch_resource<'a, 'b>(
        &'a self,
        uri: &'b http::Uri,
    ) -> Pin<Box<dyn Stream<Item = ChunkResult> + Send + 'a>>
    where
        'b: 'a,
    {
        Box::pin(stream! {
            let relpath = uri.path().trim_start_matches(std::path::MAIN_SEPARATOR);
            let path = self.base_path.join(relpath);
            let mut file = tokio::fs::File::open(path).await?;

            loop {
                let mut chunk = vec![0; self.chunk_size];
                let n = file.read(&mut chunk[..]).await?;
                if n == 0 {
                    break;
                }
                chunk.resize(n, 0);
                yield Ok(chunk);
            }
        })
    }

    async fn fetch_manifest(&self) -> Result<Vec<u8>, Error> {
        let manifest_path = self.base_path.join("manifest.json");
        Ok(tokio::fs::read(manifest_path).await?)
    }
}

#[cfg(test)]
mod test {
    use googletest::OrFail;
    use http::Uri;

    use super::*;

    use tokio_stream::StreamExt;

    #[googletest::test]
    #[tokio::test]
    async fn read_resource_using_file_backend() -> googletest::Result<()> {
        let temp_dir = tempfile::TempDir::new().or_fail()?;
        let resource_filepath = temp_dir.path().join("video.mp4");
        let v = vec![123; 8321];

        std::fs::write(&resource_filepath, &v[..]).or_fail()?;

        let backend = FileBackend::new(temp_dir.path());
        let uri = Uri::from_static("/video.mp4");
        let mut stream = backend.fetch_resource(&uri);

        let mut n_chunks = 0;
        let mut total_size = 0;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.or_fail()?;
            total_size += chunk.len();
            n_chunks += 1;
        }

        assert_eq!(total_size, v.len());
        assert_eq!(n_chunks, v.len().div_ceil(DEFAULT_CHUNK_SIZE));

        Ok(())
    }
}
