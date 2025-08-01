use actix_web::{HttpResponse, Responder, delete, get, put, web};

#[get("/content/remote")]
async fn list_remote_content(
    web::Query(_query): web::Query<vds_api::api::content::local::get::Query>,
) -> impl Responder {
    // TODO(javier-varez): Actually list remote content
    use vds_api::api::content::remote::get::Response;
    let response = Response { videos: vec![] };
    HttpResponse::Ok().json(response)
}

#[get("/content/local")]
async fn list_local_content(
    web::Query(query): web::Query<vds_api::api::content::local::get::Query>,
) -> impl Responder {
    use vds_api::api::content::local::get::{Response, Video, VideoStatus};

    let mut mock_videos = vec![
        Video {
            id: "1".to_string(),
            name: "Introduction to Mathematics".to_string(),
            size: 245 * 1024 * 1024,
            status: VideoStatus::Downloaded,
        },
        Video {
            id: "2".to_string(),
            name: "Basic Science Concepts".to_string(),
            size: 312 * 1024 * 1024,
            status: VideoStatus::Downloading,
        },
        Video {
            id: "3".to_string(),
            name: "English Grammar Fundamentals".to_string(),
            size: 189 * 1024 * 1024,
            status: VideoStatus::Downloaded,
        },
        Video {
            id: "4".to_string(),
            name: "History of Ancient Civilizations".to_string(),
            size: 456 * 1024 * 1024,
            status: VideoStatus::Failed,
        },
        Video {
            id: "5".to_string(),
            name: "Environmental Science Basics".to_string(),
            size: 378 * 1024 * 1024,
            status: VideoStatus::Downloaded,
        },
    ];

    let response = if let Some(id) = query.id {
        let video = mock_videos.into_iter().find(|v| v.id == id);
        Response::Single(video)
    } else {
        if let Some(limit) = query.limit {
            mock_videos.resize_with(limit.min(mock_videos.len()), || unreachable!());
        }
        Response::Collection(mock_videos)
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
