use actix_http::HttpMessage;
use actix_web::{dev::ServiceRequest, error, web::Data, Error};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use chrono::{Duration, Utc};
use jsonwebtoken::{self, decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::path::Path;
use crate::AuthData;
use crate::{config::Config, models::user::User, DBPool};

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub user_id: String,
    pub perms: Vec<String>,
    pub exp: i64,
}

impl Claims {
    pub fn new(user_id: String, permissions: Vec<String>, valid_for: i64) -> Self {
        Self {
            user_id,
            perms: permissions,
            exp: (Utc::now() + Duration::hours(valid_for)).timestamp(),
        }
    }

    pub fn create_token(&self, key_path: &Path) -> Result<String, Error> {
        let key = std::fs::read(key_path)?;
        let enc_key = EncodingKey::from_rsa_pem(&key).unwrap();
        encode(&Header::new(Algorithm::RS512), &self, &enc_key)
            .map_err(|e| error::ErrorUnauthorized(e.to_string()))
    }

    pub fn create_refresh_token(&self, key_path: &Path) -> Result<String, Error> {
        let key = std::fs::read(key_path)?;
        let enc_key = EncodingKey::from_rsa_pem(&key).unwrap();
        encode(
            &Header::new(Algorithm::RS512),
            &Self {
                user_id: self.user_id.clone(),
                perms: vec!["REFRESH".to_string()],
                exp: (Utc::now() + Duration::hours(24 * 93)).timestamp(),
            },
            &enc_key,
        )
        .map_err(|e| error::ErrorUnauthorized(e.to_string()))
    }

    pub fn from_token(token: &str, key_path: &Path) -> Result<Self, Error> {
        let key = std::fs::read(key_path).unwrap();
        let dec_key = DecodingKey::from_rsa_pem(&key).unwrap();
        decode::<Self>(token, &dec_key, &Validation::new(Algorithm::RS512))
            .map(|data| data.claims)
            .map_err(|e| error::ErrorUnauthorized(e.to_string()))
    }
}

pub async fn validator(req: ServiceRequest, creds: BearerAuth) -> Result<ServiceRequest, Error> {
    let token = creds.token();
    let config = req.app_data::<Data<Config>>().unwrap();
    let pool = req.app_data::<Data<DBPool>>().unwrap();
    let claims = Claims::from_token(token, &config.jwt.public_key)?;

    // Check if user exists/is banned
    match User::get(claims.user_id, pool) {
        Ok(u) => {
            if u.permissions.contains(&"banned".to_string()) {
                return Err(error::ErrorUnauthorized("banned_user".to_string()));
            }
            req.extensions_mut().insert(AuthData(u));
        }
        Err(e) => return Err(error::ErrorUnauthorized(e)),
    }

    Ok(req)
}
