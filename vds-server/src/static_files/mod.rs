use actix_web::web;

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

pub fn register_static_files(app: &mut web::ServiceConfig) {
    let generated = generate();

    app.service(
        actix_web_static_files::ResourceFiles::new("/", generated).resolve_not_found_to_root(),
    );
}
