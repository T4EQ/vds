use actix_web::{App, HttpServer, dev::Server, web};

use std::net::TcpListener;

pub mod cfg;
pub mod db;

mod api;
mod manifest;
mod static_files;

pub fn run_app(listener: TcpListener, db: &'static db::Database) -> anyhow::Result<Server> {
    let content_path = web::Data::new(db);
    Ok(HttpServer::new(move || {
        App::new()
            .app_data(content_path.clone())
            .configure(api::register_handlers)
            .configure(static_files::register_static_files)
    })
    .listen(listener)?
    .run())
}
