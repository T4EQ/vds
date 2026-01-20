use actix_web::{
    HttpResponse, Responder, get, post,
    web::{self, Bytes, BytesMut},
};
use tokio::io::AsyncReadExt;

use vds_api::api::content::meta::get::{GroupedSection, LocalVideoMeta, Progress, VideoStatus};

use crate::{api::ApiData, downloader::UserCommand};

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
            view_count: value.view_count,
        }
    }
}

#[get("/version")]
async fn get_version() -> impl Responder {
    let info = crate::build_info::get();
    HttpResponse::Ok().json(info)
}

#[get("/content/meta")]
async fn list_content_metadata(api_data: web::Data<ApiData>) -> impl Responder {
    use vds_api::api::content::meta::get::Response;

    let sections = match api_data.db.current_manifest_sections().await {
        Ok(sections) => sections,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("Unexpected error querying content list: {e:?}"));
        }
    };

    let videos = sections
        .into_iter()
        .map(|(name, content)| {
            let content = content.into_iter().map(|v| v.into()).collect();
            GroupedSection { name, content }
        })
        .collect();

    HttpResponse::Ok().json(Response { videos })
}

#[get("/content/meta/{id}")]
async fn content_metadata_for_id(
    api_data: web::Data<ApiData>,
    id: web::Path<String>,
) -> impl Responder {
    use vds_api::api::content::meta::id::get::Response;
    let Ok(id) = id.into_inner().try_into() else {
        return HttpResponse::BadRequest().body("Invalid video ID");
    };

    let meta = match api_data.db.find_video(id).await {
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
async fn get_content(api_data: web::Data<ApiData>, id: web::Path<String>) -> impl Responder {
    let Ok(id) = id.into_inner().try_into() else {
        return HttpResponse::BadRequest().body("Invalid video ID");
    };
    let Ok(crate::db::Video {
        download_status: crate::db::DownloadStatus::Downloaded(filepath),
        ..
    }) = api_data.db.find_video(id).await
    else {
        return HttpResponse::NotFound().body("Requested video ID is not available");
    };

    let mut file = match tokio::fs::File::open(filepath).await {
        Ok(file) => file,
        Err(e) if e.kind() == tokio::io::ErrorKind::NotFound => {
            return HttpResponse::InternalServerError().body("Requested video is not on disk");
        }
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("Unexpected error opening file: {e:?}"));
        }
    };

    const RESPONSE_CHUNK_SIZE: usize = 4096;
    let s = async_stream::stream! {
        loop {
            // Note we are using a new bytes instance each time on purpose. We could have used
            // `split()` to get the current bytes out and reuse the instance. However, that makes
            // the bytes turn into a shared instance, which only releases the bytes once all
            // references to each of the chunks are dropped.
            //
            // This would not meet the intent of this code, which is to reduce the memory footprint
            // of this HTTP method, as some files might be hundreds of megabytes or even gigabytes
            // in size, and we only have 1 GiB of RAM for the entire platform.
            let mut bytes = BytesMut::with_capacity(RESPONSE_CHUNK_SIZE);
            let Ok(n) = file.read_buf(&mut bytes).await else {
                yield Err::<Bytes, anyhow::Error>(anyhow::anyhow!("Unable to read data from file"));
                return;
            };
            if n == 0 {
                return;
            }

            yield Ok::<Bytes, anyhow::Error>(bytes.freeze());
        }
    };

    HttpResponse::Ok()
        .content_type("video/mp4")
        .streaming(Box::pin(s))
}

#[post("/content/{id}/view")]
async fn increment_view_cnt(api_data: web::Data<ApiData>, id: web::Path<String>) -> impl Responder {
    let Ok(id) = id.into_inner().try_into() else {
        return HttpResponse::BadRequest().body("Invalid video ID");
    };
    let Ok(crate::db::Video {
        download_status: crate::db::DownloadStatus::Downloaded(_),
        ..
    }) = api_data.db.increment_view_count(id).await
    else {
        return HttpResponse::NotFound().body("Requested video ID is not available");
    };
    HttpResponse::Ok().finish()
}

#[get("/manifest/latest")]
async fn get_manifest(api_data: web::Data<ApiData>) -> impl Responder {
    let manifest = api_data.db.current_manifest().await;

    let manifest_file = manifest
        .as_ref()
        .and_then(|m| serde_json::to_string(m).ok())
        .unwrap_or_else(|| "".to_string());

    HttpResponse::Ok()
        .content_type("application/json")
        .body(manifest_file)
}

#[post("/manifest/fetch")]
async fn fetch_manifest(api_data: web::Data<ApiData>) -> impl Responder {
    match api_data.cmd_sender.send(UserCommand::FetchManifest) {
        Ok(()) => HttpResponse::Ok().finish(),
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("Unable to handle request: {e}"))
        }
    }
}
