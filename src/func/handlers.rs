use actix_web::{ post, web, HttpResponse, http::header, HttpRequest, HttpMessage };
use crate::{AppState, CachedToken};
use serde::Deserialize;
use std::{ env};
use serde_json::json;
use chrono::{ Duration, Utc };
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::models::User;
use crate::auth::auth::{ hash_password, verify_password, generate_jwt };
use crate::middleware::middleware::JWTAuthMiddleware;  




// ===================== //
//  Obtener token con cache
// ===================== //
async fn get_paypal_token(state: &AppState) -> String {
    let mut cache = state.token_cache.lock().await;

    // Si hay un token válido, devolverlo
    if let Some(cached) = cache.as_ref() {
        if cached.is_valid() {
            return cached.access_token.clone();
        }
    }

    // Caso contrario, pedir uno nuevo
    let resp = state.client
        .post(format!("{}/v1/oauth2/token", state.env.paypal_api_mode))
        .basic_auth(&state.env.paypal_client_id, Some(&state.env.paypal_secret))
        .form(&[("grant_type", "client_credentials")])
        .send().await
        .expect("Error solicitando token");

    let json: serde_json::Value = resp.json().await.expect("Error parseando JSON de token");
    let access_token = json["access_token"]
        .as_str()
        .expect("No se encontró access_token")
        .to_string();
    let expires_in = json["expires_in"].as_i64().unwrap_or(300); // segundos

    // Guardar en cache
    let new_token = CachedToken {
        access_token: access_token.clone(),
        expires_at: Utc::now() + Duration::seconds(expires_in - 60), // margen 1min
    };
    *cache = Some(new_token);

    access_token
}

#[derive(Deserialize)]
pub struct RegisterInput {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginInput {
    pub email: String,
    pub password: String,
}

/// Registrar usuario
#[post("/register")]
pub async fn register_user(
    pool: web::Data<PgPool>,
    data: web::Json<RegisterInput>
) -> HttpResponse {
    let password_hash = hash_password(&data.password);

    let user_id = Uuid::new_v4();

    let query = sqlx
        ::query_as::<_, User>(
            "INSERT INTO users (id, username, email, password_hash, created_at, updated_at)
        VALUES ($1, $2, $3, $4, NOW(), NOW())
        RETURNING id, username, email, password_hash, created_at, updated_at"
        )
        .bind(user_id)
        .bind(&data.username)
        .bind(&data.email)
        .bind(&password_hash)
        .fetch_one(pool.get_ref()).await;

    match query {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}

/// Login usuario
#[post("/login")]
pub async fn login_user(pool: web::Data<PgPool>, data: web::Json<LoginInput>) -> HttpResponse {
    let query = sqlx
        ::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
        .bind(&data.email)
        .fetch_one(pool.get_ref()).await;

    match query {
        Ok(user) => {
            if verify_password(&data.password, &user.password) {
                let token = generate_jwt(&user.id.to_string());
                HttpResponse::Ok().json(serde_json::json!({ "token": token }))
            } else {
                HttpResponse::Unauthorized().body("Credenciales inválidas")
            }
        }
        Err(_) => HttpResponse::Unauthorized().body("Usuario no encontrado"),
    }
}

/// Obtener perfil
#[post("/profile")]
pub async fn get_user_profile(req: HttpRequest) -> HttpResponse {
    let extensions = req.extensions();
    let user_data = extensions
        .get::<JWTAuthMiddleware>()
        .expect("Usuario no autenticado");
    
    HttpResponse::Ok().json(&user_data.user)
}

// ===================== //
//   Crear orden
// ===================== //
#[actix_web::get("/create-order")]
async fn created_order(state: web::Data<AppState>) -> HttpResponse {
    let host_env = env::var("HOST").expect("HOST no está definido en .env");

    let body =
        json!({
        "intent": "CAPTURE",
        "purchase_units": [{
            "amount": { "currency_code": "USD", "value": "100.00" }
        }],
        "application_context": {
            "brand_name": "Mi tienda en línea",
            "landing_page": "NO_PREFERENCE",
            "user_action": "PAY_NOW",
            "return_url": format!("{}capture-order", host_env),
            "cancel_url": format!("{}cancel-order", host_env)
        }
    });

    let access_token = get_paypal_token(&state).await;

    let res = state.client
        .post(format!("{}/v2/checkout/orders", state.env.paypal_api_mode))
        .bearer_auth(&access_token)
        .json(&body)
        .send().await
        .expect("Error al enviar la solicitud a PayPal");

    let response_json: serde_json::Value = res
        .json().await
        .expect("Error al leer la respuesta de PayPal");

    // Buscar link de aprobación
    if let Some(links) = response_json["links"].as_array() {
        if let Some(approve) = links.iter().find(|l| l["rel"] == "approve") {
            if let Some(href) = approve["href"].as_str() {
                return HttpResponse::Found()
                    .append_header(("Location", href)) // redirección HTTP 302
                    .finish();
            }
        }
    }

    HttpResponse::InternalServerError().body(
        "No se encontró link de aprobación en la respuesta de PayPal"
    )
}

// ===================== //
//   Capturar orden
// ===================== //
#[derive(Deserialize)]
struct CaptureParams {
    token: String,
}

#[actix_web::get("/capture-order")]
async fn capture_order(params: web::Query<CaptureParams>) -> HttpResponse {
    HttpResponse::Ok()
        .content_type(header::ContentType::html())
        .body(
            format!("Orden capturada exitosamente. ¡Gracias por su compra! Token: {}", params.token)
        )
}

// ===================== //
//   Cancelar orden
// ===================== //
#[derive(Deserialize)]
struct CancelParams {
    token: String,
}

#[actix_web::get("/cancel-order")]
async fn cancel_order(params: web::Query<CancelParams>) -> HttpResponse {
    HttpResponse::Ok()
        .content_type(header::ContentType::html())
        .body(format!("Orden cancelada. ¡Gracias por su visita! Token: {}", params.token))
}
