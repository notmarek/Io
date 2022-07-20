use std::path::Path;

use crate::schema::files;
use crate::utils::indexer::scan_file;
use crate::DBPool;
use anitomy::Anitomy;
use diesel::{prelude::*, AsChangeset, Identifiable};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Queryable, Serialize, Clone, Identifiable, AsChangeset)]
#[table_name = "files"]
pub struct File {
    pub id: String,
    pub parent: String,                // parent folder in relation to library root
    pub library_id: String,            // which library is this file a part of
    pub path: String,                  // in relation to library
    pub folder: bool,                  // is this a folder or a file?
    pub last_update: i64,              // unix timestamp of last update
    pub title: Option<String>,         // title extracted with Anitomy
    pub season: Option<String>,        // season --//--
    pub episode: Option<f32>,          // episode --//--
    pub release_group: Option<String>, // group --//--
}

#[derive(AsChangeset, Clone)]
#[table_name = "files"]
pub struct FileChangeset {
    pub last_update: i64,              // unix timestamp of last update
    pub title: Option<String>,         // title extracted with Anitomy
    pub season: Option<String>,        // season --//--
    pub episode: Option<f32>,          // episode --//--
    pub release_group: Option<String>, // group --//--
}

#[derive(Debug, Deserialize, Insertable, Clone)]
#[table_name = "files"]
pub struct NewFile {
    pub id: String,
    pub parent: String,     // parent folder in relation to library root
    pub library_id: String, // which library is this file a part of
    pub path: String,       // in relation to library
    pub folder: bool,       // is this a folder or a file?
    pub last_update: i64,
}

impl File {
    pub fn new(
        f_parent: String,
        f_library_id: String,
        f_path: String,
        f_folder: bool,
        pool: &DBPool,
    ) -> Self {
        let db = pool.get().unwrap();
        use crate::schema::files::dsl::*;
        match files.filter(path.eq(&f_path)).first::<Self>(&db) {
            Ok(l) => l,
            Err(_) => {
                match diesel::insert_into(files)
                    .values(NewFile {
                        id: Uuid::new_v4().to_string(),
                        parent: f_parent,
                        library_id: f_library_id,
                        path: f_path,
                        folder: f_folder,
                        last_update: 0,
                    })
                    .get_result::<Self>(&db)
                {
                    Ok(l) => l,
                    Err(e) => panic!("What the fuck man. {}", e),
                }
            }
        }
    }

    pub fn get(fid: String, pool: &DBPool) -> Result<Self, String> {
        let db = pool.get().unwrap();
        use crate::schema::files::dsl::*;
        files
            .filter(id.eq(fid))
            .first::<Self>(&db)
            .map_err(|_| String::from("not_found"))
    }

    pub fn scan(&mut self, pool: &DBPool) {
        let mut anitomy = Anitomy::new();
        if let Ok(scanned) = scan_file(Path::new(&self.path), &mut anitomy) {
            self.last_update = scanned.last_update;
            self.title = scanned.title;
            self.season = scanned.season;
            self.episode = scanned.episode;
            self.release_group = scanned.release_group;
            let db = pool.get().unwrap();
            *self = self.save_changes::<Self>(&*db).unwrap();
        }

        todo!("handle errors in scan_file")
    }

    pub fn get_folder_content(&self, pool: &DBPool) -> Vec<Self> {
        let db = pool.get().unwrap();
        use crate::schema::files::dsl::*;
        files
            .filter(parent.eq(path))
            .get_results::<Self>(&db)
            .unwrap()
    }
}
