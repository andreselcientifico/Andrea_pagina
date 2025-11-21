use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::{config::dtos::{CourseWithVideos, CreateCourseDTO}, models::models::{Achievement, Course, Payment, User, UserAchievement, UserRole, Videos}};

#[derive(Debug, Clone)]
pub struct DBClient {
    pool: Pool<Postgres>,
}

impl DBClient {
    pub fn new(pool: Pool<Postgres>) -> Self {
        DBClient { pool }
    }
}

#[async_trait]
pub trait UserExt {
    async fn get_user(
        &self,
        user_id: Option<Uuid>,
        name: Option<&str>,
        email: Option<&str>,
        token: Option<&str>,
    ) -> Result<Option<User>, sqlx::Error>;

    async fn get_users(
        &self,
        page: u32,
        limit: usize,
    ) -> Result<Vec<User>, sqlx::Error>;

    async fn save_user<T: Into<String> + Send>(
        &self,
        name: T,
        email: T,
        password: T,
        verification_token: T,
        token_expiry: Option<DateTime<Utc>>,
        role: Option<UserRole>,
    ) -> Result<User, sqlx::Error>;

    async fn get_user_count(&self) -> Result<i64, sqlx::Error>;

    async fn update_user_name<T: Into<String> + Send>(
        &self,
        user_id: Uuid,
        name: T,
    ) -> Result<User, sqlx::Error>;

    async fn update_user_role(
        &self,
        user_id: Uuid,
        role: UserRole,
    ) -> Result<User, sqlx::Error>;

    async fn update_user_password(
        &self,
        user_id: Uuid,
        password: String,
    ) -> Result<User, sqlx::Error>;

    async fn update_user_profile(
        &self,
        user_id: Uuid,
        name: Option<String>,
        phone: Option<String>,
        location: Option<String>,
        bio: Option<String>,
        birth_date: Option<chrono::NaiveDate>,
        profile_image_url: Option<String>,
    ) -> Result<User, sqlx::Error>;

    #[allow(dead_code)]
    async fn verifed_token(
        &self,
        token: &str,
    ) -> Result<(), sqlx::Error>;

    #[allow(dead_code)]
    async fn add_verifed_token(
        &self,
        user_id: Uuid,
        token: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<(), sqlx::Error>;
}

#[async_trait]
impl UserExt for DBClient {
    async fn get_user(
    &self,
    user_id: Option<Uuid>,
    name: Option<&str>,
    email: Option<&str>,
    token: Option<&str>,
) -> Result<Option<User>, sqlx::Error> {
    let mut user: Option<User> = None;
    if let Some(user_id) = user_id {
        user = sqlx::query_as!(
            User,
            r#"
            SELECT 
                id, 
                name, 
                email, 
                phone,
                location,
                bio,
                birth_date,
                password, 
                verified, 
                created_at, 
                updated_at, 
                verification_token, 
                token_expiry, 
                role as "role: UserRole",
                profile_image_url
            FROM users
            WHERE id = $1
            "#,
            user_id
        ).fetch_optional(&self.pool).await?;
    } else if let Some(name) = name {
        user = sqlx::query_as!(
            User,
            r#"
            SELECT 
                id, 
                name, 
                email, 
                phone,
                location,
                bio,
                birth_date,
                password, 
                verified, 
                created_at, 
                updated_at, 
                verification_token, 
                token_expiry, 
                role as "role: UserRole",
                profile_image_url
            FROM users
            WHERE name = $1
            "#,
            name
        ).fetch_optional(&self.pool).await?;
    } else if let Some(email) = email {
        user = sqlx::query_as!(
            User,
            r#"
            SELECT 
                id, 
                name, 
                email, 
                phone,
                location,
                bio,
                birth_date,
                password, 
                verified, 
                created_at, 
                updated_at, 
                verification_token, 
                token_expiry, 
                role as "role: UserRole",
                profile_image_url
            FROM users
            WHERE email = $1
            "#,
            email
        ).fetch_optional(&self.pool).await?;
    } else if let Some(token) = token {
        user = sqlx::query_as!(
            User,
            r#"
            SELECT 
                id, 
                name, 
                email, 
                phone,
                location,
                bio,
                birth_date,
                password, 
                verified, 
                created_at, 
                updated_at, 
                verification_token, 
                token_expiry, 
                role as "role: UserRole",
                profile_image_url
            FROM users
            WHERE verification_token = $1
            "#,
            token
        ).fetch_optional(&self.pool).await?;
    }

    Ok(user)
}

    async fn get_users(
        &self,
        page: u32,
        limit: usize,
    ) -> Result<Vec<User>, sqlx::Error> {
        let offset = (page - 1) * limit as u32;

        let users = sqlx::query_as!(
            User,
            r#"SELECT 
                id, 
                name, 
                email, 
                phone,
                location,
                bio,
                birth_date,
                password, 
                verified, 
                created_at, 
                updated_at, 
                verification_token, 
                token_expiry, 
                role as "role: UserRole",
                profile_image_url
            FROM users
            ORDER BY created_at DESC LIMIT $1 OFFSET $2"#,
            limit as i64,
            offset as i64,
        ).fetch_all(&self.pool)
        .await?;

        Ok(users)
    }

    async fn save_user<T: Into<String> + Send>(
        &self,
        name: T,
        email: T,
        password: T,
        verification_token: T,
        token_expiry: Option<DateTime<Utc>>,
        role: Option<UserRole>,
    ) -> Result<User, sqlx::Error> {
        let role = role.unwrap_or(UserRole::User);
        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (name, email, password, verification_token, token_expiry, role) 
            VALUES ($1, $2, $3, $4, $5, $6) 
            RETURNING
                id, 
                name, 
                email, 
                phone,
                location,
                bio,
                birth_date,
                password, 
                verified, 
                created_at, 
                updated_at, 
                verification_token, 
                token_expiry, 
                role as "role: UserRole",
                profile_image_url
            "#,
            name.into(),
            email.into(),
            password.into(),
            verification_token.into(),
            token_expiry,
            role as _
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(user)
    }

    async fn get_user_count(&self) -> Result<i64, sqlx::Error> {
        let count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM users"#
        )
       .fetch_one(&self.pool)
       .await?;

        Ok(count.unwrap_or(0))
    }

    async fn update_user_name<T: Into<String> + Send>(
        &self,
        user_id: Uuid,
        new_name: T
    ) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            r#"
            UPDATE users
            SET name = $1, updated_at = Now()
            WHERE id = $2
            RETURNING
                id, 
                name, 
                email, 
                phone,
                location,
                bio,
                birth_date,
                password, 
                verified, 
                created_at, 
                updated_at, 
                verification_token, 
                token_expiry, 
                role as "role: UserRole",
                profile_image_url
            "#,
            new_name.into(),
            user_id
        ).fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    async fn update_user_role(
        &self,
        user_id: Uuid,
        new_role: UserRole
    ) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            r#"
            UPDATE users
            SET role = $1, updated_at = Now()
            WHERE id = $2
            RETURNING 
                id, 
                name, 
                email, 
                phone,
                location,
                bio,
                birth_date,
                password, 
                verified, 
                created_at, 
                updated_at, 
                verification_token, 
                token_expiry, 
                role as "role: UserRole",
                profile_image_url
            "#,
            new_role as UserRole,
            user_id
        ).fetch_one(&self.pool)
       .await?;

        Ok(user)
    }

    async fn update_user_profile(
        &self,
        user_id: Uuid,
        name: Option<String>,
        phone: Option<String>,
        location: Option<String>,
        bio: Option<String>,
        birth_date: Option<chrono::NaiveDate>,
        profile_image_url: Option<String>,
    ) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            r#"
            UPDATE users
            SET
                name = COALESCE($1, name),
                phone = COALESCE($2, phone),
                location = COALESCE($3, location),
                bio = COALESCE($4, bio),
                birth_date = COALESCE($5, birth_date),
                profile_image_url = COALESCE($6, profile_image_url),
                updated_at = NOW()
            WHERE id = $7
            RETURNING
                id,
                name,
                email,
                phone,
                location,
                bio,
                birth_date,
                password,
                verified,
                created_at,
                updated_at,
                verification_token,
                token_expiry,
                role as "role: UserRole",
                profile_image_url
            "#,
            name,
            phone,
            location,
            bio,
            birth_date,
            profile_image_url,
            user_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    async fn update_user_password(
        &self,
        user_id: Uuid,
        new_password: String
    ) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            r#"
            UPDATE users
            SET password = $1, updated_at = Now()
            WHERE id = $2
            RETURNING
                id, 
                name, 
                email, 
                phone,
                location,
                bio,
                birth_date,
                password, 
                verified, 
                created_at, 
                updated_at, 
                verification_token, 
                token_expiry, 
                role as "role: UserRole",
                profile_image_url
            "#,
            new_password,
            user_id
        ).fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    async fn verifed_token(
        &self,
        token: &str,
    ) -> Result<(), sqlx::Error> {
        let _ =sqlx::query!(
            r#"
            UPDATE users
            SET verified = true, 
                updated_at = Now(),
                verification_token = NULL,
                token_expiry = NULL
            WHERE verification_token = $1
            "#,
            token
        ).execute(&self.pool)
       .await;

        Ok(())
    }

    async fn add_verifed_token(
        &self,
        user_id: Uuid,
        token: &str,
        token_expiry: DateTime<Utc>,
    ) -> Result<(), sqlx::Error> {
        let _ = sqlx::query!(
            r#"
            UPDATE users
            SET verification_token = $1, token_expiry = $2, updated_at = Now()
            WHERE id = $3
            "#,
            token,
            token_expiry,
            user_id,
        ).execute(&self.pool)
       .await?;

        Ok(())
    }
}

#[async_trait]
pub trait CourseExt {
    async fn create_course(
        &self,
        dto: CreateCourseDTO,
    ) -> Result<CourseWithVideos, sqlx::Error>;

    async fn get_course(&self, course_id: Uuid) -> Result<Option<Course>, sqlx::Error>;

    async fn get_user_courses(&self, user_id: Uuid) -> Result<Vec<Course>, sqlx::Error>;

    async fn get_courses(
        &self,
        page: u32,
        limit: usize,
    ) -> Result<Vec<Course>, sqlx::Error>;

    async fn update_course(
        &self,
        course_id: Uuid,
        title: Option<String>,
        description: Option<String>,
        price: Option<f64>,
    ) -> Result<Course, sqlx::Error>;

    async fn delete_course(&self, course_id: Uuid) -> Result<(), sqlx::Error>;

    #[allow(dead_code)]
    async fn get_course_count(&self) -> Result<i64, sqlx::Error>;
}

#[async_trait]
impl CourseExt for DBClient {
    async fn create_course(
    &self,
    dto: CreateCourseDTO,
    ) -> Result<CourseWithVideos, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        // Insertar curso
        let course = sqlx::query_as::<_, Course>(
            r#"
            INSERT INTO courses
                (id, title, description, long_description, level, price, duration, students, rating, image, category, features, created_at, updated_at)
            VALUES
                ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14)
            RETURNING *
            "#
        )
        .bind(id)
        .bind(dto.title)
        .bind(dto.description)
        .bind(dto.long_description)
        .bind(dto.level)
        .bind(dto.price)
        .bind(dto.duration)
        .bind(dto.students.unwrap_or(0))
        .bind(dto.rating.unwrap_or(5.0))
        .bind(dto.image)
        .bind(dto.category)
        .bind(dto.features.map(|f| serde_json::to_value(f).unwrap_or(serde_json::Value::Null)))
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        // Insertar videos
        let mut videos: Vec<Videos> = Vec::new();
        for (idx, v) in dto.videos.into_iter().enumerate() {
            let video = sqlx::query_as::<_, Videos>(
                r#"
                INSERT INTO videos (course_id, "order", title, url, duration, created_at, updated_at)
                VALUES ($1,$2,$3,$4,$5,$6,$7)
                RETURNING *
                "#
            )
            .bind(course.id)
            .bind(v.order.unwrap_or((idx + 1) as i32))
            .bind(v.title)
            .bind(v.url)
            .bind(v.duration)
            .bind(now)
            .bind(now)
            .fetch_one(&self.pool)
            .await?;

            videos.push(video);
        }

        Ok(CourseWithVideos { course, videos })
    }

    async fn get_course(&self, course_id: Uuid) -> Result<Option<Course>, sqlx::Error> {
        let course = sqlx::query_as::<_, Course>(
            r#"SELECT * FROM courses WHERE id = $1"#,
        )
        .bind(course_id)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(course)
    }

    async fn get_user_courses(&self, user_id: Uuid) -> Result<Vec<Course>, sqlx::Error> {
        let courses = sqlx::query_as::<_, Course>(
            r#"
            SELECT c.*
            FROM courses c
            INNER JOIN user_courses uc ON uc.course_id = c.id
            WHERE uc.user_id = $1
            ORDER BY c.created_at DESC
            "#
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(courses)
    }

    async fn get_courses(
        &self,
        page: u32,
        limit: usize,
    ) -> Result<Vec<Course>, sqlx::Error> {
        let offset = ((page - 1) * limit as u32) as i64;
        
        let courses = sqlx::query_as::<_, Course>(
            r#"SELECT * FROM courses 
               ORDER BY created_at DESC LIMIT $1 OFFSET $2"#,
        )
        .bind(limit as i64)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;
        
        Ok(courses)
    }

    async fn update_course(
        &self,
        course_id: Uuid,
        title: Option<String>,
        description: Option<String>,
        price: Option<f64>,
    ) -> Result<Course, sqlx::Error> {
        let course = sqlx::query_as::<_, Course>(
            r#"UPDATE courses 
               SET title = COALESCE($2, title), 
                   description = COALESCE($3, description), 
                   price = COALESCE($4, price), 
                   updated_at = $5 
               WHERE id = $1 
               RETURNING *"#,
        )
        .bind(course_id)
        .bind(title)
        .bind(description)
        .bind(price)
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await?;
        
        Ok(course)
    }

    async fn delete_course(&self, course_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM courses WHERE id = $1")
            .bind(course_id)
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }

    async fn get_course_count(&self) -> Result<i64, sqlx::Error> {
        let result = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM courses")
            .fetch_one(&self.pool)
            .await?;
        
        Ok(result)
    }
}

#[async_trait]
pub trait PaymentExt {
    async fn create_payment(
        &self,
        user_id: Uuid,
        course_id: Uuid,
        amount: f64,
        payment_method: String,
        transaction_id: String,
    ) -> Result<Payment, sqlx::Error>;

    async fn get_payment(&self, payment_id: Uuid) -> Result<Option<Payment>, sqlx::Error>;

    async fn get_user_payments(&self, user_id: Uuid) -> Result<Vec<Payment>, sqlx::Error>;

    #[allow(dead_code)]
    async fn get_course_payments(&self, course_id: Uuid) -> Result<Vec<Payment>, sqlx::Error>;

    async fn update_payment_status(
        &self,
        payment_id: Uuid,
        status: String,
    ) -> Result<Payment, sqlx::Error>;

    async fn check_user_course_payment(
        &self,
        user_id: Uuid,
        course_id: Uuid,
    ) -> Result<Option<Payment>, sqlx::Error>;

    #[allow(dead_code)]
    async fn get_payment_count(&self) -> Result<i64, sqlx::Error>;
}

#[async_trait]
impl PaymentExt for DBClient {
    async fn create_payment(
        &self,
        user_id: Uuid,
        course_id: Uuid,
        amount: f64,
        payment_method: String,
        transaction_id: String,
    ) -> Result<Payment, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        
        let payment = sqlx::query_as::<_, Payment>(
            r#"INSERT INTO payments (id, user_id, course_id, amount, payment_method, transaction_id, status, created_at) 
               VALUES ($1, $2, $3, $4, $5, $6, 'pending', $7) 
               RETURNING id, user_id, course_id, amount, payment_method, transaction_id, status, created_at, updated_at"#,
        )
        .bind(id)
        .bind(user_id)
        .bind(course_id)
        .bind(amount)
        .bind(payment_method)
        .bind(transaction_id)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(payment)
    }

    async fn get_payment(&self, payment_id: Uuid) -> Result<Option<Payment>, sqlx::Error> {
        let payment = sqlx::query_as::<_, Payment>(
            r#"SELECT id, user_id, course_id, amount, payment_method, transaction_id, status, created_at, updated_at 
               FROM payments WHERE id = $1"#,
        )
        .bind(payment_id)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(payment)
    }

    async fn get_user_payments(&self, user_id: Uuid) -> Result<Vec<Payment>, sqlx::Error> {
        let payments = sqlx::query_as::<_, Payment>(
            r#"SELECT id, user_id, course_id, amount, payment_method, transaction_id, status, created_at, updated_at 
               FROM payments WHERE user_id = $1 ORDER BY created_at DESC"#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;
        
        Ok(payments)
    }

    async fn get_course_payments(&self, course_id: Uuid) -> Result<Vec<Payment>, sqlx::Error> {
        let payments = sqlx::query_as::<_, Payment>(
            r#"SELECT id, user_id, course_id, amount, payment_method, transaction_id, status, created_at, updated_at 
               FROM payments WHERE course_id = $1 ORDER BY created_at DESC"#,
        )
        .bind(course_id)
        .fetch_all(&self.pool)
        .await?;
        
        Ok(payments)
    }

    async fn update_payment_status(
        &self,
        payment_id: Uuid,
        status: String,
    ) -> Result<Payment, sqlx::Error> {
        let payment = sqlx::query_as::<_, Payment>(
            r#"UPDATE payments SET status = $2, updated_at = $3 WHERE id = $1 
               RETURNING id, user_id, course_id, amount, payment_method, transaction_id, status, created_at, updated_at"#,
        )
        .bind(payment_id)
        .bind(status)
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await?;
        
        Ok(payment)
    }

    async fn check_user_course_payment(
        &self,
        user_id: Uuid,
        course_id: Uuid,
    ) -> Result<Option<Payment>, sqlx::Error> {
        let payment = sqlx::query_as::<_, Payment>(
            r#"SELECT id, user_id, course_id, amount, payment_method, transaction_id, status, created_at, updated_at 
               FROM payments WHERE user_id = $1 AND course_id = $2 AND status = 'completed' LIMIT 1"#,
        )
        .bind(user_id)
        .bind(course_id)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(payment)
    }

    async fn get_payment_count(&self) -> Result<i64, sqlx::Error> {
        let result = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM payments WHERE status = 'completed'"
        )
        .fetch_one(&self.pool)
        .await?;
        
        Ok(result)
    }
}

#[allow(dead_code)]
#[async_trait]
pub trait AchievementExt {
    /// Crea un nuevo logro.
    async fn create_achievement<T: Into<String> + Send>(
        &self,
        name: T,
        description: Option<T>,
        icon: Option<T>,
    ) -> Result<Achievement, sqlx::Error>;

    /// Obtiene todos los logros existentes (paginados).
    async fn get_achievements(
        &self,
        page: u32,
        limit: usize,
    ) -> Result<Vec<Achievement>, sqlx::Error>;

    /// Obtiene un logro por su ID.
    async fn get_achievement(&self, achievement_id: Uuid)
        -> Result<Option<Achievement>, sqlx::Error>;

    /// Elimina un logro existente.
    async fn delete_achievement(&self, achievement_id: Uuid) -> Result<(), sqlx::Error>;
}

/// Extensión para gestionar los logros obtenidos por usuarios.
#[async_trait]
pub trait UserAchievementExt {
    /// Asigna un logro a un usuario (sin marcarlo como ganado aún).
    #[allow(dead_code)]
    async fn assign_achievement_to_user(
        &self,
        user_id: Uuid,
        achievement_id: Uuid,
    ) -> Result<UserAchievement, sqlx::Error>;

    /// Marca un logro como ganado.
    #[allow(dead_code)]
    async fn earn_achievement(
        &self,
        user_id: Uuid,
        achievement_id: Uuid,
    ) -> Result<UserAchievement, sqlx::Error>;

    /// Obtiene todos los logros de un usuario.
    async fn get_user_achievements(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<Achievement>, sqlx::Error>;

    /// Verifica si un usuario ya ha ganado un logro específico.
    #[allow(dead_code)]
    async fn has_user_earned(
        &self,
        user_id: Uuid,
        achievement_id: Uuid,
    ) -> Result<bool, sqlx::Error>;
}

/// Implementación para la conexión principal del sistema (`DBClient`).
#[async_trait]
impl AchievementExt for DBClient {
    async fn create_achievement<T: Into<String> + Send>(
        &self,
        name: T,
        description: Option<T>,
        icon: Option<T>,
    ) -> Result<Achievement, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        let achievement = sqlx::query_as::<_, Achievement>(
            r#"
            INSERT INTO achievements (id, name, description, icon, created_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, name, description, icon, created_at
            "#
        )
        .bind(id)
        .bind(name.into())
        .bind(description.map(|d| d.into()))
        .bind(icon.map(|i| i.into()))
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(achievement)
    }

    async fn get_achievements(
        &self,
        page: u32,
        limit: usize,
    ) -> Result<Vec<Achievement>, sqlx::Error> {
        let offset = ((page - 1) * limit as u32) as i64;

        let achievements = sqlx::query_as::<_, Achievement>(
            r#"
            SELECT id, name, description, icon, created_at
            FROM achievements
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#
        )
        .bind(limit as i64)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(achievements)
    }

    async fn get_achievement(&self, achievement_id: Uuid)
        -> Result<Option<Achievement>, sqlx::Error> {
        let achievement = sqlx::query_as::<_, Achievement>(
            r#"
            SELECT id, name, description, icon, created_at
            FROM achievements
            WHERE id = $1
            "#
        )
        .bind(achievement_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(achievement)
    }

    async fn delete_achievement(&self, achievement_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM achievements WHERE id = $1")
            .bind(achievement_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

#[async_trait]
impl UserAchievementExt for DBClient {
    async fn assign_achievement_to_user(
        &self,
        user_id: Uuid,
        achievement_id: Uuid,
    ) -> Result<UserAchievement, sqlx::Error> {
        let id = Uuid::new_v4();

        let user_achievement = sqlx::query_as::<_, UserAchievement>(
            r#"
            INSERT INTO user_achievements (id, user_id, achievement_id, earned, earned_at)
            VALUES ($1, $2, $3, false, NULL)
            RETURNING id, user_id, achievement_id, earned, earned_at
            "#
        )
        .bind(id)
        .bind(user_id)
        .bind(achievement_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(user_achievement)
    }

    async fn earn_achievement(
        &self,
        user_id: Uuid,
        achievement_id: Uuid,
    ) -> Result<UserAchievement, sqlx::Error> {
        let user_achievement = sqlx::query_as::<_, UserAchievement>(
            r#"
            UPDATE user_achievements
            SET earned = true, earned_at = $3
            WHERE user_id = $1 AND achievement_id = $2
            RETURNING id, user_id, achievement_id, earned, earned_at
            "#
        )
        .bind(user_id)
        .bind(achievement_id)
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await?;

        Ok(user_achievement)
    }

    async fn get_user_achievements(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<Achievement>, sqlx::Error> {
        let achievements = sqlx::query_as::<_, Achievement>(
            r#"
            SELECT 
                a.id,
                a.name,
                a.description,
                a.icon,
                a.created_at
            FROM achievement a
            INNER JOIN user_achievement ua 
                ON ua.achievement_id = a.id
            WHERE ua.user_id = $1
            AND ua.earned = TRUE
            ORDER BY ua.earned_at DESC NULLS LAST
            "#
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(achievements)
    }

    async fn has_user_earned(
        &self,
        user_id: Uuid,
        achievement_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let exists = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM user_achievements
                WHERE user_id = $1 AND achievement_id = $2 AND earned = true
            )
            "#
        )
        .bind(user_id)
        .bind(achievement_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(exists)
    }
}