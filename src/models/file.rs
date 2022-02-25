use std::path::Path;

use crate::schema::files;
use crate::DBPool;
use crate::utils::indexer::scan_file;
use anitomy::Anitomy;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Serialize, Clone)]
pub struct File {
    pub id: i32,
    pub parent: String, // parent folder in relation to library root
    pub library_id: i32, // which library is this file a part of
    pub path: String, // in relation to library
    pub folder: bool, // is this a folder or a file?
    pub last_update: i64, // unix timestamp of last update
    pub title: Option<String>, // title extracted with Anitomy
    pub season: Option<String>, // season --//--
    pub episode: Option<f32>, // episode --//--
    pub release_group: Option<String>, // group --//--
}

#[derive(Debug, Deserialize, Insertable, Clone)]
#[table_name = "files"]
pub struct NewFile{
    pub parent: String, // parent folder in relation to library root
    pub library_id: i32, // which library is this file a part of
    pub path: String, // in relation to library
    pub folder: bool, // is this a folder or a file?
}

impl File {
    pub fn new(f_parent: String, f_library_id: i32, f_path: String, f_folder: bool, pool: &DBPool) -> Self {
        let db = pool.get().unwrap();
        use crate::schema::files::dsl::*;
        match files.filter(path.eq(&f_path)).first::<Self>(&db) {
            Ok(l) => l,
            Err(_) => {
                match diesel::insert_into(files)
                    .values(NewFile {
                        parent: f_parent,
                        library_id: f_library_id,
                        path: f_path,
                        folder: f_folder,
                    })
                    .get_result::<Self>(&db)
                {
                    Ok(l) => l,
                    Err(_) => panic!("What the fuck man."),
                }
            }
        }
    }

    pub fn scan(&mut self, pool: &DBPool) {
        let mut anitomy = Anitomy::new();
        scan_file(Path::new(&self.path), &mut anitomy);
        todo!("Use scan file response to upodate file.")
    }

}