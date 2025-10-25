use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Serialize, Deserialize};
use std::env;
use chrono::{Utc, Duration};

/// Hasear Contraseña
pub fn hash_password(password: &str) -> String {
    hash(password, DEFAULT_COST).expect("Error al hashear la contraseña")
}

/// Verificar Contraseña
pub fn verify_password(password: &str, hash: &str) -> bool {
    verify(password, hash).unwrap_or(false)
}

/// Datos dentro del token JWT
#[derive(Debug, Serialize, Deserialize, Clone)]
struct Claims {
    sub: String, // subject (user id)
    exp: usize,  // expiration time as unix timestamp
}

/// Generar Token JWT de session
pub fn generate_jwt(user_id: &str) -> String {
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET no está configurado");
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("Error al calcular la expiración del token")
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_owned(),
        exp: expiration,
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref()))
        .expect("Error al generar el token JWT")
}

/// Verificar y decodificar Token JWT
pub fn verify_jwt(token: &str) -> Option<String> {
    match decode::<Claims>(
        token,
        &DecodingKey::from_secret(env::var("JWT_SECRET").expect("JWT_SECRET no está configurado").as_ref()),
        &Validation::default(),
    ) {
        Ok(data) => Some(data.claims.sub),
        Err(_) => None,
    }
}