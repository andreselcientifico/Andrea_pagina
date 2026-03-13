use actix_web::{
    HttpMessage, HttpRequest, HttpResponse, post,
    web::{Data, Json, Path},
};
use serde::Serialize;
use std::sync::Arc;
use validator::Validate;

use crate::{
    AppState,
    config::dtos::{CreateEventRequestDTO, Response, UpdateEventStatusDTO},
    db::db::EventExt,
    errors::error::HttpError,
    mail::mails::send_event_request_notification,
    middleware::middleware::JWTAuthMiddleware,
    models::models::UserRole,
};

#[derive(Debug, Serialize)]
pub struct EventListResponse {
    pub success: bool,
    pub events: Vec<crate::config::dtos::EventRequestResponseDto>,
    pub count: usize,
}

/// Public endpoint: anyone can submit an event request
#[post("/event-request")]
pub async fn create_event_request(
    app_state: Data<Arc<AppState>>,
    Json(body): Json<CreateEventRequestDTO>,
) -> Result<HttpResponse, HttpError> {
    body.validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    // Validate event_type
    let valid_types = [
        "boda",
        "cumpleaños",
        "festival",
        "corporativo",
        "serenata",
        "otro",
    ];
    if !valid_types.contains(&body.event_type.as_str()) {
        return Err(HttpError::bad_request(
            "Tipo de evento inválido. Debe ser: boda, cumpleaños, festival, corporativo, serenata u otro".to_string(),
        ));
    }

    let event = app_state
        .db_client
        .create_event_request(
            &body.name,
            &body.email,
            body.phone.as_deref(),
            &body.event_type,
            body.event_date,
            body.location.as_deref(),
            body.guests,
            Some(&body.message),
            body.budget.as_deref(),
        )
        .await
        .map_err(|e| {
            log::error!("Error creating event request: {}", e);
            HttpError::server_error(format!("Error al crear la solicitud: {}", e))
        })?;

    // Send notification email to admin
    if let Err(e) = send_event_request_notification(
        &event.name,
        &event.email,
        event.phone.as_deref().unwrap_or("No proporcionado"),
        &event.event_type,
        event
            .event_date
            .map(|d| d.to_string())
            .as_deref()
            .unwrap_or("Por definir"),
        event.location.as_deref().unwrap_or("Por definir"),
        event
            .guests
            .map(|g| g.to_string())
            .as_deref()
            .unwrap_or("Por definir"),
        event.message.as_deref().unwrap_or(""),
        event.budget.as_deref().unwrap_or("No especificado"),
    )
    .await
    {
        log::error!("Error sending event notification email: {}", e);
        // Don't fail the request if email fails
    }

    Ok(HttpResponse::Created().json(Response {
        status: "success",
        message:
            "¡Solicitud de evento enviada exitosamente! Nos pondremos en contacto contigo pronto."
                .to_string(),
    }))
}

/// Admin endpoint: list all event requests
pub async fn get_event_requests(
    app_state: Data<Arc<AppState>>,
    req: HttpRequest,
) -> Result<HttpResponse, HttpError> {
    let extensions = req.extensions();
    let user_data = extensions
        .get::<JWTAuthMiddleware>()
        .ok_or_else(|| HttpError::unauthorized("Usuario no autenticado".to_string()))?;

    if user_data.claims.role != UserRole::Admin {
        return Err(HttpError::forbidden(
            "Solo administradores pueden ver las solicitudes".to_string(),
        ));
    }

    let events = app_state
        .db_client
        .get_event_requests()
        .await
        .map_err(|e| {
            log::error!("Error getting event requests: {}", e);
            HttpError::server_error(format!("Error obteniendo solicitudes: {}", e))
        })?;

    let count = events.len();

    let response_events: Vec<crate::config::dtos::EventRequestResponseDto> = events
        .into_iter()
        .map(|e| crate::config::dtos::EventRequestResponseDto {
            id: e.id,
            name: e.name,
            email: e.email,
            phone: e.phone,
            event_type: e.event_type,
            event_date: e.event_date,
            location: e.location,
            guests: e.guests,
            message: e.message,
            budget: e.budget,
            status: e.status,
            created_at: e.created_at,
            updated_at: e.updated_at,
        })
        .collect();

    Ok(HttpResponse::Ok().json(EventListResponse {
        success: true,
        events: response_events,
        count,
    }))
}

/// Admin endpoint: update event request status
pub async fn update_event_status(
    app_state: Data<Arc<AppState>>,
    req: HttpRequest,
    path: Path<uuid::Uuid>,
    Json(body): Json<UpdateEventStatusDTO>,
) -> Result<HttpResponse, HttpError> {
    let extensions = req.extensions();
    let user_data = extensions
        .get::<JWTAuthMiddleware>()
        .ok_or_else(|| HttpError::unauthorized("Usuario no autenticado".to_string()))?;

    if user_data.claims.role != UserRole::Admin {
        return Err(HttpError::forbidden(
            "Solo administradores pueden actualizar solicitudes".to_string(),
        ));
    }

    body.validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let valid_statuses = ["pending", "contacted", "confirmed", "rejected"];
    if !valid_statuses.contains(&body.status.as_str()) {
        return Err(HttpError::bad_request(
            "Estado inválido. Debe ser: pending, contacted, confirmed o rejected".to_string(),
        ));
    }

    let event_id = path.into_inner();

    app_state
        .db_client
        .update_event_status(event_id, &body.status)
        .await
        .map_err(|e| {
            log::error!("Error updating event status: {}", e);
            HttpError::server_error(format!("Error actualizando estado: {}", e))
        })?;

    Ok(HttpResponse::Ok().json(Response {
        status: "success",
        message: "Estado de la solicitud actualizado exitosamente".to_string(),
    }))
}
