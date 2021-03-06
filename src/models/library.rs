use std::path::Path;

use crate::schema::libraries;
use crate::utils::indexer::crawl;
use crate::DBPool;
use anitomy::Anitomy;
use diesel::prelude::*;
use log::error;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Queryable, Deserialize, Serialize, Insertable, Clone, PartialEq, Eq)]
#[table_name = "libraries"]
pub struct Library {
    pub id: String,
    pub path: String, // library root
    pub depth: i32,   // how deep should we scan? -1 for deepscan
    pub last_scan: i32,
}

impl Library {
    pub fn new(lib_path: String, lib_depth: i32, pool: &DBPool) -> Self {
        let db = pool.get().unwrap();
        use crate::schema::libraries::dsl::*;
        match libraries.filter(path.eq(&lib_path)).first::<Self>(&db) {
            Ok(l) => l,
            Err(_) => {
                match diesel::insert_into(libraries)
                    .values(Library {
                        id: Uuid::new_v4().to_string(),
                        path: lib_path,
                        depth: lib_depth,
                        last_scan: 0,
                    })
                    .get_result::<Self>(&db)
                {
                    Ok(l) => l,
                    Err(_) => panic!("What the fuck man."),
                }
            }
        }
    }

    pub fn get(lib_id: String, pool: &DBPool) -> Result<Self, String> {
        let db = pool.get().unwrap();
        use crate::schema::libraries::dsl::*;
        libraries
            .filter(id.eq(&lib_id))
            .first::<Self>(&db)
            .map_err(|_| String::from("not_found"))
    }

    pub fn get_files(&self, pool: &DBPool) -> Result<Vec<crate::models::file::File>, String> {
        let db = pool.get().unwrap();
        use crate::schema::files::dsl::*;
        files
            .filter(library_id.eq(&self.id))
            .get_results(&db)
            .map_err(|_| String::from("not_found"))
    }

    pub fn get_all(pool: &DBPool) -> Result<Vec<Self>, String> {
        let db = pool.get().unwrap();
        use crate::schema::libraries::dsl::*;
        libraries
            .load::<Self>(&db)
            .map_err(|_| String::from("unknown_error"))
    }

    pub fn delete(lib_id: String, pool: &DBPool) -> Result<usize, String> {
        let db = pool.get().unwrap();
        diesel::delete(
            crate::schema::files::dsl::files
                .filter(crate::schema::files::dsl::library_id.eq(&lib_id)),
        )
        .execute(&db)
        .map_err(|_| String::from("not_found"))?;
        use crate::schema::libraries::dsl::*;
        diesel::delete(libraries.find(&lib_id))
            .execute(&db)
            .map_err(|_| String::from("not_found"))
    }

    pub fn crawl(&self, pool: &DBPool) {
        let mut anitomy = Anitomy::new();
        match crawl(
            Path::new(&self.path),
            self.depth,
            &mut anitomy,
            pool,
            self.id.clone(),
        ) {
            Ok(_) => (),
            Err(e) => error!("{}", e),
        }
    }
}
