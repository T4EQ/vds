use std::{net::TcpListener, path::PathBuf};

use clap::Parser;

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

pub mod build_info {
    std::include!(std::concat!(std::env!("OUT_DIR"), "/built.rs"));

    pub fn print_version_info() {
        println!("{PKG_NAME}: {PKG_VERSION}");
        println!("\tAuthors: {PKG_AUTHORS}");
        println!("\tHomepage: {PKG_HOMEPAGE}");
        println!("\tLicense: {PKG_LICENSE}");
        println!("\tRepository: {PKG_REPOSITORY}");

        println!("Build info:");
        println!("\tProfile: {PROFILE}");
        println!("\trustc version: {RUSTC_VERSION}");
        println!("\tFeatures: {FEATURES_STR}");
        if let Some(git_hash) = GIT_COMMIT_HASH {
            let dirty = if GIT_DIRTY.is_some_and(|v| v) {
                "-dirty"
            } else {
                ""
            };
            println!("\tGit hash: {git_hash}{dirty}");
        } else if let Some(hash) = std::option_env!("VDS_SERVER_NIX_GIT_REVISION") {
            println!("\tGit hash: {hash}");
        } else {
            println!("\tGit hash: unknown");
        }
    }
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    if args.version {
        build_info::print_version_info();
        return Ok(());
    }
    let config = vds_server::cfg::get_config(&args.config.unwrap_or_else(default_config_path))?;

    let listener = TcpListener::bind(format!(
        "{}:{}",
        config.http_config.listen_address, config.http_config.listen_port
    ))?;

    println!("Started server at http://{}", listener.local_addr()?);
    vds_server::run_app(listener, config).await?;

    Ok(())
}
