use crate::{models::file::File, AuthData, DBPool, ErrorResponse, Response};
use actix_web::{error, web, HttpResponse};
use serde::Deserialize;

#[derive(Deserialize)]
struct FileId {
    file_id: String,
}

impl FileId {
    pub fn get(&self, pool: &DBPool) -> Result<File, String> {
        File::get(self.file_id.clone(), pool)
    }
}

#[actix_web::get("/file/{file_id}")]
async fn file(
    fid: web::Path<FileId>,
    pool: web::Data<DBPool>,
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
        }))
    }
    Ok(HttpResponse::Ok().json(Response {
        status: "ok".to_string(),
        data: file,
    }))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(file);
}
