use crate::auth::Claims;
use crate::config::Config;
use crate::models::user::User;
use crate::AuthData;
use crate::DBPool;
use crate::ErrorResponse;
use actix_web::{error, web, HttpResponse};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Tokens {
    status: String,
    token_type: String,
    token: String,
    refresh_token: String,
    expiration: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserRequest {
    pub username: Option<String>,
    pub password: Option<String>,
    pub refresh_token: Option<String>,
    pub identifier: Option<String>,
    pub captcha: Option<String>,     // TODO: captcha integration
    pub invite: Option<String>,      // TODO: invite system
    pub newpassword: Option<String>, // TODO: password changing
    pub private: Option<String>,     // TODO: account privacy settings
}

#[actix_web::put("/user")]
async fn register(
    config: web::Data<Config>,
    dbpool: web::Data<DBPool>,
    req_data: web::Json<UserRequest>,
) -> impl actix_web::Responder {
    let user = User::new(
        req_data.username.clone().unwrap(),
        req_data.password.clone().unwrap(),
        vec![],
    );
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

#[actix_web::post("/user")]
async fn login(
    config: web::Data<Config>,
    dbpool: web::Data<DBPool>,
    req_data: web::Json<UserRequest>,
) -> impl actix_web::Responder {
    if req_data.identifier.as_ref().unwrap() == "password" {
        let user = User::new(
            req_data.username.clone().unwrap(),
            req_data.password.clone().unwrap(),
            vec![],
        );
        match user.login(&dbpool, config.jwt.valid_for) {
            Ok(claims) => {
                return HttpResponse::Ok().json(Tokens {
                    status: "ok".to_string(),
                    token_type: "Bearer".to_string(),
                    token: claims.create_token(&config.jwt.private_key).unwrap(),
                    refresh_token: claims
                        .create_refresh_token(&config.jwt.private_key)
                        .unwrap(),
                    expiration: claims.exp,
                })
            }
            Err(e) => {
                return HttpResponse::Ok().json(ErrorResponse {
                    status: "error".to_string(),
                    error: e,
                })
            }
        }
    } else if req_data.identifier.as_ref().unwrap() == "refresh_token" {
        let claims = Claims::from_token(
            req_data.refresh_token.as_ref().unwrap(),
            &config.jwt.public_key,
        )
        .unwrap();

        match User::get(claims.user_id, &dbpool) {
            Ok(u) => {
                if u.permissions.contains(&"banned".to_string()) {
                    return HttpResponse::Ok().json(ErrorResponse {
                        status: "error".to_string(),
                        error: "banned_user".to_string(),
                    });
                } else if claims.perms.contains(&"REFRESH".to_string()) {
                    match u.login(&dbpool, config.jwt.valid_for) {
                        Ok(claims) => {
                            return HttpResponse::Ok().json(Tokens {
                                status: "ok".to_string(),
                                token_type: "Bearer".to_string(),
                                token: claims.create_token(&config.jwt.private_key).unwrap(),
                                refresh_token: claims
                                    .create_refresh_token(&config.jwt.private_key)
                                    .unwrap(),
                                expiration: claims.exp,
                            })
                        }
                        Err(e) => {
                            return HttpResponse::Ok().json(ErrorResponse {
                                status: "error".to_string(),
                                error: e,
                            })
                        }
                    }
                } else {
                    return HttpResponse::Ok().json(ErrorResponse {
                        status: "error".to_string(),
                        error: "invalid_token".to_string(),
                    });
                }
            }

            Err(_) => {
                return HttpResponse::Ok().json(ErrorResponse {
                    status: "error".to_string(),
                    error: "invalid_user".to_string(),
                })
            }
        };
    }
    HttpResponse::Ok().json(ErrorResponse {
        status: "error".to_string(),
        error: "unknown_identifier".to_string(),
    })
}

#[derive(Deserialize)]
struct Uid {
    user_id: String,
}

#[actix_web::get("/user/{user_id}")]
async fn user_info(
    path: web::Path<Uid>,
    AuthData(user): AuthData,
    pool: web::Data<DBPool>,
) -> actix_web::Result<impl actix_web::Responder> {
    let user_info = {
        if path.user_id == "@me" {
            user
        } else {
            match User::get(path.user_id.clone(), &pool) {
                Ok(u) => u,
                Err(e) => {
                    return Err(error::ErrorNotFound(ErrorResponse {
                        status: "error".to_string(),
                        error: e,
                    }))
                }
            }
        }
    };

    Ok(HttpResponse::Ok().json(user_info))
}

pub fn configure_na(cfg: &mut web::ServiceConfig) {
    cfg.service(register).service(login);
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(user_info);
}
