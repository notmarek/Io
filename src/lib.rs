use serde::Serialize;

#[macro_use]
extern crate diesel;
extern crate log;
extern crate argon2;
extern crate pretty_env_logger;


pub mod config;
pub mod api;
pub mod auth;
pub mod models;
pub mod schema;

pub type DBPool = diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::pg::PgConnection>>;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    status: String,
    error: String,
}

pub struct Session {
    pub startup: i64,
}