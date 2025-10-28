use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Serialize, Deserialize};
use std::fs;
use chrono::{Utc, Duration};
use jsonwebtoken::Algorithm::RS256;

/// Datos dentro del token JWT
#[derive(Debug, Serialize, Deserialize, Clone)]
struct Claims {
    sub: String, // subject (user id)
    exp: usize,  // expiration time as unix timestamp
}

/// Generar Token JWT de session
#[allow(dead_code)]
pub fn generate_jwt(user_id: &str) -> String {
    let private_key_pem = fs::read("private_key.pem").expect("Error leyendo private_key.pem");
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("Error al calcular la expiración del token")
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_owned(),
        exp: expiration,
    };

    encode(&Header::new(RS256), &claims, &EncodingKey::from_rsa_pem(&private_key_pem).expect("Clave privada invalida"))
        .expect("Error al generar el token JWT")
}

/// Verificar y decodificar Token JWT
pub fn verify_jwt(token: &str) -> Option<String> {
    let public_key_pem = fs::read("public_key.pem").expect("Error leyendo public_key.pem");
    match decode::<Claims>(
        token,
        &DecodingKey::from_rsa_pem(&public_key_pem).expect("Clave pública inválida"),
        &Validation::new(RS256),
    ) {
        Ok(data) => Some(data.claims.sub),
        Err(_) => None,
    }
}