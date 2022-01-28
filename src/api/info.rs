use crate::config::Config;
use actix_web::web;
use serde::Serialize;

#[actix_web::get("/info")]
async fn info(config: web::Data<Config>) -> impl actix_web::Responder {
    #[derive(Serialize)]
    struct Resp {
        site_name: String,
        version: String,
        uptime: i64,
        load: String,
        storage: String,
    }

    actix_web::HttpResponse::Ok().json(Resp {
        site_name: config.info.name.clone(),
        version: config.info.version.clone(),
        uptime: 8219821918,                 // TODO: get the actual uptime
        load: "1.76 2.04 2.44".to_string(), // TODO: get the actual load
        storage: config.info.storage_url.clone(),
    })
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(info);
}
