use crate::auth::Claims;
use crate::schema::users;
use crate::DBPool;
use argon2::{self, hash_encoded, Config, ThreadMode, Variant, Version};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Queryable, Serialize, Deserialize, Insertable, Clone)]
#[table_name = "users"]
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

    pub fn register(
        mut self,
        salt: String,
        pool: &DBPool,
        token_validity: i64,
    ) -> Result<Claims, String> {
        self.password = hash_password(self.password, salt);
        let db = pool.get().unwrap();
        use crate::schema::users::dsl::*;
        match users.filter(username.eq(&self.username)).first::<Self>(&db) {
            Ok(_) => Err(String::from("Username already taken.")),
            Err(_) => {
                match diesel::insert_into(users)
                    .values(self.clone())
                    .get_result::<Self>(&db)
                {
                    Ok(_) => Ok(Claims::new(
                        self.id,
                        self.permissions,
                        token_validity,
                    )),
                    Err(e) => Err(format!("{}", e)),
                }
            }
        }
    }
}
