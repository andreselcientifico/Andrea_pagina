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
mod routes;
mod services;

use actix_web::{Responder, web};
use actix_web::web::{ scope };
use actix_files::{Files};
use tera::{Context, Tera};
// use actix_web::middleware::Compress;
use actix_web::{ web::{ Data, Json }, App, HttpServer, HttpResponse };
use chrono::{ DateTime, Utc };
// use openssl::ssl::{ SslAcceptor, SslFiletype, SslMethod };
use config::config::Config;
use reqwest::Client;
use services::paypal_client::PayPalClient;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;
use db::db::DBClient;
use sqlx::postgres::PgPoolOptions;
use dotenvy;
use crate::routes::routes::{ auth_scope, course_scope, global_scope };
// use env_logger::Env;
//==================== //
//      APP STATE
// ==================== //
#[derive(Clone, Debug)]
pub struct AppState {
    pub env: Config,
    pub client: Client,
    pub token_cache: Arc<RwLock<Option<CachedToken>>>,
    pub db_client: DBClient,
    pub paypal_client: PayPalClient,
    pub tera: Tera,
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

pub async fn ping(Json(json): Json<Value>) -> impl Responder {
    // Imprime el JSON recibido en formato pretty
    match serde_json::to_string_pretty(&json) {
        Ok(pretty) => log::info!("Json pretty:\n{}", pretty),
        Err(e) => log::error!("Error convirtiendo JSON a pretty: {}", e),
    }

    // Respuesta HTTP
    HttpResponse::Ok().json(
        serde_json::json!({
        "status": "ok",
        "message": "pong",
    })
    )
}

// Esta función NO tiene macro. Se usa para rutas como /cursos o /perfil
async fn index_fallback(state: web::Data<Arc<AppState>>) -> impl Responder {
    render_index(state).await
}

// Función auxiliar para no repetir código
async fn render_index(state: web::Data<Arc<AppState>>) -> HttpResponse {
    let context = Context::new();
    match state.tera.render("index.html", &context) {
        Ok(body) => HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(body),
        Err(e) => {
            log::error!("Error de Tera: {}", e);
            HttpResponse::InternalServerError().body("Error interno")
        }
    }
}

// ===================== //
//        MAIN
// ===================== //
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if let Err(e) = dotenvy::dotenv() {
        log::warn!("No se cargó el archivo .env (esto es normal en producción): {}", e);
    }
    // env_logger::Builder::from_env(Env::default().default_filter_or("debug,actix_server=info")).init();

    // Crear conexión a Postgres
    let config = Config::init();
    let pool = match PgPoolOptions::new().connect(&config.database_url).await {
        Ok(pool) => { pool }
        Err(err) => {
            return Err(
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Database connection failed {}".replace("{}", &err.to_string())
                )
            );
        }
    };
    let db: DBClient = DBClient::new(pool.clone());
    log::info!("Ejecutando migraciones...");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("No se pudieron ejecutar las migraciones");
    let paypal_client = PayPalClient::new(
        config.paypal_client_id.clone(),
        config.paypal_secret.clone(),
        config.paypal_api_mode.contains("sandbox")
    ).await;

    let dist_path = std::env::current_dir()
    .unwrap()
    .join("dist");

    // 2. Inicializamos Tera vacío y añadimos el index.html manualmente
    let mut tera_md = Tera::default();
    let index_path = dist_path.join("index.html");
    let static_path = format!("{}", dist_path.clone().to_str().unwrap());
    let assets_path = format!("{}/assets", dist_path.clone().to_str().unwrap());

    if index_path.exists() {
        tera_md.add_template_file(index_path, Some("index.html")).expect("Error al cargar index.html");
        log::info!("✅ index.html cargado correctamente en Tera");
    } else {
        log::error!("❌ NO SE ENCONTRÓ el archivo en: {:?}", index_path);
    }
    let state = AppState {
        env: config,
        client: Client::new(),
        token_cache: Arc::new(RwLock::new(None)),
        db_client: db.clone(),
        paypal_client,
        tera: tera_md,
    };
    let app_state = Arc::new(state.clone());
    HttpServer::new(move || {
        
        App::new()
            .app_data(Data::new(app_state.clone()))
            .wrap(
                actix_cors::Cors::default()
                    .allowed_origin("http://localhost:8000")
                    .allowed_origin("https://vallenatofemenino.com")
                    .allowed_origin("https://paginaandrea-actixweb-yqoj6d-251f51-76-13-106-226.traefik.me")
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
                    .allowed_headers(vec!["Content-Type", "Authorization"])
                    .supports_credentials()
                    .max_age(3600)
            )
            .service(
                scope("/back")
                    .service(auth_scope())
                    .service(course_scope())
                    .service(
                        global_scope()
                    )
            )
            .service(Files::new("/assets", &assets_path)) 
            .service(Files::new("/", &static_path)
                        .index_file("index.html")
                        .default_handler(web::to(index_fallback)
                    ))
    })
        .workers(2)
        .bind("0.0.0.0:8000")?
        .run().await
}