use actix_web::web;
use crate::handlers::auth;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .route("/register", web::post().to(auth::register))
            .route("/login", web::post().to(auth::login))
            .route("/refresh", web::post().to(auth::refresh))
            .route("/logout", web::post().to(auth::logout))
    );
}
