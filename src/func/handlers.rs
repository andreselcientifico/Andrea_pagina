use actix_web::{ 
    HttpMessage, HttpRequest, HttpResponse, cookie::{Cookie, SameSite}, get, post, put, web::{ Data, Json, Query}
};
use std::sync::Arc;
use validator::Validate;
use crate::db::db::{CourseExt, UserAchievementExt, UserExt, CoursePurchaseExt, PasswordResetTokenExt};
use serde_json::{json};
use chrono::{ Duration, Utc };
use uuid::Uuid;
use crate::mail::mails::{ send_verification_email, send_welcome_email, send_forgot_password_email };
use crate::utils::password::{hash_password, verify_password};
use crate::utils::token::create_token_rsa;
use crate::errors::error::{ ErrorMessage, HttpError };
use crate::middleware::middleware::JWTAuthMiddleware;  
use crate::config::dtos::{ RegisterDTO, LoginDTO, Response , UserLoginResponseDto, ResetPasswordRequestDTO, FilterUserDto, UserProfileResponse, UserProfileData, FilterAchievementDto, UpdateUserProfileDto, VerifyEmailQueryDTO, ForgotPasswordRequestDTO, FilterCourseDto };
use crate::AppState;


#[get("/mycourses")]
pub async fn get_user_courses_api(
    app_state: Data<Arc<AppState>>,
    req: HttpRequest,
) -> Result<HttpResponse, HttpError> {
    let extensions = req.extensions();
    let user_data = extensions
        .get::<JWTAuthMiddleware>()
        .ok_or_else(|| HttpError::unauthorized("Usuario no autenticado".to_string()))?;

    let user_id = user_data.user.id;

    let courses = app_state.db_client.get_user_purchased_courses(user_id)
        .await
        .map_err(|e| {
            log::error!("Error al obtener cursos comprados: {}", e);
            HttpError::server_error(e.to_string())
        })?;

    // Devolver un objeto JSON con la estructura esperada
    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "courseIds": courses
    })))
}

// ===================== //
//    Handlers de Autenticación
// ===================== //

/// Registrar usuario
#[post("/register")]
pub async fn register_user(
    app_state: Data<Arc<AppState>>,
    Json(body): Json<RegisterDTO>
) -> Result<HttpResponse, HttpError> {
    body.validate()
        .map_err(|e|  HttpError::bad_request(e.to_string()))?;

     let verification_token = Uuid::new_v4().to_string();
     let expires_at = Utc::now() + Duration::hours(24);
    let password_hash = hash_password(&body.password)
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let result = app_state.db_client
        .save_user(&body.name, &body.email, &password_hash, &verification_token, Some(expires_at), None)
        .await;

    match result {
        Ok(user) => {
            let send_email_result = send_verification_email(&body.email, &body.name, &verification_token).await;

            if let Err(e) = send_email_result {
               return Err(HttpError::server_error(format!("Ocurrio un error: {}", e)))
            }
            let token = create_token_rsa(user.id, user.role,None, &app_state.env.encoding_key, app_state.env.jwt_maxage)
            .map_err(|e| HttpError::server_error(e.to_string()))?;
            Ok(HttpResponse::Created().cookie(
                Cookie::build("token", token.clone())
                .path("/")
                .max_age(time::Duration::minutes(app_state.env.jwt_maxage * 60))
                .http_only(true)
                .secure(true)
                .same_site(SameSite::None)
                .finish()
                ).json(Response {
                status: "success",
                message: "Usuario registrado exitosamente. Por favor, verifica tu email.".to_string()
            }))
        },
        Err(sqlx::Error::Database(db_err)) => {
            if db_err.is_unique_violation() {
                Err(HttpError::unique_constraint_violation(
                    ErrorMessage::EmailExist.to_string(),
                ))
            } else {
                Err(HttpError::server_error(db_err.to_string()))
            }
        },
        Err(e) => Err(HttpError::server_error(e.to_string()))
    }
}

/// Login usuario
#[post("/login")]
pub async fn login_user(app_state: Data<Arc<AppState>>, Json(body): Json<LoginDTO>) -> Result<HttpResponse, HttpError> {

    body.validate()
       .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let user =  app_state.db_client
        .get_user(None, None, Some(&body.email), None)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?
        .ok_or_else(|| HttpError::bad_request("Usuario no encontrado".to_string()))?;

    if verify_password(&body.password, &user.password)
        .map_err(|_| HttpError::bad_request(ErrorMessage::WrongCredentials.to_string()))? {
        let token = create_token_rsa(user.id, user.role,None, &app_state.env.encoding_key, app_state.env.jwt_maxage)
            .map_err(|e| HttpError::server_error(e.to_string()))?;
        // Incrementar contador de logins
        let _ = app_state.db_client.increment_user_stat(user.id, "login_streak").await;
        // Verificar logros de racha de logins
        let _ = app_state.db_client.check_and_award_achievements(user.id, "login_streak", Some(1)).await;

        Ok(
            HttpResponse::Ok()
            .cookie(
                Cookie::build("token", token.clone())
                .path("/")
                .max_age(time::Duration::minutes(app_state.env.jwt_maxage * 60))
                .http_only(true)
                .secure(true) 
                .same_site(SameSite::None)
                .finish()
                ).json(UserLoginResponseDto {
                    status: "success".to_string(),
                }
            )
        )
    } 
    else {
        Err(HttpError::bad_request(ErrorMessage::WrongCredentials.to_string()))
    }
}

#[post("/logout")]
pub async fn logout_user() -> HttpResponse {
    HttpResponse::Ok()
        .cookie(
            Cookie::build("token", "")
                .path("/")
                .max_age(time::Duration::seconds(0))
                .http_only(true)
                .secure(true)
                .same_site(SameSite::None)
                .finish()
        )
        .json(serde_json::json!({ "status": "success", "message": "Sesión cerrada" }))
}


#[get("/verify")]
pub async fn verify_email(Query(query_params): Query<VerifyEmailQueryDTO>, app_state: Data<Arc<AppState>>) -> Result<HttpResponse, HttpError> {
    query_params.validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let user = app_state.db_client
        .get_user(None, None, None,  Some(&query_params.token))
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?.ok_or(HttpError::unauthorized(ErrorMessage::InvalidToken.to_string()))?;

    if let Some(expires_at) = user.token_expiry {
        if Utc::now() > expires_at {
            return Err(HttpError::bad_request("Verificacion del token ha expirado.".to_string()));
        }
    } else {
        return Err(HttpError::bad_request("token inválido.".to_string()));
    }

    app_state.db_client
        .verifed_token(&query_params.token)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    if let Err(e) = send_welcome_email(&user.email, &user.name).await {
        return Err(HttpError::server_error(format!("Ocurrio un error: {}", e)))
    }

    let token = create_token_rsa(user.id, user.role, None,&app_state.env.encoding_key, app_state.env.jwt_maxage)
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    Ok(
            HttpResponse::Ok()
            .cookie(
                Cookie::build("token", token.clone())
                .path("/settings")
                .max_age(time::Duration::minutes(app_state.env.jwt_maxage * 60))
                .http_only(true)
                .secure(true) 
                .same_site(SameSite::None)
                .finish()
                ).json(UserLoginResponseDto {
                    status: "success".to_string(),
                }
            )
        )
}


#[post("/forgot-password")]
pub async fn forgot_password(
    app_state: Data<Arc<AppState>>,
    Json(body): Json<ForgotPasswordRequestDTO>
) -> Result<HttpResponse, HttpError> {
    body.validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let user = app_state.db_client
        .get_user(None, None, Some(&body.email), None)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?.ok_or(HttpError::bad_request("Email no encontrado.".to_string()))?;

    let reset_token = Uuid::new_v4().to_string();
    let token_hash = hash_password(&reset_token)
        .map_err(|e| HttpError::server_error(e.to_string()))?;
    let expires_at = Utc::now() + Duration::minutes(30);

    let user_id = Uuid::parse_str(&user.id.to_string()).unwrap();

    // Invalidar tokens anteriores
    app_state.db_client
        .invalidate_user_tokens(user_id)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    // Crear nuevo token
    app_state.db_client
        .create_password_reset_token(user_id, &token_hash, expires_at)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let reset_link = format!("http://localhost:8000/reset-password?token={}", &reset_token);

    let send_email_result = send_forgot_password_email(&user.email, &reset_link, &user.name).await;

    if let Err(e) = send_email_result {
        return Err(HttpError::server_error(format!("No se pudo enviar el email de restablecimiento de contraseña. Erro :{}", e)));
    }

    Ok(HttpResponse::Ok().json(Response {
        status: "success",
        message: "Se ha enviado un enlace de restablecimiento de contraseña a su correo electrónico.".to_string()
    }))
}


#[post("/reset-password")]
pub async fn reset_password(app_state: Data<Arc<AppState>>, Json(body): Json<ResetPasswordRequestDTO>) -> Result<HttpResponse, HttpError> {
    body.validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let token_hash = hash_password(&body.token)
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let reset_token = app_state.db_client
        .get_password_reset_token(&token_hash)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?.ok_or(HttpError::bad_request("Token inválido o expirado.".to_string()))?;

    let user_id = reset_token.user_id;

    let _ = app_state.db_client
        .get_user(Some(user_id), None, None, None)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?.ok_or(HttpError::bad_request("Usuario no encontrado.".to_string()))?;

    let new_password_hash = hash_password(&body.new_password)
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    app_state.db_client
        .update_user_password(user_id, new_password_hash)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    app_state.db_client
        .mark_token_used(&token_hash)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    Ok(HttpResponse::Ok().json(Response {
        status: "success",
        message: "Contraseña restablecida exitosamente.".to_string()
    }))
 
}

/// Obtener perfil
#[get("/profile")]
pub async fn get_user_profile(req: HttpRequest, app_state: Data<Arc<AppState>>) -> Result<HttpResponse, HttpError> {
    // Verifica si el middleware JWT añadió los datos del usuario autenticado
    match req.extensions().get::<JWTAuthMiddleware>() {
        Some(user_data) => {
            let user_id = user_data.user.id;

            // Obtener cursos y logros desde el cliente de base de datos en AppState
            let courses = app_state.db_client
                .get_user_courses(user_id)
                .await
                .map_err(|e| {
                    HttpError::server_error(e.to_string())
                })?;

            let achievements = app_state.db_client
                .get_user_achievements(user_id)
                .await
                .map_err(|e| {
                    HttpError::server_error(e.to_string())
                })?;

            let response = UserProfileResponse {
                status: "success".into(),
                data: UserProfileData {
                    user: FilterUserDto::filter_user(&user_data.user),
                    courses: FilterCourseDto::filter_courses(&courses),
                    achievements,
                }
            };

            Ok(HttpResponse::Ok().json(response))
        }
        None => Err(HttpError::unauthorized("Usuario no autenticado".to_string())),
    }
}

#[put("/users/profile")]
pub async fn update_user_profile(
    req: HttpRequest,
    app_state: Data<Arc<AppState>>,
    body: Json<UpdateUserProfileDto>,
) -> Result<HttpResponse, HttpError> {
    match req.extensions().get::<JWTAuthMiddleware>() {
        Some(user_data) => {
            let user_id = user_data.user.id;

            let updated_user = app_state.db_client
                .update_user_profile(
                    user_id,
                    body.name.clone(),
                    body.phone.clone(),
                    body.location.clone(),
                    body.bio.clone(),
                    body.birth_date,
                    body.profile_image_url.clone(),
                )
                .await
                .map_err(|e| HttpError::server_error(e.to_string()))?;

            Ok(HttpResponse::Ok().json(FilterUserDto::filter_user(&updated_user)))
        }
        None => Err(HttpError::unauthorized("Usuario no autenticado".to_string())),
    }
}