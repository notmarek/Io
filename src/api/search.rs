use crate::{
    eventqueue::{QueueTrait, RawEvent},
    models::{library::LibraryActions, user::UserActions},
    ArcQueue, ErrorResponse, Response, VerifiedAuthData,
};
use actix_web::{delete, get, post, put};
use actix_web::{error, web, HttpResponse};
use entity::file::Model as File;
use entity::library::Model as Library;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use utoipa::{self, IntoParams, ToSchema};