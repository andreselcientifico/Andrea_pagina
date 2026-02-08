use jsonwebtoken::{decode, Validation, DecodingKey, Algorithm};
use serde::{Serialize, Deserialize};
use std::fs;
use uuid::Uuid;

use crate::models::models::UserRole;

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