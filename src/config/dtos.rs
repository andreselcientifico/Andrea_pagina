use core::str;
use chrono::{ DateTime, Utc };
use serde::{ Deserialize, Serialize };

use crate::models::models::{ UserRole, User };

#[derive(Validate, Debug, default, Clone, Serialize, Deserialize)]
pub struct RegisterDTO {
    #[validate(length(min = 1, message = "El nombre de usuario es requerido"))]
    pub name: String,
    #[validate(
        length(min = 1, message = "El correo electrónico es requerido"),
        email(message = "El correo electrónico no es válido")
    )]
    pub email: String,
    #[validate(
        length(min = 1, message = "La contraseña es requerida"),
        length(min = 6, message = "La contraseña debe tener al menos 6 caracteres")
    )]
    pub password: String,
    #[validate(
        length(min = 1, message = "Confirmar contraseña es requerido"),
        must_match(other = "password", message = "Las contraseñas no coinciden")
    )]
    #[serde(rename = "confirmPassword")]
    pub confirm_password: String,
}

#[derive(Validate, Default, Serialize, Deserialize, Debug, Clone)]
pub struct LoginDTO {
    #[validate(
        length(min = 1, message = "El correo electrónico es requerido"),
        email(message = "El correo electrónico no es válido")
    )]
    pub email: String,
    #[validate(
        length(min = 1, message = "La contraseña es requerida"),
        length(min = 6, message = "La contraseña debe tener al menos 6 caracteres")
    )]
    pub password: String,
}

#[derive(Serialize, Deserialize, Validate)]
pub struct RequestQueryDto {
    #[validate(range(min = 1))]
    pub page: Option<usize>,
    #[validate(range(min = 1, max = 50))]
    pub limit: Option<usize>,
}

pub struct FilterUserDto {
    pub id: Option<String>,
    pub name: Option<String>,
    pub email: Option<String>,
    pub role: Option<UserRole>,
    pub verified: Option<bool>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<DateTime<Utc>>,
}

impl FilterUserDto {
    pub fn filter_user(user: &User) -> Self {
        FilterUserDto {
            id: user.id.to_string(),
            name: user.name.to_owned(),
            email: user.email.to_owned(),
            role: user.role.to_str().to_string(),
            verified: user.verified,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }

    pub fn filter_users(user: &[User]) -> Vec<FilterUserDto> {
        user.iter()
            .map(|u| FilterUserDto::filter_user(u))
            .collect()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserData {
    pub user: FilterUserDto,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponseDto {
    pub status: String,
    pub data: UserData,
}
