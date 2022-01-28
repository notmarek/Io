use crate::config::Config;
use actix_web::{web, HttpResponse};
use serde::{Serialize, Deserialize};
use crate::DBPool;

#[actix_web::put("/user")]
async fn register(config: web::Data<Config>, dbpool: web::Data<DBPool>) -> impl actix_web::Responder {
    todo!();
    let db = &dbpool.get().unwrap();

    
    ""
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(register);
}
