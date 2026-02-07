use std::sync::Arc;
use actix_web::{HttpMessage, HttpRequest, HttpResponse, Responder, web::{Data, Json}};
use serde::{Deserialize, Serialize};
use tokio::spawn;

use crate::{
    db::db::UserExt,
    errors::error::HttpError,
    mail::mails::send_admin_bulk_email,
    middleware::middleware::JWTAuthMiddleware,
    models::models::UserRole,
    AppState,
};

// =============================================
// DTOs
// =============================================

#[derive(Debug, Deserialize)]
pub struct SendBulkEmailDto {
    /// Type of notification preference to target:
    /// "email_notifications", "course_reminders", "new_content"
    pub notification_type: String,
    pub subject: String,
    pub html_content: String,
}

#[derive(Debug, Serialize)]
pub struct BulkEmailResponse {
    pub success: bool,
    pub message: String,
    pub recipients_count: usize,
}

// =============================================
// Handlers
// =============================================

/// Admin endpoint to send bulk emails to users based on notification preferences
pub async fn send_bulk_email(
    app_state: Data<Arc<AppState>>,
    req: HttpRequest,
    Json(body): Json<SendBulkEmailDto>,
) -> Result<impl Responder, HttpError> {
    // Check if user is admin
    let extensions = req.extensions();
    let user_data = extensions
        .get::<JWTAuthMiddleware>()
        .ok_or_else(|| HttpError::unauthorized("Usuario no autenticado".to_string()))?;

    if user_data.user.role != UserRole::Admin {
        return Err(HttpError::forbidden("Solo administradores pueden enviar correos masivos".to_string()));
    }

    // Validate input
    if body.subject.is_empty() {
        return Err(HttpError::bad_request("El asunto es requerido".to_string()));
    }
    if body.html_content.is_empty() {
        return Err(HttpError::bad_request("El contenido es requerido".to_string()));
    }

    // Get users with the specified notification preference
    let users = app_state
        .db_client
        .get_users_by_notification_type(&body.notification_type)
        .await
        .map_err(|e| {
            log::error!("Error getting users by notification type: {}", e);
            HttpError::server_error(format!("Error obteniendo usuarios: {}", e))
        })?;

    let recipients_count = users.len();

    if recipients_count == 0 {
        return Ok(HttpResponse::Ok().json(BulkEmailResponse {
            success: true,
            message: "No hay usuarios con esta preferencia de notificación activa".to_string(),
            recipients_count: 0,
        }));
    }

    // Clone values for async task
    let subject = body.subject.clone();
    let html_content = body.html_content.clone();

    // Spawn background task to send emails
    spawn(async move {
        for (email, name) in users {
            match send_admin_bulk_email(&email, &name, &subject, &html_content).await {
                Ok(_) => {
                    log::info!("Email sent successfully to: {}", email);
                }
                Err(e) => {
                    log::error!("Failed to send email to {}: {}", email, e);
                }
            }
            // Small delay to avoid overwhelming SMTP server
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
        log::info!("Bulk email sending completed");
    });

    Ok(HttpResponse::Ok().json(BulkEmailResponse {
        success: true,
        message: format!("Enviando correos a {} usuarios en segundo plano", recipients_count),
        recipients_count,
    }))
}

/// Get count of users by notification type (for preview before sending)
#[derive(Debug, Serialize)]
pub struct NotificationCountResponse {
    pub notification_type: String,
    pub count: usize,
}

pub async fn get_notification_recipients_count(
    app_state: Data<Arc<AppState>>,
    req: HttpRequest,
    notification_type: actix_web::web::Path<String>,
) -> Result<impl Responder, HttpError> {
    // Check if user is admin
    let extensions = req.extensions();
    let user_data = extensions
        .get::<JWTAuthMiddleware>()
        .ok_or_else(|| HttpError::unauthorized("Usuario no autenticado".to_string()))?;

    if user_data.user.role != UserRole::Admin {
        return Err(HttpError::forbidden("Solo administradores pueden ver esta información".to_string()));
    }

    let users = app_state
        .db_client
        .get_users_by_notification_type(&notification_type)
        .await
        .map_err(|e| {
            log::error!("Error getting users count: {}", e);
            HttpError::server_error(format!("Error obteniendo conteo: {}", e))
        })?;

    Ok(HttpResponse::Ok().json(NotificationCountResponse {
        notification_type: notification_type.into_inner(),
        count: users.len(),
    }))
}
