use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
    },
    Argon2,
};

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

    Ok(
        Argon2::default()
        .hash_password(password.as_bytes(), &SaltString::generate(&mut OsRng))
        .map_err(|_| ErrorMessage::HashingError)?
        .to_string()
    )
}

pub fn verify_password(
    password: &str,
    hashed_password: &str,
) -> Result<bool, ErrorMessage> {
    if password.is_empty() {
        return Err(ErrorMessage::EmptyPassword);
    }

    if password.len() > MAX_PASSWORD_LENGTH {
        return Err(ErrorMessage::ExceededMaxPasswordLength(MAX_PASSWORD_LENGTH));
    }

    Ok(Argon2::default().verify_password(password.as_bytes(), &PasswordHash::new(&hashed_password)
        .map_err(|_| ErrorMessage::InvalidHashFormat)?).map_or(false, |_| true))
}