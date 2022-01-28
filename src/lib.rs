#[macro_use]
extern crate diesel;

pub mod config;
pub mod api;
pub mod auth;
pub mod models;
pub mod schema;

pub type DBPool = diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::pg::PgConnection>>;
