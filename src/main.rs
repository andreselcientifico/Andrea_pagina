mod models;
mod func;
mod auth;
mod config;
mod test;
mod errors;
mod db;
mod utils;
mod middleware;

use actix_web::{ web, App, HttpServer };
use chrono::{ DateTime, Utc };
use openssl::ssl::{ SslAcceptor, SslFiletype, SslMethod };
use config::config::Config;
use reqwest::Client;
use std::{ sync::Arc };
use tokio::sync::Mutex;
use db::db::DBClient;
use sqlx::postgres::PgPoolOptions;
use dotenvy;
use actix_web::http::header::{ AUTHORIZATION, ACCEPT, CONTENT_TYPE };


//==================== //
//      APP STATE
// ==================== //
#[derive(Clone, Debug)]
pub struct AppState {
    pub env: Config,
    pub client: Client,
    pub token_cache: Arc<Mutex<Option<CachedToken>>>,
}

#[derive(Clone, Debug)]
pub struct CachedToken {
    access_token: String,
    expires_at: DateTime<Utc>,
}

impl CachedToken {
    fn is_valid(&self) -> bool {
        Utc::now() < self.expires_at
    }
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

    let state = AppState {
        env: Config::init(),
        client: Client::new(),
        token_cache: Arc::new(Mutex::new(None)),
    };
    // Crear conexión a Postgres
    let pool = match PgPoolOptions::new()
        .connect(&state.env.database_url).await
    {
        Ok(pool) => {
            println!("✅ Conectado a la base de datos");
            pool
        },
        Err(err) => {
            eprintln!("❌ Error al conectar a la base de datos: {:?}", err);
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "Database connection failed"));
        }
    };
    let db = DBClient::new(pool);
    
    HttpServer::new(move || {
 

        App::new()
            .wrap(
                actix_cors::Cors::default()
                .allowed_origin("http://localhost:3000")
                .allowed_origin_fn(| origin, _req_head| {
                    origin.as_bytes().ends_with(b"localhost:3000")
                })
                .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                .allowed_headers(vec![AUTHORIZATION, ACCEPT])
                .allowed_header(CONTENT_TYPE)
                .max_age(3600)
            )
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
