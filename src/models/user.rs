use diesel::prelude::*;
use diesel::r2d2;
use crate::schema::users;
use serde::{Serialize, Deserialize};

#[derive(Debug, Queryable, Serialize, Deserialize, Insertable)]
#[table_name = "users"]
pub struct User {
    pub id: String,
    pub username: String,
    pub password: String,
    pub permissions: Vec<String>,
}

