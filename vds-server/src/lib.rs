use actix_web::{App, HttpServer, web};
use anyhow::Context;
use tokio::sync::mpsc;

use std::{net::TcpListener, sync::Arc};

use crate::cfg::VdsConfig;

pub mod build_info;
pub mod cfg;
pub mod db;

mod api;
mod downloader;
mod manifest;
mod static_files;

pub async fn run_app(listener: TcpListener, config: VdsConfig) -> anyhow::Result<()> {
    let database = Arc::new(
        db::Database::open(config.db_config.clone())
            .await
            .context("While initializing database")?,
    );

    database.apply_pending_migrations().await?;

    let (user_command_sender, user_command_receiver) = mpsc::unbounded_channel();

    let downloader = downloader::run_downloader(
        config.downloader_config.clone(),
        config.aws_config.clone(),
        Arc::clone(&database),
        user_command_receiver,
    );

    let api_data = web::Data::new(api::ApiData::new(
        config.clone(),
        Arc::clone(&database),
        user_command_sender,
    ));

    let server = HttpServer::new(move || {
        App::new()
            .app_data(api_data.clone())
            .wrap(tracing_actix_web::TracingLogger::default())
            .configure(api::register_handlers)
            .configure(static_files::register_static_files)
    })
    .listen(listener)?
    .run();

    tokio::select! {
        downloader = downloader => {
            downloader?;
            panic!("Unexpected downloader task exit.");
        }
        server = server => {
            server?;
            // the server can exit due to SIGINT. Using join for these 2 futures would not
            // terminate the application because downloader would keep running indefinitely
        }
    };

    Ok(())
}
