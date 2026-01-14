use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use config::Config;
use http::Uri;

fn default_listen_addr() -> String {
    "127.0.0.1".to_string()
}

fn default_listen_port() -> u16 {
    8080
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct HttpServerConfig {
    /// Address/interface to listen for TCP connections.
    #[serde(default = "default_listen_addr")]
    pub listen_address: String,

    /// Port to listen for TCP connections.
    #[serde(default = "default_listen_port")]
    pub listen_port: u16,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct DownloaderConfig {
    /// Number of maximum concurrent downloads.
    pub concurrent_downloads: usize,

    /// The read/writeable path where the video files will be stored.
    pub content_path: PathBuf,

    /// URI of the remote server providing the manifest and content cached by the VDS.
    #[serde(with = "parse_uri")]
    pub remote_server: Uri,

    /// The initial backoff time after a download failure.
    #[serde(with = "humantime_serde")]
    pub initial_backoff: std::time::Duration,

    /// The interval at which the remote is queried for new content.
    #[serde(with = "humantime_serde")]
    pub update_interval: std::time::Duration,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct DbConfig {
    /// The maximum amount of time that the DB thread will wait until the DB is available for its
    /// operation. Sqlite does not allow concurrent reads and writes, and therefore, it might block
    /// until one completes
    #[serde(with = "humantime_serde")]
    pub busy_timeout: std::time::Duration,

    /// The path where the database contents are stored
    pub runtime_path: PathBuf,
}

impl DbConfig {
    pub fn db_path(&self) -> PathBuf {
        self.runtime_path.join("vds.db")
    }

    pub fn manifest_path(&self) -> PathBuf {
        self.runtime_path.join("current_manifest.json")
    }

    pub fn temp_manifest_path(&self) -> PathBuf {
        self.runtime_path.join("_temp_manifest.json")
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct VdsConfig {
    /// Enables debug logging/tracing.
    pub debug: bool,

    /// HTTP Server configuration
    pub http_config: HttpServerConfig,

    /// Downloader service configuration
    pub downloader_config: DownloaderConfig,

    /// Database configuration
    pub db_config: DbConfig,
}

/// Parses the configuration of the VDS, returning a VdsConfig struct.
/// Uses the given path to read a structured file format (toml, yaml, json, etc).
/// Individual values can be overriden by `VDS_`-prefixed environment variables.
pub fn get_config(path: &Path) -> Result<VdsConfig> {
    let config = Config::builder()
        .add_source(config::File::with_name(
            path.to_str()
                .context("Parsing configuration path as a str")?,
        ))
        .add_source(config::Environment::with_prefix("VDS"))
        .build()
        .context("Building the configuration of the VDS from file and environment")?;

    config
        .try_deserialize()
        .context("Deserializing the configuration as VdsConfig")
}

mod parse_uri {
    use http::Uri;

    pub fn deserialize<'de, D: serde::Deserializer<'de>>(
        d: D,
    ) -> std::result::Result<Uri, D::Error> {
        d.deserialize_str(Visitor {})
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
