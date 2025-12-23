use std::{sync::Arc};
use actix_web::{
    HttpRequest, HttpResponse, web::{self, Data, scope}
};

use crate::{
    AppState,
    config::dtos::{ ProductDTO},
    errors::error::{HttpError},
    middleware::middleware::{AuthMiddlewareFactory,  RoleCheck},
    models::models::UserRole,
    func::handlers::get_paypal_token,
};

pub fn payments_scope(app_state: Arc<AppState>) -> impl actix_web::dev::HttpServiceFactory {
    scope("/payments")
        .service(
            scope("")
                .wrap(AuthMiddlewareFactory::new(app_state.clone()))
        )
        .service(
            scope("")
                .wrap(AuthMiddlewareFactory::new(app_state.clone()))
                .wrap(RoleCheck::new(vec![UserRole::Admin]))
        )
        // webhook público (proveedor) — valida firma en handler si aplica
        .route("/webhooks/paypal", web::post().to(paypal_webhook))
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
    print!("Received PayPal webhook event: {:?}", event);
    match event["event_type"].as_str() {
        /* --- PAGOS DE PRODUCTOS / ORDENES --- */
        Some("PAYMENT.CAPTURE.COMPLETED") => {
            // Pago exitoso → concede acceso al curso
            print!("Payment completed event received.");
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
