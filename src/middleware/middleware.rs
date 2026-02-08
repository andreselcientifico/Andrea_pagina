use std::{rc::Rc, sync::Arc, future::Future};
use actix_web::{
    Error, HttpMessage, web::Data, HttpResponse, body::{EitherBody}, dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready}, http::{header, Method}
};
use futures::{FutureExt, future::{LocalBoxFuture, Ready, ready}};
use uuid::Uuid;
use std::pin::Pin;


use crate::{
    AppState, auth::auth::{UserJwtData, verify_jwt}, db::db::{CoursePurchaseExt, SubscriptionExt}, errors::error::{ErrorMessage, HttpError}, models::models::{UserRole}
};

/// Estructura que contendrá al usuario autenticado
#[derive(Debug, Clone)]
pub struct JWTAuthMiddleware {
    pub claims: UserJwtData,
}

/// Middleware principal de autenticación JWT
pub struct AuthMiddlewareFactory {
}

impl AuthMiddlewareFactory {
    /// Middleware solo para autenticación
    pub fn new() -> Self {
        Self {}
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthMiddlewareFactory
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct AuthMiddleware<S> {
    service: Rc<S>
}

impl<S, B> Service<ServiceRequest> for AuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self,  req: ServiceRequest) -> Self::Future {
        let srv = self.service.clone();

        Box::pin(async move {
            
            // ===============================
            // 1. PERMITIR OPTIONS (CRÍTICO)
            // ===============================
            if req.method() == Method::OPTIONS {
                return Ok(
                    req.into_response(
                        HttpResponse::Ok().finish().map_into_right_body()
                    )
                );
            }
            // ============================
            // 2. EXTRAER TOKEN
            // ============================
            let token = req.cookie("token")
                .map(|c| c.value().to_string())
                .or_else(|| {
                    req.headers()
                        .get(header::AUTHORIZATION)
                        .and_then(|v| v.to_str().ok())
                        .and_then(|v| v.strip_prefix("Bearer "))
                        .map(|s| s.to_string())
                });

            let token = match token {
                Some(t) => t,
                None => {
                    let err = HttpError::unauthorized(ErrorMessage::TokenNotProvided.to_string());
                    return Err(actix_web::error::ErrorUnauthorized(err.to_string()));
                }
            };

            // ===============================
            // 3. VALIDAR JWT
            // ===============================
            let claims = match verify_jwt(&token) {
                Some(c) => c,
                None => {
                    let err = HttpError::unauthorized(ErrorMessage::InvalidToken.to_string());
                    return Err(actix_web::error::ErrorUnauthorized(err.to_string()));
                }
            };

            // Guardar usuario autenticado en la request
            req.extensions_mut().insert(JWTAuthMiddleware { claims });

             // ============================
            // 4. CONTINUAR PIPELINE
            // ============================
            Ok(srv.call(req).await?.map_into_left_body())
        })
    }
}


// ==================================
// Middleware de chequeo de roles
// ==================================
#[derive(Clone)]
pub struct RoleCheck {
    roles: Vec<UserRole>,
}

impl RoleCheck {
    pub fn new(roles: Vec<UserRole>) -> Self {
        Self { roles }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RoleCheck
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = RoleCheckMiddleware<S>;
    type InitError = ();
    type Future = Pin<Box<dyn Future<Output = Result<Self::Transform, Self::InitError>> + 'static>>;

    fn new_transform(&self, service: S) -> Self::Future {
        let roles = self.roles.clone();
        Box::pin(async move {
            Ok(RoleCheckMiddleware {
                service: Rc::new(service),
                roles,
            })
        })
    }
}

pub struct RoleCheckMiddleware<S> {
    service: Rc<S>,
    roles: Vec<UserRole>,
}

impl<S, B> Service<ServiceRequest> for RoleCheckMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let srv = self.service.clone();
        let roles = self.roles.clone();

        async move {
            // Simulación: extracción de rol del usuario (por header, cookie o claims)
            // En la práctica, podrías sacar esto del "extensions" o JWT.
            let user_role = extract_user_role(&req);

            // Verificación inline (reemplaza a role_check)
            let authorized = roles.contains(&user_role);

            if !authorized {
                // Rechazar acceso
                let (req, _) = req.into_parts();
                let res = HttpResponse::Forbidden()
                    .body("Permission Denied")
                    .map_into_right_body();
                return Ok(ServiceResponse::new(req, res));
            }

            // Continuar flujo normal
            let res = srv.call(req).await?.map_into_left_body();
            Ok(res)
        }
        .boxed_local()
    }
}

// 🔹 Ejemplo básico de extracción de rol (puedes adaptarlo a tu JWT o base de datos)
fn extract_user_role(req: &ServiceRequest) -> UserRole {
    req.extensions()
        .get::<JWTAuthMiddleware>()
        .map(|auth| auth.claims.role.clone())
        .unwrap_or(UserRole::User)
}




#[derive(Clone)]
pub enum RequiredAccess {
    Role(UserRole),
    PremiumAccess,
    OwnedCourse(Uuid),
    AnyCourseAccess,
}

#[derive(Clone)]
pub struct AccessCheck {
    required: Vec<RequiredAccess>,
}

impl AccessCheck {
    pub fn new(required: Vec<RequiredAccess>) -> Self {
        Self { required }
    }
}

impl<S, B> Transform<S, ServiceRequest> for AccessCheck
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = AccessCheckMiddleware<S>;
    type InitError = ();
    type Future = Pin<Box<dyn Future<Output = Result<Self::Transform, Self::InitError>> + 'static>>;

    fn new_transform(&self, service: S) -> Self::Future {
        let required = self.required.clone();
        Box::pin(async move {
            Ok(AccessCheckMiddleware {
                service: Rc::new(service),
                required,
            })
        })
    }
}

pub struct AccessCheckMiddleware<S> {
    service: Rc<S>,
    required: Vec<RequiredAccess>,
}

impl<S, B> Service<ServiceRequest> for AccessCheckMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let srv = self.service.clone();
        let required = self.required.clone();

        async move {
            let app_data = req.app_data::<Data<Arc<AppState>>>().unwrap();
            let db_client = &app_data.db_client;
            let claims = {
                let extensions = req.extensions();
                extensions
                    .get::<JWTAuthMiddleware>()
                    .map(|auth| auth.claims.clone())
            };

            let claims = match claims {
                Some(claims) => claims,
                None => {
                    let (req, _) = req.into_parts();
                    let res = HttpResponse::Unauthorized()
                        .json(serde_json::json!({ "error": "Unauthorized" }))
                        .map_into_right_body();

                    return Ok(ServiceResponse::new(req, res));
                }
            };


            let mut allowed = false;

            // Revisar cada requisito de acceso
            for access in required.iter() {
                match access {
                    RequiredAccess::Role(role) => {
                        if &claims.role == role {
                            allowed = true;
                        }
                    }
                    RequiredAccess::PremiumAccess => {
                        // Verificar si el usuario tiene suscripción activa (status true) o cancelada pero no expirada
                        let has_access = db_client.check_user_has_active_subscription(claims.sub).await;
                        if has_access.is_ok() && has_access.unwrap() {
                            allowed = true;
                        }
                    }
                    RequiredAccess::OwnedCourse(course_id) => {
                        // Verificar si el usuario ha comprado este curso
                        let has_access = db_client.check_user_course_access(claims.sub,*course_id).await;
                        if has_access.is_ok() && has_access.is_ok() {
                            allowed = true;
                        }
                    }
                    RequiredAccess::AnyCourseAccess => {
                        // Verificar si el usuario tiene suscripción activa o cancelada pero no expirada
                        let has_subscription = db_client.check_user_has_active_subscription(claims.sub).await;
                        if has_subscription.is_ok() && has_subscription.unwrap() {
                            allowed = true;
                            continue;
                        }

                        // Si no tiene suscripción, verificar si tiene acceso a algún curso comprado
                        let purchased_courses = db_client.get_user_purchased_courses(claims.sub)
                            .await;
                        if purchased_courses.is_ok() && !purchased_courses.unwrap().is_empty() {
                            allowed = true;
                        }
                    }
                }
            }

            if !allowed {
                let (req, _) = req.into_parts();
                let res = HttpResponse::Forbidden()
                    .json(serde_json::json!({"error": "Permission denied"}))
                    .map_into_right_body();
                return Ok(ServiceResponse::new(req, res));
            }

            let res = srv.call(req).await?.map_into_left_body();
            Ok(res)
        }
        .boxed_local()
    }
}
