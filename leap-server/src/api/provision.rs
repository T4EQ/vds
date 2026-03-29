use actix_web::{HttpResponse, Responder, get, post};

use crate::provision;

#[tracing::instrument(
    fields(
        request_id = %uuid::Uuid::new_v4(),
    )
)]
#[post("network_config")]
async fn set_network_config() -> impl Responder {
    HttpResponse::Ok().await
}

#[tracing::instrument(
    fields(
        request_id = %uuid::Uuid::new_v4(),
    )
)]
#[get("get_storage_devs")]
async fn get_storage_devs() -> impl Responder {
    match provision::storage::list_disks().await {
        Ok(devs) => HttpResponse::Ok().json(devs),
        Err(error) => HttpResponse::InternalServerError().body(error.to_string()),
    }
}
