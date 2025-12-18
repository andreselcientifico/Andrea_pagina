use core::str;
use chrono::{ DateTime, Utc, NaiveDate };
use serde::{ Deserialize, Serialize };
use uuid::Uuid;
use validator::Validate; 

use crate::models::models::{ Achievement, Course, User, UserRole};

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
    #[serde(rename = "birthDate")]
    pub birth_date: Option<NaiveDate>, 
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
            phone: user.phone.to_owned(),
            location: user.location.to_owned(),
            bio: user.bio.to_owned(),
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

    pub rating: Option<f32>, // calificación inicial, por defecto 5.0
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

#[allow(dead_code)]
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

    pub rating: Option<f32>, // calificación inicial, por defecto 5.0
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
            && self.rating == Some(other.rating)
            && self.image == other.image
            && self.category == Some(other.category.clone())
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UpdateLessonDTO {
    // Si 'id' está presente, se actualiza; si es None, se crea una nueva lección.
    pub id: Option<Uuid>, 
    
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
    pub rating: f32,
    pub image: Option<String>,
    pub category: String,
    pub features: Option<Vec<String>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,

    pub modules: Vec<ModuleWithLessonsDto>,
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
    #[validate(length(min = 1, message = "El ID de transacción es requerido"))]
    pub transaction_id: String,
}

#[allow(dead_code)]
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

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct UserProfileResponse {
    pub status: String,
    pub data: UserProfileData,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct UserProfileData {
    pub user: FilterUserDto,
    pub courses: Vec<FilterCourseDto>,
    pub achievements: Vec<FilterAchievementDto>,
}

#[allow(dead_code)]
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
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct FilterCourseDto {
    pub id: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub long_description: Option<String>,
    pub price: Option<f64>,
    pub level: Option<String>,
    pub duration: Option<String>,
    pub students: Option<i32>,
    pub rating: Option<f32>,
    pub image: Option<String>,
    pub category: Option<String>,
    pub features: Option<Vec<String>>, // JSONB -> Vec<String>
    pub paypal_product_id: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<DateTime<Utc>>,
}

impl FilterCourseDto {
    pub fn filter_course(course: &Course) -> Self {
        let features: Option<Vec<String>> = course.features.as_ref().and_then(|v| {
            serde_json::from_value(v.clone()).ok()
        });
        FilterCourseDto {
            id: Some(course.id.to_string()),
            title: Some(course.title.to_owned()),
            description: Some(course.description.to_owned()),
            long_description: course.long_description.clone(),
            price: Some(course.price),
            level: Some(course.level.clone()),
            duration: course.duration.clone(),
            students: Some(course.students),
            rating: Some(course.rating),
            image: course.image.clone(),
            category: Some(course.category.clone()),
            paypal_product_id: course.paypal_product_id.clone(),
            features, // ya convertido a Option<Vec<String>>
            created_at: Some(course.created_at),
            updated_at: Some(course.updated_at),
        }
    }

    pub fn filter_courses(courses: &[Course]) -> Vec<FilterCourseDto> {
        courses.iter().map(|c| FilterCourseDto::filter_course(c)).collect()
    }
}

#[allow(dead_code)]
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