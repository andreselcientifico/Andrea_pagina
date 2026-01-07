use actix_web::{web, HttpRequest, HttpResponse, Result, HttpMessage};
use serde::{Deserialize};
use uuid::Uuid;
use crate::{AppState, db::db::{SubscriptionExt, SubscriptionPlanExt}, errors::error::HttpError, middleware::middleware::JWTAuthMiddleware};
use std::sync::Arc;

// DTOs para suscripciones
#[derive(Deserialize)]
pub struct CreateSubscriptionPlanRequest {
    pub name: String,
    pub description: Option<String>,
    pub price: f64,
    pub duration_months: i32,
    pub features: Option<serde_json::Value>,
    pub paypal_plan_id: Option<String>,
}

#[derive(Deserialize)]
pub struct SubscribeRequest {
    pub user_id: Uuid,
    pub plan_id: String,
}

// Crear un plan de suscripción (solo admin)
pub async fn create_subscription_plan(
    app_state: web::Data<Arc<AppState>>,
    req: web::Json<CreateSubscriptionPlanRequest>,
) -> Result<HttpResponse, HttpError> {
    // Crear producto en PayPal primero
    let product_id = app_state.paypal_client.create_product(&req.name, &req.description.clone().unwrap_or_else(|| req.name.clone()))
        .await
        .map_err(|e| HttpError::server_error(format!("Failed to create PayPal product: {}", e)))?;

    // Crear plan en PayPal
    let plan_id = app_state.paypal_client.create_plan(&product_id, &req.name, &req.description.clone().unwrap_or_else(|| req.name.clone()), req.price, "MONTH", req.duration_months)
        .await
        .map_err(|e| HttpError::server_error(format!("Failed to create PayPal plan: {}", e)))?;

    // Guardar en la DB con el plan_id de PayPal
    let plan = app_state.db_client
        .create_subscription_plan(&req.name, req.description.as_ref(), req.price, req.duration_months, req.features.as_ref(), Some(&plan_id))
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    Ok(HttpResponse::Created().json(plan))
}

// Obtener planes de suscripción
pub async fn get_subscription_plans(
    app_state: web::Data<Arc<AppState>>,
) -> Result<HttpResponse, HttpError> {
    let plans = app_state.db_client
        .get_subscription_plans()
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    Ok(HttpResponse::Ok().json(plans))
}

// Actualizar plan de suscripción
#[derive(Deserialize)]
pub struct UpdateSubscriptionPlanRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub price: Option<f64>,
    pub duration_months: Option<i32>,
    pub features: Option<serde_json::Value>,
    pub paypal_plan_id: Option<String>,
    pub active: Option<bool>,
}

pub async fn update_subscription_plan(
    app_state: web::Data<Arc<AppState>>,
    plan_id: web::Path<Uuid>,
    req: web::Json<UpdateSubscriptionPlanRequest>,
) -> Result<HttpResponse, HttpError> {
    let plan = app_state.db_client
        .update_subscription_plan(*plan_id, req.name.as_ref().map(|s| s.as_str()), req.description.as_ref(), req.price, req.duration_months, req.features.as_ref(), req.paypal_plan_id.as_ref().map(|s| s.as_str()), req.active)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    Ok(HttpResponse::Ok().json(plan))
}

// Eliminar plan de suscripción
pub async fn delete_subscription_plan(
    app_state: web::Data<Arc<AppState>>,
    plan_id: web::Path<Uuid>,
) -> Result<HttpResponse, HttpError> {
    // Obtener el plan para tener el paypal_plan_id
    let plans = app_state.db_client
        .get_subscription_plans()
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let plan = plans.into_iter().find(|p| p.id == *plan_id)
        .ok_or_else(|| HttpError::not_found("Plan not found".to_string()))?;

    // Eliminar plan en PayPal si existe
    if let Some(paypal_plan_id) = &plan.paypal_plan_id {
        app_state.paypal_client.delete_plan(paypal_plan_id)
            .await
            .map_err(|e| HttpError::server_error(format!("Failed to delete PayPal plan: {}", e)))?;
    }

    // Eliminar de la DB
    app_state.db_client
        .delete_subscription_plan(*plan_id)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    Ok(HttpResponse::NoContent().finish())
}


// Obtener suscripciones del usuario autenticado
pub async fn get_user_subscriptions(
    req: HttpRequest,
    app_state: web::Data<Arc<AppState>>,
) -> Result<HttpResponse, HttpError> {
    let user = req.extensions().get::<JWTAuthMiddleware>().unwrap().user.clone();
    let subscriptions = app_state.db_client
        .get_user_subscriptions(user.id)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    Ok(HttpResponse::Ok().json(subscriptions))
}

// Cancelar suscripción
pub async fn cancel_subscription(
    req: HttpRequest,
    app_state: web::Data<Arc<AppState>>,
    subscription_id: web::Path<Uuid>,
) -> Result<HttpResponse, HttpError> {
    let user = req.extensions().get::<JWTAuthMiddleware>().unwrap().user.clone();

    // Obtener la suscripción para verificar que pertenece al usuario
    let subscriptions = app_state.db_client
        .get_user_subscriptions(user.id)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let subscription = subscriptions.into_iter().find(|s| s.id == *subscription_id)
        .ok_or_else(|| HttpError::not_found("Subscription not found or does not belong to user".to_string()))?;

    // Cancelar en PayPal
    app_state.paypal_client.cancel_subscription(&subscription.paypal_subscription_id)
        .await
        .map_err(|e| HttpError::server_error(format!("Failed to cancel PayPal subscription: {}", e)))?;

    // Cancelar en la DB
    app_state.db_client
        .cancel_subscription(*subscription_id)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    Ok(HttpResponse::Ok().finish())
}