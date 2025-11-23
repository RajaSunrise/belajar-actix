use sqlx::any::AnyPoolOptions;
use sqlx::{AnyPool};
use std::env;

#[derive(Clone, Copy, PartialEq)]
pub enum DbKind {
    Sqlite,
    Postgres,
}

#[derive(Clone)]
pub struct AppState {
    pub db: AnyPool,
}

pub async fn init_db() -> AnyPool {
    // Install default drivers for AnyPool
    sqlx::any::install_default_drivers();

    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:anime.db".to_string());
    let mut kind = DbKind::Sqlite;

    // Check if it looks like postgres
    if database_url.starts_with("postgres") {
        kind = DbKind::Postgres;
    } else {
        let path = database_url.replace("sqlite:", "");
        if !std::path::Path::new(&path).exists() && path != ":memory:" {
             println!("Creating SQLite database file: {}", path);
             std::fs::File::create(&path).expect("Failed to create database file");
        }
    }

    println!("Connecting to database: {}", database_url);

    let pool_result = AnyPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await;

    // Fallback to SQLite if connection failed
    let pool = match pool_result {
        Ok(p) => p,
        Err(e) => {
            println!("Failed to connect to provided URL: {}. Error: {}", database_url, e);
            println!("Falling back to local SQLite: sqlite:anime.db");
            kind = DbKind::Sqlite;
            if !std::path::Path::new("anime.db").exists() {
                 std::fs::File::create("anime.db").expect("Failed to create database file");
            }
            AnyPoolOptions::new()
                .max_connections(5)
                .connect("sqlite:anime.db")
                .await
                .expect("Failed to connect to fallback SQLite")
        }
    };

    run_migrations(&pool, kind).await;

    pool
}

async fn run_migrations(pool: &AnyPool, kind: DbKind) {
    // Users
    let users_query = r#"
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            username TEXT NOT NULL UNIQUE,
            password TEXT NOT NULL,
            role TEXT NOT NULL DEFAULT 'user',
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
    "#;

    // Anime Series
    let anime_query = r#"
        CREATE TABLE IF NOT EXISTS anime_series (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            description TEXT,
            content_type TEXT NOT NULL,
            status TEXT NOT NULL,
            schedule_day TEXT,
            thumbnail_url TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            rating REAL DEFAULT 0.0
        );
    "#;

    // Episodes
    let ep_query = r#"
        CREATE TABLE IF NOT EXISTS episodes (
            id TEXT PRIMARY KEY,
            series_id TEXT NOT NULL,
            title TEXT NOT NULL,
            episode_number INTEGER NOT NULL,
            video_path TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(series_id) REFERENCES anime_series(id) ON DELETE CASCADE
        );
    "#;

    // Genres
    let genre_query = format!(r#"
        CREATE TABLE IF NOT EXISTS genres (
            id {id_type} PRIMARY KEY {pg_suffix},
            name TEXT NOT NULL UNIQUE
        );
        "#,
        id_type = if kind == DbKind::Postgres { "SERIAL" } else { "INTEGER" },
        pg_suffix = if kind == DbKind::Sqlite { "AUTOINCREMENT" } else { "" }
    );

    // Anime Genres Junction
    let ag_query = r#"
        CREATE TABLE IF NOT EXISTS anime_genres (
            anime_id TEXT NOT NULL,
            genre_id INTEGER NOT NULL,
            PRIMARY KEY (anime_id, genre_id),
            FOREIGN KEY(anime_id) REFERENCES anime_series(id) ON DELETE CASCADE,
            FOREIGN KEY(genre_id) REFERENCES genres(id) ON DELETE CASCADE
        );
    "#;

    let queries = vec![users_query, anime_query, ep_query, &genre_query, ag_query];

    for query in queries {
        if let Err(e) = sqlx::query(query).execute(pool).await {
            println!("Migration Warning/Error: {}", e);
        }
    }

    // Seed Genres
    let genres = vec![
        "Action", "Adventure", "Comedy", "Drama", "Fantasy",
        "Horror", "Mecha", "Music", "Romance", "Sci-Fi",
        "Slice of Life", "Sports", "Thriller", "Xianxia", "Mystery"
    ];

    for g in genres {
        let insert_query = match kind {
            DbKind::Sqlite => "INSERT OR IGNORE INTO genres (name) VALUES (?)",
            DbKind::Postgres => "INSERT INTO genres (name) VALUES (?) ON CONFLICT (name) DO NOTHING",
        };

        let _ = sqlx::query(insert_query)
            .bind(g)
            .execute(pool)
            .await;
    }
}
