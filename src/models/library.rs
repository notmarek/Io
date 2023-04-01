use crate::utils::indexer::crawl;
use anitomy::Anitomy;
use async_trait::async_trait;
use entity::prelude::{File, Library};
use entity::{file, library};
use log::error;
use sea_orm::prelude::*;
use sea_orm::DatabaseConnection;
use std::path::Path;
use uuid::Uuid;

#[async_trait]
pub trait LibraryActions {
    async fn new(
        lib_path: String,
        lib_depth: i32,
        db: &DatabaseConnection,
    ) -> Result<library::Model, String>;
    async fn get(lib_id: String, db: &DatabaseConnection) -> Result<library::Model, String>;
    async fn get_files(&self, db: &DatabaseConnection) -> Result<Vec<file::Model>, String>;
    async fn get_all(db: &DatabaseConnection) -> Result<Vec<library::Model>, String>;
    async fn delete(lib_id: String, db: &DatabaseConnection) -> Result<u64, String>;
    async fn crawl(&self, db: &DatabaseConnection);
}

#[async_trait]
impl LibraryActions for library::Model {
    async fn new(
        lib_path: String,
        lib_depth: i32,
        db: &DatabaseConnection,
    ) -> Result<library::Model, String> {
        match Library::find()
            .filter(library::Column::Path.eq(&lib_path))
            .one(db)
            .await
        {
            Ok(Some(l)) => Ok(l),
            Ok(None) => {
                let active: library::ActiveModel = library::Model {
                    id: Uuid::new_v4().to_string(),
                    path: lib_path,
                    depth: lib_depth,
                    last_scan: 0.to_string(),
                }
                .into();
                active.insert(db).await.map_err(|e| e.to_string())
            }
            Err(e) => Err(e.to_string()),
        }
    }

    async fn get(lib_id: String, db: &DatabaseConnection) -> Result<library::Model, String> {
        match Library::find_by_id(lib_id).one(db).await {
            Ok(Some(l)) => Ok(l),
            Ok(None) => Err("No such library was found.".to_string()),
            Err(e) => Err(e.to_string()),
        }
    }

    async fn get_files(&self, db: &DatabaseConnection) -> Result<Vec<file::Model>, String> {
        self.find_related(File)
            .all(db)
            .await
            .map_err(|e| e.to_string())
    }

    async fn get_all(db: &DatabaseConnection) -> Result<Vec<library::Model>, String> {
        Library::find().all(db).await.map_err(|e| e.to_string())
    }

    async fn delete(lib_id: String, db: &DatabaseConnection) -> Result<u64, String> {
        match Library::delete_by_id(lib_id).exec(db).await {
            Ok(e) => Ok(e.rows_affected),
            Err(e) => Err(e.to_string()),
        }
    }

    async fn crawl(&self, db: &DatabaseConnection) {
        let mut anitomy = Anitomy::new();
        match crawl(
            Path::new(&self.path),
            self.depth,
            &mut anitomy,
            db,
            self.id.clone(),
        ) {
            Ok(_) => (),
            Err(e) => error!("{}", e),
        }
    }
}
