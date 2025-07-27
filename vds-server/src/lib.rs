use actix_web::{App, HttpServer, dev::Server};

use std::net::TcpListener;

mod api;
mod static_files;

pub fn run_app(listener: TcpListener) -> anyhow::Result<Server> {
    Ok(HttpServer::new(move || {
        App::new()
            .configure(api::register_handlers)
            .configure(static_files::register_static_files)
    })
    .listen(listener)?
    .run())
}
