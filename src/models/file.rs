use crate::utils::indexer::scan_file;
use async_trait::async_trait;
use entity::file;
use entity::prelude::File;
use sea_orm::{prelude::*, ActiveModelTrait, DatabaseConnection, EntityTrait, QueryFilter};
use std::path::Path;
use uuid::Uuid;

#[async_trait]
#[allow(clippy::new_ret_no_self)]
pub trait FileActions {
    async fn new(
        parent: String,
        library_id: String,
        path: String,
        folder: bool,
        db: &DatabaseConnection,
    ) -> Result<file::Model, String>;
    async fn get(fid: String, db: &DatabaseConnection) -> Result<file::Model, String>;
    async fn scan(&mut self, db: &DatabaseConnection);
    async fn get_folder_content(&self, db: &DatabaseConnection)
        -> Result<Vec<file::Model>, String>;
}

#[async_trait]
impl FileActions for file::Model {
    async fn new(
        parent: String,
        library_id: String,
        path: String,
        folder: bool,
        db: &DatabaseConnection,
    ) -> Result<Self, String> {
        match File::find()
            .filter(file::Column::Path.eq(&path))
            .one(db)
            .await
        {
            Ok(Some(f)) => Ok(f),
            Ok(None) => {
                let active: file::ActiveModel = file::Model {
                    id: Uuid::new_v4().to_string(),
                    parent,
                    library_id,
                    path,
                    folder,
                    last_update: chrono::NaiveDateTime::MIN,
                    ..Default::default()
                }
                .into();
                active.insert(db).await.map_err(|e| e.to_string())
            }
            Err(e) => Err(e.to_string()),
        }
    }

    async fn get(fid: String, db: &DatabaseConnection) -> Result<file::Model, String> {
        match File::find_by_id(fid).one(db).await {
            Ok(Some(f)) => Ok(f),
            Ok(None) => Err("No such file could be found.".to_string()),
            Err(e) => Err(e.to_string()),
        }
    }

    async fn scan(&mut self, db: &DatabaseConnection) {
        if self.size.is_some() {
            return;
        }
        if let Ok(scanned) = scan_file(Path::new(&self.path)).await {
            self.title = scanned.title;
            self.season = scanned.season;
            self.episode = scanned.episode;
            self.release_group = scanned.release_group;
            self.size = scanned.size;
            log::info!("{:#?}", self);
            let mut active: file::ActiveModel = self.clone().into();
            active.last_update = sea_orm::ActiveValue::set(scanned.last_update);
            log::info!("active : {:#?}", active);
            active.update(db).await.unwrap();
        }
    }

    async fn get_folder_content(
        &self,
        db: &DatabaseConnection,
    ) -> Result<Vec<file::Model>, String> {
        File::find()
            .filter(file::Column::Parent.eq(&self.path))
            .all(db)
            .await
            .map_err(|e| e.to_string())
    }
}
