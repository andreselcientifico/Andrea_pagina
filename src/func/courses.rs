use std::sync::Arc;
use actix_web::{ web::{ self, Data, Json, Path, Query, scope }, HttpResponse };
use validator::Validate;
use uuid::Uuid;
use serde::Deserialize;
use sqlx::Error as SqlxError;

use crate::{
    AppState,
    config::dtos::{ CreateCourseDTO, UpdateCourseDTO },
    db::db::CourseExt,
    errors::error::{ ErrorMessage, HttpError },
    middleware::middleware::{ AuthMiddlewareFactory, RoleCheck, JWTAuthMiddleware, AccessCheck, RequiredAccess },
    models::models::UserRole,
};

pub fn courses_scope(app_state: Arc<AppState>) -> impl actix_web::dev::HttpServiceFactory {
    scope("/courses")
        .route("", web::get().to(get_courses))
        .service(
            scope("/videos")
                // Middleware solo para /courses/videos
                .wrap(AuthMiddlewareFactory::new(app_state.clone()))
                .wrap(RoleCheck::new(vec![UserRole::Admin]))
                .route("", web::get().to(get_courses_with_videos))
        )
        .service(
            scope("/{id}/videos")
                .wrap(AuthMiddlewareFactory::new(app_state.clone()))
                .wrap(AccessCheck::new(vec![
                    RequiredAccess::Role(UserRole::Admin),
                    RequiredAccess::PremiumAccess,   // requiere suscripción activa
                ]))
                .route("", web::get().to(get_course_with_videos))
        )
        .route("/{id}", web::get().to(get_course))
        .service(
            scope("")
                .wrap(AuthMiddlewareFactory::new(app_state.clone()))
                .wrap(RoleCheck::new(vec![UserRole::Admin]))
                .route("", web::post().to(create_course))
                .route("/{id}", web::put().to(update_course))
                .route("/{id}", web::delete().to(delete_course))
        )
}

#[derive(Deserialize)]
pub struct ListQuery {
    page: Option<u32>,
    limit: Option<usize>,
}

pub async fn get_courses(
    Query(q): Query<ListQuery>,
    app_state: Data<Arc<AppState>>
) -> Result<HttpResponse, HttpError> {
    let page = q.page.unwrap_or(1);
    let limit = q.limit.unwrap_or(10);

    let courses = app_state.db_client
        .get_courses(page, limit).await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    Ok(HttpResponse::Ok().json(courses))
}

pub async fn get_course(
    path: Path<String>,
    app_state: Data<Arc<AppState>>
) -> Result<HttpResponse, HttpError> {
    let id_str = path.into_inner();
    let course_id = Uuid::parse_str(&id_str).map_err(|e| HttpError::bad_request(e.to_string()))?;

    let course = app_state.db_client
        .get_course(course_id).await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    match course {
        Some(c) => Ok(HttpResponse::Ok().json(c)),
        None => Err(HttpError::not_found(ErrorMessage::CourseNotFound.to_string())),
    }
}


pub async fn get_courses_with_videos(
    // Query(q): Query<ListQuery>,
    app_state: Data<Arc<AppState>>
) -> Result<HttpResponse, HttpError> {

    // // Valores por defecto
    // let page = q.page.unwrap_or(1);
    // let limit = q.limit.unwrap_or(10);

    // Obtener cursos con videos desde el DBClient
    let courses = app_state.db_client
        .get_all_courses_with_modules()
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    // Respuesta HTTP 200 OK
    Ok(HttpResponse::Ok().json(courses))
}

pub async fn get_course_with_videos(
    path: Path<String>,
    app_state: Data<Arc<AppState>>,
    _auth: web::ReqData<JWTAuthMiddleware>  // requiere autenticación
) -> Result<HttpResponse, HttpError> {
    let id_str = path.into_inner();
    let course_id = Uuid::parse_str(&id_str)
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let course = app_state.db_client
        .get_course_with_videos(course_id)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    match course {
        Some(c) => Ok(HttpResponse::Ok().json(c)),
        None => Err(HttpError::not_found(ErrorMessage::CourseNotFound.to_string())),
    }
}

pub async fn create_course(
    app_state: Data<Arc<AppState>>,
    Json(body): Json<CreateCourseDTO>,
    _auth: web::ReqData<JWTAuthMiddleware> // ya validado por middleware/RoleCheck o AuthMiddlewareFactory
) -> Result<HttpResponse, HttpError> {
    body.validate().map_err(|e| HttpError::bad_request(e.to_string()))?;

    // let product_id = app_state.paypal_client
    //     .create_product(&body.title, &body.description)
    //     .await
    //     .map_err(|e| HttpError::server_error(e.to_string()))?;
    
    let course = app_state.db_client.create_course(body).await.map_err(|e| {
        let s = e.to_string();
        if s.contains("duplicate") || s.contains("unique") {
            HttpError::unique_constraint_violation(ErrorMessage::CourseAlreadyExists.to_string())
        } else {
            HttpError::server_error(s)
        }
    })?;

    Ok(HttpResponse::Created().json(course))
}

pub async fn update_course(
    path: Path<String>,
    app_state: Data<Arc<AppState>>,
    Json(body): Json<UpdateCourseDTO>,
    _auth: web::ReqData<JWTAuthMiddleware>
) -> Result<HttpResponse, HttpError> {
    body.validate().map_err(|e| HttpError::bad_request(e.to_string()))?;

    let id_str = path.into_inner();
    let course_id = Uuid::parse_str(&id_str).map_err(|e| HttpError::bad_request(e.to_string()))?;

    let updated = app_state.db_client
        .update_course(course_id,body).await
        .map_err(|e| {
            match e {
                SqlxError::RowNotFound =>
                    HttpError::not_found(ErrorMessage::CourseNotFound.to_string()),
                _ => HttpError::server_error(e.to_string()),
            }
        })?;

    Ok(HttpResponse::Ok().json(updated))
}

pub async fn delete_course(
    path: Path<String>,
    app_state: Data<Arc<AppState>>,
    _auth: web::ReqData<JWTAuthMiddleware>
) -> Result<HttpResponse, HttpError> {
    let id_str = path.into_inner();
    let course_id = Uuid::parse_str(&id_str).map_err(|e| HttpError::bad_request(e.to_string()))?;

    app_state.db_client.delete_course(course_id).await.map_err(|e| {
        match e {
            SqlxError::RowNotFound =>
                HttpError::not_found(ErrorMessage::CourseNotFound.to_string()),
            _ => HttpError::server_error(e.to_string()),
        }
    })?;

    Ok(HttpResponse::NoContent().finish())
}
