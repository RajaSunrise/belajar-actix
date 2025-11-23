use actix_web::web;

pub mod auth;
pub mod content;
pub mod admin;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/api")
        .configure(auth::config)
        .configure(content::config)
        .configure(admin::config)
    );
}
