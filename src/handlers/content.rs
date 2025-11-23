use actix_web::{web, HttpResponse, Responder};
use sqlx::AnyPool;
use crate::models::content::{AnimeSeries, Episode, Genre};
use serde_json::json;

async fn fetch_genres_for_anime(pool: &AnyPool, anime_id: &str) -> Vec<Genre> {
    sqlx::query_as(
        "SELECT g.* FROM genres g
         JOIN anime_genres ag ON g.id = ag.genre_id
         WHERE ag.anime_id = ?"
    )
    .bind(anime_id)
    .fetch_all(pool)
    .await
    .unwrap_or(vec![])
}

// Generic internal list fetcher
async fn get_content_list(pool: &AnyPool, content_type: Option<&str>) -> Vec<AnimeSeries> {
    let mut anime: Vec<AnimeSeries> = if let Some(ct) = content_type {
        sqlx::query_as("SELECT * FROM anime_series WHERE content_type = ?")
            .bind(ct)
            .fetch_all(pool)
            .await
            .unwrap_or(vec![])
    } else {
        sqlx::query_as("SELECT * FROM anime_series")
            .fetch_all(pool)
            .await
            .unwrap_or(vec![])
    };

    // Populate genres efficiently? For now N+1 is fine for low traffic/learning
    for a in &mut anime {
        a.genres = fetch_genres_for_anime(pool, &a.id).await;
    }
    anime
}

pub async fn get_anime_list(pool: web::Data<AnyPool>) -> impl Responder {
    let list = get_content_list(pool.get_ref(), Some("Anime")).await;
    HttpResponse::Ok().json(list)
}

pub async fn get_donghua_list(pool: web::Data<AnyPool>) -> impl Responder {
    let list = get_content_list(pool.get_ref(), Some("Donghua")).await;
    HttpResponse::Ok().json(list)
}

pub async fn get_movie_list(pool: web::Data<AnyPool>) -> impl Responder {
    let list = get_content_list(pool.get_ref(), Some("Movie")).await;
    HttpResponse::Ok().json(list)
}

pub async fn get_all_content(pool: web::Data<AnyPool>) -> impl Responder {
    let list = get_content_list(pool.get_ref(), None).await;
    HttpResponse::Ok().json(list)
}

pub async fn get_anime_detail(
    pool: web::Data<AnyPool>,
    path: web::Path<String>,
) -> impl Responder {
    let id = path.into_inner();
    let mut anime: Option<AnimeSeries> = sqlx::query_as("SELECT * FROM anime_series WHERE id = ?")
        .bind(&id)
        .fetch_optional(pool.get_ref())
        .await
        .unwrap_or(None);

    if let Some(ref mut a) = anime {
        a.genres = fetch_genres_for_anime(pool.get_ref(), &a.id).await;
    }

    let episodes: Vec<Episode> = sqlx::query_as("SELECT * FROM episodes WHERE series_id = ? ORDER BY episode_number")
        .bind(&id)
        .fetch_all(pool.get_ref())
        .await
        .unwrap_or(vec![]);

    match anime {
        Some(a) => HttpResponse::Ok().json(json!({ "series": a, "episodes": episodes })),
        None => HttpResponse::NotFound().json(json!({"error": "Content not found"})),
    }
}

pub async fn get_schedule(
    pool: web::Data<AnyPool>,
) -> impl Responder {
    let mut anime: Vec<AnimeSeries> = sqlx::query_as("SELECT * FROM anime_series WHERE schedule_day IS NOT NULL")
        .fetch_all(pool.get_ref())
        .await
        .unwrap_or(vec![]);

    for a in &mut anime {
        a.genres = fetch_genres_for_anime(pool.get_ref(), &a.id).await;
    }
    HttpResponse::Ok().json(anime)
}

pub async fn search_content(
    pool: web::Data<AnyPool>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    let q = query.get("q").cloned().unwrap_or_default();

    // Naive search - Note: || concatenation is standard SQL
    let mut result: Vec<AnimeSeries> = sqlx::query_as("SELECT * FROM anime_series WHERE title LIKE '%' || ? || '%'")
        .bind(&q)
        .fetch_all(pool.get_ref())
        .await
        .unwrap_or(vec![]);

    for a in &mut result {
        a.genres = fetch_genres_for_anime(pool.get_ref(), &a.id).await;
    }

    let filtered: Vec<AnimeSeries> = if let Some(t) = query.get("type") {
        result.into_iter().filter(|a| a.content_type.eq_ignore_ascii_case(t)).collect()
    } else {
        result
    };

    HttpResponse::Ok().json(filtered)
}

pub async fn get_genres(
    pool: web::Data<AnyPool>,
) -> impl Responder {
    let genres: Vec<Genre> = sqlx::query_as("SELECT * FROM genres ORDER BY name")
        .fetch_all(pool.get_ref())
        .await
        .unwrap_or(vec![]);
    HttpResponse::Ok().json(genres)
}
