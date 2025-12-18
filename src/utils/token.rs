use chrono::{Utc, Duration};
use serde::{Serialize, Deserialize};
use jsonwebtoken::{encode, decode, EncodingKey, DecodingKey, Header, Validation, Algorithm, errors::Error as JwtError};
use uuid::Uuid;

use crate::models::models::UserRole;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenClaims {
    pub sub: Uuid,
    pub role: UserRole,
    pub iat: usize,
    pub exp: usize,
    pub subscription_expires_at: Option<i64>,
}
pub fn create_token_rsa(user_id: Uuid, role:UserRole, subscription_expires_at: Option<i64>, secret: &EncodingKey, expiration_in_seconds: i64) -> Result<String, JwtError> {
    if user_id.is_nil() {
        return Err(jsonwebtoken::errors::ErrorKind::InvalidSubject.into());
    }

    let now = Utc::now();

    encode(
        &Header::new(Algorithm::RS256),
        &TokenClaims {
            sub: user_id,
            role,
            iat: now.timestamp() as usize,
            exp: (now + Duration::seconds(expiration_in_seconds)).timestamp() as usize,
            subscription_expires_at,
        },
        secret,
    )
}

#[allow(dead_code)]
pub fn decode_token<T: Into<String>>(token: T, secret: DecodingKey) -> Result<TokenClaims, JwtError> {
    let token = token.into();
    if token.is_empty() {
        return Err(jsonwebtoken::errors::ErrorKind::InvalidToken.into());
    }
    Ok(decode::<TokenClaims>(
        &token,
        &secret,
        &Validation::new(Algorithm::RS256),
    )?.claims)
}