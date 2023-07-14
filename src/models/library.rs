use crate::utils::indexer::crawl;
use async_trait::async_trait;
use entity::library::ActiveModel;
use entity::prelude::{File, Library};
use entity::{file, library};
use sea_orm::DatabaseConnection;
use sea_orm::{prelude::*, ActiveValue};
use std::path::Path;
use uuid::Uuid;

#[async_trait]
#[allow(clippy::new_ret_no_self)]
pub trait LibraryActions {
    async fn new(
        name: String,
        path: String,
        depth: i32,
        db: &DatabaseConnection,
    ) -> Result<library::Model, String>;
    async fn get(lib_id: Uuid, db: &DatabaseConnection) -> Result<library::Model, String>;
    async fn get_files(&self, db: &DatabaseConnection) -> Result<Vec<file::Model>, String>;
    async fn get_all(db: &DatabaseConnection) -> Result<Vec<library::Model>, String>;
    async fn delete(lib_id: Uuid, db: &DatabaseConnection) -> Result<u64, String>;
    async fn scan(&self, db: &DatabaseConnection);
}

#[async_trait]
impl LibraryActions for library::Model {
    async fn new(
        name: String,
        path: String,
        depth: i32,
        db: &DatabaseConnection,
    ) -> Result<library::Model, String> {
        match Library::find()
            .filter(library::Column::Path.eq(&path))
            .one(db)
            .await
        {
            Ok(Some(l)) => Ok(l),
            Ok(None) => {
                let active: library::ActiveModel = library::Model {
                    id: Uuid::new_v4(),
                    name,
                    path,
                    depth,
                    last_scan: chrono::NaiveDateTime::from_timestamp_opt(0, 1).unwrap(),
                }
                .into();
                active.insert(db).await.map_err(|e| e.to_string())
            }
            Err(e) => Err(e.to_string()),
        }
    }

    async fn get(lib_id: Uuid, db: &DatabaseConnection) -> Result<library::Model, String> {
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

    async fn delete(lib_id: Uuid, db: &DatabaseConnection) -> Result<u64, String> {
        match Library::delete_by_id(lib_id).exec(db).await {
            Ok(e) => Ok(e.rows_affected),
            Err(e) => Err(e.to_string()),
        }
    }

    async fn scan(&self, db: &DatabaseConnection) {
        log::info!("Scanning library: {}", self.name);
        let mut active: ActiveModel = self.clone().into();
        active.last_scan = ActiveValue::set(chrono::Utc::now().naive_local());
        active.update(db).await.unwrap();
        crawl(Path::new(&self.path), self.depth, db, self.id, None)
            .await
            .unwrap()
    }
}
