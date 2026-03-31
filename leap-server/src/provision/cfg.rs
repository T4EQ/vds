use std::time::Duration;

use crate::cfg::{DbConfig, DownloaderConfig, LeapConfig, RetryParams, S3Config};

impl From<&leap_api::provision::config::post::LeapConfig> for LeapConfig {
    fn from(value: &leap_api::provision::config::post::LeapConfig) -> Self {
        let has_custom_endpoint = value.s3_config.endpoint_url.is_some();
        Self {
            debug: false,
            db_config: DbConfig {
                busy_timeout: Duration::from_secs(10),
                pool_size: 16,
                runtime_path: "/var/lib/leap/runtime_path".into(),
            },
            s3_config: S3Config {
                endpoint_url: value.s3_config.endpoint_url.clone(),
                force_path_style: value
                    .s3_config
                    .force_path_style
                    .unwrap_or(has_custom_endpoint),
                access_key_id: Some(value.s3_config.access_key_id.clone()),
                secret_access_key: Some(value.s3_config.secret_access_key.clone()),
                region: value
                    .s3_config
                    .region
                    .as_deref()
                    .unwrap_or("us-east-1")
                    .to_owned(),
            },
            downloader_config: DownloaderConfig {
                concurrent_downloads: value.downloader_config.concurrent_downloads,
                remote_server: value.s3_config.bucket.clone(),
                update_interval: value.downloader_config.update_interval,
                content_path: "/var/lib/leap/content_path".into(),
                retry_params: RetryParams {
                    initial_backoff: value.downloader_config.retry_params.initial_backoff,
                    backoff_factor: value.downloader_config.retry_params.backoff_factor,
                    max_backoff: value.downloader_config.retry_params.max_backoff,
                },
            },
        }
    }
}

pub async fn provision_configuration(
    config: &leap_api::provision::config::post::LeapConfig,
) -> anyhow::Result<LeapConfig> {
    let blockdevs = super::storage::list_blockdevs().await?;
    let mounted = blockdevs
        .iter()
        .any(|b| b.mountpoints.contains(&"/var/lib/leap".to_owned()));
    if !mounted {
        anyhow::bail!("Cannot apply configuration. /var/lib/leap is not mounted");
    }

    let config: LeapConfig = config.into();

    // Check S3 access to validate configuration
    let bucket = config
        .downloader_config
        .remote_server
        .host()
        .ok_or_else(|| anyhow::anyhow!("S3 URI must specify a bucket name"))?;
    let s3_backend =
        crate::downloader::s3backend::S3Backend::new(bucket, &config.s3_config).await?;
    s3_backend.verify_bucket_access().await?;

    // Save configuration to file
    let target_dir: std::path::PathBuf = "/var/lib/leap/config/config.toml".into();
    if let Some(parent) = target_dir.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let serialized_config = toml::to_string(&config)?;
    tokio::fs::write(target_dir, serialized_config.as_bytes()).await?;

    Ok(config)
}
