use std::{io::stdout, net::TcpListener, path::PathBuf};

use clap::Parser;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser, Debug)]
struct Args {
    /// Path to the VDS configuration
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Displays version information
    #[arg(short, long)]
    version: bool,
}

fn default_config_path() -> PathBuf {
    "/var/lib/vds/config/config.toml".into()
}

fn print_version_info() {
    let info = vds_server::build_info::get();
    println!("{}:", info.name);
    println!("\tVersion: {}", info.version);
    if let Some(git_hash) = &info.git_hash {
        println!("\tGit hash: {git_hash}");
    } else {
        println!("\tGit hash: Unknown");
    }
    println!("\tAuthors:");
    for author in info.authors {
        println!("\t\t{author}");
    }
    println!("\tHomepage: {}", info.homepage);
    println!("\tLicense: {}", info.license);
    println!("\tRepository: {}", info.repository);
    println!();
    println!("Build info:");
    println!("\tProfile: {}", info.profile);
    println!("\trustc version: {}", info.rustc_version);
    println!("\tFeatures: {}", info.features);
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    if args.version {
        print_version_info();
        return Ok(());
    }
    let config = vds_server::cfg::get_config(&args.config.unwrap_or_else(default_config_path))?;

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                let level = if config.debug { "trace" } else { "info" };
                tracing_subscriber::EnvFilter::new(level)
            }),
        )
        .with(JsonStorageLayer)
        .with(BunyanFormattingLayer::new("vds-server".into(), stdout))
        .init();

    let listener = TcpListener::bind(format!(
        "{}:{}",
        config.http_config.listen_address, config.http_config.listen_port
    ))?;

    println!("Started server at http://{}", listener.local_addr()?);
    vds_server::run_app(listener, config).await?;

    Ok(())
}
