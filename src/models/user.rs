use std::fmt::Display;

use crate::auth::Claims;
use crate::schema::users;
use crate::DBPool;
use argon2::{self, hash_encoded, verify_encoded, Config, ThreadMode, Variant, Version};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Queryable, Serialize, Deserialize, Insertable, Clone)]
#[diesel(table_name = users)]
pub struct User {
    pub id: String,
    pub username: String,
    pub password: String,
    pub permissions: Vec<String>,
}

pub fn hash_password(password: String, salt: String) -> String {
    let config = Config {
        variant: Variant::Argon2id,
        version: Version::Version13,
        mem_cost: 65536,
        time_cost: 4,
        lanes: 4,
        thread_mode: ThreadMode::Sequential,
        secret: &[],
        ad: &[],
        hash_length: 32,
    };
    hash_encoded(password.as_bytes(), salt.as_bytes(), &config).unwrap()
}

impl User {
    pub fn new(username: String, password: String, permissions: Vec<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            username,
            password,
            permissions,
        }
    }

    pub fn get(uuid: String, pool: &DBPool) -> Result<Self, String> {
        let mut db = pool.get().unwrap();
        use crate::schema::users::dsl::*;
        match users.filter(id.eq(&uuid)).first::<Self>(&mut db) {
            Ok(u) => {
                if !u.has_permission_one_of(vec!["verified"]) {
                    Err(String::from("unverified_user"))
                } else {
                    Ok(u)
                }
            }
            Err(_) => Err(String::from("invalid_user")),
        }
    }

    pub fn get_all(limit: i64, page: i64, pool: &DBPool) -> Vec<Self> {
        let mut db = pool.get().unwrap();
        use crate::schema::users::dsl::*;
        users
            .limit(limit)
            .offset((page - 1) * limit)
            .load(&mut db)
            .unwrap()
    }

    pub fn refresh(self, token_validity: i64) -> Claims {
        Claims::new(self.id, self.permissions, token_validity)
    }

    pub fn login(mut self, pool: &DBPool, token_validity: i64) -> Result<Claims, String> {
        let mut db = pool.get().unwrap();
        let raw_password = self.password;
        use crate::schema::users::dsl::*;
        match users.filter(username.eq(&self.username)).first::<Self>(&mut db) {
            Ok(u) => {
                self = u;
                if !verify_encoded(&self.password, raw_password.as_bytes()).unwrap() {
                    return Err(String::from("invalid_password"));
                }
                Ok(Claims::new(self.id, self.permissions, token_validity))
            }
            Err(_) => Err(String::from("invalid_username")),
        }
    }

    pub fn register(
        mut self,
        salt: String,
        pool: &DBPool,
        token_validity: i64,
    ) -> Result<Claims, String> {
        let mut db = pool.get().unwrap();
        use crate::schema::users::dsl::*;
        match users.filter(username.eq(&self.username)).first::<Self>(&mut db) {
            Ok(_) => Err(String::from("username_exists")),
            Err(_) => {
                self.password = hash_password(self.password, salt);
                match diesel::insert_into(users)
                    .values(self.clone())
                    .get_result::<Self>(&mut db)
                {
                    Ok(_) => Ok(Claims::new(self.id, self.permissions, token_validity)),
                    Err(e) => Err(format!("{}", e)),
                }
            }
        }
    }

    pub fn has_permission_one_of<T: Display>(&self, perms: Vec<T>) -> bool {
        perms
            .iter()
            .any(|p| self.permissions.iter().any(|e| e == &p.to_string()))
    }
}
