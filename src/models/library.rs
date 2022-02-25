use std::path::Path;

use crate::schema::libraries;
use crate::DBPool;
use crate::utils::indexer::crawl;
use anitomy::Anitomy;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Serialize, Clone)]
pub struct Library {
    pub id: i32,
    pub path: String, // library root
    pub depth: i32,   // how deep should we scan? -1 for deepscan
}

#[derive(Debug, Deserialize, Insertable, Clone)]
#[table_name = "libraries"]
pub struct NewLibrary {
    pub path: String, // library root
    pub depth: i32,   // how deep should we scan? -1 for deepscan
}

impl Library {
    pub fn new(lib_path: String, lib_depth: i32, pool: &DBPool) -> Self {
        let db = pool.get().unwrap();
        use crate::schema::libraries::dsl::*;
        match libraries.filter(path.eq(&lib_path)).first::<Self>(&db) {
            Ok(l) => l,
            Err(_) => {
                match diesel::insert_into(libraries)
                    .values(NewLibrary {
                        path: lib_path,
                        depth: lib_depth,
                    })
                    .get_result::<Self>(&db)
                {
                    Ok(l) => l,
                    Err(_) => panic!("What the fuck man."),
                }
            }
        }
    }

    pub fn remove(&self, pool: &DBPool) {
        todo!("Create a remove method.")
    }

    pub fn scan(&self, pool: &DBPool) {
        let mut anitomy = Anitomy::new();
        crawl(Path::new(&self.path), self.depth, &mut anitomy).unwrap()
    }
}
