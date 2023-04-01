use crate::{config::Config, Session};
use actix_web::get;
use actix_web::web;
use actix_web::web::Data;
use chrono::Utc;
use sea_orm::DatabaseConnection;
use serde::Serialize;
use sysinfo::{System, SystemExt};
use utoipa::{self, ToSchema};

#[derive(Serialize, ToSchema)]
pub struct Info {
    site_name: String,
    version: String,
    uptime: i64,
    system_uptime: u64,
    load: String,
    storage: String,
    memory: String,
    swap: String,
    os: String,
    db_engine: String,
}

#[utoipa::path(
    tag = "Info",
    context_path = "/na",
    responses(
        (status = 200, description = "Returns information about the backend.", body = Info),
    ),
)]
#[get("/info")]
async fn info(
    config: web::Data<Config>,
    session_info: web::Data<Session>,
    db: web::Data<DatabaseConnection>,
) -> impl actix_web::Responder {
    let mut sys = System::new_all();
    sys.refresh_all();
    let load = sys.load_average();
    let db_engine = match db.into_inner().as_ref() {
        DatabaseConnection::SqlxMySqlPoolConnection(_) => String::from("mysql"),
        DatabaseConnection::SqlxPostgresPoolConnection(_) => String::from("postgres"),
        DatabaseConnection::SqlxSqlitePoolConnection(_) => String::from("sqlite"),
        _ => String::from("unknown"),
    };
    actix_web::HttpResponse::Ok().json(Info {
        site_name: config.info.name.clone(),
        version: config.info.version.clone(),
        uptime: Utc::now().timestamp() - session_info.startup,
        system_uptime: sys.uptime(),
        load: format!("{}, {}, {}", load.one, load.five, load.fifteen),
        storage: config.info.storage_url.clone(),
        memory: format!(
            "{} MB / {} MB",
            sys.used_memory() / (1024_u64.pow(2)),
            sys.total_memory() / (1024_u64.pow(2))
        ),
        swap: format!(
            "{} MB / {} MB",
            sys.used_swap() / (1024_u64.pow(2)),
            sys.total_swap() / (1024_u64.pow(2))
        ),
        os: format!(
            "{} {} {}",
            sys.name().unwrap_or_default(),
            sys.os_version().unwrap_or_default(),
            sys.kernel_version().unwrap()
        ),
        db_engine,
    })
}

pub fn configure_na(cfg: &mut web::ServiceConfig) {
    cfg.service(info);
}
