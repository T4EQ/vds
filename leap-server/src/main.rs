use std::{io::stdout, net::TcpListener, path::PathBuf};

use clap::Parser;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser, Debug)]
struct Args {
    /// Path to the LEAP configuration
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Fall back to provisioning if the LEAP configuration file could not be open and parsed.
    #[arg(short = 'f', long = "fallback")]
    provision_fallback: bool,

    /// Run provisioning instead of the main application
    #[arg(short = 'p', long = "provision")]
    provision: bool,

    /// Address
    #[arg(long = "address", default_value = "0.0.0.0")]
    address: String,

    /// Port
    #[arg(long = "port", default_value = "80")]
    port: u16,

    /// Displays version information.
    #[arg(short, long)]
    version: bool,
}

fn default_config_path() -> PathBuf {
    "/var/lib/leap/config/config.toml".into()
}

fn print_version_info() {
    let info = leap_server::build_info::get();
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

#[derive(thiserror::Error, Debug)]
enum AppError {
    #[error("The LEAP configuration could not be loaded: {0}")]
    InvalidConfiguration(anyhow::Error),
    #[error("LEAP failed during runtime: {0}")]
    RuntimeError(anyhow::Error),
}

async fn start_leap_server(args: &Args) -> Result<(), AppError> {
    let config =
        leap_server::cfg::get_config(args.config.as_ref().unwrap_or(&default_config_path()))
            .map_err(AppError::InvalidConfiguration)?;

    let open_logfile = {
        let logfile = config.db_config.logfile();
        move || -> Box<dyn std::io::Write> {
            Box::new(
                std::fs::File::options()
                    .create(true)
                    .append(true)
                    .open(&logfile)
                    .map_err(|e| format!("Unable to open logfile {logfile:?}: {e}"))
                    .unwrap(),
            )
        }
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                let level = if config.debug { "trace" } else { "info" };
                tracing_subscriber::EnvFilter::new(level)
            }),
        )
        .with(JsonStorageLayer)
        .with(BunyanFormattingLayer::new("leap-server".into(), stdout))
        .with(BunyanFormattingLayer::new(
            "leap-server".into(),
            open_logfile,
        ))
        .init();

    let listener = TcpListener::bind(format!("{}:{}", args.address, args.port))
        .map_err(|e| AppError::RuntimeError(e.into()))?;

    println!(
        "Started server at http://{}",
        listener
            .local_addr()
            .map_err(|e| AppError::RuntimeError(e.into()))?
    );
    leap_server::run_app(listener, config)
        .await
        .map_err(|e| AppError::RuntimeError(e.into()))?;
    Ok(())
}

async fn start_leap_provisioning(args: &Args) -> anyhow::Result<()> {
    let listener = TcpListener::bind(format!("{}:{}", args.address, args.port))?;
    leap_server::run_provisioning(listener).await?;
    Ok(())
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    if args.version {
        print_version_info();
        return Ok(());
    }

    if args.provision {
        start_leap_provisioning(&args).await?;
    } else {
        match start_leap_server(&args).await {
            res @ Err(AppError::InvalidConfiguration(_)) => {
                if args.provision_fallback {
                    start_leap_provisioning(&args).await?;
                }
                res
            }
            res => res,
        }?;
    }

    Ok(())
}
