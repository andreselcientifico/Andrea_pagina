use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize};
use uuid::Uuid;
use crate::{AppState, errors::error::HttpError, db::db::NotificationExt};
use std::sync::Arc;

// DTOs para notificaciones
#[derive(Deserialize)]
pub struct CreateNotificationRequest {
    pub user_id: Uuid,
    pub title: String,
    pub message: String,
    pub sent_via: String,
}

#[derive(Deserialize)]
pub struct MarkAsReadRequest {
    pub read: bool,
}

// Obtener notificaciones del usuario
pub async fn get_notifications(
    app_state: web::Data<Arc<AppState>>,
    user_id: web::Path<Uuid>,
) -> Result<HttpResponse, HttpError> {
    let notifications = app_state.db_client
        .get_user_notifications(*user_id)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    Ok(HttpResponse::Ok().json(notifications))
}

// Marcar notificación como leída
pub async fn mark_notification_as_read(
    app_state: web::Data<Arc<AppState>>,
    notification_id: web::Path<Uuid>,
) -> Result<HttpResponse, HttpError> {
    app_state.db_client
        .mark_notification_read(*notification_id)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({"status": "success"})))
}

// Crear notificación (admin)
pub async fn create_notification(
    app_state: web::Data<Arc<AppState>>,
    req: web::Json<CreateNotificationRequest>,
) -> Result<HttpResponse, HttpError> {
    let notification = app_state.db_client
        .create_notification(req.user_id, &req.title, &req.message, &req.sent_via)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    Ok(HttpResponse::Created().json(notification))
}