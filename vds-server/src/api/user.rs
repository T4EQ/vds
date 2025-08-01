use actix_web::{HttpResponse, Responder, get, web};
use std::path::{Path, PathBuf};
use tokio::io::AsyncReadExt;

async fn get_content_inner(content_path: &Path, id: &str) -> HttpResponse {
    let mut filepath = content_path.join(id);
    filepath.set_extension("mp4");

    let mut file = match tokio::fs::File::open(filepath).await {
        Ok(file) => file,
        Err(e) if e.kind() == tokio::io::ErrorKind::NotFound => {
            return HttpResponse::NotFound().finish();
        }
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("Unexpected error opening file: {e:?}"));
        }
    };

    let mut data = Vec::new();
    if let Err(e) = file.read_to_end(&mut data).await {
        return HttpResponse::InternalServerError().body(format!("Unable to read file: {e:?}"));
    };

    HttpResponse::Ok().content_type("video/mp4").body(data)
}

#[get("/content")]
async fn get_content(
    content_path: web::Data<PathBuf>,
    web::Query(query): web::Query<vds_api::api::content::get::Query>,
) -> impl Responder {
    get_content_inner(content_path.get_ref(), &query.id).await
}

#[get("/content/{id}")]
async fn get_content_path(
    content_path: web::Data<PathBuf>,
    id: web::Path<String>,
) -> impl Responder {
    get_content_inner(content_path.get_ref(), &id).await
}
