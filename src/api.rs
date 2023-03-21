use actix_web::{middleware::Compat, web};
use actix_web_httpauth::middleware::HttpAuthentication;

mod file;
mod info;
mod library;
pub mod user;

pub fn configure(cfg: &mut web::ServiceConfig) {
    let auth = HttpAuthentication::bearer(crate::auth::validator);

    cfg.service(
        web::scope("/api")
            .wrap(Compat::new(auth))
            .configure(user::configure)
            .configure(library::configure)
            .configure(file::configure),
    );
}

pub fn configure_no_auth(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/na")
            .configure(info::configure_na)
            .configure(user::configure_na)
            .service(health),
    );
}

#[actix_web::get("/health")]
async fn health() -> impl actix_web::Responder {
    actix_web::HttpResponse::Ok().body("OK")
}
