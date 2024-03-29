use crate::{
    eventqueue::{QueueTrait, RawEvent},
    models::{file::FileActions, library::LibraryActions, user::UserActions},
    ArcQueue, ErrorResponse, Response, VerifiedAuthData,
};
use actix_web::{delete, get, post, put};
use actix_web::{error, web, HttpResponse};
use entity::file::Model as File;
use entity::library::Model as Library;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use utoipa::{self, IntoParams, ToSchema};
use uuid::Uuid;

#[derive(IntoParams, Deserialize)]
pub struct LibId {
    library_id: Uuid,
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
    db: web::Data<DatabaseConnection>,
    VerifiedAuthData(_user): VerifiedAuthData,
) -> impl actix_web::Responder {
    let libraries = {
        match Library::get_all(&db).await {
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
    db: web::Data<DatabaseConnection>,
    VerifiedAuthData(_user): VerifiedAuthData,
) -> impl actix_web::Responder {
    #[derive(Serialize)]
    struct Bruh {
        library_info: Library,
        root_file: File,
        files: Vec<File>,
    }
    let library = {
        match Library::get(path.library_id, &db).await {
            Ok(u) => Response {
                status: "ok".to_string(),
                data: Bruh {
                    library_info: u.clone(),
                    root_file: File::get_from_path(u.path.clone(), &db).await.unwrap(),
                    files: u.get_files(&db).await.unwrap(),
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
    db: web::Data<DatabaseConnection>,
    queue: web::Data<ArcQueue>,
    VerifiedAuthData(user): VerifiedAuthData,
) -> impl actix_web::Responder {
    if !user.has_permission_one_of(vec!["scan_library", "*_library", "administrator"]) {
        return Err(error::ErrorForbidden(ErrorResponse {
            status: "error".to_string(),
            error: "missing_permissions".to_string(),
        }));
    }
    match Library::get(path.library_id, &db).await {
        Ok(u) => queue
            .lock()
            .await
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
    db: web::Data<DatabaseConnection>,
    VerifiedAuthData(user): VerifiedAuthData,
) -> impl actix_web::Responder {
    if !user.has_permission_one_of(vec!["delete_library", "*_library", "administrator"]) {
        return Err(error::ErrorForbidden(ErrorResponse {
            status: "error".to_string(),
            error: "missing_permissions".to_string(),
        }));
    }
    match Library::delete(path.library_id, &db).await {
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
    name: String,
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
    db: web::Data<DatabaseConnection>,
    VerifiedAuthData(user): VerifiedAuthData,
) -> impl actix_web::Responder {
    if !user.has_permission_one_of(vec!["create_library", "*_library", "administrator"]) {
        return Err(error::ErrorForbidden(ErrorResponse {
            status: "error".to_string(),
            error: "missing_permissions".to_string(),
        }));
    }
    let lib = Library::new(data.name.clone(), data.path.clone(), data.depth, &db).await;
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
