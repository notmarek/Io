use crate::config::Config;
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

#[derive(Deserialize)]
struct FileToken {
    #[serde(rename = "t")]
    pub token: Option<String>,
}

impl FileId {
    pub async fn get(&self, pool: &DatabaseConnection) -> Result<File, String> {
        File::get(self.file_id, pool).await
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Nginx {
    #[serde(rename = "t")]
    pub token: Option<String>,
    #[serde(rename = "u")]
    pub uri: Option<Uuid>,
}
#[get("/playlist/{file_id}.m3u")]
async fn playlist(
    fid: web::Path<FileId>,
    file_token: web::Query<FileToken>,
    db: web::Data<DatabaseConnection>,
    config: web::Data<Config>,
) -> actix_web::Result<impl actix_web::Responder> {
    if !User::can_access_with_file_token(file_token.token.clone().unwrap(), &db).await {
        return Err(error::ErrorUnauthorized(ErrorResponse {
            status: "error".to_string(),
            error: "Unauthorized".to_string(),
        }));
    }
    let the_file = fid.get(&db).await;
    if let Err(e) = the_file {
        return Err(error::ErrorNotFound(ErrorResponse {
            status: "error".to_string(),
            error: e,
        }));
    };
    let the_file = the_file.unwrap();
    if !the_file.folder {
        return Ok("Hellnah".to_string());
    }
    let mut pl = String::from("#EXTM3U\n\n");
    let folder_content = the_file.get_folder_content(&db).await.unwrap();

    for f in folder_content {
        if !f.folder {
            pl += &format!(
                "#EXTINF:0,[{}] {} - {}\n{}/{}?t={}\n\n",
                f.release_group.unwrap_or("UNK".to_string()),
                f.title.unwrap_or(f.path),
                f.episode.unwrap_or(0),
                config.info.storage_url,
                f.id,
                file_token.token.clone().unwrap(),
            );
        }
    }

    Ok(pl)
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
    let file_id = match req_data.uri {
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
    cfg.service(nginx).service(playlist);
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(file);
}
