use std::pin::Pin;

use crate::downloader::Error;
use crate::downloader::backend::{Backend, ChunkResult}; // Change this line

use async_stream::stream;
use aws_config::BehaviorVersion;
use aws_sdk_s3::Client;
use tokio_stream::Stream;

pub struct S3Backend {
    client: Client,
    bucket: String,
}

impl S3Backend {
    pub async fn new(
        bucket: &str,
        aws_config: Option<&crate::cfg::AwsConfig>,
    ) -> anyhow::Result<Self> {
        tracing::info!("Initializing S3 backend for bucket: {}", bucket);

        let (access_key, secret_key, region) = if let Some(cfg) = aws_config {
            tracing::debug!("✓ Using AWS credentials from config file");
            tracing::debug!("✓ AWS Region: {}", cfg.region);
            (
                cfg.access_key_id.clone(),
                cfg.secret_access_key.clone(),
                cfg.region.clone(),
            )
        } else {
            // Fall back to environment variables
            let access_key = std::env::var("AWS_ACCESS_KEY_ID").map_err(|_| {
                anyhow::anyhow!("AWS_ACCESS_KEY_ID not set in environment or config")
            })?;
            let secret_key = std::env::var("AWS_SECRET_ACCESS_KEY").map_err(|_| {
                anyhow::anyhow!("AWS_SECRET_ACCESS_KEY not set in environment or config")
            })?;
            let region = std::env::var("AWS_REGION")
                .or_else(|_| std::env::var("AWS_DEFAULT_REGION"))
                .map_err(|_| anyhow::anyhow!("AWS_REGION not set in environment or config"))?;

            tracing::debug!("✓ Using AWS credentials from environment variables");
            tracing::debug!("✓ AWS Region: {}", region);
            (access_key, secret_key, region)
        };

        // Build AWS config with explicit credentials
        let creds =
            aws_sdk_s3::config::Credentials::new(access_key, secret_key, None, None, "config-file");

        let retry_config = aws_config::retry::RetryConfig::standard().with_max_attempts(3);

        let config = aws_config::defaults(BehaviorVersion::latest())
            .region(aws_config::Region::new(region))
            .credentials_provider(creds)
            .retry_config(retry_config)
            .load()
            .await;

        let client = Client::new(&config);

        // Verify bucket access
        client
            .head_bucket()
            .bucket(bucket)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("✗ Failed to verify S3 bucket access: {}", e);
                anyhow::anyhow!("Cannot access S3 bucket '{}': {}", bucket, e)
            })?;

        tracing::info!("✓ Successfully verified access to S3 bucket: {}", bucket);

        Ok(Self {
            client,
            bucket: bucket.to_string(),
        })
    }

    async fn get_s3_object(
        &self,
        key: &str,
    ) -> Result<aws_sdk_s3::operation::get_object::GetObjectOutput, Error> {
        tracing::debug!("Fetching S3 object: s3://{}/{}", self.bucket, key);

        self.client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| {
                tracing::error!(
                    "Failed to get S3 object s3://{}/{}: {}",
                    self.bucket,
                    key,
                    e
                );
                tracing::error!("Possible reasons:");
                tracing::error!("  - File does not exist in S3");
                tracing::error!("  - Missing s3:GetObject permission");
                tracing::error!("  - Invalid AWS credentials");
                tracing::error!("  - Network connectivity issue");
                Error::IoError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "Failed to get S3 object s3://{}/{}: {}",
                        self.bucket, key, e
                    ),
                ))
            })
    }
}

#[async_trait::async_trait]
impl Backend for S3Backend {
    fn fetch_resource<'a, 'b>(
        &'a self,
        uri: &'b http::Uri,
    ) -> Pin<Box<dyn Stream<Item = ChunkResult> + Send + 'a>>
    where
        'b: 'a,
    {
        Box::pin(stream! {
            let key = uri.path().trim_start_matches('/');

            let object = match self.get_s3_object(key).await {
                Ok(obj) => {
                    tracing::info!("Successfully initiated download of s3://{}/{}", self.bucket, key);
                    obj
                }
                Err(e) => {
                    yield Err(e);
                    return;
                }
            };

            let mut body = object.body;

            loop {
                match body.next().await {
                    Some(Ok(bytes)) => {
                        yield Ok(bytes.to_vec());
                    }
                    Some(Err(e)) => {
                        tracing::error!("Error reading S3 stream for s3://{}/{}: {}", self.bucket, key, e);
                        yield Err(Error::IoError(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("Error reading S3 stream: {}", e)
                        )));
                        return;
                    }
                    None => {
                        tracing::debug!("Completed download of s3://{}/{}", self.bucket, key);
                        break;
                    }
                }
            }
        })
    }

    async fn fetch_manifest(&self) -> Result<Vec<u8>, Error> {
        tracing::info!("Fetching manifest from s3://{}/manifest.json", self.bucket);

        let result = self.get_s3_object("manifest.json").await?;

        let data = result.body.collect().await.map_err(|e| {
            tracing::error!("Failed to read manifest body: {}", e);
            Error::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to read manifest body: {}", e),
            ))
        })?;

        tracing::info!("Successfully fetched manifest from S3");
        Ok(data.into_bytes().to_vec())
    }
}
