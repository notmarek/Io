use actix_web::{error, Error};
use chrono::{Duration, Utc};
use jsonwebtoken::{self, decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub user_id: String,
    pub perms: Vec<String>,
    exp: i64,
}

impl Claims {
    pub fn new(user_id: String, permissions: Vec<String>, valid_for: i64) -> Self {
        Self {
            user_id,
            perms: permissions,
            exp: (Utc::now() + Duration::hours(valid_for)).timestamp(),
        }
    }

    pub fn create_token(self, key_path: PathBuf) -> Result<String, Error> {
        let enc_key = EncodingKey::from_rsa_pem(&std::fs::read(key_path)?).unwrap();
        encode(&Header::new(Algorithm::RS512), &self, &enc_key)
            .map_err(|e| error::ErrorUnauthorized(e.to_string()))
    }

    pub fn from_token(token: &str, key_path: PathBuf) -> Result<Self, Error> {
        let dec_key = DecodingKey::from_rsa_pem(&std::fs::read(key_path)?).unwrap();
        decode::<Self>(token, &dec_key, &Validation::new(Algorithm::RS512))
            .map(|data| data.claims)
            .map_err(|e| error::ErrorUnauthorized(e.to_string()))
    }
}
