use actix_web::web;

mod static_files {
    #![allow(
        clippy::all,
        clippy::pedantic,
        clippy::nursery,
        unused_imports,
        unused_variables
    )]
    include!(concat!(env!("OUT_DIR"), "/generated.rs"));
}

pub fn register_static_files(app: &mut web::ServiceConfig) {
    let generated = static_files::generate();
    app.service(
        actix_web_static_files::ResourceFiles::new("/", generated).resolve_not_found_to_root(),
    );
}
