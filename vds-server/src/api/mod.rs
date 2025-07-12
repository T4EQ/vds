use actix_web::{App, web};

mod management;

pub fn register_handlers<T>(app: App<T>) -> App<T>
where
    T: actix_web::dev::ServiceFactory<
            actix_web::dev::ServiceRequest,
            Config = (),
            Error = actix_web::Error,
            InitError = (),
        >,
{
    app.service(
        web::scope("api")
            .service(management::list_remote_content)
            .service(management::list_local_content)
            .service(management::delete_local_content)
            .service(management::get_content)
            .service(management::cache_content),
    )
}
