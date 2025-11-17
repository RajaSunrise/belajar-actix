use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, Result};
use sqlx::PgPool;
use models::{User, NewUser};

mod models;
mod database;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello World")
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Ini Hey Hello")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

#[get("/users")]
async fn get_users(pool: web::Data<PgPool>) -> Result<impl Responder> {
    let users = sqlx::query_as::<_, User>("SELECT id, name, email FROM users")
        .fetch_all(pool.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    Ok(HttpResponse::Ok().json(users))
}

#[post("/users")]
async fn create_user(pool: web::Data<PgPool>, new_user: web::Json<NewUser>) -> Result<impl Responder> {
    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (name, email) VALUES ($1, $2) RETURNING id, name, email"
    )
    .bind(&new_user.name)
    .bind(&new_user.email)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    Ok(HttpResponse::Created().json(user))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = database::establish_connection().await.expect("Failed to connect to database");
    println!("Database connected!");
    println!("server berjalan di http://localhost:8080");
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(hello)
            .service(echo)
            .service(get_users)
            .service(create_user)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
