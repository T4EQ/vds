use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};

use actix_web::{HttpResponse, Responder, get, post, web};
use tokio::io::AsyncReadExt;

use vds_api::api::content::meta::get::{LocalVideoMeta, Progress, VideoStatus};

static MOCK_VIDEOS: LazyLock<Vec<LocalVideoMeta>> = LazyLock::new(|| {
    vec![
        LocalVideoMeta {
            id: "1".to_string(),
            name: "Introduction to Mathematics".to_string(),
            size: 245 * 1024 * 1024,
            status: VideoStatus::Downloaded,
        },
        LocalVideoMeta {
            id: "2".to_string(),
            name: "Basic Science Concepts".to_string(),
            size: 312 * 1024 * 1024,
            status: VideoStatus::Downloading(Progress(0.5)),
        },
        LocalVideoMeta {
            id: "3".to_string(),
            name: "English Grammar Fundamentals".to_string(),
            size: 189 * 1024 * 1024,
            status: VideoStatus::Downloaded,
        },
        LocalVideoMeta {
            id: "4".to_string(),
            name: "History of Ancient Civilizations".to_string(),
            size: 456 * 1024 * 1024,
            status: VideoStatus::Failed,
        },
        LocalVideoMeta {
            id: "5".to_string(),
            name: "Environmental Science Basics".to_string(),
            size: 378 * 1024 * 1024,
            status: VideoStatus::Downloaded,
        },
    ]
});

#[get("/content/meta")]
async fn list_content_metadata(
    web::Query(query): web::Query<vds_api::api::content::meta::get::Query>,
) -> impl Responder {
    use vds_api::api::content::meta::get::Response;

    let response = {
        let mut videos = MOCK_VIDEOS.clone();
        if let Some(limit) = query.limit {
            videos.resize_with(limit.min(videos.len()), || unreachable!());
        }
        Response { videos }
    };
    HttpResponse::Ok().json(response)
}

#[get("/content/meta/{id}")]
async fn content_metadata_for_id(id: web::Path<String>) -> impl Responder {
    use vds_api::api::content::meta::single::get::Response;

    let response = Response {
        meta: MOCK_VIDEOS.iter().find(|v| v.id == *id).cloned(),
    };
    HttpResponse::Ok().json(response)
}

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

#[get("/content/{id}")]
async fn get_content(content_path: web::Data<PathBuf>, id: web::Path<String>) -> impl Responder {
    get_content_inner(content_path.get_ref(), &id).await
}

#[get("/manifest/latest")]
async fn get_manifest() -> impl Responder {
    let manifest_file = include_str!("../../../docs/manifest-example.json");
    HttpResponse::Ok()
        .content_type("application/json")
        .body(manifest_file)
}

#[post("/manifest/fetch")]
async fn fetch_manifest() -> impl Responder {
    // TODO
    HttpResponse::Ok()
}
