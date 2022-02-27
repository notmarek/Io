use crate::{models::library::Library, AuthData, DBPool, ErrorResponse, Response};
use actix_web::{error, web, HttpResponse};
use serde::Deserialize;

#[derive(Deserialize)]
struct LibId {
    library_id: String,
}

#[actix_web::get("/library/all")]
async fn libraries(
    path: web::Path<LibId>,
    pool: web::Data<DBPool>,
    AuthData(_user): AuthData,
) -> impl actix_web::Responder {
    let libraries = {
        match Library::get_all(path.library_id.clone(), &pool) {
            Ok(u) => Response {
                status: "ok".to_string(),
                data: u,
            },
            Err(e) => {
                return Err(error::ErrorNotFound(ErrorResponse {
                    status: "error".to_string(),
                    error: e,
                }))
            }
        }
    };

    Ok(HttpResponse::Ok().json(libraries))
}

#[actix_web::get("/library/{library_id}")]
async fn library(
    path: web::Path<LibId>,
    pool: web::Data<DBPool>,
    AuthData(_user): AuthData,
) -> impl actix_web::Responder {
    let library = {
        match Library::get(path.library_id.clone(), &pool) {
            Ok(u) => Response {
                status: "ok".to_string(),
                data: u,
            },
            Err(e) => {
                return Err(error::ErrorNotFound(ErrorResponse {
                    status: "error".to_string(),
                    error: e,
                }))
            }
        }
    };

    Ok(HttpResponse::Ok().json(library))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(libraries).service(library);
}
