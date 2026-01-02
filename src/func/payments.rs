use std::{sync::Arc};
use actix_web::{
    HttpRequest, HttpResponse, post, web::{self, Data, Path, ReqData}
};
use serde_json::{Value, json};
use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::{
    AppState, 
    CachedToken, 
    config::dtos::ProductDTO, 
    db::db::{CourseExt, CoursePurchaseExt}, 
    errors::error::HttpError, 
    middleware::middleware::{ JWTAuthMiddleware}
};

// ===================== //
//  Obtener token con cache
// ===================== //
/// Obtiene un token de PayPal usando cache en memoria.
/// - Usa RwLock para permitir múltiples lectores concurrentes
/// - Evita I/O dentro del lock
/// - Renueva el token solo cuando expira
pub async fn get_paypal_token(state: &AppState) -> String {
    // =========================
    // 1️⃣ PRIMER CHECK (lectura concurrente, rápido)
    // =========================
    {
        let cache = state.token_cache.read().await;

        if let Some(cached) = cache.as_ref() {
            if cached.is_valid() {
                return cached.access_token.clone();
            }
        }
    } // 🔓 el lock de lectura se libera aquí

     // =========================
    // 2️⃣ Solicitar nuevo token (SIN lock)
    // =========================
    let resp = state.client
        .post(format!("{}/v1/oauth2/token", state.env.paypal_api_mode))
        .basic_auth(
            &state.env.paypal_client_id,
            Some(&state.env.paypal_secret)
        )
        .form(&[("grant_type", "client_credentials")])
        .send()
        .await
        .expect("Error solicitando token PayPal");

    let json: serde_json::Value = resp
        .json()
        .await
        .expect("Error parseando JSON de token PayPal");

    let access_token = json["access_token"]
        .as_str()
        .expect("No se encontró access_token")
        .to_string();

    let expires_in = json["expires_in"].as_i64().unwrap_or(3600);

    let new_token = CachedToken {
        access_token: access_token.clone(),
        expires_at: Utc::now() + Duration::seconds(expires_in - 60),
    };

    // ==================================================
    // 3️⃣ SEGUNDO CHECK (lock de escritura corto)
    // ==================================================
    {
        let mut cache = state.token_cache.write().await;

        // Otro request pudo haber renovado el token
        if let Some(cached) = cache.as_ref() {
            if cached.is_valid() {
                return cached.access_token.clone();
            }
        }

        // Nadie lo renovó → este request lo guarda
        *cache = Some(new_token);
    }


    access_token
}


pub async fn create_product(
    app_state: Data<Arc<AppState>>,
    body: ProductDTO,
) -> Result<String, HttpError> {
       let access_token = get_paypal_token(&app_state).await;

       let res = app_state.client
           .post(format!("{}/v1/catalogs/products", app_state.env.paypal_api_mode))
           .bearer_auth(access_token)
           .header("Content-Type", "application/json")
           .json(&body)
           .send()
           .await
           .map_err(|e | HttpError::server_error(e.to_string()))?;

        let status = res.status();
        let text = res.text().await.unwrap_or_default();

        log::debug!("PayPal status: {} ", status);
        log::debug!("PayPal body: {} ", text);

        if !status.is_success() {
            let error_message = format!("PayPal API error: {} - {}", status, text);
            log::error!("{}", error_message);
            return Err(HttpError::server_error(error_message));
        }

        let product_response: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| HttpError::server_error(e.to_string()))?;
        let product_id = product_response.get("id").and_then(|v| v.as_str())
                .ok_or_else(|| HttpError::server_error(format!("Invalid Paypal response: {}", product_response)))?
                .to_string();

        return Ok(product_id);
}

pub async fn paypal_webhook(
    app_state: Data<Arc<AppState>>,
    body: web::Bytes,
    req: HttpRequest,
) -> Result<HttpResponse, HttpError> {
    // Extraer todas las cabeceras de firma que PayPal envía
    let transmission_id = req
        .headers()
        .get("PAYPAL-TRANSMISSION-ID")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| HttpError::bad_request("Missing PAYPAL-TRANSMISSION-ID"))?;

    let transmission_sig = req
        .headers()
        .get("PAYPAL-TRANSMISSION-SIG")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| HttpError::bad_request("Missing PAYPAL-TRANSMISSION-SIG"))?;

    let transmission_time = req
        .headers()
        .get("PAYPAL-TRANSMISSION-TIME")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| HttpError::bad_request("Missing PAYPAL-TRANSMISSION-TIME"))?;

    let cert_url = req
        .headers()
        .get("PAYPAL-CERT-URL")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| HttpError::bad_request("Missing PAYPAL-CERT-URL"))?;

    let auth_algo = req
        .headers()
        .get("PAYPAL-AUTH-ALGO")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| HttpError::bad_request("Missing PAYPAL-AUTH-ALGO"))?;

    // Verificar la firma con tu función (más abajo)
    let is_verified = verify_paypal_webhook_signature(
        &app_state,
        transmission_id,
        transmission_sig,
        transmission_time,
        cert_url,
        auth_algo,
        &body,
    ).await;

    if !is_verified {
        return Err(HttpError::bad_request("Invalid PayPal webhook signature"));
    }

    let event: serde_json::Value = serde_json::from_slice(&body)
        .map_err(|e| HttpError::bad_request(format!("Invalid payload: {}", e)))?;
    log::info!("Received PayPal webhook event: {:?}", event);
    match event["event_type"].as_str() {
        /* --- PAGOS DE PRODUCTOS / ORDENES --- */
        Some("PAYMENT.CAPTURE.COMPLETED") => {
            // Pago exitoso → concede acceso al curso
            log::info!("Payment completed event received.");
             Ok(HttpResponse::Ok().finish())
        }
        Some("PAYMENT.CAPTURE.DENIED") => {
            // Pago denegado
             Ok(HttpResponse::Ok().finish())
        }
        Some("PAYMENT.CAPTURE.PENDING") => {
            // Pago pendiente
             Ok(HttpResponse::Ok().finish())
        }
        Some("PAYMENT.CAPTURE.REFUNDED") => {
            // Reembolso de pago
             Ok(HttpResponse::Ok().finish())
        }
        Some("PAYMENT.CAPTURE.REVERSED") => {
            // Reversión de pago (disputa)
             Ok(HttpResponse::Ok().finish())
        }
        Some("CHECKOUT.ORDER.APPROVED") => {
            // Orden aprobada pero no capturada aún
             Ok(HttpResponse::Ok().finish())
        }

        /* --- SUSCRIPCIONES --- */
        Some("BILLING.SUBSCRIPTION.CREATED") => {
            // Suscripción creada
             Ok(HttpResponse::Ok().finish())
        }
        Some("BILLING.SUBSCRIPTION.ACTIVATED") => {
            // Activar acceso del plan
             Ok(HttpResponse::Ok().finish())
        }
        Some("BILLING.SUBSCRIPTION.UPDATED") => {
            // Actualizar plan/estado
             Ok(HttpResponse::Ok().finish())
        }
        Some("BILLING.SUBSCRIPTION.CANCELLED") => {
            // Marcar para terminar acceso al final del ciclo
             Ok(HttpResponse::Ok().finish())
        }
        Some("BILLING.SUBSCRIPTION.EXPIRED") => {
            // Fin de suscripción ya efectiva
             Ok(HttpResponse::Ok().finish())
        }
        Some("BILLING.SUBSCRIPTION.SUSPENDED") => {
            // Suspensión por pagos fallidos
             Ok(HttpResponse::Ok().finish())
        }
        Some("BILLING.SUBSCRIPTION.PAYMENT.FAILED") => {
            // Un pago recurrente falló
             Ok(HttpResponse::Ok().finish())
        }
        Some("PAYMENT.SALE.COMPLETED") => {
            // Pago de ciclo recurrente exitoso
             Ok(HttpResponse::Ok().finish())
        }
        Some("PAYMENT.SALE.REFUNDED") => {
            // Reembolso de pago recurrente
             Ok(HttpResponse::Ok().finish())
        }
        Some("PAYMENT.SALE.REVERSED") => {
            // Reversión de pago recurrente
             Ok(HttpResponse::Ok().finish())
        }

        /* --- OTROS EVENTOS GENERALES --- */
        Some("PAYMENT.ORDER.CANCELLED") => {
            // Orden cancelada antes de pago
             Ok(HttpResponse::Ok().finish())
        }
        Some("PAYMENT.AUTHORIZATION.CREATED") => {
            // Autorización de pago iniciada
             Ok(HttpResponse::Ok().finish())
        }
        Some("PAYMENT.AUTHORIZATION.VOIDED") => {
            // Autorización de pago anulada
             Ok(HttpResponse::Ok().finish())
        }
        // _ => {
        //     // Evento no manejado explícitamente
        //      Ok(HttpResponse::Ok().finish())
        // }
        _ => Err(HttpError::bad_request("Unsupported event type"))
    }
}


pub async fn verify_paypal_webhook_signature(
    app_state: &Data<Arc<AppState>>,
    transmission_id: &str,
    transmission_sig: &str,
    transmission_time: &str,
    cert_url: &str,
    auth_algo: &str,
    body: &web::Bytes,
) -> bool {
    // Construye el JSON para PayPal
    let webhook_event: serde_json::Value = match serde_json::from_slice(body) {
        Ok(val) => val,
        Err(_) => return false,
    };

    #[derive(serde::Serialize)]
    struct VerifyRequest<'a> {
        transmission_id: &'a str,
        transmission_time: &'a str,
        cert_url: &'a str,
        auth_algo: &'a str,
        transmission_sig: &'a str,
        webhook_id: &'a str,
        webhook_event: serde_json::Value,
    }

    let verify_body = VerifyRequest {
        transmission_id,
        transmission_time,
        cert_url,
        auth_algo,
        transmission_sig,
        webhook_id: &app_state.env.paypal_webhook_id,
        webhook_event,
    };

    // Obtiene token OAuth2 para PayPal
    let token = get_paypal_token(&app_state).await;

    let client = reqwest::Client::new();
    let url = format!("{}/v1/notifications/verify-webhook-signature", app_state.env.paypal_api_mode);

    let resp = match client
        .post(&url)
        .bearer_auth(token)
        .json(&verify_body)
        .send()
        .await
    {
        Ok(r) => r,
        Err(_) => return false,
    };

    if let Ok(json) = resp.json::<serde_json::Value>().await {
        if json.get("verification_status").and_then(|v| v.as_str()) == Some("SUCCESS") {
            return true;
        }
    }
    false
}


// ===================== //
//   Crear orden
// ===================== //
pub async fn created_order(
    state: Data<Arc<AppState>>, 
    path: Path<(Uuid,)>,
) -> HttpResponse {
    let course_id = path.into_inner().0;
    log::info!("creando orden");
    let course = match state.db_client.get_course(course_id).await {
        Ok(c) => c,
        Err(e) => {
            log::error!("Error: {}", e);
            return HttpResponse::NotFound().body("Curso no encontrado");
        }
    };
    let invoice_id = Uuid::new_v4().to_string();
    let (paypal_product_id , title, price) = match course {
        Some(c) => (c.paypal_product_id.clone(), c.title.clone(), c.price),
        None => {
            log::error!("Curso no encontrado");
            return HttpResponse::NotFound().body("Curso no encontrado");
        }
    };

    let body =
        json!({
        "intent": "CAPTURE",
        "payment_source": {
            "paypal": {
                "experience_context": {
                    "payment_method_preference": "IMMEDIATE_PAYMENT_REQUIRED",
                    "landing_page": "LOGIN",
                    "user_action": "PAY_NOW",
                    "return_url": format!("{}/paypal/capture?course_id={}", state.env.host, course_id),
                    "cancel_url": format!("{}/paypal/cancel?course_id={}", state.env.host, course_id)
                }
            }
        },
        "purchase_units": [{
            "invoice_id": invoice_id,
            "custom_id": course_id.to_string(),
            "amount": {
                "currency_code": "USD",
                "value": format!("{:.2}", price),
                "breakdown": {
                    "item_total": {
                        "currency_code": "USD",
                        "value": format!("{:.2}", price)
                    }
                }
            },
            "items": [{
                "name": title,
                "description": "Curso completo",
                "unit_amount": {
                    "currency_code": "USD",
                    "value": format!("{:.2}", price)
                },
                "quantity": "1",
                "category": "DIGITAL_GOODS",
                "sku": paypal_product_id
            }]
        }]
    });

    let access_token = get_paypal_token(&state).await;

    let res = state.client
        .post(format!("{}/v2/checkout/orders", state.env.paypal_api_mode))
        .bearer_auth(&access_token)
        .json(&body)
        .send().await
        .expect("Error al enviar la solicitud a PayPal");

    if res.status().is_client_error() || res.status().is_server_error() {
        log::error!("Respuesta inválida de PayPal: {:?}", res);
        return HttpResponse::InternalServerError().body("Error creating order");
    }

    let response_json: Value = match res.json().await {
        Ok(v) => v,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .body("Respuesta inválida de PayPal");
        }
    };
    let order_id = match response_json.get("id").and_then(|v| v.as_str()) {
        Some(id) => id.to_string(),
        None => {
            log::error!("PayPal no devolvió order id: {:?}", response_json);
            return HttpResponse::InternalServerError()
                .body("PayPal no devolvió order id");
        }
    };

    // Responder sólo con orderID
    HttpResponse::Ok().json(json!({ "id": order_id }))

}

// ===================== //
//   Capturar orden
// ===================== //

#[post("/paypal/capture/{order_id}")]
async fn capture_order(
    path: Path<(String,)>, 
    app_state: Data<Arc<AppState>>,
    user: ReqData<JWTAuthMiddleware>,
) -> HttpResponse {
    let order_id = path.into_inner().0;
    let user_id = user.user.id;
    let access_token = get_paypal_token(&app_state).await;

    let res =match  app_state.client
        .post(format!("{}/v2/checkout/orders/{}/capture", app_state.env.paypal_api_mode, order_id))
        .bearer_auth(&access_token)
        .header("Content-Type", "application/json")
        .body("{}")
        .send()
        .await
    {
        Ok(res) => res,
        Err(err) => {
            return HttpResponse::InternalServerError().json(json!({
                "error": format!("Error al capturar la orden: {:?}", err)
            }));
        }
    };

    if !res.status().is_success() {
        let error_body = match res.text().await {
            Ok(text) => text,
            Err(_) => "Error desconocido de PayPal".to_string(),
        };
        return HttpResponse::BadRequest().json(json!({
            "error": format!("PayPal devolvió un error: {}", error_body)
        }));
    }

    let data: serde_json::Value = match res.json().await {
        Ok(json) => json,
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({
                "error": format!("Error al parsear la respuesta de PayPal: {}", e)
            }));
        }
    };
    // Extraer el status de la respuesta de PayPal
    let status = data["status"].as_str().unwrap_or("").to_string();
    // Extraer el course_id del custom_id en purchase_units
   let custom_id = data["purchase_units"][0]["payments"]["captures"][0]["custom_id"]
    .as_str()
    .unwrap_or("");
    let course_id = match Uuid::parse_str(custom_id) {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::BadRequest().json(json!({
                "error": format!("No se pudo obtener el ID del curso de la orden de PayPal: {}", e)
            }));
        }
    };
    let amount = match data["purchase_units"][0]["amount"]["value"].as_str() {
        Some(value) => match value.parse::<i64>() {
            Ok(amount) => amount,
            Err(_) => 0,
        },
        None => 0,
    };

     if status == "COMPLETED" {
        match app_state.db_client.register_course_purchase(
            user_id,
            course_id,
            order_id.clone(),
            amount,
            "paypal".to_string(),
            status.clone(),
        ).await {
            Ok(_) => (),
            Err(e) => {
                return HttpResponse::InternalServerError().json(json!({
                    "error": format!("Error al registrar la compra: {}", e)
                }));
            }
        };
    } else {
        return HttpResponse::BadRequest().json(json!({
            "error": "El pago no se completó exitosamente"
        }));
    }
    // Devolver un objeto con el status y otros datos relevantes
    HttpResponse::Ok().json(json!({
        "status": status,
        "order_id": order_id,
        "data": data  // Opcional: devolver toda la respuesta de PayPal si es necesario
    }))
}