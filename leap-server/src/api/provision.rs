use std::path::PathBuf;

use actix_web::{HttpResponse, Responder, get, post, web};
use leap_api::provision::config::post::LeapConfig;
use leap_api::provision::network::post::NetworkConfig;
use serde::Deserialize;
use tokio::sync::Mutex;

use crate::api::ProvisionApiData;

#[tracing::instrument(fields(request_id = %uuid::Uuid::new_v4()))]
#[post("network")]
async fn set_network_config(
    provision_data: web::Data<Mutex<ProvisionApiData>>,
    web::Json(network_config): web::Json<NetworkConfig>,
) -> impl Responder {
    match provision_data.try_lock() {
        Ok(mut lock) => match lock.provision.configure_network(&network_config).await {
            Ok(()) => HttpResponse::Ok().body(""),
            Err(err) => HttpResponse::InternalServerError().body(format!("{err}")),
        },
        Err(_) => HttpResponse::BadRequest().body("Another provisioning operation is ongoing"),
    }
}

#[tracing::instrument(fields(request_id = %uuid::Uuid::new_v4()))]
#[get("storage/devices")]
async fn get_storage_devs(provision_data: web::Data<Mutex<ProvisionApiData>>) -> impl Responder {
    match provision_data.try_lock() {
        Ok(lock) => match lock.provision.list_blockdevs().await {
            Ok(blockdevs) => HttpResponse::Ok().json(blockdevs),
            Err(err) => HttpResponse::InternalServerError().body(format!("{err}")),
        },
        Err(_) => HttpResponse::BadRequest().body("Another provisioning operation is ongoing"),
    }
}

#[derive(Deserialize)]
struct FormatStorageQuery {
    name: PathBuf,
}

#[tracing::instrument(fields(request_id = %uuid::Uuid::new_v4()))]
#[post("storage/format")]
async fn format_storage(
    provision_data: web::Data<Mutex<ProvisionApiData>>,
    web::Query(FormatStorageQuery { name }): web::Query<FormatStorageQuery>,
) -> impl Responder {
    match provision_data.try_lock() {
        Ok(mut lock) => match lock.provision.configure_storage(&name).await {
            Ok(blockdevs) => HttpResponse::Ok().json(blockdevs),
            Err(err) => HttpResponse::InternalServerError().body(format!("{err}")),
        },
        Err(_) => HttpResponse::BadRequest().body("Another provisioning operation is ongoing"),
    }
}

#[tracing::instrument(fields(request_id = %uuid::Uuid::new_v4()))]
#[post("config")]
async fn set_configuration(
    provision_data: web::Data<Mutex<ProvisionApiData>>,
    web::Json(config): web::Json<LeapConfig>,
) -> impl Responder {
    match provision_data.try_lock() {
        Ok(mut lock) => match lock.provision.configure_leap(&config).await {
            Ok(()) => HttpResponse::Ok().body(""),
            Err(err) => HttpResponse::InternalServerError().body(format!("{err}")),
        },
        Err(_) => HttpResponse::BadRequest().body("Another provisioning operation is ongoing"),
    }
}

#[tracing::instrument(fields(request_id = %uuid::Uuid::new_v4()))]
#[post("complete")]
async fn complete_provisioning(
    provision_data: web::Data<Mutex<ProvisionApiData>>,
) -> impl Responder {
    match provision_data.try_lock() {
        Ok(mut lock) => match lock.provision.finish().await {
            Ok(()) => HttpResponse::Ok().body(""),
            Err(err) => HttpResponse::InternalServerError().body(format!("{err}")),
        },
        Err(_) => HttpResponse::BadRequest().body("Another provisioning operation is ongoing"),
    }
}

#[tracing::instrument(fields(request_id = %uuid::Uuid::new_v4()))]
#[get("status")]
async fn status(provision_data: web::Data<Mutex<ProvisionApiData>>) -> impl Responder {
    match provision_data.try_lock() {
        Ok(lock) => match lock.provision.status() {
            Ok(status) => HttpResponse::Ok().json(status),
            Err(err) => HttpResponse::InternalServerError().body(format!("{err}")),
        },
        Err(_) => HttpResponse::BadRequest().body("Another provisioning operation is ongoing"),
    }
}
