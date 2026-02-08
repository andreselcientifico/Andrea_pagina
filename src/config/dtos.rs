use core::str;
use chrono::{ DateTime, Utc, NaiveDate };
use serde::{ Deserialize, Serialize };
use uuid::Uuid;
use validator::Validate; 

use crate::models::models::{ Achievement, Certificate, Course, Subscription, User, UserRole};

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
    pub phone: Option<String>,
    pub location: Option<String>,
    pub bio: Option<String>,
    pub avatar: Option<String>,
    pub email_notifications: Option<bool>,
    pub course_reminders: Option<bool>,
    pub new_content: Option<bool>,
    #[serde(rename = "birthDate")]
    pub birth_date: Option<NaiveDate>, 
    pub role: Option<UserRole>,
    pub verified: Option<bool>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl FilterUserDto {
    pub fn filter_user(user: &User) -> Self {
        FilterUserDto {
            id: Some(user.id.to_string()),
            name: Some(user.name.to_owned()),
            email: Some(user.email.to_owned()),
            phone: user.phone.to_owned(),
            location: user.location.to_owned(),
            bio: user.bio.to_owned(),
            avatar: user.profile_image_url.to_owned(),
            email_notifications: Some(user.email_notifications),
            course_reminders: Some(user.course_reminders),
            new_content: Some(user.new_content),
            birth_date: user.birth_date,
            role: user.role.clone().into(),
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
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateLessonProgressDTO {
    pub is_completed: bool,
    pub progress: Option<f64>,
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

#[derive(Serialize, Deserialize, Validate)]
pub struct VerifyEmailQueryDTO {
    #[validate(length(min = 1, message = "El token es requerido"))]
    pub token: String,
}

#[derive(Deserialize, Serialize, Validate, Debug, Clone)]
pub struct ForgotPasswordRequestDTO {
    #[validate(
        length(min = 1, message = "El correo electrónico es requerido"),
        email(message = "El correo electrónico no es válido")
    )]
    pub email: String,
}

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


#[derive(Validate, Debug, Clone, Serialize, Deserialize)]
pub struct CreateCourseDTO {
    #[validate(length(min = 1, message = "El título del curso es requerido"))]
    pub title: String,

    #[validate(length(min = 1, message = "La descripción corta es requerida"))]
    pub description: String,

    pub long_description: Option<String>,

    #[validate(length(min = 1, message = "El nivel es requerido"))]
    pub level: String, // "básico" | "intermedio" | "avanzado"

    #[validate(range(min = 0.0, message = "El precio debe ser mayor a 0"))]
    pub price: f64,

    pub duration: Option<String>, // ej: "4 semanas"

    pub students: Option<i32>, // se puede calcular por defecto

    #[validate(url(message = "La URL de la imagen no es válida"))]
    pub image: Option<String>, // URL de imagen

    #[validate(length(min = 1, message = "La categoría es requerida"))]
    pub category: String, // "básico" | "premium"

    #[serde(default)]
    pub features: Option<Vec<String>>, // JSONB -> Vec<String>

    pub paypal_product_id: Option<String>,

    #[serde(default)]
    pub modules: Vec<CreateModuleDTO>, // array de videos
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct CoursePageRow {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub long_description: Option<String>,
    pub level: String,
    pub duration: String,
    pub students: i32,
    pub paypal_product_id: Option<String>,
    pub price: Option<f64>,
    pub image: Option<String>,
    pub category: String,
    pub features: Vec<String>,

    // Rating
    pub rating_average: f64,
    pub rating_count: i64,

    // User related
    pub purchased: bool,
    pub has_active_subscription: bool,
}


#[derive(Validate, Debug, Clone, Serialize, Deserialize)]
pub struct CreateLessonDTO {
    #[validate(length(min = 1, message = "El título de la lección es requerido"))]
    pub title: String,
    
    pub duration: Option<String>,
    pub completed: bool,
    #[serde(rename = "type")]
    #[validate(length(min = 1, message = "El tipo de lección es requerido"))]
    pub r#type: String, // video | exercise | quiz
    
    pub content_url: Option<String>,
    pub description: Option<String>,
    
    // El orden es opcional en la entrada, se puede calcular si no se proporciona
    pub order: Option<i32>, 
}

#[derive(Validate, Debug, Clone, Serialize, Deserialize)]
pub struct CreateModuleDTO {
    #[validate(length(min = 1, message = "El título del módulo es requerido"))]
    pub title: String,
    
    // El orden es opcional en la entrada, se puede calcular si no se proporciona
    pub order: Option<i32>, 
    
    #[serde(default)]
    pub lessons: Vec<CreateLessonDTO>,
}


#[derive(Validate, Debug, Clone, Serialize, Deserialize,PartialEq)]
pub struct UpdateCourseDTO {
    #[validate(length(min = 1, message = "El título del curso es requerido"))]
    pub title: Option<String>,

    #[validate(length(min = 1, message = "La descripción corta es requerida"))]
    pub description: Option<String>,

    pub long_description: Option<String>,

    #[validate(length(min = 1, message = "El nivel es requerido"))]
    pub level: Option<String>, // "básico" | "intermedio" | "avanzado"

    #[validate(range(min = 0.0, message = "El precio debe ser mayor a 0"))]
    pub price: Option<f64>,

    pub duration: Option<String>, // ej: "4 semanas"

    pub students: Option<i32>, // se puede calcular por defecto

    #[validate(url(message = "La URL de la imagen no es válida"))]
    pub image: Option<String>, // URL de imagen

    #[validate(length(min = 1, message = "La categoría es requerida"))]
    pub category: Option<String>, // "básico" | "premium"

    #[serde(default)]
    pub features: Option<Vec<String>>, // JSONB -> Vec<String>

    #[serde(default)]
    pub modules: Option<Vec<UpdateModuleDTO>>, // array de videos
}

impl PartialEq<Course> for UpdateCourseDTO {
    fn eq(&self, other: &Course) -> bool {
        self.title == Some(other.title.clone())
            && self.description == Some(other.description.clone())
            // Comparación correcta de Option<String> con String
            && self.long_description == other.long_description
            && self.level == Some(other.level.clone())
            && self.price == Some(other.price)
            && self.duration == other.duration
            && self.students == Some(other.students)
            && self.image == other.image
            && self.category == Some(other.category.clone())
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UpdateLessonDTO {
    // Si 'id' está presente, se actualiza; si es None, se crea una nueva lección.
    pub id: Option<Uuid>, 
    pub module_id: Option<Uuid>,
    // Los campos son Option<T> si se permite la actualización parcial
    pub title: Option<String>, 
    pub duration: Option<String>,
    pub completed: Option<bool>,
    #[serde(rename = "type")]
    pub r#type: Option<String>,
    pub content_url: Option<String>,
    pub description: Option<String>,
    pub order: Option<i32>, 
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UpdateModuleDTO {
    // Si 'id' está presente, se actualiza; si es None, se crea un nuevo módulo.
    pub id: Option<Uuid>, 

    pub title: Option<String>,
    pub order: Option<i32>,

    #[serde(default)]
    // Aquí el Option<Vec> permite que se omita la lista de lecciones si no se van a actualizar
    pub lessons: Option<Vec<UpdateLessonDTO>>, 
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LessonDto {
    pub id: Uuid,
    pub title: String,
    pub duration: Option<String>,
    pub completed: Option<bool>,
    pub r#type: String,
    pub content_url: Option<String>,
    pub description: Option<String>,
    pub order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModuleWithLessonsDto {
    pub id: Uuid,
    pub title: String,
    pub order: i32,
    pub lessons: Vec<LessonDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CourseWithModulesDto {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub long_description: Option<String>,
    pub price: f64,
    pub level: String,
    pub duration: Option<String>,
    pub students: i32,
    pub image: Option<String>,
    pub category: String,
    pub features: Option<Vec<String>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,

    pub total_lessons: i64,
    pub completed_lessons: i64,

    pub modules: Vec<ModuleWithLessonsDto>,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct CourseResponseDTO {
    pub id: String,
    pub name: String,
    pub description: String,
    pub price: f64,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
}


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
    #[validate(length(min = 1, message = "El ID de transacción es requerido"))]
    pub transaction_id: String,
}


#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct ProductDTO {
    #[validate(length(min = 1, message = "El nombre del producto es requerido"))]
    pub name: String,
    #[validate(length(min = 1, message = "La descripción del producto es requerida"))]
    pub description: String,
    pub type_: String, 
    pub category: String, 
    #[validate(url(message = "La URL de la imagen no es válida"))]
    pub image_url: Option<String>,
    pub home_url: Option<String>,
}


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


#[derive(Validate, Debug, Clone, Serialize, Deserialize)]
pub struct VerifyPaymentDTO {
    #[serde(default)]
    pub payment_id: Option<String>,
    #[serde(default)]
    pub transaction_id: Option<String>,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct UserPaymentStatusDTO {
    pub user_id: String,
    pub course_id: String,
    pub paid: bool,
    pub payment_date: Option<DateTime<Utc>>,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct UserProfileResponse {
    pub status: String,
    pub data: UserProfileData,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct UserProfileData {
    pub user: FilterUserDto,
    pub courses: Vec<UserCourseDto>,
    pub achievements: Vec<UserAchievementDto>,
    pub subscriptions: Vec<Subscription>,
    pub certificates: Vec<Certificate>,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserProfileDto {
    pub name: Option<String>,
    pub phone: Option<String>,
    pub location: Option<String>,
    pub bio: Option<String>,
    pub birth_date: Option<chrono::NaiveDate>,
    pub profile_image_url: Option<String>,
}

// Nuevos DTOs para courses y achievements (tipo "filter" como FilterUserDto)

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct FilterCourseDto {
    pub id: Uuid,
    pub title: Option<String>,
    pub description: Option<String>,
    pub long_description: Option<String>,
    pub price: Option<f64>,
    pub level: Option<String>,
    pub duration: Option<String>,
    pub students: Option<i32>,
    pub image: Option<String>,
    pub category: Option<String>,
    pub rating: i32,
    pub features: Option<Vec<String>>,
    pub paypal_product_id: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl FilterCourseDto {
    pub fn filter_course(course: &UserCourseDto) -> Self {
        let features: Option<Vec<String>> = course.features.as_ref().and_then(|v| {
            serde_json::from_value(v.clone()).ok()
        });
        FilterCourseDto {
            id: course.id,
            title: Some(course.title.to_owned()),
            description: Some(course.description.to_owned()),
            long_description: course.long_description.clone(),
            price: Some(course.price),
            level: Some(course.level.clone()),
            duration: course.duration.clone(),
            students: Some(course.students),
            image: course.image.clone(),
            category: Some(course.category.clone()),
            rating: course.rating.clone(),
            paypal_product_id: course.paypal_product_id.clone(),
            features,
            created_at: Some(course.created_at),
            updated_at: Some(course.updated_at),
        }
    }
    
    pub fn filter_courses(list: &[UserCourseDto]) -> Vec<FilterCourseDto> {
        list.iter().map(|c| FilterCourseDto::filter_course(c)).collect()
    }

}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserAchievementDto {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub trigger_type: String,
    pub trigger_value: i32,
    pub active: bool,
    pub earned: bool,
    pub earned_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}



#[derive(Debug, Serialize, Deserialize)]
pub struct FilterAchievementDto {
    pub id: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<DateTime<Utc>>,
    // añade otros campos que tenga tu modelo Achievement si los necesitas (p.ej. points)
}

impl FilterAchievementDto {
    pub fn filter_achievement(a: &Achievement) -> Self {
        FilterAchievementDto {
            id: Some(a.id.to_string()),
            // adapta names según tu modelo Achievement
            title: Some(a.name.to_owned()),
            description: a.description.clone(),
            created_at: Some(a.created_at),
        }
    }

    pub fn filter_achievements(list: &[Achievement]) -> Vec<FilterAchievementDto> {
        list.iter().map(|a| FilterAchievementDto::filter_achievement(a)).collect()
    }
}

#[derive(Debug, Serialize, Deserialize, Validate, sqlx::FromRow)]
pub struct CreatedCommentDto {
    #[validate(length(min = 1, message = "El comentario no puede estar vacío"))]
    pub content: String
}

#[derive(Debug, Serialize, Deserialize, Validate, sqlx::FromRow)]
pub struct CreatedRatingDto {
    pub rating: i32,
}

#[derive(Debug, Serialize, Deserialize, Validate, sqlx::FromRow)]
pub struct CommentLessonDto {
    pub id: Uuid,
    pub user_id: Uuid,
    pub lesson_id: Uuid,
    pub user_name: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct CourseRatingDto {
    pub average: f64,
    pub count: i64,
    pub user_rating: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserCourseDto {
    pub id: Uuid,
    pub title: String,                       
    pub description: String,                  
    pub long_description: Option<String>,    
    pub level: String,                        
    pub price: f64,
    pub duration: Option<String>,            
    pub students: i32,                                              
    pub image: Option<String>,                
    pub category: String,                     
    pub rating: i32,
    pub features: Option<serde_json::Value>,
    pub paypal_product_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ===================== //
// QUIZ DTOs
// ===================== //

#[derive(Debug, Serialize, Deserialize)]
pub struct QuizResponseDto {
    pub id: Option<String>,
    pub lesson_id: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub pass_percentage: Option<f64>,
    pub total_questions: i64,
    pub order: i32, // Note: Quiz doesn't have order in DB, usually follows Lesson order.
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OptionDto {
    pub id: Option<String>,
    pub text: String,
    pub order: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuestionDto {
    pub id: String,
    pub question: String,
    pub description: Option<String>,
    pub options: Vec<OptionDto>,
    // Only verify/correct_option if needed generally not sent to frontend for taking quiz
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correct_option_id: Option<String>, 
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explanation: Option<String>,
    pub order: i32,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct SubmitAnswerDto {
    pub question_id: String,
    pub selected_option_id: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct SubmitQuizDto {
    pub answers: Vec<SubmitAnswerDto>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuizResultDetailDto {
    pub question_id: String,
    pub question: String,
    pub selected_option_id: String,
    pub correct_option_id: String,
    pub is_correct: bool,
    pub explanation: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuizSubmissionResponseDto {
    pub submission_id: String,
    pub quiz_id: String,
    pub score: i32,
    pub total_score: i32,
    pub percentage: f64,
    pub passed: bool,
    pub submitted_at: DateTime<Utc>,
    pub results: Vec<QuizResultDetailDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certificate: Option<CertificateDto>, // Optional certificate if generated after submission
}

// DTOs para administración de Quizzes
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateOptionDto {
    pub text: String,
    pub is_correct: bool,
    pub order: i32,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateQuestionDto {
    pub question: String,
    pub description: Option<String>,
    pub explanation: Option<String>,
    pub order: i32,
    pub options: Vec<CreateOptionDto>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateQuizDto {
    pub lesson_id: String,
    pub title: String,
    pub description: Option<String>,
    pub pass_percentage: Option<f64>,
    pub questions: Vec<CreateQuestionDto>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct QuizAttemptDto {
    pub id: Uuid,
    pub quiz_id: Uuid,
    pub user_id: Uuid,
    pub score: i32,
    pub percentage: f64,
    pub passed: bool,
    pub submitted_at: DateTime<Utc>,
}

// ===================== //
// CERTIFICATE DTOs
// ===================== //

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct CertificateDto {
    pub id: Uuid,
    pub course_id: Uuid,
    pub user_id: Uuid,
    pub user_name: String,
    pub course_title: String,
    pub issue_date: DateTime<Utc>,
    pub completion_percentage: f64,
    pub certificate_number: String,
}