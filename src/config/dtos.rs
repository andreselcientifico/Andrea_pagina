use core::str;
use chrono::{ DateTime, Utc };
use serde::{ Deserialize, Serialize };
use validator::Validate; 

use crate::models::models::{ UserRole, User };

#[derive(Validate, Debug, Default, Clone, Serialize, Deserialize)]
pub struct RegisterDTO {
    #[validate(length(min = 1, message = "El nombre de usuario es requerido"))]
    pub name: String,
    #[validate(
        length(min = 1, message = "El correo electrónico es requerido"),
        email(message = "El correo electrónico no es válido")
    )]
    pub email: String,
    #[validate(
        length(min = 6, message = "La contraseña debe tener al menos 6 caracteres"),
    )]
    pub password: String,
    #[validate(
        length(min = 1, message = "Confirmar contraseña es requerido"),
        must_match(other = "password", message = "Las contraseñas no coinciden")
    )]
    #[serde(rename = "confirmPassword")]
    pub confirm_password: String,
}

#[derive(Validate, Debug, Default, Clone, Serialize, Deserialize)]
pub struct LoginDTO {
    #[validate(
        length(min = 1, message = "El correo electrónico es requerido"),
        email(message = "El correo electrónico no es válido")
    )]
    pub email: String,
    #[validate(
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

#[derive(Debug, Serialize, Deserialize)]
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
            id: Some(user.id.to_string()),
            name: Some(user.name.to_owned()),
            email: Some(user.email.to_owned()),
            role: Some(user.role.expect("Falta el role")),
            verified: Some(user.verified),
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

#[derive(Debug, Serialize, Deserialize)]
pub struct UserListResponseDto {
    pub status: String,
    pub users: Vec<FilterUserDto>,
    pub results: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserLoginResponseDto {
    pub status: String,
}

#[derive(Serialize, Deserialize)]
pub struct Response {
    pub status: &'static str,
    pub message: String,
}

#[derive(Validate, Debug, Default, Clone, Serialize, Deserialize)]
pub struct NameUpdateDTO {
    #[validate(length(min = 1, message = "El nombre de usuario es requerido"))]
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RoleUpdateDTO {
    #[validate(custom(message = "Rol de usuario inválido", function = "validate_user_role"))]
    pub role: UserRole,
}

fn validate_user_role(role: &UserRole) -> Result<(), validator::ValidationError> {
    match role {
        UserRole::Admin | UserRole::User => Ok(()),
    }
}

#[derive(Debug, Validate, Default, Clone, Serialize, Deserialize)]
pub struct UserPasswordUpdateDTO {
    #[validate(
        length(min = 6, message = "La contraseña debe tener al menos 6 caracteres")
    )]
    #[serde(rename = "old_Password")]
    pub old_password: String,
    #[validate(
        length(min = 6, message = "La nueva contraseña debe tener al menos 6 caracteres")
    )]
    #[serde(rename = "newPassword")]
    pub new_password: String,
    #[validate(
        length(min = 6, message = "Confirmar nueva contraseña debe tener al menos 6 caracteres"),
        must_match(other = "new_password", message = "Las contraseñas no coinciden")
    )]
    #[serde(rename = "confirmNewPassword")]
    pub confirm_new_password: String,
}
#[allow(dead_code)]
#[derive(Serialize, Deserialize, Validate)]
pub struct VerifyEmailQueryDTO {
    #[validate(length(min = 1, message = "El token es requerido"))]
    pub token: String,
}
#[allow(dead_code)]
#[derive(Deserialize, Serialize, Validate, Debug, Clone)]
pub struct ForgotPasswordRequestDTO {
    #[validate(
        length(min = 1, message = "El correo electrónico es requerido"),
        email(message = "El correo electrónico no es válido")
    )]
    pub email: String,
}
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct ResetPasswordRequestDTO {
    #[validate(length(min = 1, message = "El token es requerido"))]
    pub token: String,
    #[validate(
        length(min = 6, message = "La nueva contraseña debe tener al menos 6 caracteres")
    )]
    #[serde(rename = "newPassword")]
    pub new_password: String,
    #[validate(
        length(min = 6, message = "Confirmar nueva contraseña debe tener al menos 6 caracteres"),
        must_match(other = "new_password", message = "Las contraseñas no coinciden")
    )]
    #[serde(rename = "confirmNewPassword")]
    pub confirm_new_password: String,
}

#[allow(dead_code)]
#[derive(Validate, Debug, Default, Clone, Serialize, Deserialize)]
pub struct CreateCourseDTO {
    #[validate(length(min = 1, message = "El nombre del curso es requerido"))]
    pub name: String,
    #[validate(length(min = 1, message = "La descripción es requerida"))]
    pub description: String,
    #[validate(range(min = 0.0, message = "El precio debe ser mayor a 0"))]
    pub price: f64,
}

#[allow(dead_code)]
#[derive(Validate, Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCourseDTO {
    pub name: Option<String>,
    pub description: Option<String>,
    pub price: Option<f64>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct CourseResponseDTO {
    pub id: String,
    pub name: String,
    pub description: String,
    pub price: f64,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
}

#[allow(dead_code)]
#[derive(Validate, Debug, Clone, Serialize, Deserialize)]
pub struct CreatePaymentDTO {
    #[validate(length(min = 1, message = "El ID del curso es requerido"))]
    pub course_id: String,
    #[validate(length(min = 1, message = "El ID del usuario es requerido"))]
    pub user_id: String,
    #[validate(range(min = 0.0, message = "El monto debe ser mayor a 0"))]
    pub amount: f64,
    #[validate(length(min = 1, message = "El método de pago es requerido"))]
    pub payment_method: String,
    #[serde(default)]
    pub transaction_id: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentResponseDTO {
    pub id: String,
    pub course_id: String,
    pub user_id: String,
    pub amount: f64,
    pub status: String, // "pending", "completed", "failed"
    pub payment_method: String,
    pub transaction_id: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<DateTime<Utc>>,
}

#[allow(dead_code)]
#[derive(Validate, Debug, Clone, Serialize, Deserialize)]
pub struct VerifyPaymentDTO {
    #[serde(default)]
    pub payment_id: Option<String>,
    #[serde(default)]
    pub transaction_id: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct UserPaymentStatusDTO {
    pub user_id: String,
    pub course_id: String,
    pub paid: bool,
    pub payment_date: Option<DateTime<Utc>>,
}