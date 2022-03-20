use crate::{models::file::File, AuthData, DBPool, ErrorResponse, Response};
use actix_web::{error, web, HttpResponse};
use serde::Deserialize;

#[derive(Deserialize)]
struct FileId {
    file_id: String,
}

#[actix_web::get("/file/{file_id}")]
async fn file(
    path: web::Path<FileId>,
    pool: web::Data<DBPool>,
    AuthData(_user): AuthData,
) -> impl actix_web::Responder {

    ""
}
