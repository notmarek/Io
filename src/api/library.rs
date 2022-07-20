use crate::{
    eventqueue::{QueueTrait, RawEvent},
    models::{file::File, library::Library},
    ArcQueue, AuthData, DBPool, ErrorResponse, Response,
};
use actix_web::{error, web, HttpResponse};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct LibId {
    library_id: String,
}

#[actix_web::get("/library/all")]
async fn libraries(
    pool: web::Data<DBPool>,
    AuthData(_user): AuthData,
) -> impl actix_web::Responder {
    let libraries = {
        match Library::get_all(&pool) {
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
    #[derive(Serialize)]
    struct Bruh {
        library_info: Library,
        files: Vec<File>,
    }
    let library = {
        match Library::get(path.library_id.clone(), &pool) {
            Ok(u) => Response {
                status: "ok".to_string(),
                data: Bruh {
                    library_info: u.clone(),
                    files: u.get_files(&pool).unwrap(),
                },
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

#[actix_web::post("/library/{library_id}/scan")]
async fn scan_library(
    path: web::Path<LibId>,
    pool: web::Data<DBPool>,
    queue: web::Data<ArcQueue>,
    AuthData(user): AuthData,
) -> impl actix_web::Responder {
    if !user.has_permission_one_of(vec!["scan_library", "*_library", "administrator"]) {
        return Err(error::ErrorForbidden(ErrorResponse {
            status: "error".to_string(),
            error: "missing_permissions".to_string(),
        }));
    }
    match Library::get(path.library_id.clone(), &pool) {
        Ok(u) => queue
            .lock()
            .unwrap()
            .add_event(RawEvent::ScanLibrary { library: u }, 10),
        Err(e) => {
            return Err(error::ErrorNotFound(ErrorResponse {
                status: "error".to_string(),
                error: e,
            }))
        }
    }
    Ok(HttpResponse::Ok().json(Response {
        status: "ok".to_string(),
        data: "Started scanning",
    }))
}

#[actix_web::delete("/library/{library_id}")]
async fn delete_library(
    path: web::Path<LibId>,
    pool: web::Data<DBPool>,
    AuthData(user): AuthData,
) -> impl actix_web::Responder {
    if !user.has_permission_one_of(vec!["delete_library", "*_library", "administrator"]) {
        return Err(error::ErrorForbidden(ErrorResponse {
            status: "error".to_string(),
            error: "missing_permissions".to_string(),
        }));
    }
    match Library::delete(path.library_id.clone(), &pool) {
        Ok(u) => Ok(HttpResponse::Ok().json(Response {
            status: "ok".to_string(),
            data: u,
        })),
        Err(e) => Err(error::ErrorNotFound(ErrorResponse {
            status: "error".to_string(),
            error: e,
        })),
    }
}

#[derive(Deserialize)]
struct Lib {
    path: String,
    depth: i32,
}

#[actix_web::put("/library")]
async fn create_library(
    data: web::Json<Lib>,
    pool: web::Data<DBPool>,
    AuthData(user): AuthData,
) -> impl actix_web::Responder {
    if !user.has_permission_one_of(vec!["create_library", "*_library", "administrator"]) {
        return Err(error::ErrorForbidden(ErrorResponse {
            status: "error".to_string(),
            error: "missing_permissions".to_string(),
        }));
    }
    let lib = Library::new(data.path.clone(), data.depth, &pool);
    Ok(HttpResponse::Ok().json(Response {
        status: "ok".to_string(),
        data: lib,
    }))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(libraries)
        .service(library)
        .service(create_library)
        .service(scan_library)
        .service(delete_library);
}
