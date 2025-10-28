use std::sync::Arc;
use actix_web::{ 
   HttpResponse, middleware::{ Logger}, web::{ Data, Json, Query,  resource, scope, get, put}
};
use validator::Validate;

use crate::{
    AppState, 
    config::dtos::{FilterUserDto, NameUpdateDTO, RequestQueryDto, Response, RoleUpdateDTO, UserData, UserListResponseDto, UserPasswordUpdateDTO, UserResponseDto}, 
    db::db::UserExt, errors::error::{ErrorMessage, HttpError}, 
    middleware::middleware::{AuthMiddlewareFactory, JWTAuthMiddleware, RoleCheck}, 
    models::models::UserRole, utils::password
};


pub fn users_scope(app_state: Arc<AppState>) -> impl actix_web::dev::HttpServiceFactory {
    scope("/users")
        .wrap(Logger::default())
        .wrap(AuthMiddlewareFactory::new(app_state.clone()))
        .service(resource("/me")
        .route(get().to(get_me))
        .wrap(RoleCheck::new(vec![UserRole::User, UserRole::Admin])),
    )

    .service(resource("")
        .route(get().to(get_users))
        .wrap(RoleCheck::new(vec![UserRole::Admin])),
    )
    .service(resource("/name").route(put().to(update_user_name)))
    .service(resource("/role").route(put().to(update_user_role)))
    .service(resource("/password").route(put().to(update_user_password)))
}




pub async fn get_me(
    user: Data<JWTAuthMiddleware>
) -> Result<HttpResponse, HttpError> {
    Ok(
        HttpResponse::Ok().json(UserResponseDto {
        status: "success".to_string(),
        data: UserData {
            user: FilterUserDto::filter_user(&user.user),
        }
    }))
}

pub async fn get_users(
    Query(query_params): Query<RequestQueryDto>,
    app_state: Data<Arc<AppState>>
) -> Result<HttpResponse, HttpError> {
    query_params.validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let page = query_params.page.unwrap_or(1);
    let limit = query_params.limit.unwrap_or(10);
    
    let users = app_state.db_client
        .get_users(page as u32, limit)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let user_count = app_state.db_client
        .get_user_count()
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    Ok(HttpResponse::Ok().json(UserListResponseDto {
        status: "success".to_string(),
        users: FilterUserDto::filter_users(&users),
        results: user_count,
    }))
}

pub async fn update_user_name(
    app_state: Data<Arc<AppState>>,
    user: Data<JWTAuthMiddleware>,
    Json(body): Json<NameUpdateDTO>,
) -> Result<HttpResponse, HttpError> {
    body.validate()
       .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let user = &user.user;

    let user_id = uuid::Uuid::parse_str(&user.id.to_string()).unwrap();

    let result = app_state.db_client.
        update_user_name(user_id.clone(), &body.name)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let filtered_user = FilterUserDto::filter_user(&result);

    Ok(HttpResponse::Ok().json(UserResponseDto {
        data: UserData {
            user: filtered_user,
        },
        status: "success".to_string(),
    }))
}

pub async fn update_user_role(
    app_state: Data<Arc<AppState>>,
    user: Data<JWTAuthMiddleware>,
    Json(body): Json<RoleUpdateDTO>,
) -> Result<HttpResponse, HttpError> {
    body.validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let user = &user.user;

    let user_id = uuid::Uuid::parse_str(&user.id.to_string()).unwrap();

    let result = app_state.db_client
        .update_user_role(user_id.clone(), body.role)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let filtered_user = FilterUserDto::filter_user(&result);

    Ok(HttpResponse::Ok().json(UserResponseDto {
        data: UserData {
            user: filtered_user,
        },
        status: "success".to_string(),
    }))
}

pub async fn update_user_password(
    app_state: Data<Arc<AppState>>,
    user: Data<JWTAuthMiddleware>,
    Json(body): Json<UserPasswordUpdateDTO>,
) -> Result<HttpResponse, HttpError> {
    body.validate()
       .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let user = &user.user;

    let user_id = uuid::Uuid::parse_str(&user.id.to_string()).unwrap();

    let result = app_state.db_client
        .get_user(Some(user_id.clone()), None, None, None)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let user = result.ok_or(HttpError::unauthorized(ErrorMessage::InvalidToken.to_string()))?;

    let password_match = password::verify_password(&body.old_password, &user.password)
            .map_err(|e| HttpError::server_error(e.to_string()))?;

    if !password_match {
        return Err(HttpError::bad_request("Old password is incorrect".to_string()));
    }

    let hash_password = password::hash_password(&body.new_password)
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    app_state.db_client
        .update_user_password(user_id.clone(), hash_password)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    Ok(HttpResponse::Ok().json(Response {
        message: "Password updated Successfully".to_string(),
        status: "success",
    }))

}