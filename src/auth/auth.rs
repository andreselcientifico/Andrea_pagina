use jsonwebtoken::{decode, Validation, DecodingKey, Algorithm};
use serde::{Serialize, Deserialize};
use std::fs;
use chrono::{Utc};
use crate::{models::models::UserRole, utils::token::TokenClaims};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserJwtData {
    pub sub: Uuid,           // user id
    pub role: UserRole,        // rol del usuario
    pub exp: usize,          // expiración
    pub iat: usize,          // issued at
}

impl UserJwtData {
    pub fn id(&self) -> Uuid {
        self.sub
    }
}



pub fn is_premium(claims: &TokenClaims) -> bool {
    match claims.subscription_expires_at {
        Some(ts) => ts > Utc::now().timestamp(),
        None => false,
    }
}

/// ===============================
/// Verificar y decodificar JWT
/// ===============================
pub fn verify_jwt(token: &str) -> Option<UserJwtData> {
    // Leer clave pública
    let public_key_pem = fs::read("public.pem")
        .expect("Error leyendo public.pem");

    let validation = Validation::new(Algorithm::RS256);

    match decode::<UserJwtData>(
        token,
        &DecodingKey::from_rsa_pem(&public_key_pem)
            .expect("Clave pública inválida"),
        &validation,
    ) {
        Ok(data) => Some(data.claims),
        Err(err) => {
            log::warn!("JWT inválido: {}", err);
            None
        }
    }
}