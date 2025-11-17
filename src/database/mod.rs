use sqlx::postgres::PgPool;
use dotenvy::dotenv;
use std::env;

pub async fn establish_connection() -> Result<PgPool, sqlx::Error> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    PgPool::connect(&database_url).await
}