use std::{rc::Rc, sync::Arc};
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::{header, StatusCode},
    Error, HttpMessage,
};
use futures::future::{ready, Ready, LocalBoxFuture};
use uuid::Uuid;

use crate::{
    auth::auth::verify_jwt,
    db::db::UserExt,
    errors::error::{ErrorMessage, HttpError},
    models::models::{User, UserRole},
    AppState,
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

    /// Middleware que también valida roles
    pub fn with_roles(app_state: Arc<AppState>) -> Self {
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
pub async fn role_check(
    req: ServiceRequest,
    roles: Vec<UserRole>,
) -> Result<ServiceRequest, actix_web::Error> {
    {
    let extensions = req.extensions();
    let user_data = extensions
        .get::<JWTAuthMiddleware>()
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("Usuario no autenticado"))?;

    if let Some(role) = &user_data.user.role {
        if !roles.contains(role) {
            let err = HttpError::new(
                ErrorMessage::PermissionDenied.to_string(),
                StatusCode::FORBIDDEN,
            );
            return Err(actix_web::error::ErrorForbidden(err.to_string()));
        }
    } else {
        let err = HttpError::new("El usuario no tiene rol asignado", StatusCode::FORBIDDEN);
        return Err(actix_web::error::ErrorForbidden(err.to_string()));
    }
    }
    Ok(req)
}