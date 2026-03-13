use actix_web::{
    HttpMessage, HttpRequest, HttpResponse, post,
    web::{Data, Json, Path},
};
use resend_rs::Resend;
use std::env;
use std::sync::Arc;

use crate::{
    AppState,
    config::dtos::{ReceivedEmailResponseDto, ResendWebhookPayload, Response, ReplyEmailDto},
    db::db::InboxExt,
    errors::error::HttpError,
    middleware::middleware::JWTAuthMiddleware,
    models::models::UserRole,
    mail::sendmail::send_email,
};

/// Public endpoint: Resend webhook to receive inbound emails
pub async fn resend_webhook(
    app_state: Data<Arc<AppState>>,
    Json(payload): Json<ResendWebhookPayload>,
) -> Result<HttpResponse, HttpError> {
    log::info!("Received Resend webhook: {:?}", payload.r#type);

    if payload.r#type == "email.received" {
        let email_id = &payload.data.email_id;
        log::info!("Processing received email with ID: {}", email_id);

        let api_key = env::var("RESEND_API_KEY")
            .map_err(|_| HttpError::server_error("RESEND_API_KEY no configurada".to_string()))?;

        let resend = Resend::new(&api_key);

        // Llamar a la API de Resend para obtener el cuerpo del correo
        let email_details = resend.receiving.get(email_id).await.map_err(|e| {
            log::error!("Error fetching email details from Resend: {}", e);
            HttpError::server_error(format!("Error obteniendo detalles del email: {}", e))
        })?;

        // Guardar el correo en la base de datos
        let db_email = app_state
            .db_client
            .save_received_email(
                &email_details.id,
                &email_details.from,
                &email_details.to.join(", "),
                &email_details.subject,
                email_details.text.as_deref(),
                email_details.html.as_deref(),
            )
            .await
            .map_err(|e| {
                log::error!("Error saving received email to DB: {}", e);
                HttpError::server_error(format!("Error guardando email en DB: {}", e))
            })?;

        log::info!("Successfully saved email {}", db_email.id);
    }

    Ok(HttpResponse::Ok().json(Response {
        status: "success",
        message: "Webhook processed".to_string(),
    }))
}

/// Admin endpoint: list all received emails
pub async fn get_received_emails(
    app_state: Data<Arc<AppState>>,
    req: HttpRequest,
) -> Result<HttpResponse, HttpError> {
    let extensions = req.extensions();
    let user_data = extensions
        .get::<JWTAuthMiddleware>()
        .ok_or_else(|| HttpError::unauthorized("Usuario no autenticado".to_string()))?;

    if user_data.claims.role != UserRole::Admin {
        return Err(HttpError::forbidden(
            "Solo administradores tienen acceso".to_string(),
        ));
    }

    let emails = app_state
        .db_client
        .get_received_emails()
        .await
        .map_err(|e| {
            log::error!("Error getting received emails: {}", e);
            HttpError::server_error(format!("Error obteniendo emails: {}", e))
        })?;

    let response_emails: Vec<ReceivedEmailResponseDto> = emails
        .into_iter()
        .map(|e| ReceivedEmailResponseDto {
            id: e.id,
            resend_email_id: e.resend_email_id,
            from_address: e.from_address,
            to_address: e.to_address,
            subject: e.subject,
            text_content: e.text_content,
            html_content: e.html_content,
            is_read: e.is_read,
            created_at: e.created_at,
        })
        .collect();

    Ok(HttpResponse::Ok().json(response_emails))
}

/// Admin endpoint: mark email as read
pub async fn mark_email_read(
    app_state: Data<Arc<AppState>>,
    req: HttpRequest,
    path: Path<uuid::Uuid>,
) -> Result<HttpResponse, HttpError> {
    let extensions = req.extensions();
    let user_data = extensions
        .get::<JWTAuthMiddleware>()
        .ok_or_else(|| HttpError::unauthorized("Usuario no autenticado".to_string()))?;

    if user_data.claims.role != UserRole::Admin {
        return Err(HttpError::forbidden(
            "Solo administradores tienen acceso".to_string(),
        ));
    }

    let email_id = path.into_inner();

    app_state
        .db_client
        .mark_email_read(email_id)
        .await
        .map_err(|e| {
            log::error!("Error marking email as read: {}", e);
            HttpError::server_error(format!("Error marcando email como leido: {}", e))
        })?;

    Ok(HttpResponse::Ok().json(Response {
        status: "success",
        message: "Email marcado como leido".to_string(),
    }))
}

/// Admin endpoint: delete email
pub async fn delete_received_email(
    app_state: Data<Arc<AppState>>,
    req: HttpRequest,
    path: Path<uuid::Uuid>,
) -> Result<HttpResponse, HttpError> {
    let extensions = req.extensions();
    let user_data = extensions
        .get::<JWTAuthMiddleware>()
        .ok_or_else(|| HttpError::unauthorized("Usuario no autenticado".to_string()))?;

    if user_data.claims.role != UserRole::Admin {
        return Err(HttpError::forbidden(
            "Solo administradores tienen acceso".to_string(),
        ));
    }

    let email_id = path.into_inner();

    app_state
        .db_client
        .delete_received_email(email_id)
        .await
        .map_err(|e| {
            log::error!("Error deleting received email: {}", e);
            HttpError::server_error(format!("Error eliminando email: {}", e))
        })?;

    Ok(HttpResponse::Ok().json(Response {
        status: "success",
        message: "Email eliminado".to_string(),
    }))
}

/// Admin endpoint: reply to a received email
pub async fn reply_to_email(
    app_state: Data<Arc<AppState>>,
    req: HttpRequest,
    path: Path<uuid::Uuid>,
    Json(body): Json<ReplyEmailDto>,
) -> Result<HttpResponse, HttpError> {
    let extensions = req.extensions();
    let user_data = extensions
        .get::<JWTAuthMiddleware>()
        .ok_or_else(|| HttpError::unauthorized("Usuario no autenticado".to_string()))?;

    if user_data.claims.role != UserRole::Admin {
        return Err(HttpError::forbidden(
            "Solo administradores tienen acceso".to_string(),
        ));
    }

    let email_id = path.into_inner();

    // Verify email exists
    let emails = app_state
        .db_client
        .get_received_emails()
        .await
        .map_err(|e| {
            log::error!("Error fetching emails: {}", e);
            HttpError::server_error(format!("Error obteniendo emails: {}", e))
        })?;

    let _email = emails
        .iter()
        .find(|e| e.id == email_id)
        .ok_or_else(|| HttpError::not_found("Email no encontrado".to_string()))?;

    // Send reply from the Resend reply email (not from admin email)
    let from_email = "reply@vallenatofemenino.com".to_string();

    // Get HTML or text content for the email body
    let body_html = body
        .html_content
        .clone()
        .unwrap_or_else(|| {
            format!(
                "<html><body>{}</body></html>",
                body.text_content.as_ref().unwrap_or(&String::new())
            )
        });

    // Send reply email using Resend
    send_email(
        &body.to_address,
        &body.subject,
        &body_html,
        &[],
        from_email,
        Some("eventos@oibauzorio.resend.app")
    )
    .await
    .map_err(|e| {
        log::error!("Error sending reply email: {}", e);
        HttpError::server_error(format!("Error enviando respuesta: {}", e))
    })?;

    Ok(HttpResponse::Ok().json(Response {
        status: "success",
        message: "Respuesta enviada correctamente".to_string(),
    }))
}
