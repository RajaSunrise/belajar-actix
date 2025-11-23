use actix_web::web;
use crate::handlers::admin;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/admin")
            .route("/anime", web::post().to(admin::create_anime))
            .route("/anime/{id}", web::put().to(admin::update_anime))
            .route("/anime/{id}", web::delete().to(admin::delete_anime))
            .route("/upload", web::post().to(admin::upload_episode))
            .route("/episode", web::post().to(admin::create_episode_meta))
            .route("/metrics", web::get().to(admin::get_system_metrics))
            .route("/users", web::get().to(admin::get_users))
            .route("/users/{id}", web::delete().to(admin::delete_user))
    );
}
