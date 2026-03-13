mod auth;
mod config;
mod db;
mod errors;
mod func;
mod mail;
mod middleware;
mod models;
mod routes;
mod services;
mod test;
mod utils;

use crate::func::inbox::resend_webhook;
use crate::func::payments::paypal_webhook;
use crate::routes::routes::{auth_scope, course_scope, global_scope};
use actix_files::Files;
use actix_service::Service;
use actix_web::http::header::{CACHE_CONTROL, HeaderValue};
use actix_web::web::{post, resource, scope};
use actix_web::{
    App, HttpResponse, HttpServer,
    web::{Data, Json},
};
use actix_web::{Responder, middleware::Compress, web};
use chrono::{DateTime, Utc};
use config::config::Config;
use db::db::DBClient;
use dotenvy;
// use env_logger::Env;
use reqwest::Client;
use serde_json::Value;
use services::paypal_client::PayPalClient;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tera::{Context, Tera};
use tokio::sync::RwLock;
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
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "message": "pong",
    }))
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

fn looks_hashed_asset(path: &str) -> bool {
    // heurística simple: "-<8+ chars>." antes de la extensión
    // ejemplos: photo1-DzpscOSY.webp, index-gcXjKfVp.js
    // no ejemplos: photo1-medium.webp, hero-background-medium.webp
    let p = path.rsplit('/').next().unwrap_or(path);
    // busca un guion y al menos 8 chars antes del punto final
    if let Some((_, rest)) = p.split_once('-') {
        if let Some((maybe_hash, _ext)) = rest.split_once('.') {
            return maybe_hash.len() >= 8
                && maybe_hash
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_');
        }
    }
    false
}

// ===================== //
//        MAIN
// ===================== //
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if let Err(e) = dotenvy::dotenv() {
        log::warn!(
            "No se cargó el archivo .env (esto es normal en producción): {}",
            e
        );
    }
    //env_logger::Builder::from_env(Env::default().default_filter_or("debug,actix_server=info"))
    //   .init();

    // Crear conexión a Postgres
    let config = Config::init();
    let pool = match PgPoolOptions::new().connect(&config.database_url).await {
        Ok(pool) => pool,
        Err(err) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Database connection failed {}".replace("{}", &err.to_string()),
            ));
        }
    };
    let db: DBClient = DBClient::new(pool.clone());
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("No se pudieron ejecutar las migraciones");
    let paypal_client = PayPalClient::new(
        config.paypal_client_id.clone(),
        config.paypal_secret.clone(),
        config.paypal_api_mode.contains("sandbox"),
    )
    .await;

    let dist_path = std::env::current_dir().unwrap().join("dist");

    // 2. Inicializamos Tera vacío y añadimos el index.html manualmente
    let mut tera_md = Tera::default();
    let index_path = dist_path.join("index.html");
    let static_path = format!("{}", dist_path.clone().to_str().unwrap());
    let assets_path = format!("{}/assets", dist_path.clone().to_str().unwrap());

    if index_path.exists() {
        tera_md
            .add_template_file(index_path, Some("index.html"))
            .expect("Error al cargar index.html");
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
            .wrap(Compress::default())
            .wrap(
                actix_cors::Cors::default()
                    .allowed_origin("http://localhost:8000")
                    .allowed_origin("https://vallenatofemenino.com")
                    .allowed_origin(
                        "https://paginaandrea-actixweb-yqoj6d-251f51-76-13-106-226.traefik.me",
                    )
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
                    .allowed_headers(vec![
                        "Content-Type",
                        "Authorization",
                        "PAYPAL-TRANSMISSION-ID",
                        "PAYPAL-TRANSMISSION-SIG",
                        "PAYPAL-TRANSMISSION-TIME",
                        "PAYPAL-CERT-URL",
                        "PAYPAL-AUTH-ALGO",
                    ])
                    .supports_credentials()
                    .max_age(3600),
            )
            .service(
                scope("/back")
                    .service(resource("/payments/webhooks/paypal").route(post().to(paypal_webhook)))
                    .service(resource("/webhooks/resend").route(post().to(resend_webhook)))
                    .service(auth_scope())
                    .service(course_scope())
                    .service(global_scope()),
            )
            .wrap_fn(|req, srv| {
                let path = req.path().to_string();
                let is_assets = path.starts_with("/assets/");
                let is_index_html = path == "/" || !path.starts_with("/back"); // tu SPA fallback

                let fut = srv.call(req);
                async move {
                    let mut res = fut.await?;

                    let cc = if is_assets {
                        if looks_hashed_asset(&path) {
                            "public, max-age=31536000, immutable"
                        } else {
                            "public, max-age=3600, must-revalidate"
                        }
                    } else if is_index_html {
                        // importante para SPA: el HTML no debe quedarse pegado
                        "no-cache"
                    } else {
                        "public, max-age=0"
                    };

                    res.headers_mut()
                        .insert(CACHE_CONTROL, HeaderValue::from_static(cc));
                    Ok(res)
                }
            })
            // /assets
            .service(
                Files::new("/assets", &assets_path)
                    .use_etag(true)
                    .use_last_modified(true),
            )
            // /
            .service(
                Files::new("/", &static_path)
                    .index_file("index.html")
                    .default_handler(web::to(index_fallback))
                    .use_etag(true)
                    .use_last_modified(true),
            )
    })
    .workers(2)
    .bind("0.0.0.0:8000")?
    .run()
    .await
}
