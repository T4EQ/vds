use std::path::PathBuf;

#[derive(serde::Deserialize)]
pub struct ServerSettings {
    pub listen_address: String,
    pub port: u16,
}

#[derive(Default, serde::Deserialize)]
pub struct Settings {
    pub server_settings: ServerSettings,

    pub content_path: PathBuf,

    pub debug_file_server: PathBuf,
}

impl Default for ServerSettings {
    fn default() -> Self {
        Self {
            listen_address: "127.0.0.1".to_string(),
            port: 8080,
        }
    }
}

pub fn load_settings(settings_file: &str) -> anyhow::Result<Settings> {
    let config = config::Config::builder()
        .add_source(config::File::with_name(settings_file))
        .add_source(config::Environment::with_prefix("VDS"))
        .build()?;
    Ok(config.try_deserialize()?)
}
