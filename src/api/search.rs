use crate::{models::file::FileActions, Response, VerifiedAuthData};
use actix_web::{error::ErrorInternalServerError, post};
use actix_web::{web, HttpResponse};
use entity::file::Model as File;
use sea_orm::DatabaseConnection;
use serde::Deserialize;
use utoipa;

#[derive(Deserialize, Debug)]
pub struct SearchQuery {
    pub query: String,
    // sorting?? ordering??
}

#[utoipa::path(
    tag = "Search",
    context_path = "/api",
    responses(
        (status = 200, description = "Returns shit", body = Response<String>),
        (status = 401, description = "Access denied.", body = ErrorResponse),
    ),
    request_body(content = Lib, description = "Data needed to create a library.", content_type = "application/json"),
    security(("token" = []))
)]
#[post("/search")]
async fn search(
    data: web::Json<SearchQuery>,
    db: web::Data<DatabaseConnection>,
    VerifiedAuthData(_user): VerifiedAuthData,
) -> actix_web::Result<impl actix_web::Responder> {
    let items: Vec<File> = File::search(data.query.clone(), &db)
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(Response {
        status: "ok".to_string(),
        data: items,
    }))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(search);
}
