use jsonwebtoken::{decode, Validation, DecodingKey};
use serde::{Serialize, Deserialize};
use std::fs;
use jsonwebtoken::Algorithm::RS256;
use chrono::Utc;

use crate::utils::token::TokenClaims;

/// Datos dentro del token JWT
#[derive(Debug, Serialize, Deserialize, Clone)]
struct Claims {
    sub: String, // subject (user id)
    exp: usize,  // expiration time as unix timestamp
}

#[allow(dead_code)]
pub fn is_premium(claims: &TokenClaims) -> bool {
    match claims.subscription_expires_at {
        Some(ts) => ts > Utc::now().timestamp(),
        None => false,
    }
}

/// Verificar y decodificar Token JWT
pub fn verify_jwt(token: &str) -> Option<String> {
    let public_key_pem = fs::read("public.pem").expect("Error leyendo public_key.pem");
    match decode::<Claims>(
        token,
        &DecodingKey::from_rsa_pem(&public_key_pem).expect("Clave pública inválida"),
        &Validation::new(RS256),
    ) {
        Ok(data) => Some(data.claims.sub),
        Err(_) => None,
    }
}