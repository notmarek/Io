use actix_web::{web, middleware::Compat};
use actix_web_httpauth::middleware::HttpAuthentication;


pub fn configure(cfg: &mut web::ServiceConfig) {
    let auth = HttpAuthentication::bearer(crate::auth::validator);

	cfg.service(
		web::scope("/api")
			.wrap(Compat::new(auth))
	);

}

#[actix_web::get("/health")]
async fn health() -> impl actix_web::Responder {
	actix_web::HttpResponse::Ok()
}
