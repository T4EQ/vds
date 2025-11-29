use std::net::TcpListener;

use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    config_file: String,
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let settings = vds_server::configuration::load_settings(&args.config_file)?;
    let listener = TcpListener::bind(format!(
        "{}:{}",
        settings.server_settings.listen_address, settings.server_settings.port
    ))?;
    println!("Listening on {}", listener.local_addr()?);

    let server = vds_server::create_server(listener, &settings.content_path)?;
    let downloader = vds_server::create_downloader(&settings.debug_file_server);
    tokio::try_join!(downloader, async { Ok(server.await?) })?;

    Ok(())
}
