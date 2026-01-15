use std::sync::Arc;

use crate::{db::Database, downloader::UserCommand};

use actix_web::web;
use tokio::sync::mpsc::UnboundedSender;

mod user;

/// Shared resources used in HTTP handlers
pub struct ApiData {
    db: Arc<Database>,
    cmd_sender: UnboundedSender<UserCommand>,
}

impl ApiData {
    pub fn new(db: Arc<Database>, cmd_sender: UnboundedSender<UserCommand>) -> Self {
        Self { db, cmd_sender }
    }
}

pub fn register_handlers(app: &mut web::ServiceConfig) {
    app.service(
        web::scope("api")
            .service(user::list_content_metadata)
            .service(user::content_metadata_for_id)
            .service(user::get_content)
            .service(user::fetch_manifest)
            .service(user::get_manifest),
    );
}
