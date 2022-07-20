use std::fmt::Display;
use std::sync::{Arc, Mutex};

use actix_web::FromRequest;
use actix_web::HttpMessage;
use actix_web::HttpRequest;
use actix_web::ResponseError;
use eventqueue::Queue;
use futures::future::{ready, Ready};
use serde::Serialize;
use thiserror::Error;

#[macro_use]
extern crate diesel;
extern crate argon2;
extern crate log;
extern crate pretty_env_logger;

pub mod api;
pub mod auth;
pub mod config;
pub mod eventqueue;
pub mod models;
pub mod schema;
pub mod utils;

pub type DBPool = diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::pg::PgConnection>>;
pub type ArcQueue = Arc<Mutex<Queue>>;

#[derive(Debug, Serialize)]
pub struct Response<T: Serialize> {
    status: String,
    data: T,
}

#[derive(Debug, Serialize)]
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
}

#[derive(Clone, Debug)]
pub struct AuthData(pub models::user::User);

impl FromRequest for AuthData {
    type Error = Unauthorized;

    type Future = Ready<Result<Self, Self::Error>>;
    fn from_request(req: &HttpRequest, _payload: &mut actix_http::Payload) -> Self::Future {
        ready(req.extensions().get().map(Self::clone).ok_or(Unauthorized))
    }
}
