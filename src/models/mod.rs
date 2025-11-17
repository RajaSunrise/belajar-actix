use sqlx::FromRow;
use serde::{Serialize, Deserialize};

#[derive(FromRow, Serialize)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
}

#[derive(FromRow, Deserialize)]
pub struct NewUser {
    pub name: String,
    pub email: String,
}