use actix_web::web;
use crate::handlers::content;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .route("/anime", web::get().to(content::get_anime_list))
            .route("/donghua", web::get().to(content::get_donghua_list))
            .route("/movies", web::get().to(content::get_movie_list))
            .route("/all", web::get().to(content::get_all_content))
            .route("/content/{id}", web::get().to(content::get_anime_detail))
            .route("/schedule", web::get().to(content::get_schedule))
            .route("/search", web::get().to(content::search_content))
            .route("/genres", web::get().to(content::get_genres))
    );
}
