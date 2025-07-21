use actix_web::{HttpResponse, Responder, get, web};

static FILE: &[u8] = include_bytes!("../../../video-data/rickroll.mp4");

#[get("/content")]
async fn get_content(
    web::Query(_query): web::Query<vds_api::api::content::Query>,
) -> impl Responder {
    // FIXME: Of course, we need to serve proper video data, not a hardcoded video file
    HttpResponse::Ok().body(FILE)
}

#[get("/content/{id}")]
async fn get_content_path(_path: web::Path<String>) -> impl Responder {
    // FIXME: Of course, we need to serve proper video data, not a hardcoded video file
    HttpResponse::Ok().body(FILE)
}
