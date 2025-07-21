use actix_web::{HttpResponse, Responder, get, web};

#[derive(serde::Deserialize)]
struct LocalContentQuery {
    id: String,
}

#[get("/content")]
async fn get_content(web::Query(query): web::Query<LocalContentQuery>) -> impl Responder {
    // TODO(javier-varez): Implement
    HttpResponse::Ok().body(format!("This is the content for {}", query.id))
}
