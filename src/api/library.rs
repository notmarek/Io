use crate::{
    eventqueue::{QueueTrait, RawEvent},
    models::{file::File, library::Library},
    ArcQueue, AuthData, DatabaseConnection, ErrorResponse, Response,
};
use actix_web::{delete, get, post, put};
use actix_web::{error, web, HttpResponse};
use serde::{Deserialize, Serialize};
use utoipa::{self, IntoParams, ToSchema};

#[derive(IntoParams, Deserialize)]
pub struct LibId {
    library_id: String,
}

/// List all libraries
#[utoipa::path(
    tag = "Library",
    context_path = "/api",
    responses(
        (status = 200, description = "Returns a response with an array of libraries.", body = Response<Vec<Library>>),
        (status = 401, description = "Access denied.", body = ErrorResponse),
        (status = 404, description = "Not found.", body = ErrorResponse)
    ),
    security(("token" = []))
)]
#[get("/library/all")]
async fn libraries(
    pool: web::Data<DatabaseConnection>,
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

/// Get a library by id
#[utoipa::path(
    tag = "Library",
    context_path = "/api",
    responses(
        (status = 200, description = "Returns a response with a library and its contents.", body = Response<Library>),
        (status = 401, description = "Access denied.", body = ErrorResponse),
        (status = 404, description = "Not found.", body = ErrorResponse)
    ),
    params(LibId),
    security(("token" = []))
)]
#[get("/library/{library_id}")]
async fn library(
    path: web::Path<LibId>,
    pool: web::Data<DatabaseConnection>,
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

/// Start a library scan
#[utoipa::path(
    tag = "Library",
    context_path = "/api",
    responses(
        (status = 200, description = "Returns a response confirming scan.", body = Response),
        (status = 401, description = "Access denied.", body = ErrorResponse),
        (status = 404, description = "Not found.", body = ErrorResponse)
    ),
    params(LibId),
    security(("token" = []))
)]
#[post("/library/{library_id}/scan")]
async fn scan_library(
    path: web::Path<LibId>,
    pool: web::Data<DatabaseConnection>,
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
            .add_event(RawEvent::ScanLibraryEvent { library: u }, 10),
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

/// Delete a library
#[utoipa::path(
    tag = "Library",
    context_path = "/api",
    responses(
        (status = 200, description = "Returns a response confirming deletion.", body = Response),
        (status = 401, description = "Access denied.", body = ErrorResponse),
        (status = 404, description = "Not found.", body = ErrorResponse)
    ),
    params(LibId),
    security(("token" = []))
)]
#[delete("/library/{library_id}")]
async fn delete_library(
    path: web::Path<LibId>,
    pool: web::Data<DatabaseConnection>,
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

#[derive(Deserialize, ToSchema)]
pub struct Lib {
    path: String,
    depth: i32,
}

/// Create a library
#[utoipa::path(
    tag = "Library",
    context_path = "/api",
    responses(
        (status = 200, description = "Returns a response including the newly created library", body = Response<Library>),
        (status = 401, description = "Access denied.", body = ErrorResponse),
    ),
    request_body(content = Lib, description = "Data needed to create a library.", content_type = "application/json"),
    security(("token" = []))
)]
#[put("/library")]
async fn create_library(
    data: web::Json<Lib>,
    pool: web::Data<DatabaseConnection>,
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
