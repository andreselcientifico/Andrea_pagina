use actix_web::{ 
    HttpMessage, HttpRequest, HttpResponse, cookie::{Cookie, SameSite}, get, post, put, web::{ Data, Json, Path, Query, ReqData}
};
use std::sync::Arc;
use validator::Validate;
use crate::{AppState, CachedToken, config::dtos::{FilterCourseDto, ForgotPasswordRequestDTO, VerifyEmailQueryDTO}, db::db::{CourseExt, UserAchievementExt, UserExt}};
use serde_json::{Value, json};
use chrono::{ Duration, Utc };
use uuid::Uuid;
use crate::mail::mails::{ send_verification_email, send_welcome_email, send_forgot_password_email };
use crate::utils::password::{hash_password, verify_password};
use crate::utils::token::create_token_rsa;
use crate::errors::error::{ ErrorMessage, HttpError };
use crate::middleware::middleware::JWTAuthMiddleware;  
use crate::config::dtos::{ RegisterDTO, LoginDTO, Response , UserLoginResponseDto, ResetPasswordRequestDTO, FilterUserDto, UserProfileResponse, UserProfileData, FilterAchievementDto, UpdateUserProfileDto };


// ===================== //
//  Obtener token con cache
// ===================== //
pub async fn get_paypal_token(state: &AppState) -> String {
    let mut cache = state.token_cache.lock().await;

    // Si hay un token válido, devolverlo
    if let Some(cached) = cache.as_ref() {
        if cached.is_valid() {
            return cached.access_token.clone();
        }
    }

    // Caso contrario, pedir uno nuevo
    let resp = state.client
        .post(format!("{}/v1/oauth2/token", state.env.paypal_api_mode))
        .basic_auth(&state.env.paypal_client_id, Some(&state.env.paypal_secret))
        .form(&[("grant_type", "client_credentials")])
        .send().await
        .expect("Error solicitando token");

    let json: serde_json::Value = resp.json().await.expect("Error parseando JSON de token");
    let access_token = json["access_token"]
        .as_str()
        .expect("No se encontró access_token")
        .to_string();
    let expires_in = json["expires_in"].as_i64().unwrap_or(3600); // segundos

    // Guardar en cache
    let new_token = CachedToken {
        access_token: access_token.clone(),
        expires_at: Utc::now() + Duration::seconds(expires_in - 60), // margen 1min
    };
    *cache = Some(new_token);

    access_token
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


#[get("/api/auth/verify")]
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

    let verification_token = Uuid::new_v4().to_string();
    let expires_at = Utc::now() + Duration::minutes(30);

    let user_id = Uuid::parse_str(&user.id.to_string()).unwrap();

    app_state.db_client
        .add_verifed_token(user_id, &verification_token, expires_at)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let reset_link = format!("http://localhost:8000/reset-password?token={}", &verification_token);

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

    let user = app_state.db_client
        .get_user(None, None, Some(&body.token), None)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?.ok_or(HttpError::bad_request("Token inválido.".to_string()))?;

    if let Some(expires_at) = user.token_expiry {
        if Utc::now() > expires_at {
            return Err(HttpError::bad_request("El token de restablecimiento de contraseña ha expirado.".to_string()));
        }
    } else {
        return Err(HttpError::bad_request("Token inválido.".to_string()));
    }

    let user_id = Uuid::parse_str(&user.id.to_string()).unwrap();

    let new_password_hash = hash_password(&body.new_password)
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    app_state.db_client
        .update_user_password(user_id.clone(), new_password_hash)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    app_state.db_client
        .verifed_token(&body.token)
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
                    achievements: FilterAchievementDto::filter_achievements(&achievements),
                },
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

// ===================== //
//   Crear orden
// ===================== //
#[post("/courses/{course_id}/create-order")]
async fn created_order(state: Data<AppState>, path: Path<(Uuid,)>, user: ReqData<JWTAuthMiddleware>) -> HttpResponse {
    let course_id = path.into_inner().0;
    let user_id = user.user.id;
    
    let course = match state.db_client.get_course(course_id).await {
        Ok(c) => c,
        Err(_) => {
            return HttpResponse::NotFound().body("Curso no encontrado");
        }
    };

    let (paypal_product_id , title, price) = match course {
        Some(c) => (c.paypal_product_id.clone(), c.title.clone(), c.price),
        None => {
            return HttpResponse::NotFound().body("Curso no encontrado");
        }
    };

    let body =
        json!({
        "intent": "CAPTURE",
        "payment_source": {
            "paypal": {
                "experience_context": {
                    "payment_method_preference": "IMMEDIATE_PAYMENT_REQUIRED",
                    "landing_page": "LOGIN",
                    "user_action": "PAY_NOW",
                    "return_url": format!("{}/paypal/capture?course_id={}", state.env.host, course_id),
                    "cancel_url": format!("{}/paypal/cancel?course_id={}", state.env.host, course_id)
                }
            }
        },
        "purchase_units": [{
            "invoice_id": course_id.to_string(),
            "custom_id": user_id.to_string(),
            "amount": {
                "currency_code": "USD",
                "value": format!("{:.2}", price),
                "breakdown": {
                    "item_total": {
                        "currency_code": "USD",
                        "value": format!("{:.2}", price)
                    }
                }
            },
            "items": [{
                "name": title,
                "description": "Curso completo",
                "unit_amount": {
                    "currency_code": "USD",
                    "value": format!("{:.2}", price)
                },
                "quantity": "1",
                "category": "DIGITAL_GOODS",
                "sku": paypal_product_id
            }]
        }]
    });

    let access_token = get_paypal_token(&state).await;

    let res = state.client
        .post(format!("{}/v2/checkout/orders", state.env.paypal_api_mode))
        .bearer_auth(&access_token)
        .json(&body)
        .send().await
        .expect("Error al enviar la solicitud a PayPal");

    if res.status().is_client_error() || res.status().is_server_error() {
        return HttpResponse::InternalServerError().body("Error creating order");
    }

    let response_json: Value = res.json().await.unwrap();

    let order_id = response_json["id"].as_str().unwrap_or_default().to_string();

    // Responder sólo con orderID
    HttpResponse::Ok().json(json!({ "orderID": order_id }))
}

// ===================== //
//   Capturar orden
// ===================== //

#[post("/paypal/capture/{order_id}")]
async fn capture_order(path: Path<(String,)>, app_state: Data<AppState>) -> HttpResponse {
    let order_id = path.into_inner().0;
    let access_token = get_paypal_token(&app_state).await;

    let res = app_state.client
        .post(format!("{}/v2/checkout/orders/{}/capture", app_state.env.paypal_api_mode, order_id))
        .bearer_auth(&access_token)
        .send().await;

     match res {
        Ok(response) => {
            // Parsear la respuesta de PayPal
            let data: serde_json::Value = response.json().await.unwrap();
            HttpResponse::Ok().json(data)
        },
        Err(err) => {
            // Manejo de errores
            HttpResponse::InternalServerError().json(json!({
                "error": format!("Error al capturar la orden: {:?}", err)
            }))
        }
    }
}