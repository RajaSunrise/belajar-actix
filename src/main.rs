use actix_web::{web, App, HttpServer, HttpResponse};
use actix_files as fs;
use actix_cors::Cors;
use dotenvy::dotenv;
use std::env;

mod db;
mod models;
mod handlers;
mod auth;
mod services;
mod routes;
mod middleware;

#[cfg(test)]
mod tests;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    // env_logger::init(); // Replaced by file logger

    // Initialize File Logger
    let _guard = middleware::logger::init_file_logger();

    // Initialize Database
    let pool: sqlx::AnyPool = db::init_db().await;
    let data_pool = web::Data::new(pool);

    // Initialize Redis
    let redis_pool = services::redis::init_redis().await;
    let data_redis = web::Data::new(redis_pool.clone()); // clone client (cheap)

    // Rate Limiter
    let limiter = middleware::limiter::RateLimit { pool: redis_pool.clone() };

    // Create directories if they don't exist
    std::fs::create_dir_all("uploads").unwrap();
    std::fs::create_dir_all("static").unwrap();

    let port = env::var("PORT").unwrap_or("8080".to_string());
    println!("Starting server on port {}", port);

    HttpServer::new(move || {
        let cors = Cors::permissive();

        App::new()
            .wrap(cors)
            .wrap(limiter.clone()) // Rate Limiter Global
            .wrap(middleware::auth::JwtAuth) // Global Auth Middleware (logic inside skips public routes)
            .app_data(data_pool.clone())
            .app_data(data_redis.clone())
            // Static files
            .service(fs::Files::new("/static", "./static").show_files_listing())
            .service(fs::Files::new("/uploads", "./uploads").show_files_listing())
            // API Routes
            .route("/", web::get().to(|| async { HttpResponse::Ok().body("Anime Streaming API Running") }))
            .configure(routes::config)
    })
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await
}
