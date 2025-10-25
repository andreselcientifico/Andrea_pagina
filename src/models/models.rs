use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};


// ===================== //
//    ROLES DE USUARIO
// ===================== //
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::Type, Copy, PartialEq)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    User,
}

// ==================== //
//   Métodos para UserRole
// ==================== //
impl UserRole {
    pub fn to_str(&self) -> &str {
        match self {
            UserRole::Admin => "admin",
            UserRole::User => "user",
        }
    }
}

///===================== ///
///   Modelo de usuario
/// ==================== ///

#[derive(Serialize, Deserialize, sqlx::FromRow, Debug, sqlx::Type, Clone)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub verified: bool,
    pub password: String,
    pub role: Option<UserRole>,
    pub verification_token: Option<String>,
    pub token_expiry: Option<DateTime<Utc>>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<DateTime<Utc>>,
}

///===================== ///
///  Modelo de SUSCRIPCIÓN
/// =================== ///
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Subscription {
    pub id: Uuid,
    pub user_id: Uuid,
    pub paypal_subscription_id: String,
    pub status: bool,
    pub plan_id: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

///===================== ///
///  Modelo de CURSO
/// =================== ///
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Course {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub price: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

///===================== ///
/// Modelo: RELACION USUARIO-CURSO (Compras)
/// =================== ///
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserCourse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub course_id: Uuid,
    pub purchase_date: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

///===================== ///
///  MODELO: PROGRESO DE CURSO
/// =================== ///
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct CourseProgress {
    pub id: Uuid,
    pub user_id: Uuid,
    pub course_id: Uuid,
    pub progress_percentage: f32,
    pub last_accessed: DateTime<Utc>,
}