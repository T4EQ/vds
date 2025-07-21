use actix_web::{HttpResponse, Responder, delete, get, put, web};

#[get("/content/remote")]
async fn list_remote_content() -> impl Responder {
    // TODO(javier-varez): Actually list remote content
    use vds_api::api::content::remote::get::Response;
    let response = Response { videos: vec![] };
    HttpResponse::Ok().json(response)
}

#[get("/content/local")]
async fn list_local_content(
    web::Query(query): web::Query<vds_api::api::content::local::get::Query>,
) -> impl Responder {
    use vds_api::api::content::local::get::Response;

    // TODO(javier-varez): Actually list db content
    let response = if let Some(_id) = query.id {
        Response::Single(None)
    } else {
        Response::Collection(vec![])
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
