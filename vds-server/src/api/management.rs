use actix_web::{HttpResponse, Responder, delete, get, post, web};

#[get("/content/remote")]
async fn list_remote_content() -> impl Responder {
    // TODO(javier-varez): Actually list remote content
    HttpResponse::Ok().body("No remote content")
}

#[get("/content/local")]
async fn list_local_content() -> impl Responder {
    // TODO(javier-varez): Actually list db content
    HttpResponse::Ok().body("No local content")
}

#[delete("/content/local/{name}")]
async fn delete_local_content(name: web::Path<String>) -> impl Responder {
    // TODO(javier-varez): Actually remove db content
    HttpResponse::Ok().body(format!("Deleted {name}"))
}

#[post("/content/cache")]
async fn cache_content() -> impl Responder {
    // TODO(javier-varez): Figure out how this request will look like
    HttpResponse::Ok().body("")
}

#[get("/content/get")]
async fn get_content() -> impl Responder {
    // TODO(javier-varez): Figure out how to serve content as video
    HttpResponse::Ok().body("")
}
