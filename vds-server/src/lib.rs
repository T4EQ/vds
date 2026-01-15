use actix_web::{App, HttpServer, web};
use anyhow::Context;
use tokio::sync::mpsc;

use std::{net::TcpListener, sync::Arc};

use crate::cfg::VdsConfig;

pub mod cfg;
pub mod db;

mod api;
mod downloader;
mod manifest;
mod static_files;

pub async fn run_app(listener: TcpListener, config: VdsConfig) -> anyhow::Result<()> {
    let database = Arc::new(
        db::Database::open(config.db_config)
            .await
            .context("While initializing database")?,
    );

    database.apply_pending_migrations().await?;

    let (user_command_sender, user_command_receiver) = mpsc::unbounded_channel();

    let downloader_fut = downloader::run_downloader(
        config.downloader_config,
        Arc::clone(&database),
        user_command_receiver,
    );

    let database = web::Data::new(api::ApiData::new(
        Arc::clone(&database),
        user_command_sender,
    ));

    let server = HttpServer::new({
        let database = database.clone();
        move || {
            App::new()
                .app_data(database.clone())
                .configure(api::register_handlers)
                .configure(static_files::register_static_files)
        }
    })
    .listen(listener)?
    .run();

    tokio::select! {
        downloader = downloader_fut => {
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
