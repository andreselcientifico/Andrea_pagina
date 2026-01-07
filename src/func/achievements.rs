use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize};
use uuid::Uuid;
use crate::{AppState, db::db::{AchievementExt, UserAchievementExt, UserExt}, errors::error::HttpError};
use std::sync::Arc;

// DTOs para logros
#[derive(Deserialize)]
pub struct CreateAchievementRequest {
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub trigger_type: String,
    pub trigger_value: i32,
    pub active: Option<bool>,
}

#[derive(Deserialize)]
pub struct UpdateAchievementRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub trigger_type: Option<String>,
    pub trigger_value: Option<i32>,
    pub active: Option<bool>,
}

#[derive(Deserialize)]
pub struct AssignAchievementRequest {
    pub user_id: Uuid,
    pub achievement_id: Uuid,
}

#[derive(Deserialize)]
pub struct EarnAchievementRequest {
    pub user_id: Uuid,
    pub achievement_id: Uuid,
}

// Crear un nuevo logro (solo admin)
pub async fn create_achievement(
    app_state: web::Data<Arc<AppState>>,
    req: web::Json<CreateAchievementRequest>,
) -> Result<HttpResponse, HttpError> {
    let achievement = app_state.db_client
        .create_achievement(&req.name, req.description.as_ref(), req.icon.as_ref(), &req.trigger_type, req.trigger_value, req.active.unwrap_or(true))
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    Ok(HttpResponse::Created().json(achievement))
}

// Obtener todos los logros
pub async fn get_achievements(
    app_state: web::Data<Arc<AppState>>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> Result<HttpResponse, HttpError> {
    let page = query.get("page").and_then(|p| p.parse().ok()).unwrap_or(1);
    let limit = query.get("limit").and_then(|l| l.parse().ok()).unwrap_or(10);

    let achievements = app_state.db_client
        .get_achievements(page, limit)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    Ok(HttpResponse::Ok().json(achievements))
}

// Asignar logro a usuario
pub async fn assign_achievement_to_user(
    app_state: web::Data<Arc<AppState>>,
    req: web::Json<AssignAchievementRequest>,
) -> Result<HttpResponse, HttpError> {
    let user_achievement = app_state.db_client
        .assign_achievement_to_user(req.user_id, req.achievement_id)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    Ok(HttpResponse::Ok().json(user_achievement))
}

// Marcar logro como ganado
pub async fn earn_achievement(
    app_state: web::Data<Arc<AppState>>,
    req: web::Json<EarnAchievementRequest>,
) -> Result<HttpResponse, HttpError> {
    let user_achievement = app_state.db_client
        .earn_achievement(req.user_id, req.achievement_id)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    Ok(HttpResponse::Ok().json(user_achievement))
}

// Obtener logros de un usuario
pub async fn get_user_achievements(
    app_state: web::Data<Arc<AppState>>,
    user_id: web::Path<Uuid>,
) -> Result<HttpResponse, HttpError> {
    let achievements = app_state.db_client
        .get_user_achievements(*user_id)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    Ok(HttpResponse::Ok().json(achievements))
}

// Obtener un logro específico
pub async fn get_achievement(
    app_state: web::Data<Arc<AppState>>,
    achievement_id: web::Path<Uuid>,
) -> Result<HttpResponse, HttpError> {
    let achievement = app_state.db_client
        .get_achievement(*achievement_id)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    match achievement {
        Some(a) => Ok(HttpResponse::Ok().json(a)),
        None => Err(HttpError::not_found("Logro no encontrado".to_string())),
    }
}

// Actualizar un logro
pub async fn update_achievement(
    app_state: web::Data<Arc<AppState>>,
    achievement_id: web::Path<Uuid>,
    req: web::Json<UpdateAchievementRequest>,
) -> Result<HttpResponse, HttpError> {
    let achievement = app_state.db_client
        .update_achievement(*achievement_id, req.name.as_ref(), req.description.as_ref(), req.icon.as_ref(), req.trigger_type.as_ref().map(|s| s.as_str()), req.trigger_value, req.active)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    Ok(HttpResponse::Ok().json(achievement))
}

// Eliminar un logro
pub async fn delete_achievement(
    app_state: web::Data<Arc<AppState>>,
    achievement_id: web::Path<Uuid>,
) -> Result<HttpResponse, HttpError> {
    app_state.db_client
        .delete_achievement(*achievement_id)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    Ok(HttpResponse::NoContent().finish())
}

// Obtener logros de usuario con detalles completos
pub async fn get_user_achievements_with_details(
    app_state: web::Data<Arc<AppState>>,
    user_id: web::Path<Uuid>,
) -> Result<HttpResponse, HttpError> {
    let user_achievements = app_state.db_client
        .get_user_achievements_with_details(*user_id)
        .await
        .map_err(|e| {
            log::error!("Error al obtener los logros del usuario: {}", e);
            HttpError::server_error(e.to_string())
        })?;
    Ok(HttpResponse::Ok().json(user_achievements))
}

// Verificar y otorgar logros automáticamente
#[derive(Deserialize)]
pub struct CheckAchievementsRequest {
    pub action: String,
    pub value: Option<i32>,
}

pub async fn check_and_award_achievements(
    app_state: web::Data<Arc<AppState>>,
    user_id: web::Path<Uuid>,
    req: web::Json<CheckAchievementsRequest>,
) -> Result<HttpResponse, HttpError> {
    let awarded = app_state.db_client
        .check_and_award_achievements(*user_id, &req.action, req.value)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    Ok(HttpResponse::Ok().json(awarded))
}

// Función de debug para verificar estado de logros
#[derive(Deserialize)]
pub struct DebugAchievementsRequest {
    pub user_id: Uuid,
}

pub async fn debug_user_achievements(
    app_state: web::Data<Arc<AppState>>,
    req: web::Json<DebugAchievementsRequest>,
) -> Result<HttpResponse, HttpError> {
    // Obtener estadísticas del usuario usando la nueva función
    let user_stats = app_state.db_client
        .get_user_stats(req.user_id)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    // Obtener logros disponibles
    let achievements = app_state.db_client
        .get_achievements(1, 100)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    // Obtener logros del usuario
    let user_achievements = app_state.db_client
        .get_user_achievements_with_details(req.user_id)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let debug_info = serde_json::json!({
        "user_stats": user_stats,
        "available_achievements": achievements,
        "user_achievements": user_achievements
    });

    Ok(HttpResponse::Ok().json(debug_info))
}