use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use config::Config;
use http::Uri;
use secrecy::{ExposeSecret, SecretString};

pub const DEFAULT_CONFIG_PATH: &str = "/var/lib/leap/config/config.toml";

fn default_path_style() -> bool {
    false
}

fn default_aws_region() -> String {
    "us-east-1".to_string()
}

pub fn serialize_secret_str<S>(
    data: &Option<SecretString>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match data.as_ref() {
        Some(secret) => serializer.serialize_some(secret.expose_secret()),
        None => serializer.serialize_none(),
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct RetryParams {
    /// The initial backoff time after a download failure.
    #[serde(with = "humantime_serde")]
    pub initial_backoff: std::time::Duration,

    /// The adjustement factor for the backoff after a failure. Must be larger than 1 so that
    /// the backoff actually increments exponentially
    pub backoff_factor: f64,

    /// The maximum backoff time after a download failure.
    #[serde(with = "humantime_serde")]
    pub max_backoff: std::time::Duration,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct DownloaderConfig {
    /// Number of maximum concurrent downloads.
    pub concurrent_downloads: usize,

    /// The read/writeable path where the video files will be stored.
    pub content_path: PathBuf,

    /// URI of the remote server providing the manifest and content cached by the LEAP.
    #[serde(with = "parse_uri")]
    pub remote_server: Uri,

    /// The interval at which the remote is queried for new content.
    #[serde(with = "humantime_serde")]
    pub update_interval: std::time::Duration,

    /// Retry parameters when a download fails.
    pub retry_params: RetryParams,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct DbConfig {
    /// The maximum amount of time that the DB thread will wait until the DB is available for its
    /// operation. Sqlite does not allow concurrent reads and writes, and therefore, it might block
    /// until one completes
    #[serde(with = "humantime_serde")]
    pub busy_timeout: std::time::Duration,

    /// The number of connections that are created for the database. Limits the amount of
    /// concurrent database connections.
    pub pool_size: usize,

    /// The path where the database contents are stored
    pub runtime_path: PathBuf,
}

impl DbConfig {
    pub fn db_path(&self) -> PathBuf {
        self.runtime_path.join("leap.db")
    }

    pub fn manifest_path(&self) -> PathBuf {
        self.runtime_path.join("current_manifest.json")
    }

    pub fn temp_manifest_path(&self) -> PathBuf {
        self.runtime_path.join("_temp_manifest.json")
    }

    pub fn logfile(&self) -> PathBuf {
        self.runtime_path.join("leap_runtime.log")
    }
}

/// Configuration to access the S3 server. Note the bucket is handled separately in the main
/// configuration.
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct S3Config {
    /// S3 Endpoint URL. Defaults to AWS if not given.
    pub endpoint_url: Option<String>,

    /// Uses path-style access to buckets instead of dns-based access. Use this if your endpoint is
    /// not AWS and you are unable to connect to your bucket (MinIO, Ceph, etc).
    #[serde(default = "default_path_style")]
    pub force_path_style: bool,

    /// Access key ID.
    #[serde(serialize_with = "serialize_secret_str")]
    pub access_key_id: Option<SecretString>,

    /// Secret Access key.
    #[serde(serialize_with = "serialize_secret_str")]
    pub secret_access_key: Option<SecretString>,

    /// AWS region. Defaults to `us-east-1`.
    #[serde(default = "default_aws_region")]
    pub region: String,
}

/// Configuration of the LEAP application.
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct LeapConfig {
    /// Enables debug logging/tracing.
    pub debug: bool,

    /// Downloader service configuration.
    pub downloader_config: DownloaderConfig,

    /// Database configuration.
    pub db_config: DbConfig,

    /// S3 configuration.
    pub s3_config: S3Config,
}

/// Parses the configuration of the LEAP, returning a LeapConfig struct.
/// Uses the given path to read a structured file format (toml, yaml, json, etc).
/// Individual values can be overriden by `LEAP_`-prefixed environment variables.
pub fn get_config(path: &Path) -> Result<LeapConfig> {
    let config = Config::builder()
        .add_source(config::File::with_name(
            path.to_str()
                .context("Parsing configuration path as a str")?,
        ))
        .add_source(config::Environment::with_prefix("LEAP"))
        .build()
        .context("Building the configuration of the LEAP from file and environment")?;

    config
        .try_deserialize()
        .context("Deserializing the configuration as LeapConfig")
}

mod parse_uri {
    use http::Uri;

    pub fn deserialize<'de, D: serde::Deserializer<'de>>(
        d: D,
    ) -> std::result::Result<Uri, D::Error> {
        d.deserialize_str(Visitor {})
    }

    pub fn serialize<S>(data: &Uri, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = format!("{data}");
        serializer.serialize_str(&s)
    }

    struct Visitor {}

    impl<'de> serde::de::Visitor<'de> for Visitor {
        type Value = Uri;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            writeln!(formatter, "A valid URI")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            v.try_into()
                .map_err(|e| E::custom(format!("{v} is an invalid URI: {e}")))
        }
    }
}
