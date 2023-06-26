use crate::utils::indexer::scan_file;
use async_trait::async_trait;
use entity::file::{self, SelfReferencingLink};
use entity::prelude::File;
use sea_orm::{
    prelude::*, ActiveModelTrait, ActiveValue, DatabaseConnection, EntityTrait, QueryFilter,
};
use std::path::Path;
use uuid::Uuid;

#[async_trait]
#[allow(clippy::new_ret_no_self)]
pub trait FileActions {
    async fn new(
        parent_file_id: Option<String>,
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
        parent_file_id: Option<String>,
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
                    parent: parent_file_id,
                    library_id,
                    // parent_file_id,
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
            let mut active: file::ActiveModel = self.clone().into();
            active.title = ActiveValue::set(scanned.title);
            active.season = ActiveValue::set(scanned.season);
            active.episode = ActiveValue::set(scanned.episode);
            active.release_group = ActiveValue::set(scanned.release_group);
            active.size = ActiveValue::set(scanned.size);
            active.last_update = ActiveValue::set(scanned.last_update);
            active.update(db).await.unwrap();
        }
    }

    async fn get_folder_content(
        &self,
        db: &DatabaseConnection,
    ) -> Result<Vec<file::Model>, String> {
        // self.find_related(File)
        //     .all(db)
        //     .await
        //     .map_err(|e| e.to_string())

        self.find_linked(SelfReferencingLink)
            .all(db)
            .await
            .map_err(|e| e.to_string())
    }
}
