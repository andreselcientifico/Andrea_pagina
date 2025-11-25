use std::{rc::Rc, sync::Arc};
use actix_web::{
    Error, HttpMessage, HttpResponse, body::EitherBody, dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready}, http::header
};
use futures::{FutureExt, future::{LocalBoxFuture, Ready, ready}};
use uuid::Uuid;
use std::pin::Pin;


use crate::{
    AppState, auth::auth::verify_jwt, db::db::UserExt, errors::error::{ErrorMessage, HttpError}, models::models::{User, UserRole}, utils::token::decode_token
};

/// Estructura que contendrá al usuario autenticado
#[derive(Debug, Clone)]
pub struct JWTAuthMiddleware {
    pub user: User,
}

/// Middleware principal de autenticación JWT
pub struct AuthMiddlewareFactory {
    pub app_state: Arc<AppState>,
}

impl AuthMiddlewareFactory {
    /// Middleware solo para autenticación
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self { app_state }
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthMiddlewareFactory
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddleware {
            service: Rc::new(service),
            app_state: self.app_state.clone(),
        }))
    }
}

pub struct AuthMiddleware<S> {
    service: Rc<S>,
    app_state: Arc<AppState>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self,  req: ServiceRequest) -> Self::Future {
        let app_state = self.app_state.clone();
        let srv = self.service.clone();

        Box::pin(async move {
            // Buscar token desde cookie o header Authorization
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

            // Verificar JWT
            let user_id = match verify_jwt(&token) {
                Some(id) => id,
                None => {
                    let err = HttpError::unauthorized(ErrorMessage::InvalidToken.to_string());
                    return Err(actix_web::error::ErrorUnauthorized(err.to_string()));
                }
            };

            let user_id = match Uuid::parse_str(&user_id) {
                Ok(id) => id,
                Err(_) => {
                    let err = HttpError::unauthorized(ErrorMessage::InvalidToken.to_string());
                    return Err(actix_web::error::ErrorUnauthorized(err.to_string()));
                }
            };

            // Buscar usuario en la base de datos
            let user = app_state.db_client
                .get_user(Some(user_id), None, None, None)
                .await
                .map_err(|_| actix_web::error::ErrorUnauthorized("Usuario no encontrado"))?;

            let user = match user {
                Some(u) => u,
                None => {
                    let err = HttpError::unauthorized(ErrorMessage::UserNoLongerExist.to_string());
                    return Err(actix_web::error::ErrorUnauthorized(err.to_string()));
                }
            };

            // Guardar usuario autenticado en la request
            req.extensions_mut().insert(JWTAuthMiddleware { user });

            // Continuar con la request
            Ok(srv.call(req).await?)
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
    type Future = Pin<Box<dyn Future<Output = Result<Self::Transform, Self::InitError>>>>;

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
    let app_data = req.app_data::<actix_web::web::Data<Arc<AppState>>>();
    if app_data.is_none() {
        return UserRole::User;
    }

    let app_state = app_data.unwrap().as_ref();

    let cookie = req.cookie("token");
    if cookie.is_none() {
        return UserRole::User;
    }

    let token = cookie.unwrap().value().to_string();

    let decoded = decode_token(
        token,
        app_state.env.decoding_key.clone()
    );

    if decoded.is_err() {
        return UserRole::User;
    }

    let claims = decoded.unwrap();

    claims.role
}
