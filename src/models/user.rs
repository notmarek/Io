use crate::auth::Claims;
use argon2::{self, hash_encoded, verify_encoded, Config, ThreadMode, Variant, Version};
use async_trait::async_trait;
use entity::file_tokens;
use entity::prelude::FileTokens;
use entity::prelude::User;
use entity::user;
use sea_orm::prelude::*;
use std::fmt::Display;

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

#[async_trait]
#[allow(clippy::new_ret_no_self)]
pub trait UserActions {
    fn has_permission_one_of<T: Display>(&self, perms: Vec<T>) -> bool;
    async fn register(
        mut self,
        salt: String,
        db: &DatabaseConnection,
        token_validity: i64,
    ) -> Result<Claims, String>;
    async fn login(
        mut self,
        db: &DatabaseConnection,
        token_validity: i64,
    ) -> Result<Claims, String>;
    fn refresh(self, token_validity: i64) -> Claims;
    async fn can_access_with_file_token(file_token: String, db: &DatabaseConnection) -> bool;
    async fn get_file_token(&self, db: &DatabaseConnection) -> String;
    async fn get_all(limit: i64, page: i64, pool: &DatabaseConnection) -> Vec<user::Model>;
    async fn get(uuid: String, db: &DatabaseConnection) -> Result<user::Model, String>;
    fn new(username: String, password: String, permissions: Vec<String>) -> user::Model;
}

#[async_trait]
impl UserActions for user::Model {
    fn has_permission_one_of<T: Display>(&self, perms: Vec<T>) -> bool {
        perms
            .iter()
            .any(|p| self.permissions.split(',').any(|e| e == p.to_string()))
    }

    fn new(username: String, password: String, permissions: Vec<String>) -> user::Model {
        user::Model {
            id: Uuid::new_v4().to_string(),
            username,
            password,
            permissions: permissions.join(","),
        }
    }

    async fn get_file_token(&self, db: &DatabaseConnection) -> String {
        if let Ok(Some(ft)) = self.find_related(FileTokens).one(db).await {
            ft.token.clone()
        } else {
            "".to_string()
        }
    }

    async fn can_access_with_file_token(file_token: String, db: &DatabaseConnection) -> bool {
        let ft = match FileTokens::find()
            .filter(file_tokens::Column::Token.eq(&file_token))
            .one(db)
            .await
        {
            Ok(Some(ft)) => ft,
            _ => return false,
        };

        let user = match ft.find_related(User).one(db).await {
            Ok(Some(u)) => u,
            _ => return false,
        };
        !user.has_permission_one_of(vec!["banned"])
    }
    async fn get(uuid: String, db: &DatabaseConnection) -> Result<user::Model, String> {
        match User::find_by_id(&uuid).one(db).await {
            Ok(Some(u)) => Ok(u),
            Ok(None) => Err(String::from("invalid_user")),
            Err(e) => Err(e.to_string()),
        }
    }

    fn refresh(self, token_validity: i64) -> Claims {
        Claims::new(self.id, self.permissions, token_validity)
    }

    async fn get_all(_limit: i64, _page: i64, db: &DatabaseConnection) -> Vec<user::Model> {
        User::find().all(db).await.unwrap()
    }

    async fn register(
        mut self,
        salt: String,
        db: &DatabaseConnection,
        token_validity: i64,
    ) -> Result<Claims, String> {
        match User::find()
            .filter(user::Column::Username.eq(&self.username))
            .one(db)
            .await
        {
            Ok(Some(_)) => Err(String::from("username_exists")),
            Ok(None) => {
                self.password = hash_password(self.password.clone(), salt);
                let active: user::ActiveModel = self.clone().into();

                match active.insert(db).await {
                    Ok(_) => {
                        let ft = file_tokens::Model {
                            id: Uuid::new_v4().to_string(),
                            owner: self.id.clone(),
                            token: Uuid::new_v4().to_string().replace("-", ""),
                        };
                        let fta: file_tokens::ActiveModel = ft.into();
                        fta.insert(db).await.unwrap();

                        Ok(Claims::new(
                            self.id.clone(),
                            self.permissions.clone(),
                            token_validity,
                        ))
                    }
                    Err(e) => Err(e.to_string()),
                }
            }
            Err(e) => Err(e.to_string()),
        }
    }

    async fn login(
        mut self,
        db: &DatabaseConnection,
        token_validity: i64,
    ) -> Result<Claims, String> {
        let raw_password = self.password;
        match User::find()
            .filter(user::Column::Username.eq(self.username))
            .one(db)
            .await
        {
            Ok(Some(u)) => {
                self = u;
                if !verify_encoded(&self.password, raw_password.as_bytes()).unwrap() {
                    return Err(String::from("invalid_password"));
                }
                Ok(Claims::new(self.id, self.permissions, token_validity))
            }
            Ok(None) => Err(String::from("invalid_username")),
            Err(e) => Err(e.to_string()),
        }
    }
}
