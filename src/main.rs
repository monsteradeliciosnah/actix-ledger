use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool};
use uuid::Uuid;

#[derive(Clone)]
struct AppState { pool: SqlitePool }

#[derive(Serialize, Deserialize)]
struct Entry { id: Uuid, memo: String, amount: i64 }

#[derive(Deserialize)]
struct NewEntry { memo: String, amount: i64 }

#[get("/health")]
async fn health() -> impl Responder { HttpResponse::Ok().json(serde_json::json!({"status":"ok"})) }

#[get("/entries")]
async fn list(state: web::Data<AppState>) -> impl Responder {
    let rows = sqlx::query!("SELECT id, memo, amount FROM entries ORDER BY rowid DESC")
        .fetch_all(&state.pool).await.unwrap();
    let items: Vec<Entry> = rows.into_iter().map(|r| Entry{ id: Uuid::parse_str(&r.id).unwrap(), memo: r.memo, amount: r.amount}).collect();
    HttpResponse::Ok().json(items)
}

#[post("/entries")]
async fn create(state: web::Data<AppState>, body: web::Json<NewEntry>) -> impl Responder {
    let id = Uuid::new_v4();
    sqlx::query!("INSERT INTO entries(id, memo, amount) VALUES (?1, ?2, ?3)", id.to_string(), body.memo.clone(), body.amount)
        .execute(&state.pool).await.unwrap();
    HttpResponse::Ok().json(Entry{id, memo: body.memo.clone(), amount: body.amount})
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = SqlitePool::connect("sqlite://ledger.db").await.unwrap();
    sqlx::query("CREATE TABLE IF NOT EXISTS entries (id TEXT PRIMARY KEY, memo TEXT NOT NULL, amount INTEGER NOT NULL)").execute(&pool).await.unwrap();
    let state = AppState{ pool };
    HttpServer::new(move || App::new().app_data(web::Data::new(state.clone())).service(health).service(list).service(create))
        .bind(("0.0.0.0", 8080))?.run().await
}
