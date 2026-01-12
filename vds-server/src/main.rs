use std::{net::TcpListener, path::PathBuf, time::Duration};

use anyhow::Context;
use clap::Parser;
use static_cell::StaticCell;
use vds_server::db::Database;

#[derive(Parser, Debug)]
struct Args {
    /// Path to the VDS configuration
    #[arg(short, long)]
    config: Option<PathBuf>,
}

fn default_config_path() -> PathBuf {
    "/var/lib/vds/config/config.toml".into()
}

static DATABASE: StaticCell<Database> = StaticCell::new();

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let config = vds_server::cfg::get_config(&args.config.unwrap_or_else(default_config_path))?;

    let listener = TcpListener::bind(format!(
        "{}:{}",
        config.http_config.listen_address, config.http_config.listen_port
    ))?;

    const TIMEOUT: Duration = Duration::from_secs(2);
    let db_path = config.runtime_path.join("vds.db");

    let database = DATABASE.init(
        Database::open(db_path.to_str().unwrap(), TIMEOUT)
            .await
            .context("While initializing database")?,
    );

    println!("Started server at http://{}", listener.local_addr()?);
    vds_server::run_app(listener, database)?.await?;

    Ok(())
}
