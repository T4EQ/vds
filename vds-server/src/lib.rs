use actix_web::{App, HttpServer, web};
use anyhow::Context;

use std::{net::TcpListener, sync::Arc};

use crate::cfg::VdsConfig;

pub mod cfg;
pub mod db;

mod api;
mod downloader;
mod manifest;
mod static_files;

pub async fn run_app(listener: TcpListener, config: VdsConfig) -> anyhow::Result<()> {
    let database = db::Database::open(config.db_config)
        .await
        .context("While initializing database")?;

    database.apply_pending_migrations().await?;

    let database = web::Data::new(database);

    let downloader_fut =
        downloader::run_downloader(config.downloader_config, Arc::clone(&*database));

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
            // the server can exit due to SIGINT. Using joing for these 2 futures would not
            // terminate the application because downloader would keep running indefinitely
        }
    };

    Ok(())
}
