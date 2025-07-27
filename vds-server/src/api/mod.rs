use actix_web::web;

mod management;
mod user;

pub fn register_handlers(app: &mut web::ServiceConfig) {
    app.service(
        web::scope("api")
            .service(management::list_remote_content)
            .service(management::list_local_content)
            .service(management::delete_local_content)
            .service(management::cache_content)
            .service(user::get_content)
            .service(user::get_content_path),
    );
}
