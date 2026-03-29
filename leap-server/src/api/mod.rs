use std::sync::Arc;

use crate::{cfg::LeapConfig, db::Database, downloader::UserCommand};

use actix_web::web;
use tokio::sync::mpsc::UnboundedSender;

mod provision;
mod user;

/// Shared resources used in HTTP handlers
pub struct ApiData {
    config: LeapConfig,
    db: Arc<Database>,
    cmd_sender: UnboundedSender<UserCommand>,
}

impl ApiData {
    pub fn new(
        config: LeapConfig,
        db: Arc<Database>,
        cmd_sender: UnboundedSender<UserCommand>,
    ) -> Self {
        Self {
            config,
            db,
            cmd_sender,
        }
    }
}

fn common_api_handlers() -> actix_web::Scope {
    web::scope("api").service(user::get_version)
}

pub fn register_handlers(app: &mut web::ServiceConfig) {
    app.service(
        common_api_handlers()
            .service(user::list_content_metadata)
            .service(user::content_metadata_for_id)
            .service(user::get_content)
            .service(user::increment_view_cnt)
            .service(user::fetch_manifest)
            .service(user::get_manifest)
            .service(user::log_file),
    );
}

pub fn register_provisioning_handlers(app: &mut web::ServiceConfig) {
    app.service(common_api_handlers());
    app.service(
        web::scope("provision")
            .service(provision::set_network_config)
            .service(provision::get_storage_devs),
    );
}
