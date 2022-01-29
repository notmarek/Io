use crate::config::Config;
use actix_web::web;
use serde::Serialize;
use sysinfo::{NetworkExt, NetworksExt, ProcessExt, System, SystemExt};

#[actix_web::get("/info")]
async fn info(config: web::Data<Config>) -> impl actix_web::Responder {
    let mut sys = System::new_all();
    sys.refresh_all();
    let load = sys.load_average();
    #[derive(Serialize)]
    struct Resp {
        site_name: String,
        version: String,
        uptime: i64,
        system_uptime: u64,
        load: String,
        storage: String,
        memory: String,
        swap: String,
        os: String,
    }

    actix_web::HttpResponse::Ok().json(Resp {
        site_name: config.info.name.clone(),
        version: config.info.version.clone(),
        uptime: 133769420,                 // TODO: get the actual uptime
        system_uptime: sys.uptime(),
        load: format!("{}, {}, {}", load.one, load.five, load.fifteen),
        storage: config.info.storage_url.clone(),
        memory: format!("{}MB / {}MB", sys.used_memory() / 1024, sys.total_memory() /  1024),
        swap: format!("{}MB / {}MB", sys.used_swap() / 1024, sys.total_swap() / 1024),
        os: format!("{} {} {}", sys.name().unwrap_or_default(), sys.os_version().unwrap_or_default(), sys.kernel_version().unwrap())
    })
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(info);
}
