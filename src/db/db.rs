use std::collections::HashMap;

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use sqlx::{Pool, Postgres, query_scalar, query_as, query, Error, Row};
use uuid::Uuid;

use crate::{
    config::dtos::{
        CertificateDto, CommentLessonDto, CoursePageRow, CourseRatingDto, CourseWithModulesDto, CreateCourseDTO, CreateLessonDTO, CreateModuleDTO, CreateQuizDto, LessonDto, ModuleWithLessonsDto, OptionDto, QuestionDto, QuizAttemptDto, QuizResponseDto, UpdateCourseDTO, UserCourseDto, UserProfileData, VerifiedUserData
    },
    models::models::{
        Achievement, Certificate, Course, ForgotPasswordResult, Lesson, Module, PasswordResetToken, Payment, Question, QuizAttempt, Subscription, SubscriptionPlan, User, UserAchievement, UserCourse, UserRole
    }
};

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
    ) -> Result<Option<User>, Error>;

    async fn get_users(
        &self,
        page: u32,
        limit: usize,
    ) -> Result<Vec<User>, Error>;

    async fn save_user<T: Into<String> + Send>(
        &self,
        name: T,
        email: T,
        password: T,
        verification_token: T,
        token_expiry: Option<DateTime<Utc>>,
        role: Option<UserRole>,
    ) -> Result<User, Error>;

    async fn get_user_count(&self) -> Result<i64, Error>;

    async fn update_user_name<T: Into<String> + Send>(
        &self,
        user_id: Uuid,
        name: T,
    ) -> Result<User, Error>;

    async fn update_user_role(
        &self,
        user_id: Uuid,
        role: UserRole,
    ) -> Result<User, Error>;

    async fn update_user_password(
        &self,
        user_id: Uuid,
        password: String,
    ) -> Result<User, Error>;

    async fn update_user_profile(
        &self,
        user_id: Uuid,
        name: Option<String>,
        phone: Option<String>,
        location: Option<String>,
        bio: Option<String>,
        birth_date: Option<chrono::NaiveDate>,
        profile_image_url: Option<String>,
    ) -> Result<User, Error>;


    async fn verify_email_atomic(
        &self,
        token: &str,
    ) -> Result<Option<VerifiedUserData>, Error>;


    async fn update_verification_token(
        &self,
        email: &str,
        new_token: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<Option<(String, String)>, Error>;

    async fn increment_user_stat(
        &self,
        user_id: Uuid,
        stat_type: &str,
    ) -> Result<i32, Error>;

    async fn get_user_stats(
        &self,
        user_id: Uuid,
    ) -> Result<HashMap<String, i32>, Error>;

    async fn get_user_complete_profile(
        &self,
        user_id: Uuid,
    ) -> Result<UserProfileData, Error>;

    /// Gets users who have a specific notification preference enabled.
    /// notification_type can be: "email_notifications", "course_reminders", "new_content"
    async fn get_users_by_notification_type(
        &self,
        notification_type: &str,
    ) -> Result<Vec<(String, String)>, Error>;

    /// Updates user notification preferences
    async fn update_notification_settings(
        &self,
        user_id: Uuid,
        email_notifications: Option<bool>,
        course_reminders: Option<bool>,
        new_content: Option<bool>,
    ) -> Result<(), Error>;
}

#[async_trait]
impl UserExt for DBClient {
    /// Obtiene un usuario usando **exactamente un criterio de búsqueda**.
    /// 
    /// Criterios permitidos (solo uno):
    /// - `user_id`
    /// - `name`
    /// - `email`
    /// - `token`
    ///
    /// Retorna:
    /// - `Ok(Some(User))` si se encuentra
    /// - `Ok(None)` si no existe
    /// - `Err` si se envían 0 o más de 1 criterios
    async fn get_user(
        &self,
        user_id: Option<Uuid>,
        name: Option<&str>,
        email: Option<&str>,
        token: Option<&str>,
    ) -> Result<Option<User>, Error> {
        // ============================
        // 1. VALIDACIÓN LIGERA
        // ============================
        let provided = [
            user_id.is_some(),
            name.is_some(),
            email.is_some(),
            token.is_some(),
        ]
        .iter()
        .filter(|&&v| v)
        .count();

        if provided != 1 {
            return Err(Error::Protocol(
                "Debe enviarse exactamente un criterio de búsqueda".into(),
            ));
        }

        // ============================
        // 2. QUERY ÚNICA
        // ============================
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT 
                id, name, email, phone, location, bio, birth_date,
                password, verified,
                email_notifications, course_reminders, new_content,
                created_at, updated_at,
                verification_token, token_expiry,
                role as "role: UserRole",
                profile_image_url,
                subscription_expires_at
            FROM users
            WHERE
                ($1::uuid IS NULL OR id = $1)
            AND ($2::text IS NULL OR name = $2)
            AND ($3::text IS NULL OR email = $3)
            AND ($4::text IS NULL OR verification_token = $4)
            LIMIT 1
            "#,
            user_id,
            name,
            email,
            token
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            log::error!("ERROR get_user: {}", e);
            e
        })?;

        Ok(user)
    }

    async fn get_users(
        &self,
        page: u32,
        limit: usize,
    ) -> Result<Vec<User>, Error> {
        let offset = (page - 1) * limit as u32;
        let mut tx = self.pool.begin().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;

        let users = query_as!(
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
                email_notifications, course_reminders, new_content,
                created_at, 
                updated_at, 
                verification_token, 
                token_expiry, 
                role as "role: UserRole",
                profile_image_url,
                subscription_expires_at
            FROM users
            ORDER BY created_at DESC LIMIT $1 OFFSET $2"#,
            limit as i64,
            offset as i64,
        ).fetch_all(&mut *tx)
        .await.map_err(|e| {
            log::error!("ERROR: {}", e);
            e
        })?
        ;
        tx.commit().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
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
    ) -> Result<User, Error> {
        let mut tx = self.pool.begin().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        let role = role.unwrap_or(UserRole::User);
        let user = query_as!(
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
                email_notifications, course_reminders, new_content,
                created_at, 
                updated_at, 
                verification_token, 
                token_expiry, 
                role as "role: UserRole",
                profile_image_url,
                subscription_expires_at
            "#,
            name.into(),
            email.into(),
            password.into(),
            verification_token.into(),
            token_expiry,
            role as _
        )
        .fetch_one(&mut *tx)
        .await.map_err(|e| {
            log::error!("ERROR: {}", e);
            e
        })?
        ;
        tx.commit().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        Ok(user)
    }

    async fn get_user_count(&self) -> Result<i64, Error> {
        let mut tx = self.pool.begin().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        let count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM users"#
        )
       .fetch_one(&mut *tx)
       .await.map_err(|e| {
            log::error!("ERROR: {}", e);
            e
        })?
        ;
        tx.commit().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        Ok(count.unwrap_or(0))
    }

    async fn update_user_name<T: Into<String> + Send>(
        &self,
        user_id: Uuid,
        new_name: T
    ) -> Result<User, Error> {
        let mut tx = self.pool.begin().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        let user = query_as!(
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
                email_notifications, course_reminders, new_content,
                created_at, 
                updated_at, 
                verification_token, 
                token_expiry, 
                role as "role: UserRole",
                profile_image_url,
                subscription_expires_at
            "#,
            new_name.into(),
            user_id
        ).fetch_one(&mut *tx)
        .await.map_err(|e| {
            log::error!("ERROR: {}", e);
            e
        })?
        ;
        tx.commit().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        Ok(user)
    }

    async fn update_user_role(
        &self,
        user_id: Uuid,
        new_role: UserRole
    ) -> Result<User, Error> {
        let mut tx = self.pool.begin().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        let user = query_as!(
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
                email_notifications, course_reminders, new_content,
                created_at, 
                updated_at, 
                verification_token, 
                token_expiry, 
                role as "role: UserRole",
                profile_image_url,
                subscription_expires_at
            "#,
            new_role as UserRole,
            user_id
        ).fetch_one(&mut *tx)
       .await.map_err(|e| {
            log::error!("ERROR: {}", e);
            e
        })?
        ;
        tx.commit().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
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
    ) -> Result<User, Error> {
        let mut tx = self.pool.begin().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        let user = query_as!(
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
                email_notifications, course_reminders, new_content,
                created_at,
                updated_at,
                verification_token,
                token_expiry,
                role as "role: UserRole",
                profile_image_url,
                subscription_expires_at
            "#,
            name,
            phone,
            location,
            bio,
            birth_date,
            profile_image_url,
            user_id
        )
        .fetch_one(&mut *tx)
        .await.map_err(|e| {
            log::error!("ERROR: {}", e);
            e
        })?
        ;
        tx.commit().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        Ok(user)
    }

    async fn update_user_password(
        &self,
        user_id: Uuid,
        new_password: String
    ) -> Result<User, Error> {
        let mut tx = self.pool.begin().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        let user = query_as!(
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
                email_notifications, course_reminders, new_content,
                created_at, 
                updated_at, 
                verification_token, 
                token_expiry, 
                role as "role: UserRole",
                profile_image_url,
                subscription_expires_at
            "#,
            new_password,
            user_id
        ).fetch_one(&mut *tx)
        .await.map_err(|e| {
            log::error!("ERROR: {}", e);
            e
        })?
        ;
        tx.commit().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        Ok(user)
    }

    async fn verify_email_atomic(
        &self,
        token: &str,
    ) -> Result<Option<VerifiedUserData>, Error> {
        let row = sqlx::query(
            r#"
            UPDATE users
            SET verified = true, 
                updated_at = NOW(),
                verification_token = NULL,
                token_expiry = NULL
            WHERE verification_token = $1
              AND token_expiry > NOW()
              AND verified = false
            RETURNING id, name, email, role as "role: String"
            "#
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            log::error!("Error verificando el token: {}", e);
            e
        })?;

        // Mapeamos el resultado si existe
        Ok(if let Some(r) = row {
            let role_str: String = r.try_get("role")?;
            let role = match role_str.as_str() {
                "Admin" => crate::db::db::UserRole::Admin, // Ajusta según tu enum real
                _ => crate::db::db::UserRole::User,        // Ajusta según tu enum real
            };

            Some(VerifiedUserData {
                id: r.try_get("id")?,
                name: r.try_get("name")?,
                email: r.try_get("email")?,
                role,
            })
        } else {
            None
        })
    }

    async fn update_verification_token(
        &self,
        email: &str,
        new_token: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<Option<(String, String)>, Error> {
        // Actualizamos de una vez y devolvemos el nombre y el correo.
        // Solo afecta a usuarios que tengan verified = false
        let row = sqlx::query!(
            r#"
            UPDATE users
            SET verification_token = $1, 
                token_expiry = $2, 
                updated_at = NOW()
            WHERE email = $3 AND verified = false
            RETURNING name, email
            "#,
            new_token,
            expires_at,
            email
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            log::error!("Error update_verification_token: {}", e);
            e
        })?;

        Ok(row.map(|r| (r.name, r.email)))
    }

    async fn increment_user_stat(
        &self,
        user_id: Uuid,
        stat_type: &str,
    ) -> Result<i32, Error> {
        let mut tx = self.pool.begin().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;

        let value = sqlx::query_scalar!(
            r#"
            INSERT INTO user_stats (user_id, stat_type, value, updated_at)
            VALUES ($1, $2, 1, NOW())
            ON CONFLICT (user_id, stat_type)
            DO UPDATE
                SET value = user_stats.value + 1,
                    updated_at = NOW()
            RETURNING value
            "#,
            user_id,
            stat_type
        )
        .fetch_one(&mut *tx)
        .await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;

        tx.commit().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        Ok(value as i32)
    }

    async fn get_user_stats(
        &self,
        user_id: Uuid,
    ) -> Result<HashMap<String, i32>, Error> {

        let mut stats = HashMap::new();

        // Estadísticas incrementales
        let rows = sqlx::query!(
            "SELECT stat_type, value FROM user_stats WHERE user_id = $1",
            user_id
        )
        .fetch_all(&self.pool)
        .await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;

        for row in rows {
            stats.insert(row.stat_type, row.value);
        }

        // Cursos completados
        let courses_completed = sqlx::query_scalar::<_, i32>(
            r#"
            SELECT COUNT(DISTINCT uc.course_id)
            FROM user_courses uc
            WHERE uc.user_id = $1
            AND EXISTS (
                SELECT 1
                FROM user_lesson_progress lp
                JOIN lessons l ON lp.lesson_id = l.id
                JOIN modules m ON l.module_id = m.id
                WHERE lp.user_id = $1
                AND lp.is_completed = true
                AND m.course_id = uc.course_id
                GROUP BY m.course_id
                HAVING COUNT(*) = (
                    SELECT COUNT(*)
                    FROM lessons l2
                    JOIN modules m2 ON l2.module_id = m2.id
                    WHERE m2.course_id = uc.course_id
                )
            )
            "#
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(0);

        stats.insert("course_completed".to_string(), courses_completed);

        // Lecciones completadas
        let lessons_completed = sqlx::query_scalar::<_, i32>(
            "SELECT COUNT(*) FROM user_lesson_progress WHERE user_id = $1 AND is_completed = true",
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(0);

        stats.insert("lesson_completed".to_string(), lessons_completed);

        // Cursos inscritos
        let courses_enrolled = sqlx::query_scalar::<_, i32>(
            "SELECT COUNT(*) FROM user_courses WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(0);

        stats.insert("courses_enrolled".to_string(), courses_enrolled);

        // Comentarios
        let comments_created = sqlx::query_scalar::<_, i32>(
            "SELECT COUNT(*) FROM comments WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(0);

        stats.insert("comments_created".to_string(), comments_created);


        Ok(stats)
    }

    async fn get_user_complete_profile(
        &self,
        user_id: Uuid,
    ) -> Result<UserProfileData, Error> {

        let row = sqlx::query!(
            r#"
            SELECT profile
            FROM (
                SELECT json_build_object(
                'user', json_build_object(
                    'id', u.id,
                    'name', u.name,
                    'email', u.email,
                    'phone', u.phone,
                    'location', u.location,
                    'bio', u.bio,
                    'avatar', u.profile_image_url,
                    'created_at', u.created_at,
                    'email_notifications', u.email_notifications,
                    'course_reminders', u.course_reminders,
                    'new_content', u.new_content,
                    'role', u.role
                ),

                'courses', COALESCE((
                    SELECT json_agg(course_row ORDER BY created_at DESC)
                    FROM (
                        SELECT
                            c.created_at,
                            jsonb_build_object(
                                'id', c.id,
                                'title', c.title,
                                'description', c.description,
                                'long_description', c.long_description,
                                'level', c.level,
                                'duration', c.duration,
                                'students', c.students,
                                'paypal_product_id', c.paypal_product_id,
                                'price', c.price,
                                'image', c.image,
                                'category', c.category,
                                'rating', COALESCE(AVG(cr.rating), 0)::int,
                                'rating_count', COUNT(cr.id),
                                'created_at', c.created_at,
                                'updated_at', c.updated_at,
                                'features', c.features,

                                -- 🔑 CLAVE
                                'is_assigned', (uc.user_id IS NOT NULL)

                            ) AS course_row
                        FROM courses c

                        -- cursos asignados (si existen)
                        LEFT JOIN user_courses uc
                            ON uc.course_id = c.id
                            AND uc.user_id = u.id

                        LEFT JOIN course_ratings cr
                            ON cr.course_id = c.id

                        WHERE
                            -- 🔥 lógica principal
                            (
                                -- si tiene suscripción → todos los cursos
                                EXISTS (
                                    SELECT 1
                                    FROM subscription s
                                    WHERE s.user_id = u.id
                                    AND (
                                        s.status = true
                                        OR (s.status = false AND s.end_time > NOW())
                                    )
                                )
                                -- si NO tiene suscripción → solo asignados
                                OR uc.user_id IS NOT NULL
                            )

                        GROUP BY c.id, uc.user_id
                    ) sub
                ), '[]'::json),



                'achievements', COALESCE((
                SELECT json_agg(
                    jsonb_build_object(
                        'id', a.id,
                        'name', a.name,
                        'description', a.description,
                        'icon', a.icon,
                        'trigger_type', a.trigger_type,
                        'trigger_value', a.trigger_value,
                        'active', a.active,
                        'earned', COALESCE(ua.earned, false),
                        'earned_at', ua.earned_at,
                        'created_at', a.created_at
                    )
                    ORDER BY a.created_at ASC
                )
                FROM achievement a
                LEFT JOIN user_achievement ua
                    ON ua.achievement_id = a.id
                    AND ua.user_id = u.id
                WHERE a.active = true
            ), '[]'::json),

                'subscriptions', COALESCE((
                    SELECT json_agg(s.*)
                    FROM subscription s
                    WHERE s.user_id = u.id
                ), '[]'::json),

                'certificates', COALESCE((
                    SELECT json_agg(cer.*)
                    FROM certificates cer
                    WHERE cer.user_id = u.id
                ), '[]'::json)

                ) AS profile
                FROM users u
                WHERE u.id = $1
            ) t
            "#,
            user_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(
            |e|
            {
                log::error!("Error: {}", e);
                e
            }
        )?;

        let profile: UserProfileData = match serde_json::from_value(row.profile.unwrap()) {
            Ok(p) => p,
            Err(e) => {
                log::error!("Error deserializando profile JSON: {}", e);
                return Err(Error::Decode(Box::new(e)));
            }
        };

        Ok(profile)
    }

    async fn get_users_by_notification_type(
        &self,
        notification_type: &str,
    ) -> Result<Vec<(String, String)>, Error> {
        // Validate notification_type to prevent SQL injection
        let column = match notification_type {
            "email_notifications" => "email_notifications",
            "course_reminders" => "course_reminders",
            "new_content" => "new_content",
            _ => {
                return Err(Error::Protocol(
                    "Invalid notification type. Must be one of: email_notifications, course_reminders, new_content".into(),
                ));
            }
        };

        // Build query dynamically based on valid column
        let query = format!(
            r#"
            SELECT email, name
            FROM users
            WHERE {} = true AND verified = true
            ORDER BY created_at DESC
            "#,
            column
        );

        let rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| {
                log::error!("Error getting users by notification type: {}", e);
                e
            })?;

        let users: Vec<(String, String)> = rows
            .iter()
            .map(|row| {
                let email: String = row.get("email");
                let name: String = row.get("name");
                (email, name)
            })
            .collect();

        Ok(users)
    }

    async fn update_notification_settings(
        &self,
        user_id: Uuid,
        email_notifications: Option<bool>,
        course_reminders: Option<bool>,
        new_content: Option<bool>,
    ) -> Result<(), Error> {
        sqlx::query(
            r#"
            UPDATE users
            SET 
                email_notifications = COALESCE($1, email_notifications),
                course_reminders = COALESCE($2, course_reminders),
                new_content = COALESCE($3, new_content),
                updated_at = NOW()
            WHERE id = $4
            "#
        )
        .bind(email_notifications)
        .bind(course_reminders)
        .bind(new_content)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            log::error!("Error updating notification settings: {}", e);
            e
        })?;

        Ok(())
    }

}

// ===================== //
//      COURSES EXT 
// ===================== //

#[async_trait]
pub trait CourseExt {
    async fn create_course(
        &self,
        dto: CreateCourseDTO,
    ) -> Result<CreateCourseDTO, Error>;

    async fn get_course(&self, course_id: Uuid) -> Result<Option<Course>, Error>;

    async fn get_courses_page(&self, user_id: Option<Uuid>, page: u32, limit: u32) -> Result<Vec<CoursePageRow>, Error>;

    async fn get_courses(
        &self,
        page: u32,
        limit: usize,
    ) -> Result<Vec<UserCourseDto>, Error>;

    async fn get_all_courses_with_modules(
        &self,
    ) -> Result<Vec<CourseWithModulesDto>, Error> ;

    async fn get_course_with_videos(
        &self,
        course_id: Uuid,
        user_id: Option<Uuid>,
    ) -> Result<Option<CourseWithModulesDto>, Error>;

    async fn get_course_with_videos_preview(
        &self,
        course_id: Uuid,
        user_id: Option<Uuid>,
    ) -> Result<Option<CourseWithModulesDto>, sqlx::Error>;

    async fn update_course(
        &self,
        course_id: Uuid,
        dto: UpdateCourseDTO,
    ) -> Result<CourseWithModulesDto, Error>;

    async fn delete_course(&self, course_id: Uuid) -> Result<(), Error>;

    async fn create_lesson_comment(
        &self,
        lesson_id: Uuid,
        user_id: Uuid,
        comment: String,
    ) -> Result<CommentLessonDto, Error>;

    async fn get_lesson_comments(
        &self, 
        lesson_id: Uuid
    ) -> Result<Vec<CommentLessonDto>, Error>;

    async fn delete_lesson_comment(
        &self, 
        comment_id: Uuid
    ) -> Result<(), Error>;

    async fn create_or_update_rating(
        &self,
        course_id: Uuid,
        user_id: Uuid,
        rating: i32,
    ) -> Result<(), Error>;

    async fn get_rating(
        &self, 
        course_id: Uuid,
        user_id: Option<Uuid>,
    ) -> Result<CourseRatingDto, Error>;
    
}

// ===================== //
//   IMPLEMENTATION COURSES EXT
// ===================== //
#[async_trait]
impl CourseExt for DBClient {
    async fn create_course(
        &self,
        dto: CreateCourseDTO,
    ) -> Result<CreateCourseDTO, Error> {
        let course_id = Uuid::new_v4();
        let now = Utc::now();

        // 1. INICIAR TRANSACCIÓN
        let mut tx = match self.pool.begin().await {
            Ok(t) => t,
            Err(e) => {
                return Err(e);
            }
        };

        // 2. INSERTAR CURSO
        // Nota: Manejo seguro de features
        let features_json = match &dto.features {
            Some(f) => serde_json::to_value(f).unwrap_or(serde_json::Value::Array(vec![])),
            None => serde_json::Value::Array(vec![]),
        };

        let course_insert_result = sqlx::query_as::<_, Course>(
            r#"
            INSERT INTO courses
                (id, title, description, long_description, level, price, duration, students, image, category, features, paypal_product_id, created_at, updated_at)
            VALUES
                ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            RETURNING *
            "#
        )
        .bind(course_id)
        .bind(&dto.title)
        .bind(&dto.description)
        .bind(&dto.long_description)
        .bind(&dto.level)
        .bind(dto.price) // Asegúrate que dto.price sea compatible con DECIMAL
        .bind(&dto.duration)
        .bind(dto.students.unwrap_or(0))
        .bind(&dto.image)
        .bind(&dto.category)
        .bind(features_json)
        .bind(&dto.paypal_product_id)
        .bind(now)
        .bind(now)
        .fetch_one(&mut *tx)
        .await;

        let course = match course_insert_result {
            Ok(c) => c,
            Err(e) => {
                let _ = tx.rollback().await; 
                return Err(e);
            }
        };

        // 3. INSERTAR MÓDULOS Y LECCIONES
        let mut modules_dtos: Vec<CreateModuleDTO> = Vec::new();

        for (module_idx, module_dto) in dto.modules.into_iter().enumerate() {
            // Forzamos el orden basado en el índice para evitar error de UNIQUE constraint
            let module_order = (module_idx + 1) as i32; 

            let module_insert = sqlx::query_as::<_, Module>(
                r#"
                INSERT INTO modules (course_id, title, "order")
                VALUES ($1, $2, $3)
                RETURNING *
                "#
            )
            .bind(course_id)
            .bind(&module_dto.title)
            .bind(module_order)
            .fetch_one(&mut *tx)
            .await;

            let module_model = match module_insert {
                Ok(m) => m,
                Err(e) => {
                    let _ = tx.rollback().await;
                    return Err(e);
                }
            };

            let mut lessons_dtos: Vec<CreateLessonDTO> = Vec::new();

            for (lesson_idx, lesson) in module_dto.lessons.into_iter().enumerate() {
                // Forzamos el orden también aquí
                let lesson_order = (lesson_idx + 1) as i32;

                let lesson_insert = sqlx::query_as::<_, Lesson>(
                    r#"
                    INSERT INTO lessons (module_id, title, duration, "type", content_url, description, "order")
                    VALUES ($1, $2, $3, $4, $5, $6, $7)
                    RETURNING *
                    "#
                )
                .bind(module_model.id)
                .bind(&lesson.title)
                .bind(&lesson.duration)
                .bind(&lesson.r#type)
                .bind(&lesson.content_url)
                .bind(&lesson.description)
                .bind(lesson_order)
                .fetch_one(&mut *tx)
                .await;

                let lesson_model = match lesson_insert {
                    Ok(l) => l,
                    Err(e) => {
                        let _ = tx.rollback().await;
                        return Err(e);
                    }
                };

                lessons_dtos.push(CreateLessonDTO {
                    title: lesson_model.title,
                    duration: lesson_model.duration,
                    completed: false,
                    r#type: lesson_model.r#type,
                    content_url: lesson_model.content_url,
                    description: lesson_model.description,
                    order: Some(lesson_order),
                });
            }

            modules_dtos.push(CreateModuleDTO {
                title: module_model.title,
                order: Some(module_order),
                lessons: lessons_dtos,
            });
        }

        // 4. CONFIRMAR TRANSACCIÓN
        if let Err(e) = tx.commit().await {
            return Err(e);
        }

        Ok(CreateCourseDTO {
            title: course.title,
            description: course.description,
            long_description: course.long_description,
            level: course.level,
            price: course.price,
            duration: course.duration,
            students: Some(course.students),
            image: course.image,
            category: course.category,
            features: course.features.and_then(|f| serde_json::from_value(f).ok()),
            paypal_product_id: None,
            modules: modules_dtos,
        })
    }

    async fn get_course(&self, course_id: Uuid) -> Result<Option<Course>, Error> {
        let mut tx = self.pool.begin().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        let course = sqlx::query_as::<_, Course>(
            r#"SELECT * FROM courses WHERE id = $1"#,
        )
        .bind(course_id)
        .fetch_optional(&mut *tx)
        .await.map_err(|e| {
            log::error!("ERROR: {}", e);
            e
        })?
        ;
        tx.commit().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        Ok(course)
    }

    async fn get_courses_page(
        &self,
        user_id: Option<Uuid>,
        page: u32,
        limit: u32,
    ) -> Result<Vec<CoursePageRow>, Error> {
        let offset = ((page - 1) * limit) as i64;

        let rows = sqlx::query_as::<_, CoursePageRow>(
            r#"
            WITH user_sub AS (
                SELECT
                    CASE
                        WHEN EXISTS (
                            SELECT 1
                            FROM subscription s
                            WHERE s.user_id = $1
                            AND (
                                s.status = true
                                OR (s.status = false AND s.end_time > NOW())
                            )
                        )
                        THEN true
                        ELSE false
                    END AS has_active_subscription
            )
            SELECT
                c.id,
                c.title,
                c.description,
                c.long_description,
                c.level,
                c.duration,
                c.students,
                c.paypal_product_id,
                c.price,
                c.image,
                c.category,
                ARRAY(
                    SELECT jsonb_array_elements_text(c.features)
                ) AS features,
                COALESCE(AVG(cr.rating), 0)::float8 AS rating_average,
                COUNT(cr.id) AS rating_count,
                CASE
                    WHEN uc.course_id IS NOT NULL THEN true
                    ELSE false
                END AS purchased,
                us.has_active_subscription
            FROM courses c
            LEFT JOIN course_ratings cr
                ON cr.course_id = c.id
            LEFT JOIN user_courses uc
                ON uc.course_id = c.id
            AND uc.user_id = $1
            CROSS JOIN user_sub us
            GROUP BY
                c.id,
                uc.course_id,
                us.has_active_subscription
            ORDER BY c.created_at DESC
            LIMIT $2 OFFSET $3
            "#
        )
        .bind(user_id)
        .bind(limit as i64)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            log::error!("ERROR: {}", e);
            e
        })?;

        Ok(rows)
    }



    async fn get_courses(
        &self,
        page: u32,
        limit: usize,
    ) -> Result<Vec<UserCourseDto>, Error> {
        let mut tx = self.pool.begin().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        let offset = ((page - 1) * limit as u32) as i64;
        let courses = sqlx::query_as::<_, UserCourseDto>(
            r#"
            SELECT
                c.id,
                c.title,
                c.description,
                c.long_description,
                c.level,
                c.duration,
                c.students,
                c.paypal_product_id,
                c.price,
                c.image,
                c.category,
                COALESCE(AVG(cr.rating), 0)::int AS rating,
                COUNT(cr.id) AS rating_count,
                c.created_at,
                c.updated_at,
                c.features
                -- user course
                
            FROM courses c
            LEFT JOIN course_ratings cr
                ON cr.course_id = c.id
            GROUP BY c.id
            ORDER BY c.created_at DESC
            LIMIT $1 OFFSET $2
            "#
        )
        .bind(limit as i64)
        .bind(offset)
        .fetch_all(&mut *tx)
        .await.map_err(|e| {
            log::error!("ERROR: {}", e);
            e
        })?;
        tx.commit().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        Ok(courses)
    }

    /// Mucho más eficiente: 3 queries en vez de un JOIN enorme.
    async fn get_all_courses_with_modules(
        &self,
    ) -> Result<Vec<CourseWithModulesDto>, Error> {
        let mut tx = self.pool.begin().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        // 1️⃣ Traer cursos
        let rows = sqlx::query!(
            r#"
            SELECT 
                c.id AS course_id,
                c.title AS course_title,
                c.description,
                c.long_description,
                c.level,
                c.price,
                c.duration,
                c.students,
                c.image,
                c.category,
                c.features,
                c.paypal_product_id,
                c.created_at,
                c.updated_at,

                m.id AS "module_id?: Uuid",
                m.title AS "module_title?",
                m."order" AS "module_order?",

                l.id AS "lesson_id?: Uuid",
                l.title AS "lesson_title?",
                l.duration AS "lesson_duration?",
                l."type" AS "lesson_type?",
                l.content_url AS "content_url?",
                l.description AS "lesson_description?",
                l."order" AS "lesson_order?"

            FROM courses c
            LEFT JOIN modules m ON m.course_id = c.id
            LEFT JOIN lessons l ON l.module_id = m.id
            ORDER BY c.created_at DESC, m."order" ASC, l."order" ASC
            "#
        )
        .fetch_all(&mut *tx)
        .await.map_err(|e| {
            log::error!("ERROR: {}", e);
            e
        })?;
        // 2️⃣ Procesar filas en estructuras separadas
        // -------------- AGRUPACIÓN EFICIENTE ---------------
        
        let mut courses_map: HashMap<Uuid, CourseWithModulesDto> = HashMap::new();
        for row in rows {

            // 1️⃣ Asegurar que el curso exista en el mapa
            let course = courses_map
                .entry(row.course_id)
                .or_insert_with(|| CourseWithModulesDto {
                    id: row.course_id,
                    title: row.course_title.clone(),
                    description: row.description.clone(),
                    long_description: row.long_description.clone(),
                    price: row.price,
                    level: row.level.unwrap(),
                    duration: row.duration,
                    students: row.students.unwrap_or(0),
                    image: row.image.clone(),
                    category: row.category.unwrap(),
                    features: row.features
                        .as_ref()
                        .and_then(|v| serde_json::from_value(v.clone()).ok()),
                    created_at: row.created_at.unwrap(),
                    updated_at: row.updated_at.unwrap(),
                    total_lessons: 0,
                    completed_lessons: 0,
                    modules: vec![],
                });

            // 2️⃣ Si hay un módulo
            if let Some(module_id) = row.module_id {
                let module = course.modules
                    .iter_mut()
                    .find(|m| m.id == module_id);

                let module_ref = match module {
                    Some(m) => m,
                    None => {
                        course.modules.push(ModuleWithLessonsDto {
                            id: module_id,
                            title: row.module_title.unwrap_or("Title".to_string()),
                            order: row.module_order.unwrap_or(1),
                            lessons: vec![],
                        });
                        course.modules.last_mut().unwrap()
                    }
                };

                // 3️⃣ Si hay una lección
                if let Some(lesson_id) = row.lesson_id {
                    module_ref.lessons.push(LessonDto {
                        id: lesson_id,
                        title: row.lesson_title.clone().unwrap_or("Title".to_string()),
                        duration: row.lesson_duration,
                        completed: None,
                        r#type: row.lesson_type.clone().unwrap(),
                        content_url: row.content_url.clone(),
                        description: row.lesson_description.clone(),
                        order: row.lesson_order.unwrap(),
                    });
                }
            }
        }
        tx.commit().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        Ok(courses_map.into_values().collect())
    }

    async fn get_course_with_videos(
        &self,
        course_id: Uuid,
        user_id: Option<Uuid>,
    ) -> Result<Option<CourseWithModulesDto>, Error> {
        let mut tx = self.pool.begin().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;

        let rows = sqlx::query!(
            r#"
            SELECT 
                c.id AS course_id,
                c.title AS course_title,
                c.description,
                c.long_description,
                c.level,
                c.price,
                c.duration,
                c.students,
                c.image,
                c.category,
                c.features,
                c.created_at,
                c.updated_at,

                m.id AS "module_id?: Uuid",
                m.title AS "module_title?",
                m."order" AS "module_order?",

                l.id AS "lesson_id?: Uuid",
                l.title AS "lesson_title?",
                l.duration AS "lesson_duration?",
                l."type" AS "lesson_type?",
                l.content_url AS "content_url?",
                l.description AS "lesson_description?",
                l."order" AS "lesson_order?",

                ulp.is_completed AS "lesson_completed?"

            FROM courses c
            LEFT JOIN modules m ON m.course_id = c.id
            LEFT JOIN lessons l ON l.module_id = m.id
            LEFT JOIN user_lesson_progress ulp
                ON ulp.lesson_id = l.id
            AND ulp.user_id = $2
            WHERE c.id = $1
            ORDER BY m."order" ASC, l."order" ASC
            "#,
            course_id,
            user_id
        )
        .fetch_all(&mut *tx)
        .await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;

        if rows.is_empty() {
            return Ok(None);
        }
        let mut total_lessons = 0;
        let mut completed_lessons = 0;
        let mut course_opt: Option<CourseWithModulesDto> = None;

        for row in rows {
            // 1️⃣ Crear curso si no existe
            let course = course_opt.get_or_insert_with(|| CourseWithModulesDto {
                id: row.course_id,
                title: row.course_title.clone(),
                description: row.description.clone(),
                long_description: row.long_description.clone(),
                price: row.price,
                level: row.level.clone().unwrap_or_default(),
                duration: row.duration,
                students: row.students.unwrap_or(0),
                image: row.image.clone(),
                category: row.category.clone().unwrap_or_default(),
                features: row.features
                    .as_ref()
                    .and_then(|v| serde_json::from_value(v.clone()).ok()),
                created_at: row.created_at.unwrap(),
                updated_at: row.updated_at.unwrap(),
                total_lessons: 0,
                completed_lessons: 0,
                modules: vec![],
            });

            // 2️⃣ Módulo
            if let Some(module_id) = row.module_id {
                let module = course.modules.iter_mut().find(|m| m.id == module_id);

                let module_ref = match module {
                    Some(m) => m,
                    None => {
                        course.modules.push(ModuleWithLessonsDto {
                            id: module_id,
                            title: row.module_title.clone().unwrap_or_else(|| "Título".into()),
                            order: row.module_order.unwrap_or(1),
                            lessons: vec![],
                        });
                        course.modules.last_mut().unwrap()
                    }
                };

                // 3️⃣ Lección
                if let Some(lesson_id) = row.lesson_id {
                    total_lessons +=1;
                    if row.lesson_completed.unwrap_or(false) {
                        completed_lessons += 1;
                    }
                    module_ref.lessons.push(LessonDto {
                        id: lesson_id,
                        title: row.lesson_title.clone().unwrap_or_else(|| "Lección".into()),
                        duration: row.lesson_duration,
                        completed: row.lesson_completed,
                        r#type: row.lesson_type.clone().unwrap_or_else(|| "video".into()),
                        content_url: row.content_url.clone(),
                        description: row.lesson_description.clone(),
                        order: row.lesson_order.unwrap_or(1),
                    });
                }
            }
        }
        if let Some(course) = &mut course_opt {
            course.total_lessons = total_lessons;
            course.completed_lessons = completed_lessons;
        }

        tx.commit().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        Ok(course_opt)
    }

    async fn get_course_with_videos_preview(
        &self,
        course_id: Uuid,
        user_id: Option<Uuid>,
    ) -> Result<Option<CourseWithModulesDto>, sqlx::Error> {
        let mut tx = self.pool.begin().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;

        // Usamos una CTE para calcular lesson_index y luego en la selección final
        // exponemos content_url y description solo cuando lesson_index = 1.
        let rows = sqlx::query!(
            r#"
            WITH course_data AS (
                SELECT
                    c.id AS course_id,
                    c.title AS course_title,
                    c.description,
                    c.long_description,
                    c.level,
                    c.price,
                    c.duration,
                    c.students,
                    c.image,
                    c.category,
                    c.features,
                    c.created_at,
                    c.updated_at,

                    m.id AS module_id,
                    m.title AS module_title,
                    m."order" AS module_order,

                    l.id AS lesson_id,
                    l.title AS lesson_title,
                    l.duration AS lesson_duration,
                    l."type" AS lesson_type,
                    l.content_url AS lesson_content_url,
                    l.description AS lesson_description,
                    l."order" AS lesson_order,

                    ulp.is_completed AS lesson_completed,

                    ROW_NUMBER() OVER (ORDER BY m."order" ASC NULLS LAST, l."order" ASC NULLS LAST) AS lesson_index
                FROM courses c
                LEFT JOIN modules m ON m.course_id = c.id
                LEFT JOIN lessons l ON l.module_id = m.id
                LEFT JOIN user_lesson_progress ulp
                    ON ulp.lesson_id = l.id
                    AND ulp.user_id = $2
                WHERE c.id = $1
            )
            SELECT
                course_id,
                course_title,
                description,
                long_description,
                level,
                price,
                duration,
                students,
                image,
                category,
                features,
                created_at,
                updated_at,

                module_id AS "module_id?: Uuid",
                module_title AS "module_title?",
                module_order AS "module_order?",

                lesson_id AS "lesson_id?: Uuid",
                lesson_title AS "lesson_title?",
                lesson_duration AS "lesson_duration?",
                lesson_type AS "lesson_type?",
                -- Exponer content_url solo para la primera lección del curso
                CASE WHEN lesson_index = 1 THEN lesson_content_url ELSE NULL END AS "content_url?: String",
                -- Exponer description solo para la primera lección del curso
                CASE WHEN lesson_index = 1 THEN lesson_description ELSE NULL END AS "lesson_description?: String",
                lesson_order AS "lesson_order?",

                lesson_completed AS "lesson_completed?",
                lesson_index AS "lesson_index?"
            FROM course_data
            ORDER BY module_order ASC NULLS LAST, lesson_order ASC NULLS LAST
            "#,
            course_id,
            user_id
        )
        .fetch_all(&mut *tx)
        .await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;

        if rows.is_empty() {
            tx.commit().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
            return Ok(None);
        }

        let mut total_lessons: i64 = 0;
        let mut completed_lessons: i64 = 0;
        let mut course_opt: Option<CourseWithModulesDto> = None;

        for row in rows {
            // Crear curso si no existe aún
            let course = course_opt.get_or_insert_with(|| CourseWithModulesDto {
                id: row.course_id,
                title: row.course_title.clone(),
                description: row.description.clone(),
                long_description: row.long_description.clone(),
                price: row.price,
                level: row.level.clone().unwrap_or_default(),
                duration: row.duration.clone(),
                students: row.students.unwrap_or(0),
                image: row.image.clone(),
                category: row.category.clone().unwrap_or_default(),
                features: row.features
                    .as_ref()
                    .and_then(|v| serde_json::from_value(v.clone()).ok()),
                created_at: row.created_at.unwrap_or_else(|| chrono::Utc::now()),
                updated_at: row.updated_at.unwrap_or_else(|| chrono::Utc::now()),
                total_lessons: 0,
                completed_lessons: 0,
                modules: vec![],
            });

            // Si hay módulo en esta fila
            if let Some(module_id) = row.module_id {
                // Buscar módulo existente
                let module = course.modules.iter_mut().find(|m| m.id == module_id);

                let module_ref = match module {
                    Some(m) => m,
                    None => {
                        course.modules.push(ModuleWithLessonsDto {
                            id: module_id,
                            title: row.module_title.clone().unwrap_or_else(|| "Título".into()),
                            order: row.module_order.unwrap_or(1),
                            lessons: vec![],
                        });
                        course.modules.last_mut().unwrap()
                    }
                };

                // Si hay lección en esta fila
                if let Some(lesson_id) = row.lesson_id {
                    total_lessons += 1;
                    if row.lesson_completed.unwrap_or(false) {
                        completed_lessons += 1;
                    }

                    // Nota: content_url y lesson_description ya vienen nulos para todas
                    // las lecciones excepto la primera (por la CASE en SQL).
                    module_ref.lessons.push(LessonDto {
                        id: lesson_id,
                        title: row.lesson_title.clone().unwrap_or_else(|| "Lección".into()),
                        duration: row.lesson_duration.clone(),
                        completed: row.lesson_completed,
                        r#type: row.lesson_type.clone().unwrap_or_else(|| "video".into()),
                        content_url: row.content_url.clone(),         // solo Some para la primera lección
                        description: row.lesson_description.clone(),  // solo Some para la primera lección
                        order: row.lesson_order.unwrap_or(1),
                    });
                }
            }
        }

        if let Some(course) = &mut course_opt {
            course.total_lessons = total_lessons;
            course.completed_lessons = completed_lessons;
        }

        tx.commit().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        Ok(course_opt)
    }


    async fn update_course(
        &self,
        course_id: Uuid,
        mut dto: UpdateCourseDTO,
    ) -> Result<CourseWithModulesDto, Error> {
        let mut tx = self.pool.begin().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        let now = Utc::now();

        // Asegurar que cada módulo tenga un UUID
        if let Some(mods) = dto.modules.as_mut() {
            for m in mods.iter_mut() {
                if m.id.is_none() {
                    m.id = Some(Uuid::new_v4());
                }
                if let Some(lessons) = m.lessons.as_mut() {
                    for l in lessons.iter_mut() {
                        if l.id.is_none() {
                            l.id = Some(Uuid::new_v4());
                        }
                        // Propagar el module_id correcto
                        l.module_id = m.id;
                    }
                }
            }
        }

        // Serializar módulos y lecciones a JSON
        let modules_json = serde_json::to_value(&dto.modules).unwrap_or(serde_json::json!([]));
        let lessons_json = {
            let lessons_vec: Vec<_> = dto.modules
                .as_ref()
                .map(|mods| {
                    mods.iter().flat_map(|m| {
                        m.lessons.as_ref()
                            .map(|lessons| lessons.iter().map(|l| {
                                serde_json::json!({
                                    "id": l.id,
                                    "module_id": l.module_id,
                                    "title": l.title.clone(),
                                    "duration": l.duration.clone(),
                                    "type": l.r#type.clone(),
                                    "content_url": l.content_url.clone(),
                                    "description": l.description.clone(),
                                    "order": l.order
                                })
                            }).collect::<Vec<_>>())
                            .unwrap_or_default()
                    }).collect::<Vec<_>>()
                })
                .unwrap_or_default();
            serde_json::to_value(lessons_vec).unwrap_or(serde_json::json!([]))
        };

        let sql = r#"
            WITH
            course_update AS (
                UPDATE courses SET
                    title = COALESCE($2, title),
                    description = COALESCE($3, description),
                    long_description = COALESCE($4, long_description),
                    level = COALESCE($5, level),
                    price = COALESCE($6, price),
                    duration = COALESCE($7, duration),
                    students = COALESCE($8, students),
                    image = COALESCE($9, image),
                    category = COALESCE($10, category),
                    features = COALESCE($11::jsonb, features),
                    updated_at = $12
                WHERE id = $1
                RETURNING *
            ),

            module_input AS (
                SELECT
                    (m->>'id')::uuid AS id,
                    m->>'title' AS title,
                    (m->>'order')::int AS module_order,
                    $1 AS course_id
                FROM jsonb_array_elements($13::jsonb) AS m
            ),
            module_upsert AS (
                INSERT INTO modules (id, course_id, title, "order")
                SELECT
                    id,
                    course_id,
                    title,
                    module_order
                FROM module_input
                ON CONFLICT (id) DO UPDATE SET
                    title = EXCLUDED.title,
                    "order" = EXCLUDED."order"
                RETURNING id, title
            ),

            module_ids AS (
                SELECT id, title FROM module_upsert
            ),

            module_deleted AS (
                DELETE FROM modules
                WHERE course_id = $1
                AND id NOT IN (SELECT id FROM module_input)
                RETURNING id
            ),

            lesson_input AS (
                SELECT
                    (l->>'id')::uuid AS id,
                    (l->>'module_id')::uuid AS module_id,
                    l->>'title' AS title,
                    l->>'duration' AS duration,
                    l->>'type' AS type,
                    l->>'content_url' AS content_url,
                    l->>'description' AS description,
                    (l->>'order')::int AS lesson_order
                FROM jsonb_array_elements($14::jsonb) AS l
            ),
            lesson_upsert AS (
                INSERT INTO lessons (id, module_id, title, duration, "type", content_url, description, "order")
                SELECT
                    lesson_input.id,
                    lesson_input.module_id,
                    lesson_input.title,
                    lesson_input.duration,
                    lesson_input.type,
                    lesson_input.content_url,
                    lesson_input.description,
                    lesson_input.lesson_order
                FROM lesson_input
                JOIN module_ids ON lesson_input.module_id = module_ids.id
                ON CONFLICT (id) DO UPDATE SET
                    module_id = EXCLUDED.module_id,
                    title = EXCLUDED.title,
                    duration = EXCLUDED.duration,
                    "type" = EXCLUDED."type",
                    content_url = EXCLUDED.content_url,
                    description = EXCLUDED.description,
                    "order" = EXCLUDED."order"
                RETURNING lessons.id
            ),


            lesson_deleted AS (
                DELETE FROM lessons
                WHERE module_id IN (SELECT id FROM module_upsert)
                AND id NOT IN (SELECT id FROM lesson_input)
                RETURNING id
            )

            SELECT * FROM course_update;
        "#;

        let _ = sqlx::query(sql)
            .bind(course_id)
            .bind(dto.title)
            .bind(dto.description)
            .bind(dto.long_description)
            .bind(dto.level)
            .bind(dto.price)
            .bind(dto.duration)
            .bind(dto.students)
            .bind(dto.image)
            .bind(dto.category)
            .bind(dto.features.map(|f| serde_json::to_value(f).unwrap()))
            .bind(now)
            .bind(modules_json)
            .bind(lessons_json)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                log::error!("ERROR: {}", e);
                e
            });

        tx.commit().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        Ok(
            self.get_all_courses_with_modules()
                .await
                .map_err(|e| { log::error!("ERROR: {}", e); e })?
                .into_iter()
                .find(|c| c.id == course_id)
                .expect("Curso debería existir después de la actualización")
        )
    }



    async fn delete_course(&self, course_id: Uuid) -> Result<(), Error> {
        let mut tx = self.pool.begin().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        sqlx::query("DELETE FROM courses WHERE id = $1")
            .bind(course_id)
            .execute(&mut *tx)
            .await.map_err(|e| {
            log::error!("ERROR: {}", e);
            e
        })?
        ;
        tx.commit().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        Ok(())
    }

    async fn create_lesson_comment(
        &self,
        lesson_id: Uuid,
        user_id: Uuid,
        comment: String,
    ) -> Result<CommentLessonDto, Error> {
        let mut tx = self.pool.begin().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        let now = Utc::now();
        let result = sqlx::query_as::<_, CommentLessonDto>(
                r#"
                WITH inserted AS (
                    INSERT INTO lesson_comments (lesson_id, user_id, content, created_at)
                    VALUES ($1, $2, $3, NOW())
                    RETURNING id,lesson_id, user_id, content, created_at
                )
                SELECT
                    inserted.id,
                    inserted.lesson_id,
                    inserted.user_id,
                    u.name AS user_name,
                    inserted.content,
                    inserted.created_at
                FROM inserted
                JOIN users u ON u.id = inserted.user_id
                "#,
            )
            .bind(lesson_id)
            .bind(user_id)
            .bind(comment)
            .bind(now)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| {
                log::error!("ERROR: {}", e);
                e
            }
        )?;
        tx.commit().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        Ok(result)
    }

    async fn get_lesson_comments(&self, lesson_id: Uuid) -> Result<Vec<CommentLessonDto>, Error> {
        let mut tx = self.pool.begin().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        let result = sqlx::query_as::<_, CommentLessonDto>(
            r#"
            SELECT
                lc.id,
                lc.lesson_id,
                lc.user_id,
                u.name AS user_name,
                lc.content,
                lc.created_at
            FROM lesson_comments lc
            JOIN users u ON u.id = lc.user_id
            WHERE lc.lesson_id = $1
            ORDER BY lc.created_at DESC
            "#,
        )
            .bind(lesson_id)
            .fetch_all(&mut *tx)
            .await
            .map_err(|e| {
                log::error!("ERROR: {}", e);
                e
            })?;
        tx.commit().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        Ok(result)
    }

    async fn delete_lesson_comment(&self, comment_id: Uuid) -> Result<(), Error> {
        let mut tx = self.pool.begin().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        sqlx::query("DELETE FROM lesson_comments WHERE id = $1")
            .bind(comment_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                log::error!("ERROR: {}", e);
                e
            })?;
        tx.commit().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        Ok(())
    }

    async fn create_or_update_rating(
        &self,
        course_id: Uuid,
        user_id: Uuid,
        rating: i32,
    ) -> Result<(), Error> {

        sqlx::query!(
            r#"
            INSERT INTO course_ratings (course_id, user_id, rating)
            VALUES ($1, $2, $3)
            ON CONFLICT (course_id, user_id)
            DO UPDATE SET
                rating = EXCLUDED.rating,
                updated_at = NOW()
            "#,
            course_id,
            user_id,
            rating
        )
        .execute(&self.pool)
        .await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;

        Ok(())
    }


    async fn get_rating(
        &self,
        course_id: Uuid,
        user_id: Option<Uuid>,
    ) -> Result<CourseRatingDto, Error> {

        // 1. Rating global
        let summary = sqlx::query!(
            r#"
            SELECT
                COALESCE(AVG(rating), 0)::float AS average,
                COUNT(*)::bigint AS count
            FROM course_ratings
            WHERE course_id = $1
            "#,
            course_id
        )
        .fetch_one(&self.pool)
        .await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;

        // 2. Rating del usuario (opcional)
        let user_rating = if let Some(user_id) = user_id {
            sqlx::query_scalar!(
                r#"
                SELECT rating
                FROM course_ratings
                WHERE course_id = $1 AND user_id = $2
                "#,
                course_id,
                user_id
            )
            .fetch_optional(&self.pool)
            .await?
        } else {
            None
        };

        Ok(CourseRatingDto {
            average: summary.average.unwrap_or(0.0),
            count: summary.count.unwrap_or(0),
            user_rating,
        })
    }

}
// ===================== //


#[async_trait]
pub trait AchievementExt {
    /// Crea un nuevo logro.
    async fn create_achievement<T: Into<String> + Send>(
        &self,
        name: T,
        description: Option<T>,
        icon: Option<T>,
        trigger_type: &str,
        trigger_value: i32,
        active: bool,
    ) -> Result<Achievement, Error>;

    /// Actualiza un logro.
    async fn update_achievement<T: Into<String> + Send>(
        &self,
        achievement_id: Uuid,
        name: Option<T>,
        description: Option<T>,
        icon: Option<T>,
        trigger_type: Option<&str>,
        trigger_value: Option<i32>,
        active: Option<bool>,
    ) -> Result<Achievement, Error>;

    /// Obtiene todos los logros existentes (paginados).
    async fn get_achievements(
        &self,
        page: u32,
        limit: usize,
    ) -> Result<Vec<Achievement>, Error>;

    /// Obtiene un logro por su ID.
    async fn get_achievement(&self, achievement_id: Uuid)
        -> Result<Option<Achievement>, Error>;

    /// Elimina un logro existente.
    async fn delete_achievement(&self, achievement_id: Uuid) -> Result<(), Error>;
}

/// Extensión para gestionar los logros obtenidos por usuarios.
#[async_trait]
pub trait UserAchievementExt {
    /// Asigna un logro a un usuario (sin marcarlo como ganado aún).
    
    async fn assign_achievement_to_user(
        &self,
        user_id: Uuid,
        achievement_id: Uuid,
    ) -> Result<UserAchievement, Error>;

    /// Marca un logro como ganado.
    
    async fn earn_achievement(
        &self,
        user_id: Uuid,
        achievement_id: Uuid,
    ) -> Result<UserAchievement, Error>;

    /// Obtiene logros de usuario con detalles completos.
    async fn get_user_achievements_with_details(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<serde_json::Value>, Error>;

    /// Verifica y otorga logros automáticamente basados en acciones.
    async fn check_and_award_achievements(
        &self,
        user_id: Uuid,
        action: &str,
        value: Option<i32>,
    ) -> Result<Vec<Achievement>, Error>;
}


/// Implementación para la conexión principal del sistema (`DBClient`).
#[async_trait]
impl AchievementExt for DBClient {
    async fn create_achievement<T: Into<String> + Send>(
        &self,
        name: T,
        description: Option<T>,
        icon: Option<T>,
        trigger_type: &str,
        trigger_value: i32,
        active: bool,
    ) -> Result<Achievement, Error> {
        let mut tx = self.pool.begin().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        let id = Uuid::new_v4();
        let now = Utc::now();

        let achievement = sqlx::query_as::<_, Achievement>(
            r#"
            INSERT INTO achievement (id, name, description, icon, trigger_type, trigger_value, active, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, name, description, icon, trigger_type, trigger_value, active, created_at
            "#
        )
        .bind(id)
        .bind(name.into())
        .bind(description.map(|d| d.into()))
        .bind(icon.map(|i| i.into()))
        .bind(trigger_type)
        .bind(trigger_value)
        .bind(active)
        .bind(now)
        .fetch_one(&mut *tx)
        .await.map_err(|e| {
            log::error!("ERROR: {}", e);
            e
        })?
        ;
        tx.commit().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        Ok(achievement)
    }

    async fn update_achievement<T: Into<String> + Send>(
        &self,
        achievement_id: Uuid,
        name: Option<T>,
        description: Option<T>,
        icon: Option<T>,
        trigger_type: Option<&str>,
        trigger_value: Option<i32>,
        active: Option<bool>,
    ) -> Result<Achievement, Error> {
        let mut tx = self.pool.begin().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;

        let achievement = sqlx::query_as::<_, Achievement>(
            r#"
            UPDATE achievement
            SET name = COALESCE($2, name),
                description = COALESCE($3, description),
                icon = COALESCE($4, icon),
                trigger_type = COALESCE($5, trigger_type),
                trigger_value = COALESCE($6, trigger_value),
                active = COALESCE($7, active)
            WHERE id = $1
            RETURNING id, name, description, icon, trigger_type, trigger_value, active, created_at
            "#,
        )
        .bind(achievement_id)
        .bind(name.map(|n| n.into()))
        .bind(description.map(|d| d.into()))
        .bind(icon.map(|i| i.into()))
        .bind(trigger_type)
        .bind(trigger_value)
        .bind(active)
        .fetch_one(&mut *tx)
        .await.map_err(|e| {
            log::error!("ERROR: {}", e);
            e
        })?;
        tx.commit().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        Ok(achievement)
    }

    async fn get_achievements(
        &self,
        page: u32,
        limit: usize,
    ) -> Result<Vec<Achievement>, Error> {
        let mut tx = self.pool.begin().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        let offset = ((page - 1) * limit as u32) as i64;

        let achievements = sqlx::query_as::<_, Achievement>(
            r#"
            SELECT id, name, description, icon, trigger_type, trigger_value, active, created_at
            FROM achievement
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#
        )
        .bind(limit as i64)
        .bind(offset)
        .fetch_all(&mut *tx)
        .await.map_err(|e| {
            log::error!("ERROR: {}", e);
            e
        })?
        ;
        tx.commit().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        Ok(achievements)
    }

    async fn get_achievement(&self, achievement_id: Uuid)
        -> Result<Option<Achievement>, Error> {
        let mut tx = self.pool.begin().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        let achievement = sqlx::query_as::<_, Achievement>(
            r#"
            SELECT id, name, description, icon, trigger_type, trigger_value, active, created_at
            FROM achievement
            WHERE id = $1
            "#
        )
        .bind(achievement_id)
        .fetch_optional(&mut *tx)
        .await.map_err(|e| {
            log::error!("ERROR: {}", e);
            e
        })?
        ;
        tx.commit().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        Ok(achievement)
    }

    async fn delete_achievement(&self, achievement_id: Uuid) -> Result<(), Error> {
        let mut tx = self.pool.begin().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        sqlx::query("DELETE FROM achievement WHERE id = $1")
            .bind(achievement_id)
            .execute(&mut *tx)
            .await.map_err(|e| {
            log::error!("ERROR: {}", e);
            e
        })?
        ;
        tx.commit().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        Ok(())
    }
}

#[async_trait]
impl UserAchievementExt for DBClient {
    async fn assign_achievement_to_user(
        &self,
        user_id: Uuid,
        achievement_id: Uuid,
    ) -> Result<UserAchievement, Error> {
        let mut tx = self.pool.begin().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        let id = Uuid::new_v4();

        let user_achievement = sqlx::query_as::<_, UserAchievement>(
            r#"
            INSERT INTO user_achievement (id, user_id, achievement_id, earned, earned_at)
            VALUES ($1, $2, $3, false, NULL)
            RETURNING id, user_id, achievement_id, earned, earned_at
            "#
        )
        .bind(id)
        .bind(user_id)
        .bind(achievement_id)
        .fetch_one(&mut *tx)
        .await.map_err(|e| {
            log::error!("ERROR: {}", e);
            e
        })?
        ;
        tx.commit().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        Ok(user_achievement)
    }

    async fn earn_achievement(
        &self,
        user_id: Uuid,
        achievement_id: Uuid,
    ) -> Result<UserAchievement, Error> {
        let mut tx = self.pool.begin().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;

        let user_achievement = sqlx::query_as::<_, UserAchievement>(
            r#"
            INSERT INTO user_achievement (id, user_id, achievement_id, earned, earned_at)
            VALUES ($1, $2, $3, true, $4)
            ON CONFLICT (user_id, achievement_id)
            DO UPDATE SET earned = true, earned_at = EXCLUDED.earned_at
            RETURNING id, user_id, achievement_id, earned, earned_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(achievement_id)
        .bind(Utc::now())
        .fetch_one(&mut *tx)
        .await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;

        tx.commit().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        Ok(user_achievement)
    }

    async fn get_user_achievements_with_details(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<serde_json::Value>, Error> {
        let mut tx = self.pool.begin().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        let rows = sqlx::query(
            r#"
            SELECT
                a.id as "a_id",
                a.name,
                a.description,
                a.icon,
                a.trigger_type,
                a.trigger_value,
                a.active,
                a.created_at as "a_created_at",
                ua.earned,
                ua.earned_at
            FROM achievement a
            LEFT JOIN user_achievement ua ON ua.achievement_id = a.id AND ua.user_id = $1
            ORDER BY a.created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&mut *tx)
        .await.map_err(|e| {
            log::error!("ERROR: {}", e);
            e
        })?;

        let result: Vec<serde_json::Value> = rows
            .into_iter()
            .map(|row| {
                serde_json::json!({
                    "id": row.get::<Uuid, _>("a_id"),
                    "name": row.get::<String, _>("name"),
                    "description": row.get::<Option<String>, _>("description"),
                    "icon": row.get::<Option<String>, _>("icon"),
                    "trigger_type": row.get::<String, _>("trigger_type"),
                    "trigger_value": row.get::<i32, _>("trigger_value"),
                    "active": row.get::<bool, _>("active"),
                    "created_at": row.get::<chrono::DateTime<chrono::Utc>, _>("a_created_at"),
                    "earned": row.get::<Option<bool>, _>("earned").unwrap_or(false),
                    "earned_at": row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("earned_at")
                })
            })
            .collect();

        tx.commit().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        Ok(result)
    }

    async fn check_and_award_achievements(
        &self,
        user_id: Uuid,
        action: &str,
        value: Option<i32>,
    ) -> Result<Vec<Achievement>, Error> {
        let mut tx = self.pool.begin().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        let fallback_value = value.unwrap_or(1);

        // 1️⃣ Calcular el valor actual del usuario
        let current_value: i32 = match action {
            "course_completed" => {
                sqlx::query_scalar::<_, i64>(
                    r#"
                    SELECT COUNT(DISTINCT uc.course_id)
                    FROM user_courses uc
                    WHERE uc.user_id = $1
                    AND EXISTS (
                        SELECT 1
                        FROM user_lesson_progress lp
                        JOIN lessons l ON lp.lesson_id = l.id
                        JOIN modules m ON l.module_id = m.id
                        WHERE lp.user_id = $1
                        AND lp.is_completed = true
                        AND m.course_id = uc.course_id
                        GROUP BY m.course_id
                        HAVING COUNT(*) = (
                            SELECT COUNT(*)
                            FROM lessons l2
                            JOIN modules m2 ON l2.module_id = m2.id
                            WHERE m2.course_id = uc.course_id
                        )
                    )
                    "#
                )
                .bind(user_id)
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })? as i32
            }

            "lesson_completed" => {
                sqlx::query_scalar::<_, i64>(
                    "SELECT COUNT(*) FROM user_lesson_progress WHERE user_id = $1 AND is_completed = true"
                )
                .bind(user_id)
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })? as i32
            }

            "courses_enrolled" => {
                sqlx::query_scalar::<_, i64>(
                    "SELECT COUNT(*) FROM user_courses WHERE user_id = $1"
                )
                .bind(user_id)
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })? as i32
            }
            
            "comments_created" => {
                sqlx::query_scalar::<_, i64>(
                    "SELECT COUNT(*) FROM lesson_comments WHERE user_id = $1"
                )
                .bind(user_id)
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })? as i32
            }

            "login_streak" => {
                let count: i32 = sqlx::query_scalar(
                    r#"
                    SELECT COALESCE(
                        (SELECT value
                        FROM user_stats
                        WHERE user_id = $1 AND stat_type = 'login_streak'),
                        0
                    )
                    "#
                )
                .bind(user_id)
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;

                count as i32
            }

            _ => fallback_value,
        };

        // 2️⃣ Obtener logros alcanzables
        let achievements = sqlx::query_as::<_, Achievement>(
            r#"
            SELECT id, name, description, icon, trigger_type, trigger_value, active, created_at
            FROM achievement
            WHERE trigger_type = $1
              AND trigger_value <= $2
              AND active = true
            "#
        )
        .bind(action)
        .bind(current_value)
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;

        let mut awarded = Vec::new();

        // 3️⃣ Insertar / actualizar logros de forma atómica
        for achievement in achievements {
            let was_awarded = sqlx::query_scalar::<_, bool>(
                r#"
                INSERT INTO user_achievement (user_id, achievement_id, earned, earned_at)
                VALUES ($1, $2, true, NOW())
                ON CONFLICT (user_id, achievement_id)
                DO UPDATE
                SET earned = true,
                    earned_at = COALESCE(user_achievement.earned_at, NOW())
                WHERE user_achievement.earned = false
                RETURNING true
                "#
            )
            .bind(user_id)
            .bind(achievement.id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?
            .unwrap_or(false);

            if was_awarded {
                awarded.push(achievement);
            }
        }

        tx.commit().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        Ok(awarded)
    }
}

#[async_trait]
pub trait CoursePurchaseExt {
    async fn register_course_purchase(
        &self,
        user_id: Uuid,
        course_id: Uuid,
        transaction_id: String,
        amount: i64,
        payment_method: String,
        status: String,
    ) -> Result<(), Error>;

    async fn check_user_course_access (
        &self,
        user_id: Uuid,
        course_id: Uuid,
    ) -> Result<Option<bool>, Error>;

    async fn get_user_purchased_courses(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<Uuid>, Error>;

    async fn update_lesson_progress(
        &self,
        user_id: Uuid,
        lesson_id: Uuid,
        is_completed: bool,
        progress: Option<f64>,
    ) -> Result<(), Error>;
}

#[async_trait]
impl CoursePurchaseExt for DBClient {

    async fn register_course_purchase(
        &self,
        user_id: Uuid,
        course_id: Uuid,
        transaction_id: String,
        amount: i64,
        payment_method: String,
        status: String,
    ) -> Result<(), Error> {
        let mut tx = self.pool.begin().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        // Verificar que el curso existe
        let course_exists = query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM courses WHERE id = $1)",
            course_id
        )
        .fetch_one(&mut *tx)
        .await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;

        if !course_exists.unwrap_or(false) {
            return Err(Error::RowNotFound);
        }

        // Registrar la compra en la tabla payments y user_courses
        query_as::<_, Payment>(
            r#"
            INSERT INTO payments
            (id, user_id, course_id, amount, payment_method, transaction_id, status, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, user_id, course_id, amount, payment_method, transaction_id, status, created_at, updated_at
            "#
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(course_id)
        .bind(amount)
        .bind(payment_method)
        .bind(transaction_id)
        .bind(status)
        .bind(Utc::now())
        .bind(Utc::now())
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            log::error!("ERROR: {}", e);
            e
        })?;

        // Registrar en user_courses si no existe
        let user_course_exists = query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM user_courses WHERE user_id = $1 AND course_id = $2)",
            user_id,
            course_id
        )
        .fetch_one(&mut *tx)
        .await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;

        if !user_course_exists.unwrap_or(false) {
            query!(
                r#"
                INSERT INTO user_courses (id, user_id, course_id, purchased_at)
                VALUES ($1, $2, $3, $4)
                "#,
                Uuid::new_v4(),
                user_id,
                course_id,
                Utc::now(),
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
            query!(
                r#"
                UPDATE courses
                SET students = students + 1
                WHERE id = $1
                "#,
                course_id
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        }

        // Inicializar progreso del curso si no existe
        let progress_exists = query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM course_progress WHERE user_id = $1 AND course_id = $2)",
            user_id,
            course_id
        )
        .fetch_one(&mut *tx)
        .await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;

        if !progress_exists.unwrap_or(false) {
            // Obtener el número total de lecciones del curso
            let total_lessons = query_scalar!(
                r#"
                SELECT COUNT(l.*)
                FROM courses c
                JOIN modules m ON m.course_id = c.id
                JOIN lessons l ON l.module_id = m.id
                WHERE c.id = $1
                "#,
                course_id
            )
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;

            let total_lessons_i32 = total_lessons.map(|v| v as i32);

            query!(
                r#"
                INSERT INTO course_progress
                (id, user_id, course_id, progress_percentage, total_lessons, completed_lessons, last_accessed, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                "#,
                Uuid::new_v4(),
                user_id,
                course_id,
                0.0,  // progreso inicial 0%
                total_lessons_i32,
                Some(0),  // 0 lecciones completadas inicialmente
                Utc::now(),
                Utc::now(),
                Utc::now()
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        }
        tx.commit().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;

        // Verificar logros de cursos inscritos
        let _ = self.check_and_award_achievements(user_id, "courses_enrolled", None).await;

        Ok(())
    }

    async fn check_user_course_access(
        &self,
        user_id: Uuid,
        course_id: Uuid
    ) -> Result<Option<bool>, Error> {
        let mut tx = self.pool.begin().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        // 1. Verificar si el usuario es admin
        let is_admin = query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM users WHERE id = $1 AND role = 'admin')",
            user_id
        )
        .fetch_one(&mut *tx)
        .await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;

        if is_admin.unwrap_or(false) {
            return Ok(Some(true));
        }

        // 2. Verificar si el usuario tiene una suscripción activa
        let has_active_subscription = query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM subscription WHERE user_id = $1 AND status = true AND end_time > NOW())",
            user_id
        )
        .fetch_one(&mut *tx)
        .await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;

        if has_active_subscription.unwrap_or(false) {
            return Ok(Some(true));
        }
        
        // 3. Verificar si el usuario ha comprado este curso específico
        let check = query_scalar!(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM user_courses
                WHERE user_id = $1 AND course_id = $2
            )
            "#,
            user_id,
            course_id
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            log::error!("Error: {}", e);
            e
        });
        tx.commit().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        return check
    }

    async fn get_user_purchased_courses(
        &self,
        user_id: Uuid
    ) -> Result<Vec<Uuid>, Error> {
        let mut tx = self.pool.begin().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        let purcha = query_as::<_, UserCourse>(
            r#"
            SELECT id, user_id, course_id, purchased_at, created_at, updated_at
            FROM user_courses
            WHERE user_id = $1
            "#
        )
        .bind(user_id)
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| {
            log::error!("Error: {}", e);
            e
        })
        .map(|user_courses| {
            user_courses.into_iter().map(|uc| uc.course_id).collect()
        });
        tx.commit().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        return purcha
    }
    
    async fn update_lesson_progress(
        &self,
        user_id: Uuid,
        lesson_id: Uuid,
        is_completed: bool,
        progress: Option<f64>,
    ) -> Result<(), Error> {
        let mut tx = self.pool.begin().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;

        // Actualizar o crear el progreso de la lección
        let _ = sqlx::query!(
            r#"
            INSERT INTO user_lesson_progress (id, user_id, lesson_id, is_completed, progress, last_accessed)
            VALUES ($1, $2, $3, $4, $5, NOW())
            ON CONFLICT (user_id, lesson_id)
            DO UPDATE SET
                is_completed = $4,
                progress = $5,
                last_accessed = NOW(),
                updated_at = NOW(),
                completed_at = CASE WHEN $4 = true THEN NOW() ELSE user_lesson_progress.completed_at END
            "#,
            Uuid::new_v4(),
            user_id,
            lesson_id,
            is_completed,
            progress
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            log::error!("Error: {}", e);
            e
        });

        // Obtener el curso y el número total de lecciones
        let module_id = sqlx::query_scalar!(
            r#"
            SELECT module_id FROM lessons WHERE id = $1
            "#,
            lesson_id
        )
        .fetch_one(&mut *tx)
        .await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;

        let course_id = sqlx::query_scalar!(
            r#"
            SELECT course_id FROM modules WHERE id = $1
            "#,
            module_id
        )
        .fetch_one(&mut *tx)
        .await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;

        let total_lessons = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) FROM lessons WHERE module_id IN (SELECT id FROM modules WHERE course_id = $1)
            "#,
            course_id
        )
        .fetch_one(&mut *tx)
        .await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;

        let completed_lessons = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) FROM user_lesson_progress
            WHERE user_id = $1 AND is_completed = true AND lesson_id IN (
                SELECT id FROM lessons WHERE module_id IN (
                    SELECT id FROM modules WHERE course_id = $2
                )
            )
            "#,
            user_id,
            course_id
        )
        .fetch_one(&mut *tx)
        .await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;

        // Desempaquetar los valores Option a i64
        let completed_lessons_value = completed_lessons.unwrap_or(0);
        let total_lessons_value = total_lessons.unwrap_or(1);

        // Calcular el porcentaje de progreso
        let progress_percentage = if total_lessons_value > 0 {
            (completed_lessons_value as f32 / total_lessons_value as f32) * 100.0
        } else {
            0.0
        };


        // Actualizar el progreso del curso
        sqlx::query!(
            r#"
            INSERT INTO course_progress (id, user_id, course_id, progress_percentage, total_lessons, completed_lessons, last_accessed)
            VALUES ($1, $2, $3, $4, $5, $6, NOW())
            ON CONFLICT (user_id, course_id)
            DO UPDATE SET
                progress_percentage = $4,
                completed_lessons = $6,
                last_accessed = NOW(),
                updated_at = NOW(),
                completed_at = CASE WHEN $4 = 100 THEN NOW() ELSE course_progress.completed_at END
            "#,
            Uuid::new_v4(),
            user_id,
            course_id,
            progress_percentage,
            total_lessons_value as i32,
            completed_lessons_value as i32
        )
        .execute(&mut *tx)
        .await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        tx.commit().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        // Otorgar logros después del commit
        if is_completed {
            let _ = self
                .check_and_award_achievements(user_id, "lesson_completed", None)
                .await;
        }

        if progress_percentage >= 100.0 {
            let _ = self
                .check_and_award_achievements(user_id, "course_completed", None)
                .await;
        }
    
        Ok(())
    }

}

#[async_trait]
pub trait SubscriptionPlanExt {
    async fn create_subscription_plan(
        &self,
        name: &str,
        description: Option<&String>,
        price: f64,
        duration_months: i32,
        features: Option<&serde_json::Value>,
        paypal_plan_id: Option<&str>,
        trial_days: Option<i32>,
    ) -> Result<SubscriptionPlan, Error>;

    async fn update_subscription_plan(
        &self,
        plan_id: Uuid,
        name: Option<&str>,
        description: Option<&String>,
        price: Option<f64>,
        duration_months: Option<i32>,
        features: Option<&serde_json::Value>,
        paypal_plan_id: Option<&str>,
        trial_days: Option<i32>,
        active: Option<bool>,
    ) -> Result<SubscriptionPlan, Error>;

    async fn delete_subscription_plan(&self, plan_id: Uuid) -> Result<(), Error>;

    async fn get_subscription_plans(&self) -> Result<Vec<SubscriptionPlan>, Error>;
}

#[async_trait]
pub trait SubscriptionExt {
    async fn create_subscription(
        &self,
        user_id: Uuid,
        plan_id: &String,
        paypal_id: &String,
    ) -> Result<Subscription, Error>;

    async fn get_user_subscriptions(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<Subscription>, Error>;

    async fn cancel_subscription(
        &self,
        subscription_id: Uuid,
    ) -> Result<(), Error>;

    async fn update_subscription_end_time(
        &self,
        paypal_subscription_id: &str,
    ) -> Result<(), Error>;

    async fn update_subscription_status(
        &self,
        paypal_subscription_id: &str,
        status: bool,
    ) -> Result<(), Error>;

    async fn expire_subscription(
        &self,
        paypal_subscription_id: &str,
    ) -> Result<(), Error>;

    async fn check_user_has_active_subscription(
        &self,
        user_id: Uuid,
    ) -> Result<bool, Error>;
}

#[async_trait]
impl SubscriptionPlanExt for DBClient {
    async fn create_subscription_plan(
        &self,
        name: &str,
        description: Option<&String>,
        price: f64,
        duration_months: i32,
        features: Option<&serde_json::Value>,
        paypal_plan_id: Option<&str>,
        trial_days: Option<i32>,
    ) -> Result<SubscriptionPlan, Error> {
        let mut tx = self.pool.begin().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        let id = Uuid::new_v4();
        let now = Utc::now();

        let plan = sqlx::query_as::<_, SubscriptionPlan>(
            r#"
            INSERT INTO subscription_plans (id, name, description, price, duration_months, features, paypal_plan_id, trial_days, active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, true, $9, $10)
            RETURNING id, name, description, price, duration_months, features, paypal_plan_id, trial_days, active, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(description)
        .bind(price)
        .bind(duration_months)
        .bind(features)
        .bind(paypal_plan_id)
        .bind(trial_days)
        .bind(now)
        .bind(now)
        .fetch_one(&mut *tx)
        .await.map_err(|e| {
            log::error!("ERROR: {}", e);
            e
        })?;
        tx.commit().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        Ok(plan)
    }

    async fn update_subscription_plan(
        &self,
        plan_id: Uuid,
        name: Option<&str>,
        description: Option<&String>,
        price: Option<f64>,
        duration_months: Option<i32>,
        features: Option<&serde_json::Value>,
        paypal_plan_id: Option<&str>,
        trial_days: Option<i32>,
        active: Option<bool>,
    ) -> Result<SubscriptionPlan, Error> {
        let mut tx = self.pool.begin().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        let now = Utc::now();

        let plan = sqlx::query_as::<_, SubscriptionPlan>(
            r#"
            UPDATE subscription_plans
            SET name = COALESCE($2, name),
                description = COALESCE($3, description),
                price = COALESCE($4, price),
                duration_months = COALESCE($5, duration_months),
                features = COALESCE($6, features),
                paypal_plan_id = COALESCE($7, paypal_plan_id),
                trial_days = COALESCE($8, trial_days),
                active = COALESCE($9, active),
                updated_at = $10
            WHERE id = $1
            RETURNING id, name, description, price, duration_months, features, paypal_plan_id, trial_days, active, created_at, updated_at
            "#,
        )
        .bind(plan_id)
        .bind(name)
        .bind(description)
        .bind(price)
        .bind(duration_months)
        .bind(features)
        .bind(paypal_plan_id)
        .bind(trial_days)
        .bind(active)
        .bind(now)
        .fetch_one(&mut *tx)
        .await.map_err(|e| {
            log::error!("ERROR: {}", e);
            e
        })?;
        tx.commit().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        Ok(plan)
    }

    async fn delete_subscription_plan(&self, plan_id: Uuid) -> Result<(), Error> {
        let mut tx = self.pool.begin().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        sqlx::query("DELETE FROM subscription_plans WHERE id = $1")
            .bind(plan_id)
            .execute(&mut *tx)
            .await.map_err(|e| {
                log::error!("ERROR: {}", e);
                e
            })?;
        tx.commit().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        Ok(())
    }

    async fn get_subscription_plans(&self) -> Result<Vec<SubscriptionPlan>, Error> {
        let mut tx = self.pool.begin().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        let plans = sqlx::query_as::<_, SubscriptionPlan>(
            r#"
            SELECT id, name, description, price, duration_months, features, paypal_plan_id, trial_days, active, created_at, updated_at
            FROM subscription_plans
            WHERE active = true
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&mut *tx)
        .await.map_err(|e| {
            log::error!("ERROR: {}", e);
            e
        })?;
        tx.commit().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        Ok(plans)
    }
}

#[async_trait]
impl SubscriptionExt for DBClient {
    async fn create_subscription(
        &self,
        user_id: Uuid,
        paypal_id: &String, // El ID que empieza con I-XXXX
        plan_id: &String,   // El ID del plan en tu sistema
    ) -> Result<Subscription, Error> {
        let mut tx = self.pool.begin().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        let id = Uuid::new_v4();
        let now = Utc::now();

        // Cancelar cualquier suscripción activa del usuario
        sqlx::query(
            r#"
            UPDATE subscription
            SET status = false, updated_at = $2
            WHERE user_id = $1 AND status = true
            "#,
        )
        .bind(user_id)
        .bind(now)
        .execute(&mut *tx)
        .await.map_err(|e| {
            log::error!("ERROR: {}", e);
            e
        })?;

        // Calcular end_time basado en duration_months del plan
        let duration_months: Option<i32> = sqlx::query_scalar(
            r#"
            SELECT duration_months FROM subscription_plans WHERE paypal_plan_id = $1
            "#,
        )
        .bind(plan_id)
        .fetch_optional(&mut *tx)
        .await.map_err(|e| {
            log::error!("ERROR fetching plan duration: {}", e);
            e
        })?;

        let end_time = duration_months.map(|months| {
            now + chrono::Duration::days((months * 30) as i64)
        });

        let subscription = sqlx::query_as::<_, Subscription>(
            r#"
            INSERT INTO subscription (
                id, 
                user_id, 
                paypal_subscription_id, 
                status, 
                plan_id, 
                start_time, 
                end_time, 
                auto_renew,
                created_at, 
                updated_at
            )
            VALUES ($1, $2, $3, true, $4, $5, $6, true, $7, $8)
            RETURNING id, user_id, paypal_subscription_id, status, plan_id, start_time, end_time, auto_renew, created_at, updated_at
            "#,
        )
        .bind(id)        // $1
        .bind(user_id)   // $2
        .bind(paypal_id) // $3
        .bind(plan_id)   // $4
        .bind(now)       // $5 (start_time)
        .bind(end_time)  // $6 (end_time calculado)
        .bind(now)       // $7 (created_at)
        .bind(now)       // $8 (updated_at)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            log::error!("ERROR EN INSERT: {}", e);
            e
        })?;

        tx.commit().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        Ok(subscription)
    }

    async fn get_user_subscriptions(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<Subscription>, Error> {
        let mut tx = self.pool.begin().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        let subscriptions = sqlx::query_as::<_, Subscription>(
            r#"
            SELECT id, user_id, paypal_subscription_id, status, plan_id, start_time, end_time, auto_renew, created_at, updated_at
            FROM subscription
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&mut *tx)
        .await.map_err(|e| {
            log::error!("ERROR: {}", e);
            e
        })?;
        tx.commit().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        Ok(subscriptions)
    }

    async fn cancel_subscription(
        &self,
        subscription_id: Uuid,
    ) -> Result<(), Error> {
        let mut tx = self.pool.begin().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        let now = Utc::now();

        // Solo marcar auto_renew = false, la suscripción sigue activa hasta end_time
        sqlx::query(
            r#"
            UPDATE subscription
            SET auto_renew = false, updated_at = $2
            WHERE id = $1
            "#,
        )
        .bind(subscription_id)
        .bind(now)
        .execute(&mut *tx)
        .await.map_err(|e| {
            log::error!("ERROR: {}", e);
            e
        })?;
        tx.commit().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        Ok(())
    }

    async fn update_subscription_end_time(
        &self,
        paypal_subscription_id: &str,
    ) -> Result<(), Error> {
        let mut tx = self.pool.begin().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        let now = Utc::now();

        // Obtener plan_id de la suscripción
        let plan_id: Option<String> = sqlx::query_scalar(
            r#"
            SELECT plan_id FROM subscription WHERE paypal_subscription_id = $1
            "#,
        )
        .bind(paypal_subscription_id)
        .fetch_optional(&mut *tx)
        .await.map_err(|e| {
            log::error!("ERROR: {}", e);
            e
        })?;

        if let Some(plan_id) = plan_id {
            // Obtener duration_months del plan usando paypal_plan_id
            let duration_months: Option<i32> = sqlx::query_scalar(
                r#"
                SELECT duration_months FROM subscription_plans WHERE paypal_plan_id = $1
                "#,
            )
            .bind(&plan_id)
            .fetch_optional(&mut *tx)
            .await.map_err(|e| {
                log::error!("ERROR: {}", e);
                e
            })?;

            if let Some(duration_months) = duration_months {
                let end_time = now + chrono::Duration::days((duration_months * 30) as i64); // Aproximado

                sqlx::query(
                    r#"
                    UPDATE subscription
                    SET end_time = $2, updated_at = $3
                    WHERE paypal_subscription_id = $1
                    "#,
                )
                .bind(paypal_subscription_id)
                .bind(end_time)
                .bind(now)
                .execute(&mut *tx)
                .await.map_err(|e| {
                    log::error!("ERROR: {}", e);
                    e
                })?;
            }
        }

        tx.commit().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        Ok(())
    }

    async fn update_subscription_status(
        &self,
        paypal_subscription_id: &str,
        status: bool,
    ) -> Result<(), Error> {
        let mut tx = self.pool.begin().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        let now = Utc::now();

        sqlx::query(
            r#"
            UPDATE subscription
            SET status = $2, updated_at = $3
            WHERE paypal_subscription_id = $1
            "#,
        )
        .bind(paypal_subscription_id)
        .bind(status)
        .bind(now)
        .execute(&mut *tx)
        .await.map_err(|e| {
            log::error!("ERROR: {}", e);
            e
        })?;
        tx.commit().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        Ok(())
    }

    async fn expire_subscription(
        &self,
        paypal_subscription_id: &str,
    ) -> Result<(), Error> {
        let mut tx = self.pool.begin().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        let now = Utc::now();

        sqlx::query(
            r#"
            UPDATE subscription
            SET status = false, end_time = $2, updated_at = $2
            WHERE paypal_subscription_id = $1 AND end_time IS NULL
            "#,
        )
        .bind(paypal_subscription_id)
        .bind(now)
        .execute(&mut *tx)
        .await.map_err(|e| {
            log::error!("ERROR: {}", e);
            e
        })?;
        tx.commit().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        Ok(())
    }

    async fn check_user_has_active_subscription(
        &self,
        user_id: Uuid,
    ) -> Result<bool, Error> {
        let mut tx = self.pool.begin().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        let has_active = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM subscription 
                WHERE user_id = $1 AND status = true AND end_time > NOW()
            )
            "#,
        )
        .bind(user_id)
        .fetch_one(&mut *tx)
        .await.map_err(|e| {
            log::error!("ERROR: {}", e);
            e
        })?;
        tx.commit().await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        Ok(has_active)
    }
}

#[async_trait]
pub trait PasswordResetTokenExt {
    async fn generate_reset_token_atomic(
        &self,
        user_email: &str,
        new_token_id: Uuid,
        token_hash: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<Option<ForgotPasswordResult>, Error>;

    async fn reset_password_with_token(
        &self,
        token_hash: &str,
        new_password_hash: &str,
    ) -> Result<Option<Uuid>, Error>;
}

#[async_trait]
impl PasswordResetTokenExt for DBClient {
    async fn generate_reset_token_atomic(
        &self,
        user_email: &str,
        new_token_id: Uuid,
        token_hash: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<Option<ForgotPasswordResult>, Error> {
        
        let row = sqlx::query(
            r#"
            WITH target_user AS (
                SELECT id, name, email
                FROM users
                WHERE email = $1
                LIMIT 1
            ),
            invalidated_tokens AS (
                UPDATE password_reset_tokens
                SET used = true
                WHERE user_id = (SELECT id FROM target_user)
                  AND used = false
                RETURNING id
            ),
            max_version AS (
                SELECT COALESCE(MAX(version), 0) + 1 as next_version
                FROM password_reset_tokens
                WHERE user_id = (SELECT id FROM target_user)
            ),
            inserted_token AS (
                INSERT INTO password_reset_tokens 
                    (id, user_id, token_hash, version, expires_at, used, created_at)
                SELECT 
                    $2, 
                    (SELECT id FROM target_user), 
                    $3, 
                    (SELECT next_version FROM max_version), 
                    $4, 
                    false, 
                    NOW()
                WHERE EXISTS (SELECT 1 FROM target_user)
                RETURNING id, user_id, token_hash, version, expires_at, used, created_at
            )
            SELECT 
                u.name AS user_name, 
                u.email AS user_email,
                t.id AS token_id,
                t.user_id,
                t.token_hash,
                t.version,
                t.expires_at,
                t.used,
                t.created_at
            FROM target_user u
            JOIN inserted_token t ON t.user_id = u.id
            "#
        )
        .bind(user_email)
        .bind(new_token_id)
        .bind(token_hash)
        .bind(expires_at)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            log::error!("Error atomic reset token: {}", e);
            e
        })?;

        Ok(if let Some(r) = row {
            Some(ForgotPasswordResult {
                user_name: r.try_get("user_name")?,
                user_email: r.try_get("user_email")?,
                token_data: PasswordResetToken {
                    id: r.try_get("token_id")?,
                    user_id: r.try_get("user_id")?,
                    token_hash: r.try_get("token_hash")?,
                    version: r.try_get("version")?,
                    expires_at: r.try_get("expires_at")?,
                    used: r.try_get("used")?,
                    created_at: r.try_get("created_at")?,
                }
            })
        } else {
            None
        })
    }

    async fn reset_password_with_token(
        &self,
        token_hash: &str,
        new_password_hash: &str,
    ) -> Result<Option<Uuid>, Error> {
        // Devuelve Some(user_id) si se actualizó; None si token inválido/expirado/usado
        let row = sqlx::query(
            r#"
            WITH consumed AS (
              UPDATE password_reset_tokens
              SET used = TRUE
              WHERE token_hash = $1
                AND used = FALSE
                AND expires_at > NOW()
              RETURNING user_id
            ),
            updated AS (
              UPDATE users
              SET password = $2, updated_at = NOW()
              WHERE id = (SELECT user_id FROM consumed)
              RETURNING id
            )
            SELECT id FROM updated;
            "#
        )
        .bind(token_hash)
        .bind(new_password_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            log::error!("ERROR: {}", e);
            e
        })?;

        Ok(row.map(|r| r.get::<Uuid, _>("id")))
    }
}

// ===================== //
//      QUIZ EXT
// ===================== //

#[async_trait]
pub trait QuizExt {
    async fn get_quiz_by_lesson(&self, lesson_id: Uuid) -> Result<Option<QuizResponseDto>, Error>;
    async fn get_quiz(&self, quiz_id: Uuid) -> Result<Option<QuizResponseDto>, Error>;
    async fn get_quiz_questions(&self, quiz_id: Uuid) -> Result<Vec<QuestionDto>, Error>;
    async fn submit_quiz_attempt(
        &self, 
        user_id: Uuid, 
        quiz_id: Uuid, 
        score: i32, 
        total_score: i32, 
        percentage: f64, 
        passed: bool, 
        answers: serde_json::Value
    ) -> Result<QuizAttempt, Error>;
    async fn get_user_attempts(&self, user_id: Uuid, quiz_id: Uuid) -> Result<Vec<QuizAttemptDto>, Error>;
    async fn get_attempt(&self, attempt_id: Uuid) -> Result<Option<QuizAttempt>, Error>;

    // Helpers to link quiz -> course and compute completion
    async fn get_course_id_by_quiz(&self, quiz_id: Uuid) -> Result<Option<Uuid>, Error>;
    async fn get_total_quizzes_in_course(&self, course_id: Uuid) -> Result<i64, Error>;
    async fn get_user_passed_quizzes_count(&self, user_id: Uuid, course_id: Uuid) -> Result<i64, Error>;

    // Admin CRUD for quizzes including nested questions/options
    async fn create_quiz_with_questions(&self, create_quiz: CreateQuizDto) -> Result<QuizResponseDto, Error>;
    async fn update_quiz_with_questions(&self, quiz_id: Uuid, create_quiz: CreateQuizDto) -> Result<QuizResponseDto, Error>;
    async fn delete_quiz(&self, quiz_id: Uuid) -> Result<(), Error>;
}

#[async_trait]
impl QuizExt for DBClient {
    async fn get_quiz_by_lesson(&self, lesson_id: Uuid) -> Result<Option<QuizResponseDto>, Error> {
         let quiz = sqlx::query_as!(
            QuizResponseDto,
            r#"
            SELECT 
                id::text, 
                lesson_id::text, 
                title, 
                description, 
                pass_percentage,
                (SELECT COUNT(*) FROM questions WHERE quiz_id = q.id) as "total_questions!",
                1 as "order!" -- Placeholder
            FROM quizzes q
            WHERE lesson_id = $1
            "#,
            lesson_id
        )
        .fetch_optional(&self.pool)
        .await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        Ok(quiz)
    }

    async fn get_quiz(&self, quiz_id: Uuid) -> Result<Option<QuizResponseDto>, Error> {
        let quiz = sqlx::query_as!(
            QuizResponseDto,
            r#"
            SELECT 
                id::text, 
                lesson_id::text, 
                title, 
                description, 
                pass_percentage,
                (SELECT COUNT(*) FROM questions WHERE quiz_id = q.id) as "total_questions!",
                1 as "order!" -- Placeholder
            FROM quizzes q
            WHERE id = $1
            "#,
            quiz_id
        )
        .fetch_optional(&self.pool)
        .await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        Ok(quiz)
    }

    async fn get_quiz_questions(&self, quiz_id: Uuid) -> Result<Vec<QuestionDto>, Error> {
        // Fetch questions
        let questions = sqlx::query_as!(
            Question,
            r#"SELECT * FROM questions WHERE quiz_id = $1 ORDER BY "order" ASC"#,
            quiz_id
        )
        .fetch_all(&self.pool)
        .await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;

        let mut question_dtos = Vec::new();

        for q in questions {
            let options = sqlx::query_as!(
                OptionDto,
                r#"SELECT id::text, text, "order" FROM question_options WHERE question_id = $1 ORDER BY "order" ASC"#,
                q.id
            )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;

             // Only fetch correct option if needed, but for now we follow the structure
            let correct_option = sqlx::query_scalar!(
                r#"SELECT id::text FROM question_options WHERE question_id = $1 AND is_correct = true LIMIT 1"#,
                q.id
            )
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| {
                log::error!("Error: {}", e);
                e
            })?;

            question_dtos.push(QuestionDto {
                id: q.id.to_string(),
                question: q.question,
                description: q.description,
                options,
                correct_option_id: correct_option.unwrap(),
                explanation: q.explanation,
                order: q.order,
            });
        }
        
        Ok(question_dtos)
    }

    async fn submit_quiz_attempt(
        &self, 
        user_id: Uuid, 
        quiz_id: Uuid, 
        score: i32, 
        total_score: i32, 
        percentage: f64, 
        passed: bool, 
        answers: serde_json::Value
    ) -> Result<QuizAttempt, Error> {
        let attempt = sqlx::query_as!(
            QuizAttempt,
            r#"
            INSERT INTO quiz_attempts (id, quiz_id, user_id, score, total_score, percentage, passed, answers, submitted_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW())
            RETURNING *
            "#,
            Uuid::new_v4(),
            quiz_id,
            user_id,
            score,
            total_score,
            percentage,
            passed,
            answers
        )
        .fetch_one(&self.pool)
        .await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        Ok(attempt)
    }

    async fn get_user_attempts(&self, user_id: Uuid, quiz_id: Uuid) -> Result<Vec<QuizAttemptDto>, Error> {
         let attempts = sqlx::query_as!(
            QuizAttemptDto,
            r#"
            SELECT id, quiz_id, user_id, score, percentage, passed, submitted_at 
            FROM quiz_attempts 
            WHERE user_id = $1 AND quiz_id = $2
            ORDER BY submitted_at DESC
            "#,
            user_id,
            quiz_id
        )
        .fetch_all(&self.pool)
        .await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        Ok(attempts)
    }

    async fn get_attempt(&self, attempt_id: Uuid) -> Result<Option<QuizAttempt>, Error> {
        let attempt = sqlx::query_as!(
            QuizAttempt,
            r#"SELECT * FROM quiz_attempts WHERE id = $1"#,
            attempt_id
        )
        .fetch_optional(&self.pool)
        .await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        Ok(attempt)       
    }

    async fn get_course_id_by_quiz(&self, quiz_id: Uuid) -> Result<Option<Uuid>, Error> {
        let course_id = sqlx::query_scalar!(
            r#"
            SELECT c.id::uuid FROM courses c
            JOIN modules m ON m.course_id = c.id
            JOIN lessons l ON l.module_id = m.id
            JOIN quizzes q ON q.lesson_id = l.id
            WHERE q.id = $1
            LIMIT 1
            "#,
            quiz_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            log::error!("Error: {}", e);
            e
        })?;

        Ok(course_id)
    }

    async fn get_total_quizzes_in_course(&self, course_id: Uuid) -> Result<i64, Error> {
        let total = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM quizzes q
               JOIN lessons l ON q.lesson_id = l.id
               JOIN modules m ON l.module_id = m.id
               WHERE m.course_id = $1"#,
            course_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| { log::error!("Error: {}", e); e })?;

        Ok(total.unwrap())
    }

    async fn get_user_passed_quizzes_count(&self, user_id: Uuid, course_id: Uuid) -> Result<i64, Error> {
        let count = sqlx::query_scalar!(
            r#"SELECT COUNT(DISTINCT q.id) FROM quiz_attempts qa
               JOIN quizzes q ON qa.quiz_id = q.id
               JOIN lessons l ON q.lesson_id = l.id
               JOIN modules m ON l.module_id = m.id
               WHERE qa.user_id = $1 AND m.course_id = $2 AND qa.passed = true"#,
            user_id,
            course_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| { log::error!("Error: {}", e); e })?;

        Ok(count.unwrap())
    }

    async fn create_quiz_with_questions(&self, create_quiz: CreateQuizDto) -> Result<QuizResponseDto, Error> {
        let mut tx = self.pool.begin().await.map_err(|e| { log::error!("Error: {}", e); e })?;
        let quiz_id = Uuid::new_v4();
        let lesson_id = Uuid::parse_str(&create_quiz.lesson_id)
            .map_err(|e| { log::error!("UUID parse error: {}", e); e });

        sqlx::query!(
            r#"INSERT INTO quizzes (id, lesson_id, title, description, pass_percentage) VALUES ($1, $2, $3, $4, $5)"#,
            quiz_id,
            lesson_id.unwrap(),
            create_quiz.title,
            create_quiz.description,
            create_quiz.pass_percentage.unwrap_or(70.0)
        )
        .execute(tx.as_mut())
        .await
        .map_err(|e| { log::error!("Error: {}", e); e })?;

        for (_q_idx, qdto) in create_quiz.questions.into_iter().enumerate() {
            let question_id = Uuid::new_v4();
            sqlx::query!(
                r#"INSERT INTO questions (id, quiz_id, question, description, explanation, "order") VALUES ($1, $2, $3, $4, $5, $6)"#,
                question_id,
                quiz_id,
                qdto.question,
                qdto.description,
                qdto.explanation,
                qdto.order
            )
            .execute(tx.as_mut())
            .await
            .map_err(|e| { log::error!("Error: {}", e); e })?;

            for opt in qdto.options.into_iter() {
                sqlx::query!(
                    r#"INSERT INTO question_options (id, question_id, text, is_correct, "order") VALUES ($1, $2, $3, $4, $5)"#,
                    Uuid::new_v4(),
                    question_id,
                    opt.text,
                    opt.is_correct,
                    opt.order
                )
                .execute(tx.as_mut())
                .await
                .map_err(|e| { log::error!("Error: {}", e); e })?;
            }
        }

        tx.commit().await.map_err(|e| { log::error!("Error: {}", e); e })?;
        // Return created quiz DTO
        let res = self.get_quiz(quiz_id).await?;
        match res {
            Some(q) => Ok(q),
            None => Err(sqlx::Error::RowNotFound),
        }
    }

    async fn update_quiz_with_questions(&self, quiz_id: Uuid, create_quiz: CreateQuizDto) -> Result<QuizResponseDto, Error> {
        let mut tx = self.pool.begin().await.map_err(|e| { log::error!("Error: {}", e); e })?;

        sqlx::query!(
            r#"UPDATE quizzes SET title = $1, description = $2, pass_percentage = $3, updated_at = NOW() WHERE id = $4"#,
            create_quiz.title,
            create_quiz.description,
            create_quiz.pass_percentage.unwrap_or(70.0),
            quiz_id
        )
        .execute(tx.as_mut())
        .await
        .map_err(|e| { log::error!("Error: {}", e); e })?;

        // Delete existing questions (this cascades question_options due to FK)
        sqlx::query!(r#"DELETE FROM questions WHERE quiz_id = $1"#, quiz_id)
            .execute(tx.as_mut())
            .await
            .map_err(|e| { log::error!("Error: {}", e); e })?;

        for qdto in create_quiz.questions.into_iter() {
            let question_id = Uuid::new_v4();
            sqlx::query!(
                r#"INSERT INTO questions (id, quiz_id, question, description, explanation, "order") VALUES ($1, $2, $3, $4, $5, $6)"#,
                question_id,
                quiz_id,
                qdto.question,
                qdto.description,
                qdto.explanation,
                qdto.order
            )
            .execute(tx.as_mut())
            .await
            .map_err(|e| { log::error!("Error: {}", e); e })?;

            for opt in qdto.options.into_iter() {
                sqlx::query!(
                    r#"INSERT INTO question_options (id, question_id, text, is_correct, "order") VALUES ($1, $2, $3, $4, $5)"#,
                    Uuid::new_v4(),
                    question_id,
                    opt.text,
                    opt.is_correct,
                    opt.order
                )
                .execute(tx.as_mut())
                .await
                .map_err(|e| { log::error!("Error: {}", e); e })?;
            }
        }

        tx.commit().await.map_err(|e| { log::error!("Error: {}", e); e })?;

        let res = self.get_quiz(quiz_id).await?;
        match res {
            Some(q) => Ok(q),
            None => Err(sqlx::Error::RowNotFound),
        }
    }

    async fn delete_quiz(&self, quiz_id: Uuid) -> Result<(), Error> {
        sqlx::query!(r#"DELETE FROM quizzes WHERE id = $1"#, quiz_id)
            .execute(&self.pool)
            .await
            .map_err(|e| { log::error!("Error: {}", e); e })?;
        Ok(())
    }
}

// ===================== //
// CERTIFICATE EXT
// ===================== //

#[async_trait]
pub trait CertificateExt {
     async fn get_user_certificates(&self, user_id: Uuid) -> Result<Vec<CertificateDto>, Error>;
     async fn get_certificate(&self, certificate_id: Uuid) -> Result<Option<CertificateDto>, Error>;
     async fn generate_certificate(
         &self, 
         user_id: Uuid, 
         course_id: Uuid, 
         completion_percentage: i64
    ) -> Result<Certificate, Error>;
}

#[async_trait]
impl CertificateExt for DBClient {
    async fn get_user_certificates(&self, user_id: Uuid) -> Result<Vec<CertificateDto>, Error> {
        let certificates = sqlx::query_as!(
            CertificateDto,
            r#"
            SELECT 
                c.id, 
                c.course_id, 
                c.user_id, 
                u.name as user_name, 
                co.title as course_title, 
                c.issue_date, 
                c.completion_percentage, 
                c.certificate_number
            FROM certificates c
            JOIN users u ON c.user_id = u.id
            JOIN courses co ON c.course_id = co.id
            WHERE c.user_id = $1
            ORDER BY c.issue_date DESC
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        Ok(certificates)
    }

    async fn get_certificate(&self, certificate_id: Uuid) -> Result<Option<CertificateDto>, Error> {
         let certificates = sqlx::query_as!(
            CertificateDto,
            r#"
             SELECT 
                c.id, 
                c.course_id, 
                c.user_id, 
                u.name as user_name, 
                co.title as course_title, 
                c.issue_date, 
                c.completion_percentage, 
                c.certificate_number
            FROM certificates c
            JOIN users u ON c.user_id = u.id
            JOIN courses co ON c.course_id = co.id
            WHERE c.id = $1
            "#,
            certificate_id
        )
        .fetch_optional(&self.pool)
        .await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        Ok(certificates)
    }

    async fn generate_certificate(
         &self, 
         user_id: Uuid, 
         course_id: Uuid, 
         completion_percentage: i64
    ) -> Result<Certificate, Error> {
        // Format: CERT-YYYY-COURSEID-USERID-SEQ (Simplified)
        let now = Utc::now();
        let cert_number = format!("CERT-{}-{}-{}", now.format("%Y"), course_id.as_simple().to_string().chars().take(4).collect::<String>(), Uuid::new_v4().as_simple().to_string().chars().take(5).collect::<String>()).to_uppercase();

        let certificates = sqlx::query_as!(
            Certificate,
            r#"
            INSERT INTO certificates (id, user_id, course_id, certificate_number, completion_percentage, issue_date)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (user_id, course_id) DO UPDATE 
                SET completion_percentage = $5
            RETURNING *
            "#,
            Uuid::new_v4(),
            user_id,
            course_id,
            cert_number,
            completion_percentage as f64,
            now
        )
        .fetch_one(&self.pool)
        .await
            .map_err(|e| {
                    log::error!("Error: {}", e);
                    e
                })?;
        Ok(certificates)
    }
}