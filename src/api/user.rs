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

#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LimitQuery {
    pub limit: Option<i64>,
    pub page: Option<i64>,
}

#[actix_web::put("/user")]
async fn register(
    config: web::Data<Config>,
    dbpool: web::Data<DBPool>,
    req_data: web::Json<RegisterRequest>,
) -> impl actix_web::Responder {
    let user = User::new(req_data.username.clone(), req_data.password.clone(), vec![]);
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
        Err(e) => HttpResponse::NotAcceptable().json(ErrorResponse {
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
    match req_data
        .identifier
        .as_ref()
        .unwrap_or(&String::new())
        .as_str()
    {
        "password" => {
            let user = User::new(
                req_data.username.clone().unwrap(),
                req_data.password.clone().unwrap(),
                vec![],
            );
            match user.login(&dbpool, config.jwt.valid_for) {
                Ok(claims) => HttpResponse::Ok().json(Tokens {
                    status: "ok".to_string(),
                    token_type: "Bearer".to_string(),
                    token: claims.create_token(&config.jwt.private_key).unwrap(),
                    refresh_token: claims
                        .create_refresh_token(&config.jwt.private_key)
                        .unwrap(),
                    expiration: claims.exp,
                }),

                Err(e) => HttpResponse::Unauthorized().json(ErrorResponse {
                    status: "error".to_string(),
                    error: e,
                }),
            }
        }
        "refresh_token" => {
            let claims = Claims::from_token(
                req_data.refresh_token.as_ref().unwrap(),
                &config.jwt.public_key,
            )
            .unwrap();

            match User::get(claims.user_id, &dbpool) {
                Ok(u) => {
                    if u.permissions.contains(&"banned".to_string()) {
                        HttpResponse::Ok().json(ErrorResponse {
                            status: "error".to_string(),
                            error: "banned_user".to_string(),
                        })
                    } else if claims.perms.contains(&"REFRESH".to_string()) {
                        let c = u.refresh(config.jwt.valid_for);
                        HttpResponse::Ok().json(Tokens {
                            status: "ok".to_string(),
                            token_type: "Bearer".to_string(),
                            token: c.create_token(&config.jwt.private_key).unwrap(),
                            refresh_token: c.create_refresh_token(&config.jwt.private_key).unwrap(),
                            expiration: c.exp,
                        })
                    } else {
                        HttpResponse::Ok().json(ErrorResponse {
                            status: "error".to_string(),
                            error: "invalid_token".to_string(),
                        })
                    }
                }

                Err(e) => HttpResponse::Unauthorized().json(ErrorResponse {
                    status: "error".to_string(),
                    error: e,
                }),
            }
        }
        _ => HttpResponse::Unauthorized().json(ErrorResponse {
            status: "error".to_string(),
            error: "unknown_identifier".to_string(),
        }),
    }
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
            if !user.has_permission_one_of(vec!["view_users", "*_users", "administrator"]) {
                return Err(error::ErrorForbidden(ErrorResponse {
                    status: "error".to_string(),
                    error: "missing_permissions".to_string(),
                }));
            }
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

#[actix_web::get("/users")]
async fn user_list(
    AuthData(user): AuthData,
    pool: web::Data<DBPool>,
    query: web::Query<LimitQuery>,
) -> actix_web::Result<impl actix_web::Responder> {
    if !user.has_permission_one_of(vec!["view_users", "*_users", "administrator"]) {
        return Err(error::ErrorForbidden(ErrorResponse {
            status: "error".to_string(),
            error: "missing_permissions".to_string(),
        }));
    }
    let users = User::get_all(query.limit.unwrap_or(25), query.page.unwrap_or(1), &pool);
    Ok(HttpResponse::Ok().json(users))
}

pub fn configure_na(cfg: &mut web::ServiceConfig) {
    cfg.service(register).service(login);
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(user_info).service(user_list);
}
