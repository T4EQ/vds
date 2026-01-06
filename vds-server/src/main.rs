use std::{net::TcpListener, path::PathBuf};

use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    /// Path to the VDS configuration
    #[arg(short, long)]
    config: Option<PathBuf>,
}

fn default_config_path() -> PathBuf {
    "/var/lib/vds/config/config.toml".into()
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let config = vds_server::cfg::get_config(&args.config.unwrap_or_else(default_config_path))?;

    let listener = TcpListener::bind(format!(
        "{}:{}",
        config.http_config.listen_address, config.http_config.listen_port
    ))?;
    println!("Listening on {}", listener.local_addr()?);
    vds_server::run_app(listener, config)?.await?;

    Ok(())
}
