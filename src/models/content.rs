use serde::{Deserialize, Serialize};
use sqlx::FromRow;
// use chrono::NaiveDateTime;

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct AnimeSeries {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub content_type: String, // "Anime", "Donghua", "Movie"
    pub status: String,       // "Ongoing", "Tamat"
    pub schedule_day: Option<String>,
    pub thumbnail_url: Option<String>,
    pub created_at: Option<String>, // String
    pub rating: Option<f32>,
    #[sqlx(skip)]
    pub genres: Vec<Genre>,
}

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct Genre {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct Episode {
    pub id: String,
    pub series_id: String,
    pub title: String,
    pub episode_number: i32,
    pub video_path: String,
    pub created_at: Option<String>, // String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAnimeRequest {
    pub title: String,
    pub description: Option<String>,
    pub content_type: String,
    pub status: String,
    pub schedule_day: Option<String>,
    pub rating: Option<f32>,
    pub genre_ids: Option<Vec<i64>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateAnimeRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub content_type: Option<String>,
    pub status: Option<String>,
    pub schedule_day: Option<String>,
    pub rating: Option<f32>,
    pub genre_ids: Option<Vec<i64>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateEpisodeRequest {
    pub series_id: String,
    pub title: String,
    pub episode_number: i32,
}
