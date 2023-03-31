use crate::auth::Claims;
use crate::config::Config;
use crate::models::user::User;
use crate::AuthData;
use crate::DatabaseConnection;
use crate::ErrorResponse;
use actix_web::{error, web, HttpResponse};
use actix_web::{get, post, put};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(ToSchema, Serialize, Deserialize)]
pub struct Tokens {
    status: String,
    token_type: String,
    token: String,
    refresh_token: String,
    expiration: i64,
}

#[derive(ToSchema, Serialize, Deserialize, Debug)]
pub struct UserRequest {
    pub username: Option<String>,
    pub password: Option<String>,
    pub refresh_token: Option<String>,
    pub identifier: String,
    pub captcha: Option<String>,     // TODO: captcha integration
    pub invite: Option<String>,      // TODO: invite system
    pub newpassword: Option<String>, // TODO: password changing
    pub private: Option<String>,     // TODO: account privacy settings
}

#[derive(ToSchema, Serialize, Deserialize, Debug)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LimitQuery {
    pub limit: Option<i64>,
    pub page: Option<i64>,
}

/// Register
/// Let's you register a new user account
#[utoipa::path(
    tag = "User",
    context_path = "/na",
    responses(
        (status = 200, description = "Account created succesfully", body = Tokens),
        (status = 406, description = "Couldn't create an account", body = ErrorResponse)
    ),
    request_body(content = RegisterRequest, description = "Registration request", content_type = "application/json"),
)]
#[put("/user")]
async fn register(
    config: web::Data<Config>,
    DatabaseConnection: web::Data<DatabaseConnection>,
    req_data: web::Json<RegisterRequest>,
) -> impl actix_web::Responder {
    let user = User::new(req_data.username.clone(), req_data.password.clone(), vec![]);
    match user.register("epicsalt#".to_string(), &DatabaseConnection, config.jwt.valid_for) {
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

/// Login
/// Let's you login using your password
/// or refresh your token with the refresh token
#[utoipa::path(
    tag = "User",
    context_path = "/na",
    responses(
        (status = 200, description = "Logged in.", body = Tokens),
        (status = 401, description = "Couldn't login successfully - see error", body = ErrorResponse)
    ),
    request_body(content = UserRequest, description = "Login request", content_type = "application/json"),
)]
#[post("/user")]
async fn login(
    config: web::Data<Config>,
    DatabaseConnection: web::Data<DatabaseConnection>,
    req_data: web::Json<UserRequest>,
) -> impl actix_web::Responder {
    match req_data.identifier.as_str() {
        "password" => {
            let user = User::new(
                req_data.username.clone().unwrap(),
                req_data.password.clone().unwrap(),
                vec![],
            );
            match user.login(&DatabaseConnection, config.jwt.valid_for) {
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

            match User::get(claims.user_id, &DatabaseConnection) {
                Ok(u) => {
                    if u.permissions.contains(&"banned".to_string()) {
                        HttpResponse::Unauthorized().json(ErrorResponse {
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
                        HttpResponse::Unauthorized().json(ErrorResponse {
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

#[derive(IntoParams, Deserialize)]
struct Uid {
    user_id: String,
}

/// Get User By ID
#[utoipa::path(
    tag = "User",
    context_path = "/api",
    responses(
        (status = 200, description = "Returns a user.", body = User),
        (status = 404, description = "No such user found", body = ErrorResponse)
    ),
    params(Uid),
    security(("token" = []))
)]
#[get("/user/{user_id}")]
async fn user_info(
    path: web::Path<Uid>,
    AuthData(user): AuthData,
    pool: web::Data<DatabaseConnection>,
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

/// Get a list of all users
#[utoipa::path(
    tag = "User",
    context_path = "/api",
    responses(
        (status = 200, description = "Returns a user.", body = [User]),
        (status = 401, description = "Access denied.", body = ErrorResponse)
    ),
    security(("token" = []))
)]
#[get("/users")]
async fn user_list(
    AuthData(user): AuthData,
    pool: web::Data<DatabaseConnection>,
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
