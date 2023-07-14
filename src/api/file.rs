use crate::{
    models::{file::FileActions, user::UserActions},
    ErrorResponse, Response, VerifiedAuthData,
};
use actix_web::{error, get, post, web, HttpResponse};
use entity::file::Model as File;
use entity::user::Model as User;
use log::info;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use utoipa::{self, IntoParams};
use uuid::Uuid;

#[derive(Deserialize, IntoParams)]
struct FileId {
    file_id: Uuid,
}

impl FileId {
    pub async fn get(&self, pool: &DatabaseConnection) -> Result<File, String> {
        File::get(self.file_id.clone(), pool).await
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Nginx {
    #[serde(rename = "t")]
    pub token: Option<String>,
    #[serde(rename = "u")]
    pub uri: Option<Uuid>,
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
    db: web::Data<DatabaseConnection>,
    VerifiedAuthData(_user): VerifiedAuthData,
) -> impl actix_web::Responder {
    let file = fid.get(&db).await;
    info!("{:?}", file);
    if let Err(e) = file {
        return Err(error::ErrorNotFound(ErrorResponse {
            status: "error".to_string(),
            error: e,
        }));
    };
    let file = file.unwrap();
    #[derive(Serialize)]
    struct Folder {
        folder: File,
        children: Vec<File>,
    }
    if file.folder {
        return Ok(HttpResponse::Ok().json(Response {
            status: "ok".to_string(),
            data: Folder {
                folder: file.clone(),
                children: file.get_folder_content(&db).await.unwrap(),
            },
        }));
    }
    Ok(HttpResponse::Ok().json(Response {
        status: "ok".to_string(),
        data: file,
    }))
}

#[post("/file/nginx")]
async fn nginx(
    req_data: web::Json<Nginx>,
    db: web::Data<DatabaseConnection>,
) -> actix_web::Result<impl actix_web::Responder> {
    if !User::can_access_with_file_token(req_data.token.clone().unwrap(), &db).await {
        return Err(error::ErrorUnauthorized(ErrorResponse {
            status: "error".to_string(),
            error: "".to_string(),
        }));
    }
    let file_id = match req_data.uri.clone() {
        Some(u) => FileId { file_id: u },
        _ => return Err(error::ErrorUnauthorized("nope")),
    };
    let f = file_id.get(&db).await;
    if let Err(e) = f {
        return Err(error::ErrorNotFound(ErrorResponse {
            status: "error".to_string(),
            error: e,
        }));
    };
    let f = f.unwrap();

    Ok(HttpResponse::Ok()
        .insert_header(("X-Path", f.path))
        .finish())
}

pub fn configure_na(cfg: &mut web::ServiceConfig) {
    cfg.service(nginx);
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(file);
}
