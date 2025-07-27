use std::{net::TcpListener, path::PathBuf};

use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, default_value = "127.0.0.1")]
    listen_address: String,

    #[arg(short, long, default_value = "8080")]
    port: u16,

    #[arg(short, long)]
    content_path: PathBuf,
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let listener = TcpListener::bind(format!("{}:{}", args.listen_address, args.port))?;
    println!("Listening on {}", listener.local_addr()?);
    vds_server::run_app(listener, &args.content_path)?.await?;
    Ok(())
}
