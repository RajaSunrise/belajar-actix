use actix_web::{web, HttpResponse, Responder};
use sqlx::AnyPool;
use uuid::Uuid;
use crate::models::content::{CreateAnimeRequest, CreateEpisodeRequest, UpdateAnimeRequest};
use crate::models::user::User;
use crate::services::video::save_video;
use actix_multipart::Multipart;
use sys_info;
use serde_json::json;

pub async fn create_anime(
    pool: web::Data<AnyPool>,
    req: web::Json<CreateAnimeRequest>,
) -> impl Responder {
    let id = Uuid::new_v4().to_string();

    // Transaction
    let mut tx = match pool.begin().await {
        Ok(t) => t,
        Err(e) => return HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    };

    let res = sqlx::query(
        "INSERT INTO anime_series (id, title, description, content_type, status, schedule_day, rating) VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&req.title)
    .bind(&req.description)
    .bind(&req.content_type)
    .bind(&req.status)
    .bind(&req.schedule_day)
    .bind(req.rating.unwrap_or(0.0))
    .execute(&mut *tx)
    .await;

    if let Err(e) = res {
        return HttpResponse::InternalServerError().json(json!({"error": e.to_string()}));
    }

    if let Some(g_ids) = &req.genre_ids {
        for gid in g_ids {
            let g_res = sqlx::query("INSERT INTO anime_genres (anime_id, genre_id) VALUES (?, ?)")
                .bind(&id)
                .bind(gid)
                .execute(&mut *tx)
                .await;
            if let Err(e) = g_res {
                 let _ = tx.rollback().await;
                 return HttpResponse::InternalServerError().json(json!({"error": e.to_string()}));
            }
        }
    }

    if let Err(e) = tx.commit().await {
        return HttpResponse::InternalServerError().json(json!({"error": e.to_string()}));
    }

    HttpResponse::Ok().json(json!({"message": "Content created", "id": id}))
}

pub async fn update_anime(
    pool: web::Data<AnyPool>,
    path: web::Path<String>,
    req: web::Json<UpdateAnimeRequest>,
) -> impl Responder {
    let id = path.into_inner();

    let mut tx = match pool.begin().await {
        Ok(t) => t,
        Err(e) => return HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    };

    let result = sqlx::query(
        "UPDATE anime_series SET
            title = COALESCE(?, title),
            description = COALESCE(?, description),
            content_type = COALESCE(?, content_type),
            status = COALESCE(?, status),
            schedule_day = COALESCE(?, schedule_day),
            rating = COALESCE(?, rating)
        WHERE id = ?"
    )
    .bind(&req.title)
    .bind(&req.description)
    .bind(&req.content_type)
    .bind(&req.status)
    .bind(&req.schedule_day)
    .bind(req.rating)
    .bind(&id)
    .execute(&mut *tx)
    .await;

    if let Err(e) = result {
         return HttpResponse::InternalServerError().json(json!({"error": e.to_string()}));
    }

    // Handle genres update: clear old, insert new (simplest strategy)
    if let Some(g_ids) = &req.genre_ids {
        let del = sqlx::query("DELETE FROM anime_genres WHERE anime_id = ?")
            .bind(&id)
            .execute(&mut *tx)
            .await;

        if let Err(e) = del {
             let _ = tx.rollback().await;
             return HttpResponse::InternalServerError().json(json!({"error": e.to_string()}));
        }

        for gid in g_ids {
            let g_res = sqlx::query("INSERT INTO anime_genres (anime_id, genre_id) VALUES (?, ?)")
                .bind(&id)
                .bind(gid)
                .execute(&mut *tx)
                .await;
            if let Err(e) = g_res {
                 let _ = tx.rollback().await;
                 return HttpResponse::InternalServerError().json(json!({"error": e.to_string()}));
            }
        }
    }

    if let Err(e) = tx.commit().await {
        return HttpResponse::InternalServerError().json(json!({"error": e.to_string()}));
    }

    HttpResponse::Ok().json(json!({"message": "Content updated"}))
}

pub async fn delete_anime(
    pool: web::Data<AnyPool>,
    path: web::Path<String>,
) -> impl Responder {
    let id = path.into_inner();
    let result = sqlx::query("DELETE FROM anime_series WHERE id = ?")
        .bind(&id)
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(_) => HttpResponse::Ok().json(json!({"message": "Content deleted"})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

pub async fn upload_episode(
    pool: web::Data<AnyPool>,
    payload: Multipart,
) -> impl Responder {
    match save_video(payload).await {
        Ok(path) => HttpResponse::Ok().json(json!({"path": path})),
        Err(_) => HttpResponse::InternalServerError().json(json!({"error": "Upload failed"})),
    }
}

pub async fn create_episode_meta(
    pool: web::Data<AnyPool>,
    req: web::Json<CreateEpisodeRequest>,
    video_path: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    let path = video_path.get("path").cloned().unwrap_or_default();
    let id = Uuid::new_v4().to_string();

    let result = sqlx::query(
        "INSERT INTO episodes (id, series_id, title, episode_number, video_path) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&req.series_id)
    .bind(&req.title)
    .bind(req.episode_number)
    .bind(&path)
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().json(json!({"message": "Episode created", "id": id})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

pub async fn get_system_metrics() -> impl Responder {
    let load = sys_info::loadavg().unwrap_or(sys_info::LoadAvg { one: 0.0, five: 0.0, fifteen: 0.0 });
    let mem = sys_info::mem_info().unwrap_or(sys_info::MemInfo { total: 0, free: 0, avail: 0, buffers: 0, cached: 0, swap_total: 0, swap_free: 0 });
    let disk = sys_info::disk_info().unwrap_or(sys_info::DiskInfo { total: 0, free: 0 });

    HttpResponse::Ok().json(json!({
        "load_avg": load.one,
        "memory_used_kb": mem.total - mem.free,
        "memory_total_kb": mem.total,
        "disk_free_kb": disk.free,
        "status": "Healthy"
    }))
}

pub async fn get_users(
    pool: web::Data<AnyPool>,
) -> impl Responder {
    let users: Vec<User> = sqlx::query_as("SELECT * FROM users")
        .fetch_all(pool.get_ref())
        .await
        .unwrap_or(vec![]);
    HttpResponse::Ok().json(users)
}

pub async fn delete_user(
    pool: web::Data<AnyPool>,
    path: web::Path<String>,
) -> impl Responder {
    let id = path.into_inner();
    let result = sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(&id)
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(_) => HttpResponse::Ok().json(json!({"message": "User deleted"})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}
