use std::sync::Arc;
use actix_web::{  HttpResponse, put, web::{ self, Data, Json, Path, Query, ReqData } };
use validator::Validate;
use uuid::Uuid;
use serde::Deserialize;
use sqlx::Error as SqlxError;
use serde_json::json;

use crate::{
    AppState, 
    config::dtos::{ CreateCourseDTO, CreatedCommentDto, CreatedRatingDto, ProductDTO, UpdateCourseDTO, UpdateLessonProgressDTO }, 
    db::db::{CourseExt, CoursePurchaseExt}, 
    errors::error::{ ErrorMessage, HttpError }, 
    func::payments::{create_product }, 
    middleware::middleware::{ JWTAuthMiddleware },
};

//===================COMMENTS===================//

pub async fn create_lesson_comment(
    path: Path<String>,
    app_state: Data<Arc<AppState>>,
    _auth: web::ReqData<JWTAuthMiddleware>, // requiere autenticación
    Json(body): Json<CreatedCommentDto>
) -> Result<HttpResponse, HttpError> {
    let course_id = Uuid::parse_str(&path.into_inner())
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    Ok(HttpResponse::Ok().json(
        app_state.db_client
        .create_lesson_comment(course_id, _auth.user.id, body.content).await
        .map_err(|e| HttpError::server_error(e.to_string()))?
    ))
}

pub async fn get_lesson_comments(
    path: Path<String>,
    app_state: Data<Arc<AppState>>
) -> Result<HttpResponse, HttpError> {
    let course_id = Uuid::parse_str(&path.into_inner())
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    Ok(HttpResponse::Ok().json(
        app_state.db_client
        .get_lesson_comments(course_id).await
        .map_err(|e| HttpError::server_error(e.to_string()))?
    ))
}

pub async fn delete_comment(
    path: Path<(Uuid,Uuid)>,
    app_state: Data<Arc<AppState>>,
    _auth: web::ReqData<JWTAuthMiddleware>  // requiere autenticación
) -> Result<HttpResponse, HttpError> {
    let (_, commentid) = path.into_inner();
    let comment_id = Uuid::parse_str(&commentid.to_string())
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    app_state.db_client
        .delete_lesson_comment(comment_id).await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    Ok(HttpResponse::Ok().json(()))
}

pub async fn create_or_update_rating(
    path: Path<String>,
    app_state: Data<Arc<AppState>>,
    _auth: web::ReqData<JWTAuthMiddleware>, // requiere autenticación
    Json(body): Json<CreatedRatingDto>
) -> Result<HttpResponse, HttpError> {
    let course_id = Uuid::parse_str(&path.into_inner())
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    app_state.db_client
        .create_or_update_rating(course_id, _auth.user.id, body.rating).await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    Ok(HttpResponse::Ok().json(()))
}

pub async fn get_rating(
    path: Path<String>,
    app_state: Data<Arc<AppState>>,
    _auth: web::ReqData<JWTAuthMiddleware>
) -> Result<HttpResponse, HttpError> {
    let course_id = Uuid::parse_str(&path.into_inner())
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let rating = app_state.db_client
        .get_rating(course_id, Some(_auth.user.id)).await
        .map_err(|e| 
            {   
                log::debug!("get_course_with_videos_preview error: {:?}", e);
                log::error!("Error al obtener el curso: {}", e);
                HttpError::server_error(e.to_string())
            }
        )?;

    Ok(HttpResponse::Ok().json(rating))
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


pub async fn get_courses_with_modules(
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

pub async fn get_course_with_modules(
    path: Path<String>,
    app_state: Data<Arc<AppState>>,
    _auth: web::ReqData<JWTAuthMiddleware>  // requiere autenticación
) -> Result<HttpResponse, HttpError> {
    let id_str = path.into_inner();
    let course_id = Uuid::parse_str(&id_str)
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let course = app_state.db_client
        .get_course_with_videos(course_id,Some( _auth.user.id))
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    match course {
        Some(c) => Ok(HttpResponse::Ok().json(c)),
        None => Err(HttpError::not_found(ErrorMessage::CourseNotFound.to_string())),
    }
}

pub async fn get_course_with_modules_preview(
    path: Path<String>,
    app_state: Data<Arc<AppState>>,
    _auth: web::ReqData<JWTAuthMiddleware>
) -> Result<HttpResponse, HttpError> {
    let id_str = path.into_inner();
    let course_id = Uuid::parse_str(&id_str)
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let course = app_state.db_client
        .get_course_with_videos_preview(course_id,Some( _auth.user.id))
        .await
        .map_err(|e| 
            {
                log::debug!("get_course_with_videos_preview error: {:?}", e);
                log::error!("Error al obtener el curso: {}", e);
                HttpError::server_error(e.to_string())
            }
        )?;

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
    let host = app_state.env.host.trim_end_matches('/');
    let product_body = ProductDTO {
        name: body.title.clone(),
        description: body.description.clone(),
        type_: "SERVICE".to_string(),
        category: "EDUCATIONAL_AND_TEXTBOOKS".to_string(),
        image_url: body.image.clone(),
        home_url: Some(if host.starts_with("https://") {
            format!("{}/courses/", host)
        } else {
            format!("https://{}/courses/", host)
        })
    };
    log::debug!("PayPal request body: {:?}", product_body);
    let product_id = create_product(app_state.clone(), product_body).await.map_err(|e| {
        HttpError::server_error(format!("Failed to create product: {}", e.to_string()))
    })?;
    let new_body = CreateCourseDTO {
        paypal_product_id: Some(product_id.clone()),
        ..body.clone()
    };

    let course = app_state.db_client.create_course(new_body).await.map_err(|e| {
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


pub async fn update_lesson_progress(
    path: Path<(String,String)>,
    user: ReqData<JWTAuthMiddleware>,
    state: Data<Arc<AppState>>,
    Json(progress_data): Json<UpdateLessonProgressDTO>,
) -> Result<HttpResponse, HttpError> {
    log::debug!("ejecutando update_lesson_progress");
    let  (_,lesson_id) = path.into_inner();
    let user_id = user.user.id;

    let lesson_uuid = Uuid::parse_str(&lesson_id)
        .map_err(|_| HttpError::bad_request("ID de lección inválido".to_string()))?;
    log::debug!("user_id: {}", user_id);
    log::debug!("lesson_uuid: {}", lesson_uuid);
    log::debug!("progress_data: {:?}", progress_data);
    state.db_client.update_lesson_progress(
        user_id,
        lesson_uuid,
        progress_data.is_completed,
        progress_data.progress,
    )
    .await
    .map_err(|e| HttpError::server_error(e.to_string()))?;
    
    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "lessonId": lesson_uuid,
        "progress": progress_data.progress,
    })))
}

