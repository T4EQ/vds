use actix_web::{App, HttpServer, dev::Server, web};

use std::net::TcpListener;

pub mod cfg;

mod api;
mod db;
mod manifest;
mod static_files;

use cfg::VdsConfig;

pub fn run_app(listener: TcpListener, config: VdsConfig) -> anyhow::Result<Server> {
    let content_path = web::Data::new(config);
    Ok(HttpServer::new(move || {
        App::new()
            .app_data(content_path.clone())
            .configure(api::register_handlers)
            .configure(static_files::register_static_files)
    })
    .listen(listener)?
    .run())
}
