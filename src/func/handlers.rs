use crate::AppState;
use crate::config::dtos::{
    FilterUserDto, ForgotPasswordRequestDTO, LoginDTO, RegisterDTO, ResendVerificationDTO,
    ResetPasswordRequestDTO, Response, UpdateUserProfileDto, UserLoginResponseDto,
    UserProfileResponse, VerifyEmailQueryDTO,
};
use crate::errors::error::{ErrorMessage, HttpError};
use crate::mail::mails::{send_forgot_password_email, send_verification_email, send_welcome_email};
use crate::middleware::middleware::JWTAuthMiddleware;
use crate::utils::password::{hash_password, hash_token, verify_password};
use crate::utils::token::{base_url, create_token_rsa};
use crate::{
    auth::auth::{UserJwtData, verify_jwt},
    db::db::{CourseExt, CoursePurchaseExt, PasswordResetTokenExt, UserAchievementExt, UserExt},
};
use actix_web::{
    HttpMessage, HttpRequest, HttpResponse,
    cookie::{Cookie, SameSite},
    get,
    http::header,
    post, put,
    web::{Data, Json, Query},
};
use chrono::{Duration, Utc};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

#[get("/mycourses")]
pub async fn get_user_courses_api(
    app_state: Data<Arc<AppState>>,
    req: HttpRequest,
) -> Result<HttpResponse, HttpError> {
    let extensions = req.extensions();
    let user_data = extensions
        .get::<JWTAuthMiddleware>()
        .ok_or_else(|| HttpError::unauthorized("Usuario no autenticado".to_string()))?;

    let user_id = user_data.claims.sub;

    let courses = app_state
        .db_client
        .get_user_purchased_courses(user_id)
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

fn get_optional_user(req: &HttpRequest) -> Option<UserJwtData> {
    let token = req
        .cookie("token")
        .map(|c| c.value().to_string())
        .or_else(|| {
            req.headers()
                .get(header::AUTHORIZATION)
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.strip_prefix("Bearer "))
                .map(|s| s.to_string())
        })?;

    verify_jwt(&token)
}

#[get("/courses-page")]
pub async fn get_courses_page(
    app_state: Data<Arc<AppState>>,
    req: HttpRequest,
) -> Result<HttpResponse, HttpError> {
    let user = get_optional_user(&req);

    let user_id = user.as_ref().map(|u| u.id());

    let courses = app_state
        .db_client
        .get_courses_page(user_id, 1, 100)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let has_active_subscription = courses
        .first()
        .map(|c| c.has_active_subscription)
        .unwrap_or(false);

    let purchased_course_ids: Vec<Uuid> = courses
        .iter()
        .filter(|c| c.purchased)
        .map(|c| c.id)
        .collect();

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "user": {
            "authenticated": user.is_some(),
            "hasActiveSubscription": has_active_subscription
        },
        "purchasedCourseIds": purchased_course_ids,
        "courses": courses
    })))
}

// ===================== //
//    Handlers de Autenticación
// ===================== //

/// Registrar usuario
#[post("/register")]
pub async fn register_user(
    app_state: Data<Arc<AppState>>,
    Json(body): Json<RegisterDTO>,
) -> Result<HttpResponse, HttpError> {
    body.validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let verification_token = Uuid::new_v4().to_string();
    let expires_at = Utc::now() + Duration::hours(24);
    let password_hash =
        hash_password(&body.password).map_err(|e| HttpError::server_error(e.to_string()))?;

    let result = app_state
        .db_client
        .save_user(
            &body.name,
            &body.email,
            &password_hash,
            &verification_token,
            Some(expires_at),
            None,
        )
        .await;

    match result {
        Ok(user) => {
            let send_email_result =
                send_verification_email(&body.email, &app_state, &body.name, &verification_token)
                    .await;

            if let Err(e) = send_email_result {
                return Err(HttpError::server_error(format!("Ocurrio un error: {}", e)));
            }
            let token = create_token_rsa(
                user.id,
                user.role,
                None,
                &app_state.env.encoding_key,
                app_state.env.jwt_maxage,
            )
            .map_err(|e| HttpError::server_error(e.to_string()))?;
            Ok(HttpResponse::Created()
                .cookie(
                    Cookie::build("token", token.clone())
                        .path("/")
                        .max_age(time::Duration::minutes(app_state.env.jwt_maxage * 60))
                        .http_only(true)
                        .secure(true)
                        .same_site(SameSite::None)
                        .finish(),
                )
                .json(Response {
                    status: "success",
                    message: "Usuario registrado exitosamente. Por favor, verifica tu email."
                        .to_string(),
                }))
        }
        Err(sqlx::Error::Database(db_err)) => {
            if db_err.is_unique_violation() {
                Err(HttpError::unique_constraint_violation(
                    ErrorMessage::EmailExist.to_string(),
                ))
            } else {
                Err(HttpError::server_error(db_err.to_string()))
            }
        }
        Err(e) => Err(HttpError::server_error(e.to_string())),
    }
}

/// Login usuario
#[post("/login")]
pub async fn login_user(
    app_state: Data<Arc<AppState>>,
    Json(body): Json<LoginDTO>,
) -> Result<HttpResponse, HttpError> {
    body.validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let user = app_state
        .db_client
        .get_user(None, None, Some(&body.email), None)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?
        .ok_or_else(|| HttpError::bad_request("Usuario no encontrado".to_string()))?;

    if verify_password(&body.password, &user.password)
        .map_err(|_| HttpError::bad_request(ErrorMessage::WrongCredentials.to_string()))?
    {
        let token = create_token_rsa(
            user.id,
            user.role,
            None,
            &app_state.env.encoding_key,
            app_state.env.jwt_maxage,
        )
        .map_err(|e| HttpError::server_error(e.to_string()))?;
        // Incrementar contador de logins
        let _ = app_state
            .db_client
            .increment_user_stat(user.id, "login_streak")
            .await;
        // Verificar logros de racha de logins
        let _ = app_state
            .db_client
            .check_and_award_achievements(user.id, "login_streak", Some(1))
            .await;

        Ok(HttpResponse::Ok()
            .cookie(
                Cookie::build("token", token.clone())
                    .path("/")
                    .max_age(time::Duration::minutes(app_state.env.jwt_maxage * 60))
                    .http_only(true)
                    .secure(true)
                    .same_site(SameSite::None)
                    .finish(),
            )
            .json(UserLoginResponseDto {
                status: "success".to_string(),
            }))
    } else {
        Err(HttpError::bad_request(
            ErrorMessage::WrongCredentials.to_string(),
        ))
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
                .finish(),
        )
        .json(serde_json::json!({ "status": "success", "message": "Sesión cerrada" }))
}

#[post("/resend-verification")]
pub async fn resend_verification(
    app_state: Data<Arc<AppState>>,
    Json(body): Json<ResendVerificationDTO>,
) -> Result<HttpResponse, HttpError> {
    // 1. Validar el DTO
    body.validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    // 2. Generar nuevos datos para el token
    let new_verification_token = Uuid::new_v4().to_string();
    let new_expires_at = Utc::now() + Duration::hours(24);

    // 3. Actualizar en BD (1 sola consulta atómica)
    let update_result = app_state
        .db_client
        .update_verification_token(&body.email, &new_verification_token, new_expires_at)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    // 4. Analizar el resultado
    match update_result {
        Some((user_name, user_email)) => {
            // El usuario existía y no estaba verificado. Mandamos el correo.
            let send_email_result = send_verification_email(
                &user_email,
                &app_state,
                &user_name,
                &new_verification_token,
            )
            .await;

            if let Err(e) = send_email_result {
                return Err(HttpError::server_error(format!(
                    "Error enviando el correo: {}",
                    e
                )));
            }

            Ok(HttpResponse::Ok().json(Response {
                status: "success",
                message: "Correo de verificación reenviado exitosamente.".to_string(),
            }))
        }
        None => {
            // Si devuelve None significa que:
            // a) El correo no existe en BD
            // b) El correo ya está verificado (`verified = true`)
            // Por seguridad, damos un mensaje genérico para no filtrar qué correos están registrados.
            Ok(HttpResponse::Ok().json(Response {
                status: "success",
                message: "Si tu correo no estaba verificado, te hemos enviado un nuevo enlace."
                    .to_string(),
            }))
        }
    }
}

#[get("/verify")]
pub async fn verify_email(
    Query(query_params): Query<VerifyEmailQueryDTO>,
    app_state: Data<Arc<AppState>>,
) -> Result<HttpResponse, HttpError> {
    query_params
        .validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let verified_user = app_state
        .db_client
        .verify_email_atomic(&query_params.token)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    // Si devuelve None, el token es inválido, ya se usó, o ya expiró.
    let user = verified_user.ok_or(HttpError::unauthorized(
        "El token de verificación es inválido o ha expirado.".to_string(),
    ))?;

    // Enviamos el correo de bienvenida
    if let Err(e) = send_welcome_email(&user.email, &user.name).await {
        return Err(HttpError::server_error(format!(
            "Ocurrio un error al enviar correo: {}",
            e
        )));
    }

    // Generamos el JWT
    let token = create_token_rsa(
        user.id,
        user.role,
        None,
        &app_state.env.encoding_key,
        app_state.env.jwt_maxage,
    )
    .map_err(|e| HttpError::server_error(e.to_string()))?;

    // Retornamos la respuesta
    Ok(HttpResponse::Ok()
        .cookie(
            Cookie::build("token", token.clone())
                .path("/") // Te recomiendo cambiar "/settings" a "/" a menos que sea a propósito
                .max_age(time::Duration::minutes(app_state.env.jwt_maxage * 60))
                .http_only(true)
                .secure(true)
                .same_site(SameSite::None)
                .finish(),
        )
        .json(UserLoginResponseDto {
            status: "success".to_string(),
        }))
}

#[post("/forgot-password")]
pub async fn forgot_password(
    app_state: Data<Arc<AppState>>,
    Json(body): Json<ForgotPasswordRequestDTO>,
) -> Result<HttpResponse, HttpError> {
    body.validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let reset_token = Uuid::new_v4().to_string();
    let new_token_id = Uuid::new_v4();

    let token_hash =
        hash_token(&reset_token).map_err(|e| HttpError::server_error(e.to_string()))?;

    let expires_at = Utc::now() + Duration::minutes(30);

    let user_info = app_state
        .db_client
        .generate_reset_token_atomic(&body.email, new_token_id, &token_hash, expires_at)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?
        .ok_or_else(|| HttpError::bad_request("Email no encontrado.".to_string()))?;

    let host = base_url(&app_state.env.host);
    let reset_link = format!("{host}resetear-contrasena?token={reset_token}");

    let send_email_result =
        send_forgot_password_email(&user_info.user_email, &reset_link, &user_info.user_name).await;

    if let Err(e) = send_email_result {
        return Err(HttpError::server_error(format!(
            "No se pudo enviar el email de restablecimiento. Error: {}",
            e
        )));
    }

    Ok(HttpResponse::Ok().json(Response {
        status: "success",
        message:
            "Se ha enviado un enlace de restablecimiento de contraseña a su correo electrónico."
                .to_string(),
    }))
}

#[put("/reset-password")]
pub async fn reset_password(
    app_state: Data<Arc<AppState>>,
    Json(body): Json<ResetPasswordRequestDTO>,
) -> Result<HttpResponse, HttpError> {
    body.validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let token_hash = hash_token(&body.token).map_err(|e| HttpError::server_error(e.to_string()))?;

    let new_password_hash =
        hash_password(&body.new_password).map_err(|e| HttpError::server_error(e.to_string()))?;

    let updated_user_id = app_state
        .db_client
        .reset_password_with_token(&token_hash, &new_password_hash)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    if updated_user_id.is_none() {
        return Err(HttpError::bad_request(
            "Token inválido o expirado.".to_string(),
        ));
    }

    Ok(HttpResponse::Ok().json(Response {
        status: "success",
        message: "Contraseña restablecida exitosamente.".to_string(),
    }))
}

/// Obtener perfil
#[get("/profile")]
pub async fn get_user_profile(
    req: HttpRequest,
    app_state: Data<Arc<AppState>>,
) -> Result<HttpResponse, HttpError> {
    // Verifica si el middleware JWT añadió los datos del usuario autenticado
    match req.extensions().get::<JWTAuthMiddleware>() {
        Some(user_data) => {
            let user_id = user_data.claims.sub;

            // Utilizar la consulta unificada
            let profile_data = app_state
                .db_client
                .get_user_complete_profile(user_id)
                .await
                .map_err(|e| HttpError::server_error(e.to_string()))?;

            let response = UserProfileResponse {
                status: "success".into(),
                data: profile_data,
            };

            Ok(HttpResponse::Ok().json(response))
        }
        None => Err(HttpError::unauthorized(
            "Usuario no autenticado".to_string(),
        )),
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
            let user_id = user_data.claims.sub;

            let updated_user = app_state
                .db_client
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
        None => Err(HttpError::unauthorized(
            "Usuario no autenticado".to_string(),
        )),
    }
}
