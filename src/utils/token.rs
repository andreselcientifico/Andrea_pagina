use chrono::{Utc, Duration};
use serde::{Serialize, Deserialize};
use jsonwebtoken::{encode, decode, EncodingKey, DecodingKey, Header, Validation, Algorithm, errors::Error as JwtError};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenClaims {
    pub sub: String,
    pub iat: usize,
    pub exp: usize,
}
pub fn create_token_rsa(user_id: &str, secret: &EncodingKey, expiration_in_seconds: i64) -> Result<String, JwtError> {
    if user_id.is_empty() {
        return Err(jsonwebtoken::errors::ErrorKind::InvalidSubject.into());
    }

    let now = Utc::now();

    encode(
        &Header::new(Algorithm::RS256),
        &TokenClaims {
            sub: user_id.to_owned(),
            iat: now.timestamp() as usize,
            exp: (now + Duration::seconds(expiration_in_seconds)).timestamp() as usize,
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