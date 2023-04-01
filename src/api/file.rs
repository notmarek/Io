use crate::{models::file::File, AuthData, ErrorResponse, Response};
use actix_web::{error, get, web, HttpResponse};
use sea_orm::DatabaseConnection;
use serde::Deserialize;
use utoipa::{self, IntoParams};

#[derive(Deserialize, IntoParams)]
struct FileId {
    file_id: String,
}

impl FileId {
    pub fn get(&self, pool: &DatabaseConnection) -> Result<File, String> {
        File::get(self.file_id.clone(), pool)
    }
}

#[utoipa::path(
    tag = "File",
    context_path = "/api",
    responses(
        (status = 200, description = "Returns a response confirming deletion.", body = Response),
        (status = 401, description = "Access denied.", body = ErrorResponse),
        (status = 404, description = "Not found.", body = ErrorResponse)
    ),
    params(FileId),
    security(("token" = []))
)]
#[get("/file/{file_id}")]
async fn file(
    fid: web::Path<FileId>,
    pool: web::Data<DatabaseConnection>,
    AuthData(_user): AuthData,
) -> impl actix_web::Responder {
    let file = fid.get(&pool);
    if let Err(e) = file {
        return Err(error::ErrorNotFound(ErrorResponse {
            status: "error".to_string(),
            error: e,
        }));
    };
    let file = file.unwrap();
    if file.folder {
        return Ok(HttpResponse::Ok().json(Response {
            status: "ok".to_string(),
            data: file.get_folder_content(&pool),
        }));
    }
    Ok(HttpResponse::Ok().json(Response {
        status: "ok".to_string(),
        data: file,
    }))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(file);
}
