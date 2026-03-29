use actix_web::{HttpResponse, Responder, post};

#[tracing::instrument(
    fields(
        request_id = %uuid::Uuid::new_v4(),
    )
)]
#[post("network_config")]
async fn set_network_config() -> impl Responder {
    HttpResponse::Ok().await
}
