use actix_web::web;

mod user;

pub fn register_handlers(app: &mut web::ServiceConfig) {
    app.service(
        web::scope("api")
            .service(user::list_content_metadata)
            .service(user::content_metadata_for_id)
            .service(user::get_content)
            .service(user::fetch_manifest)
            .service(user::get_manifest),
    );
}
