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
#[allow(dead_code)]
impl UserRole {
    pub fn to_str(&self) -> &str {
        match self {
            UserRole::Admin => "admin",
            UserRole::User => "user",
        }
    }
}

// ===================== //
// MODELOS PRINCIPALES
// ===================== //

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

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserSettings {
    pub id: Uuid,
    pub user_id: Uuid,
    pub email_notifications: bool,
    pub push_notifications: bool,
    pub course_reminders: bool,
    pub new_content: bool,
    pub two_factor_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ===================== //
// CURSOS Y PROGRESO
// ===================== //
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Course {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub price: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserCourse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub course_id: Uuid,
    pub purchase_date: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct CourseProgress {
    pub id: Uuid,
    pub user_id: Uuid,
    pub course_id: Uuid,
    pub progress_percentage: f32,
    pub total_lessons: Option<i32>,
    pub completed_lessons: Option<i32>,
    pub last_accessed: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ===================== //
// LOGROS
// ===================== //
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Achievement {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserAchievement {
    pub id: Uuid,
    pub user_id: Uuid,
    pub achievement_id: Uuid,
    pub earned: bool,
    pub earned_at: Option<DateTime<Utc>>,
}

// ===================== //
// NOTIFICACIONES
// ===================== //
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Notification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub message: String,
    pub sent_via: String,
    pub sent_at: DateTime<Utc>,
    pub read: bool,
}

// ===================== //
// SUSCRIPCIONES
// ===================== //
#[allow(dead_code)]
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

// ===================== //
// RELACIONES ENTRE MODELOS
// ===================== //
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct UserWithSettings {
    pub user: User,
    pub settings: Option<UserSettings>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct UserCourseWithProgress {
    pub user_course: UserCourse,
    pub progress: Option<CourseProgress>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct UserWithAchievements {
    pub user: User,
    pub achievements: Vec<UserAchievement>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct FullUserProfile {
    pub user: User,
    pub settings: Option<UserSettings>,
    pub courses: Vec<UserCourseWithProgress>,
    pub achievements: Vec<UserAchievement>,
    pub notifications: Vec<Notification>,
    pub subscriptions: Vec<Subscription>,
}
