use crate::config::Config;
use crate::models::user::User;
use crate::DBPool;
use crate::ErrorResponse;
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Tokens {
    status: String,
    token_type: String,
    token: String,
    refresh_token: String,
    expiration: i64,
}

#[actix_web::put("/user")]
async fn register(
    config: web::Data<Config>,
    dbpool: web::Data<DBPool>,
) -> impl actix_web::Responder {
    // todo!();
    let user = User::new("Marek".to_string(), "password".to_string(), vec![]);
    // let jwt: Claims = ;
    match user.register("epicsalt#".to_string(), &dbpool, config.jwt.valid_for) {
        Ok(claims) => HttpResponse::Ok().json(Tokens {
            status: "ok".to_string(),
            token_type: "Bearer".to_string(),
            token: claims.create_token(&config.jwt.private_key).unwrap(),
            refresh_token: claims
                .create_refresh_token(&config.jwt.private_key)
                .unwrap(),
            expiration: claims.exp,
        }),
        Err(e) => HttpResponse::Ok().json(ErrorResponse {
            status: "error".to_string(),
            error: e,
        }),
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(register);
}
