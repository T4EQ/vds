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

    let server = vds_server::create_server(listener, &args.content_path)?;
    let downloader = vds_server::create_downloader(&args.content_path);
    tokio::try_join!(downloader, async { Ok(server.await?) })?;

    Ok(())
}
