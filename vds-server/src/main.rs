use std::net::TcpListener;

use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, default_value = "0.0.0.0")]
    listen_address: String,

    #[arg(short, long, default_value = "8080")]
    port: u16,
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let listener = TcpListener::bind(format!("{}:{}", args.listen_address, args.port))?;
    println!("Listening on {}", listener.local_addr()?);
    vds_server::run_app(listener)?.await?;
    Ok(())
}
