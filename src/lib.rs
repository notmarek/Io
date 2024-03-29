use actix_web::FromRequest;
use actix_web::HttpMessage;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::ResponseError;
use eventqueue::Queue;
use futures::future::{ready, Ready};
use models::user::UserActions;
use serde::Serialize;
use std::fmt::Display;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;
// #[macro_use]
// extern crate diesel;
extern crate argon2;
extern crate log;
extern crate pretty_env_logger;

pub mod api;
pub mod auth;
pub mod config;
pub mod data_sources;
pub mod docs;
pub mod eventqueue;
pub mod models;
// pub mod schema;
pub mod utils;
use log::error;
use utoipa::ToSchema;

// pub type DatabaseConnection = ;
pub type ArcQueue = Arc<Mutex<Queue>>;

#[derive(ToSchema, Debug, Serialize)]
pub struct Response<T: Serialize> {
    status: String,
    data: T,
}

#[derive(ToSchema, Debug, Serialize)]
pub struct ErrorResponse {
    status: String,
    error: String,
}

impl Display for ErrorResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        serde_json::to_string(self).unwrap().fmt(f)
    }
}

impl<T: Serialize> Display for Response<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        serde_json::to_string(self).unwrap().fmt(f)
    }
}
pub struct Session {
    pub startup: i64,
}

#[derive(Error, Debug)]
#[error("unauthorized")]
pub struct Unauthorized;

impl ResponseError for Unauthorized {
    fn status_code(&self) -> actix_http::StatusCode {
        actix_http::StatusCode::UNAUTHORIZED
    }
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(actix_web::http::header::ContentType::json())
            .body(format!(r#"{{ "status": "error", "error": "{self}" }}"#))
    }
}

#[derive(Clone, Debug)]
pub struct AuthData(pub entity::user::Model);

impl FromRequest for AuthData {
    type Error = Unauthorized;

    type Future = Ready<Result<Self, Self::Error>>;
    fn from_request(req: &HttpRequest, _payload: &mut actix_http::Payload) -> Self::Future {
        ready(req.extensions().get().map(Self::clone).ok_or(Unauthorized))
    }
}

#[derive(Clone, Debug)]
pub struct VerifiedAuthData(pub entity::user::Model);
impl FromRequest for VerifiedAuthData {
    type Error = Unauthorized;
    type Future = Ready<Result<Self, Self::Error>>;
    fn from_request(req: &HttpRequest, _payload: &mut actix_http::Payload) -> Self::Future {
        let ad: Result<AuthData, Self::Error> = req
            .extensions()
            .get()
            .map(AuthData::clone)
            .ok_or(Unauthorized);
        let vad: Result<Self, Self::Error> = match ad {
            Ok(AuthData(user)) => match user.has_permission_one_of(vec!["verified"]) {
                true => Ok(VerifiedAuthData(user)),
                false => Err(Unauthorized),
            },
            Err(e) => Err(e),
        };
        ready(vad)
    }
}
