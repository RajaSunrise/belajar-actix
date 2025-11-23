use actix_web::{web, HttpResponse, Responder, HttpRequest, HttpMessage};
use sqlx::AnyPool;
use uuid::Uuid;
use bcrypt::{hash, verify, DEFAULT_COST};
use crate::models::user::{User, LoginRequest, RegisterRequest};
use crate::models::auth::RefreshRequest;
use crate::auth::create_jwt;
use crate::services::redis::{RedisPool, store_refresh_token, get_refresh_token, revoke_token};
use serde_json::json;

pub async fn register(
    pool: web::Data<AnyPool>,
    req: web::Json<RegisterRequest>,
) -> impl Responder {
    let hashed_password = hash(&req.password, DEFAULT_COST).unwrap();
    let id = Uuid::new_v4().to_string();
    let role = "user";

    // Check if it's the first user
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(pool.get_ref())
        .await
        .unwrap_or((0,));

    let final_role = if count.0 == 0 { "superuser" } else { role };

    let result = sqlx::query("INSERT INTO users (id, username, password, role) VALUES (?, ?, ?, ?)")
        .bind(&id)
        .bind(&req.username)
        .bind(&hashed_password)
        .bind(final_role)
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(_) => HttpResponse::Ok().json(json!({"message": "User registered", "id": id, "role": final_role})),
        Err(_) => HttpResponse::BadRequest().json(json!({"error": "Username taken"})),
    }
}

pub async fn login(
    pool: web::Data<AnyPool>,
    redis: web::Data<RedisPool>,
    req: web::Json<LoginRequest>,
) -> impl Responder {
    let user: Option<User> = sqlx::query_as("SELECT * FROM users WHERE username = ?")
        .bind(&req.username)
        .fetch_optional(pool.get_ref())
        .await
        .unwrap_or(None);

    match user {
        Some(u) => {
            if verify(&req.password, &u.password).unwrap_or(false) {
                // Generate Access Token (Short lived)
                let access_token = create_jwt(&u.id, &u.role).unwrap();

                // Generate Refresh Token (Long lived)
                let refresh_token = Uuid::new_v4().to_string();

                // Store Refresh Token in Redis (7 days = 604800s)
                let _ = store_refresh_token(redis.get_ref(), &u.id, &refresh_token, 604800).await;

                HttpResponse::Ok().json(json!({
                    "access_token": access_token,
                    "refresh_token": refresh_token,
                    "user_id": u.id,
                    "role": u.role
                }))
            } else {
                HttpResponse::Unauthorized().json(json!({"error": "Invalid credentials"}))
            }
        }
        None => HttpResponse::Unauthorized().json(json!({"error": "User not found"})),
    }
}

pub async fn refresh(
    pool: web::Data<AnyPool>,
    redis: web::Data<RedisPool>,
    req: web::Json<RefreshRequest>,
) -> impl Responder {
    // Validate stored refresh token
    let stored_token = get_refresh_token(redis.get_ref(), &req.user_id).await;

    match stored_token {
        Ok(token) if token == req.refresh_token => {
            // Valid! Generate new tokens.

            // Get user role to generate new JWT
            let user: Option<User> = sqlx::query_as("SELECT * FROM users WHERE id = ?")
                .bind(&req.user_id)
                .fetch_optional(pool.get_ref())
                .await
                .unwrap_or(None);

            if let Some(u) = user {
                let new_access = create_jwt(&u.id, &u.role).unwrap();
                // Rotate refresh token? (Optional, let's keep it simple and re-use or rotate)
                // Let's rotate.
                let new_refresh = Uuid::new_v4().to_string();
                let _ = store_refresh_token(redis.get_ref(), &u.id, &new_refresh, 604800).await;

                HttpResponse::Ok().json(json!({
                    "access_token": new_access,
                    "refresh_token": new_refresh
                }))
            } else {
                 HttpResponse::Unauthorized().json(json!({"error": "User not found"}))
            }
        }
        _ => HttpResponse::Unauthorized().json(json!({"error": "Invalid refresh token"}))
    }
}

pub async fn logout(
    redis: web::Data<RedisPool>,
    req: HttpRequest,
) -> impl Responder {
    if let Some(claims) = req.extensions().get::<crate::models::TokenClaims>() {
         let _ = revoke_token(redis.get_ref(), &claims.sub).await;
         HttpResponse::Ok().json(json!({"message": "Logged out"}))
    } else {
         HttpResponse::Unauthorized().finish()
    }
}
