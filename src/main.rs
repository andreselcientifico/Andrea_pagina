mod models;
mod func;
mod auth;
mod config;
mod test;
mod errors;
mod db;

use actix_web::{ web, App, HttpServer };
use openssl::ssl::{ SslAcceptor, SslFiletype, SslMethod };
use reqwest::Client;
use std::{ env, sync::Arc };
use tokio::sync::Mutex;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use dotenvy;
use crate::func::handlers::{ AppState };


//==================== //
//      DB POOL
// ==================== //
#[derive(Clone)]
pub struct PgPoolWrapper {
    pub db_pool: PgPool,
}

// ===================== //
//        MAIN
// ===================== //
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().expect("No se pudo cargar el archivo .env");

    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder.set_private_key_file("key.pem", SslFiletype::PEM).unwrap();
    builder.set_certificate_chain_file("cert.pem").unwrap();

    let client = Client::new();

    // Crear conexión a Postgres
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL no está definido");
    let pool = PgPoolOptions::new()
        .connect(&database_url).await
        .expect("No se pudo conectar a la base de datos");

    let state = AppState {
        client,
        paypal_api_mode: env
            ::var("PAYPAL_API_MODE")
            .unwrap_or("https://api-m.sandbox.paypal.com".to_string()),
        paypal_client_id: env
            ::var("PAYPAL_API_CLIENT_ID")
            .expect("PAYPAL_API_CLIENT_ID no definido"),
        paypal_secret: env::var("PAYPAL_API_SECRET").expect("PAYPAL_API_SECRET no definido"),
        token_cache: Arc::new(Mutex::new(None)),
    };
    let db = PgPoolWrapper { db_pool: pool };

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .app_data(web::Data::new(db.clone()))
            .service(func::handlers::created_order)
            .service(func::handlers::capture_order)
            .service(func::handlers::cancel_order)
            .service(func::handlers::register_user)
            .service(func::handlers::login_user)
            .service(func::handlers::get_user_profile)
    })
        .workers(8)
        .bind("127.0.0.1:8000")?
        .run().await
}
