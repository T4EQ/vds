use std::sync::LazyLock;

use actix_web::{HttpResponse, Responder, get, post, web};
use tokio::io::AsyncReadExt;

use vds_api::api::content::meta::get::{LocalVideoMeta, Progress, VideoStatus};

use crate::db::Database;

impl From<crate::db::DownloadStatus> for VideoStatus {
    fn from(value: crate::db::DownloadStatus) -> Self {
        match value {
            crate::db::DownloadStatus::Pending => VideoStatus::Pending,
            crate::db::DownloadStatus::InProgress((completed, total)) => {
                VideoStatus::Downloading(Progress(completed as f64 / total as f64))
            }
            crate::db::DownloadStatus::Downloaded(_) => VideoStatus::Downloaded,
            crate::db::DownloadStatus::Failed(msg) => VideoStatus::Failed(msg),
        }
    }
}

impl From<crate::db::Video> for LocalVideoMeta {
    fn from(value: crate::db::Video) -> Self {
        LocalVideoMeta {
            id: value.id.to_string(),
            name: value.name,
            size: value.file_size as usize,
            status: value.download_status.into(),
        }
    }
}

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
            status: VideoStatus::Failed("Because of reasons".to_string()),
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
async fn content_metadata_for_id(
    database: web::Data<Database>,
    id: web::Path<String>,
) -> impl Responder {
    use vds_api::api::content::meta::id::get::Response;
    let Ok(id) = id.into_inner().try_into() else {
        return HttpResponse::BadRequest().body("Invalid video ID");
    };

    let meta = match database.find_video(id).await {
        Ok(meta) => Some(meta.into()),
        Err(crate::db::Error::Diesel(diesel::result::Error::NotFound)) => None,
        Err(err) => {
            return HttpResponse::InternalServerError()
                .body(format!("Error querying the video from database: {err}"));
        }
    };

    HttpResponse::Ok().json(Response { meta })
}

#[get("/content/{id}")]
async fn get_content(database: web::Data<Database>, id: web::Path<String>) -> impl Responder {
    let Ok(id) = id.into_inner().try_into() else {
        return HttpResponse::BadRequest().body("Invalid video ID");
    };

    let Ok(crate::db::Video {
        download_status: crate::db::DownloadStatus::Downloaded(filepath),
        ..
    }) = database.increment_view_count(id).await
    else {
        return HttpResponse::NotFound().body("Requested video ID is not available");
    };

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

#[get("/manifest/latest")]
async fn get_manifest() -> impl Responder {
    // FIXME: Do not use a hardcoded manifest
    let manifest_file = include_str!("../../../docs/manifest-example.json");
    HttpResponse::Ok()
        .content_type("application/json")
        .body(manifest_file)
}

#[post("/manifest/fetch")]
async fn fetch_manifest() -> impl Responder {
    // TODO: Implement
    HttpResponse::Ok()
}
