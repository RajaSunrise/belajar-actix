use redis::AsyncCommands;
use std::env;

pub type RedisPool = redis::Client;

pub async fn init_redis() -> RedisPool {
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());
    redis::Client::open(redis_url).expect("Invalid Redis URL")
}

pub async fn store_refresh_token(client: &RedisPool, user_id: &str, token: &str, ttl_seconds: usize) -> Result<(), redis::RedisError> {
    let mut con = client.get_multiplexed_async_connection().await?;
    let key = format!("refresh:{}", user_id);
    con.set_ex(key, token, ttl_seconds as u64).await
}

pub async fn get_refresh_token(client: &RedisPool, user_id: &str) -> Result<String, redis::RedisError> {
    let mut con = client.get_multiplexed_async_connection().await?;
    let key = format!("refresh:{}", user_id);
    con.get(key).await
}

pub async fn revoke_token(client: &RedisPool, user_id: &str) -> Result<(), redis::RedisError> {
    let mut con = client.get_multiplexed_async_connection().await?;
    let key = format!("refresh:{}", user_id);
    con.del(key).await
}
