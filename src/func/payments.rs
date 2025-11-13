use std::sync::Arc;
use actix_web::{
    web::{self, Data, Json, Path, scope},
    HttpResponse,
};
use validator::Validate;
use uuid::Uuid;

use crate::{
    AppState,
    config::dtos::{CreatePaymentDTO, VerifyPaymentDTO},
    db::db::PaymentExt,
    errors::error::{ErrorMessage, HttpError},
    middleware::middleware::{AuthMiddlewareFactory, JWTAuthMiddleware, RoleCheck},
    models::models::UserRole,
};

pub fn payments_scope(app_state: Arc<AppState>) -> impl actix_web::dev::HttpServiceFactory {
    scope("/payments")
        // rutas protegidas por JWT (usuario)
        .service(
            scope("")
                .wrap(AuthMiddlewareFactory::new(app_state.clone()))
                .route("", web::post().to(create_payment))
                .route("/user", web::get().to(get_user_payments))
        )
        // rutas administrativas (JWT + admin)
        .service(
            scope("")
                .wrap(AuthMiddlewareFactory::new(app_state.clone()))
                .wrap(RoleCheck::new(vec![UserRole::Admin]))
                .route("/{id}", web::get().to(get_payment))
        )
        // webhook público (proveedor) — valida firma en handler si aplica
        .route("/webhook", web::post().to(verify_payment))
}

pub async fn create_payment(
    app_state: Data<Arc<AppState>>,
    user: web::ReqData<JWTAuthMiddleware>,
    Json(body): Json<CreatePaymentDTO>,
) -> Result<HttpResponse, HttpError> {
    body.validate().map_err(|e| HttpError::bad_request(e.to_string()))?;

    let user_id = Uuid::parse_str(&user.user.id.to_string()).map_err(|e| HttpError::server_error(e.to_string()))?;
    let course_id = Uuid::parse_str(&body.course_id).map_err(|e| HttpError::bad_request(e.to_string()))?;

    // opcional: comprobar si ya pagó
    let already = app_state.db_client
        .check_user_course_payment(user_id, course_id)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    if already.is_some() {
        return Err(HttpError::bad_request(ErrorMessage::CourseAlreadyPurchased.to_string()));
    }

    let payment = app_state.db_client
        .create_payment(user_id, course_id, body.amount, body.payment_method.clone(), body.transaction_id.clone().unwrap_or_default())
        .await
        .map_err(|e| {
            let s = e.to_string();
            if s.contains("duplicate") || s.contains("unique") {
                HttpError::unique_constraint_violation(ErrorMessage::PaymentAlreadyProcessed.to_string())
            } else {
                HttpError::server_error(s)
            }
        })?;

    Ok(HttpResponse::Created().json(payment))
}

pub async fn verify_payment(
    app_state: Data<Arc<AppState>>,
    Json(body): Json<VerifyPaymentDTO>,
) -> Result<HttpResponse, HttpError> {
    body.validate().map_err(|e| HttpError::bad_request(e.to_string()))?;

    // verificar por payment_id o transaction_id según lo reciba el proveedor
    let payment_id = if let Some(pid) = &body.payment_id {
        Some(Uuid::parse_str(pid).map_err(|_| HttpError::bad_request(ErrorMessage::InvalidPaymentMethod.to_string()))?)
    } else {
        None
    };

    if let Some(pid) = payment_id {
        let updated = app_state.db_client
            .update_payment_status(pid, "completed".to_string())
            .await
            .map_err(|e| HttpError::server_error(e.to_string()))?;

        // opcional: insertar en user_courses aquí

        return Ok(HttpResponse::Ok().json(updated));
    }

    // si viene transaction_id debes implementar get_by_transaction en DBClient (no incluido aquí)
    if let Some(tx) = &body.transaction_id {
        // placeholder: intenta parsear como uuid (si tu transaction_id es uuid)
        if let Ok(pid) = Uuid::parse_str(tx) {
            let updated = app_state.db_client
                .update_payment_status(pid, "completed".to_string())
                .await
                .map_err(|e| HttpError::server_error(e.to_string()))?;
            return Ok(HttpResponse::Ok().json(updated));
        }

        return Err(HttpError::bad_request(ErrorMessage::InvalidPaymentMethod.to_string()));
    }

    Err(HttpError::bad_request(ErrorMessage::PaymentNotFound.to_string()))
}

pub async fn get_user_payments(
    app_state: Data<Arc<AppState>>,
    user: web::ReqData<JWTAuthMiddleware>,
) -> Result<HttpResponse, HttpError> {
    let user_id = Uuid::parse_str(&user.user.id.to_string()).map_err(|e| HttpError::server_error(e.to_string()))?;

    let payments = app_state.db_client
        .get_user_payments(user_id)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    Ok(HttpResponse::Ok().json(payments))
}

pub async fn get_payment(
    path: Path<String>,
    app_state: Data<Arc<AppState>>,
) -> Result<HttpResponse, HttpError> {
    let id_str = path.into_inner();
    let payment_id = Uuid::parse_str(&id_str).map_err(|e| HttpError::bad_request(e.to_string()))?;

    let payment = app_state.db_client
        .get_payment(payment_id)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    match payment {
        Some(p) => Ok(HttpResponse::Ok().json(p)),
        None => Err(HttpError::not_found(ErrorMessage::PaymentNotFound.to_string()))
    }
}