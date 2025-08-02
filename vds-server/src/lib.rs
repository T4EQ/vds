use actix_web::{App, HttpServer, dev::Server, web};

use std::{net::TcpListener, path::Path};

mod api;
mod static_files;

pub fn run_app(listener: TcpListener, content_path: &Path) -> anyhow::Result<Server> {
    let content_path = web::Data::new(content_path.to_owned());
    Ok(HttpServer::new(move || {
        App::new()
            .app_data(content_path.clone())
            .configure(api::register_handlers)
            .configure(static_files::register_static_files)
    })
    .listen(listener)?
    .run())
}
