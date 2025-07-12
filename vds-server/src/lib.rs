use actix_web::{App, HttpServer, dev::Server};

use std::sync::Arc;

use std::net::TcpListener;

mod api;

pub fn run_app(listener: TcpListener, files_path: String) -> anyhow::Result<Server> {
    let files_path = Arc::new(files_path);
    Ok(HttpServer::new(move || {
        api::register_handlers(App::new())
            .service(actix_files::Files::new("/", &*files_path).index_file("index.html"))
    })
    .listen(listener)?
    .run())
}
