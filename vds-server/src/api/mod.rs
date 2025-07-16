use actix_web::{App, web};
use actix_web::{HttpResponse, Responder, put};

mod management;

#[derive(serde::Deserialize, serde::Serialize)]
struct Auth {
    auth_success: bool,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct AuthRequest {
    username: String,
    password: String,
}

#[put("/user/auth")]
async fn user_authenticate(req: web::Json<AuthRequest>) -> impl Responder {
    // FIXME: Do not hardcode
    if req.username == "Javi" && req.password == "test" {
        HttpResponse::Ok().json(Auth { auth_success: true })
    } else {
        HttpResponse::Ok().json(Auth {
            auth_success: false,
        })
    }
}

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
            .service(management::cache_content)
            .service(user_authenticate),
    )
}
