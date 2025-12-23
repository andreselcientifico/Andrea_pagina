mod models;
mod func;
mod auth;
mod config;
mod test;
mod errors;
mod db;
mod utils;
mod middleware;
mod mail;

use actix_web::Responder;
use actix_web::middleware::Compress;
use actix_web::web::scope;
use actix_web::{ web,web::{Data, Json},  App, HttpServer, HttpResponse };
use chrono::{ DateTime, Utc };
use openssl::ssl::{ SslAcceptor, SslFiletype, SslMethod };
use config::config::Config;
use reqwest::Client;
use serde_json::Value;
use std::sync::Arc ;
use tokio::sync::Mutex;
use db::db::DBClient;
use sqlx::postgres::PgPoolOptions;
use dotenvy;
use middleware::middleware::AuthMiddlewareFactory;
use crate::func::users::users_scope;
use crate::func::courses::courses_scope;
use crate::func::payments::payments_scope;
use env_logger::Env; 


//==================== //
//      APP STATE
// ==================== //
#[derive(Clone, Debug)]
pub struct AppState {
    pub env: Config,
    pub client: Client,
    pub token_cache: Arc<Mutex<Option<CachedToken>>>,
    pub db_client: DBClient, 
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

pub async fn ping(
    Json(json): Json<Value>,
) -> impl Responder {
    // Imprime el JSON recibido en formato pretty
    match serde_json::to_string_pretty(&json) {
        Ok(pretty) => println!("Json pretty:\n{}", pretty),
        Err(e) => eprintln!("Error convirtiendo JSON a pretty: {}", e),
    }

    // Respuesta HTTP
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "message": "pong",
    }))
}

// ===================== //
//        MAIN
// ===================== //
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().expect("No se pudo cargar el archivo .env");
    let current_dir = std::env::current_dir().expect("No se pudo obtener el directorio actual");
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

    let key_path = current_dir.join("key.pem");
    let cert_path = current_dir.join("cert.pem");

    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder.set_private_key_file(key_path, SslFiletype::PEM).expect("No se pudo leer key.pem");
    builder.set_certificate_chain_file(cert_path).expect("No se pudo leer cert.pem");
   
    // Crear conexión a Postgres
    let pool = match PgPoolOptions::new()
        .connect(&Config::init().database_url).await
    {
        Ok(pool) => {
            pool
        },
        Err(err) => {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "Database connection failed {}".replace("{}", &err.to_string())));
        }
    };
    let db: DBClient = DBClient::new(pool);
    
     let state = AppState {
        env: Config::init(),
        client: Client::new(),
        token_cache: Arc::new(Mutex::new(None)),
        db_client: db.clone(),
    };
     let app_state = Arc::new(state.clone());
    HttpServer::new(move || {
        App::new()
            .wrap(Compress::default())
            .wrap(
                actix_cors::Cors::permissive()
                .allowed_origin_fn(| origin, _req_head| {
                    let origin = origin.to_str().unwrap_or("");
                    origin.as_bytes().ends_with(b"localhost:8080")
                })
                .allowed_origin_fn(| origin, _req_head| {
                    let origin = origin.to_str().unwrap_or("");
                    origin.as_bytes().ends_with(b".trycloudflare.com")
                })
                .allow_any_method()
                .allow_any_header()
                .supports_credentials()
                .max_age(3600)
            )
            .app_data(Data::new(app_state.clone()))
            .route("/ping", web::post().to(ping))
            .service(func::handlers::register_user)
            .service(func::handlers::login_user)
            .service(func::handlers::verify_email)
            .service(func::handlers::logout_user)
            .service(users_scope(app_state.clone()))
            .service(courses_scope(app_state.clone()))
            .service(payments_scope(app_state.clone()))
            .service(
            scope("/api")
                .wrap(AuthMiddlewareFactory::new(app_state.clone()))
                .service(func::handlers::get_user_profile)
                .service(func::handlers::update_user_profile)
                .service(func::handlers::created_order)
                .service(func::handlers::capture_order)
                .service(
                        func::handlers::get_user_courses_api
                )
        )
            
    })
        .workers(8)
        // .bind_openssl("127.0.0.1:8000", builder)?
        .bind("0.0.0.0:8000")?
        .run().await
}
