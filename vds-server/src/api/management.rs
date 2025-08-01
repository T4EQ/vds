use std::sync::LazyLock;

use actix_web::{HttpResponse, Responder, delete, get, put, web};

use vds_api::api::content::local::get::{LocalVideoMeta, Progress, VideoStatus};

#[get("/content/remote")]
async fn list_remote_content(
    web::Query(_query): web::Query<vds_api::api::content::remote::get::Query>,
) -> impl Responder {
    // TODO(javier-varez): Actually list remote content
    use vds_api::api::content::remote::get::Response;
    let response = Response { videos: vec![] };
    HttpResponse::Ok().json(response)
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

#[get("/content/local")]
async fn list_local_content(
    web::Query(query): web::Query<vds_api::api::content::local::get::Query>,
) -> impl Responder {
    use vds_api::api::content::local::get::Response;

    let response = {
        let mut videos = MOCK_VIDEOS.clone();
        if let Some(limit) = query.limit {
            videos.resize_with(limit.min(videos.len()), || unreachable!());
        }
        Response { videos }
    };
    HttpResponse::Ok().json(response)
}

#[get("/content/local/single")]
async fn get_local_content_meta(
    web::Query(query): web::Query<vds_api::api::content::local::single::get::Query>,
) -> impl Responder {
    use vds_api::api::content::local::single::get::Response;
    let response = {
        let video = MOCK_VIDEOS.iter().find(|v| v.id == query.id).cloned();
        Response { video }
    };
    HttpResponse::Ok().json(response)
}

#[delete("/content/local")]
async fn delete_local_content(
    web::Query(_query): web::Query<vds_api::api::content::local::delete::Query>,
) -> impl Responder {
    // TODO(javier-varez): Actually remove db content
    HttpResponse::NoContent()
}

#[put("/content/local")]
async fn cache_content(
    web::Query(_query): web::Query<vds_api::api::content::local::put::Query>,
) -> impl Responder {
    // TODO(javier-varez): Actually implement
    HttpResponse::Ok()
}
