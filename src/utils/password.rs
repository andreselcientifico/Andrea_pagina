use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use sha2::{Digest, Sha256};

use crate::errors::error::ErrorMessage;

const MAX_PASSWORD_LENGTH: usize = 64;

pub fn hash_password(password: impl Into<String>) -> Result<String, ErrorMessage> {
    let password = password.into();

    if password.is_empty() {
        return Err(ErrorMessage::EmptyPassword);
    }

    if password.len() > MAX_PASSWORD_LENGTH {
        return Err(ErrorMessage::ExceededMaxPasswordLength(MAX_PASSWORD_LENGTH));
    }

    Ok(Argon2::default()
        .hash_password(password.as_bytes(), &SaltString::generate(&mut OsRng))
        .map_err(|_| ErrorMessage::HashingError)?
        .to_string())
}

pub fn hash_token(token: impl Into<String>) -> Result<String, ErrorMessage> {
    let token = token.into();

    if token.is_empty() {
        return Err(ErrorMessage::EmptyPassword);
    }

    if token.len() > MAX_PASSWORD_LENGTH {
        return Err(ErrorMessage::ExceededMaxPasswordLength(MAX_PASSWORD_LENGTH));
    }
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    Ok(hex::encode(hasher.finalize()))
}

pub fn verify_password(password: &str, hashed_password: &str) -> Result<bool, ErrorMessage> {
    if password.is_empty() {
        return Err(ErrorMessage::EmptyPassword);
    }

    if password.len() > MAX_PASSWORD_LENGTH {
        return Err(ErrorMessage::ExceededMaxPasswordLength(MAX_PASSWORD_LENGTH));
    }

    Ok(Argon2::default()
        .verify_password(
            password.as_bytes(),
            &PasswordHash::new(&hashed_password).map_err(|_| ErrorMessage::InvalidHashFormat)?,
        )
        .map_or(false, |_| true))
}
