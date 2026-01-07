use std::collections::HashMap;

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use sqlx::{Pool, Postgres, query_scalar, query_as, query, Error, Row};
use uuid::Uuid;

use crate::{config::dtos::{CommentLessonDto, CourseRatingDto, CourseWithModulesDto, CreateCourseDTO, CreateLessonDTO, CreateModuleDTO, LessonDto, ModuleWithLessonsDto, UpdateCourseDTO, UserAchievementDto, UserCourseDto},  models::models::{Achievement, Course, CourseProgress, Lesson, Module, Notification, PasswordResetToken, Payment, Subscription, SubscriptionPlan, User, UserAchievement, UserCourse, UserRole}};

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

    #[allow(dead_code)]
    async fn verifed_token(
        &self,
        token: &str,
    ) -> Result<(), Error>;

    #[allow(dead_code)]
    async fn add_verifed_token(
        &self,
        user_id: Uuid,
        token: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<(), Error>;

    async fn increment_user_stat(
        &self,
        user_id: Uuid,
        stat_type: &str,
    ) -> Result<i32, Error>;

    async fn get_user_stats(
        &self,
        user_id: Uuid,
    ) -> Result<HashMap<String, i32>, Error>;
}

#[async_trait]
impl UserExt for DBClient {
    async fn get_user(
        &self,
        user_id: Option<Uuid>,
        name: Option<&str>,
        email: Option<&str>,
        token: Option<&str>,
    ) -> Result<Option<User>, Error> {
        let mut tx = self.pool.begin().await?;
        let mut user: Option<User> = None;
        if let Some(user_id) = user_id {
            user = query_as!(
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
                    profile_image_url,
                    subscription_expires_at
                FROM users
                WHERE id = $1
                "#,
                user_id
            ).fetch_optional(&mut *tx).await.map_err(|e| {
                log::error!("ERROR: {}", e);
                e
            })?
            ;
        } else if let Some(name) = name {
            user = query_as!(
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
                    profile_image_url,
                    subscription_expires_at
                FROM users
                WHERE name = $1
                "#,
                name
            ).fetch_optional(&mut *tx).await.map_err(|e| {
                log::error!("ERROR: {}", e);
                e
            })?
            ;
        } else if let Some(email) = email {
            user = query_as!(
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
                    profile_image_url,
                    subscription_expires_at
                FROM users
                WHERE email = $1
                "#,
                email
            ).fetch_optional(&mut *tx).await.map_err(|e| {
                log::error!("ERROR: {}", e);
                e
            })?
            ;
        } else if let Some(token) = token {
            user = query_as!(
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
                    profile_image_url,
                    subscription_expires_at
                FROM users
                WHERE verification_token = $1
                "#,
                token
            ).fetch_optional(&mut *tx).await.map_err(|e| {
                log::error!("ERROR: {}", e);
                e
            })?
            ;
        }
        tx.commit().await?;
        Ok(user)
    }

    async fn get_users(
        &self,
        page: u32,
        limit: usize,
    ) -> Result<Vec<User>, Error> {
        let offset = (page - 1) * limit as u32;
        let mut tx = self.pool.begin().await?;

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
        tx.commit().await?;
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
        let mut tx = self.pool.begin().await?;
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
        tx.commit().await?;
        Ok(user)
    }

    async fn get_user_count(&self) -> Result<i64, Error> {
        let mut tx = self.pool.begin().await?;
        let count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM users"#
        )
       .fetch_one(&mut *tx)
       .await.map_err(|e| {
            log::error!("ERROR: {}", e);
            e
        })?
        ;
        tx.commit().await?;
        Ok(count.unwrap_or(0))
    }

    async fn update_user_name<T: Into<String> + Send>(
        &self,
        user_id: Uuid,
        new_name: T
    ) -> Result<User, Error> {
        let mut tx = self.pool.begin().await?;
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
        tx.commit().await?;
        Ok(user)
    }

    async fn update_user_role(
        &self,
        user_id: Uuid,
        new_role: UserRole
    ) -> Result<User, Error> {
        let mut tx = self.pool.begin().await?;
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
        tx.commit().await?;
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
        let mut tx = self.pool.begin().await?;
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
        tx.commit().await?;
        Ok(user)
    }

    async fn update_user_password(
        &self,
        user_id: Uuid,
        new_password: String
    ) -> Result<User, Error> {
        let mut tx = self.pool.begin().await?;
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
        tx.commit().await?;
        Ok(user)
    }

    async fn verifed_token(
        &self,
        token: &str,
    ) -> Result<(), Error> {
        let mut tx = self.pool.begin().await?;
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
        ).execute(&mut *tx)
       .await;
        tx.commit().await?;
        Ok(())
    }

    async fn add_verifed_token(
        &self,
        user_id: Uuid,
        token: &str,
        token_expiry: DateTime<Utc>,
    ) -> Result<(), Error> {
        let mut tx = self.pool.begin().await?;
        let _ = sqlx::query!(
            r#"
            UPDATE users
            SET verification_token = $1, token_expiry = $2, updated_at = Now()
            WHERE id = $3
            "#,
            token,
            token_expiry,
            user_id,
        ).execute(&mut *tx)
       .await.map_err(|e| {
            log::error!("ERROR: {}", e);
            e
        })?
        ;
        tx.commit().await?;
        Ok(())
    }

    async fn increment_user_stat(
        &self,
        user_id: Uuid,
        stat_type: &str,
    ) -> Result<i32, Error> {
        let mut tx = self.pool.begin().await?;

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
        .await?;

        tx.commit().await?;
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
        .await?;

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

    async fn get_user_courses(&self, user_id: Uuid) -> Result<Vec<UserCourseDto>, Error>;

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

    #[allow(dead_code)]
    async fn get_course_count(&self) -> Result<i64, Error>;

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
        let mut tx = self.pool.begin().await?;
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
        tx.commit().await?;
        Ok(course)
    }

    async fn get_user_courses(
        &self,
        user_id: Uuid
    ) -> Result<Vec<UserCourseDto>, Error> {

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
            INNER JOIN user_courses uc
                ON uc.course_id = c.id
            LEFT JOIN course_ratings cr
                ON cr.course_id = c.id
            WHERE uc.user_id = $1
            GROUP BY c.id
            ORDER BY c.created_at DESC
            "#
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            log::error!("ERROR get_user_courses: {}", e);
            e
        })?;

        Ok(courses)
    }


    async fn get_courses(
        &self,
        page: u32,
        limit: usize,
    ) -> Result<Vec<UserCourseDto>, Error> {
        let mut tx = self.pool.begin().await?;
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
        tx.commit().await?;
        Ok(courses)
    }

    /// Mucho más eficiente: 3 queries en vez de un JOIN enorme.
    async fn get_all_courses_with_modules(
        &self,
    ) -> Result<Vec<CourseWithModulesDto>, Error> {
        let mut tx = self.pool.begin().await?;
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
        tx.commit().await?;
        Ok(courses_map.into_values().collect())
    }

    async fn get_course_with_videos(
        &self,
        course_id: Uuid,
        user_id: Option<Uuid>,
    ) -> Result<Option<CourseWithModulesDto>, Error> {
        let mut tx = self.pool.begin().await?;

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
        .await?;

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

        tx.commit().await?;
        Ok(course_opt)
    }

    async fn get_course_with_videos_preview(
        &self,
        course_id: Uuid,
        user_id: Option<Uuid>,
    ) -> Result<Option<CourseWithModulesDto>, sqlx::Error> {
        let mut tx = self.pool.begin().await?;

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
        .await?;

        if rows.is_empty() {
            tx.commit().await?;
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

        tx.commit().await?;
        Ok(course_opt)
    }


    async fn update_course(
        &self,
        course_id: Uuid,
        mut dto: UpdateCourseDTO,
    ) -> Result<CourseWithModulesDto, Error> {
        let mut tx = self.pool.begin().await?;
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

        tx.commit().await?;
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
        let mut tx = self.pool.begin().await?;
        sqlx::query("DELETE FROM courses WHERE id = $1")
            .bind(course_id)
            .execute(&mut *tx)
            .await.map_err(|e| {
            log::error!("ERROR: {}", e);
            e
        })?
        ;
        tx.commit().await?;
        Ok(())
    }

    async fn get_course_count(&self) -> Result<i64, Error> {
        let mut tx = self.pool.begin().await?;
        let result = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM courses")
            .fetch_one(&mut *tx)
            .await.map_err(|e| {
            log::error!("ERROR: {}", e);
            e
        })?
        ;
        tx.commit().await?;
        Ok(result)
    }

    async fn create_lesson_comment(
        &self,
        lesson_id: Uuid,
        user_id: Uuid,
        comment: String,
    ) -> Result<CommentLessonDto, Error> {
        let mut tx = self.pool.begin().await?;
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
        tx.commit().await?;
        Ok(result)
    }

    async fn get_lesson_comments(&self, lesson_id: Uuid) -> Result<Vec<CommentLessonDto>, Error> {
        let mut tx = self.pool.begin().await?;
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
        tx.commit().await?;
        Ok(result)
    }

    async fn delete_lesson_comment(&self, comment_id: Uuid) -> Result<(), Error> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("DELETE FROM lesson_comments WHERE id = $1")
            .bind(comment_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                log::error!("ERROR: {}", e);
                e
            })?;
        tx.commit().await?;
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
        .await?;

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
        .await?;

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

#[allow(dead_code)]
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
    #[allow(dead_code)]
    async fn assign_achievement_to_user(
        &self,
        user_id: Uuid,
        achievement_id: Uuid,
    ) -> Result<UserAchievement, Error>;

    /// Marca un logro como ganado.
    #[allow(dead_code)]
    async fn earn_achievement(
        &self,
        user_id: Uuid,
        achievement_id: Uuid,
    ) -> Result<UserAchievement, Error>;

    /// Obtiene todos los logros de un usuario.
    async fn get_user_achievements(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<UserAchievementDto>, Error>;

    /// Verifica si un usuario ya ha ganado un logro específico.
    #[allow(dead_code)]
    async fn has_user_earned(
        &self,
        user_id: Uuid,
        achievement_id: Uuid,
    ) -> Result<bool, Error>;

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

#[async_trait]
pub trait NotificationExt {
    async fn get_user_notifications(&self, user_id: Uuid) -> Result<Vec<Notification>, Error>;
    async fn mark_notification_read(&self, notification_id: Uuid) -> Result<(), Error>;
    async fn create_notification(&self, user_id: Uuid, title: &str, message: &str, sent_via: &str) -> Result<Notification, Error>;
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
        let mut tx = self.pool.begin().await?;
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
        tx.commit().await?;
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
        let mut tx = self.pool.begin().await?;

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
        tx.commit().await?;
        Ok(achievement)
    }

    async fn get_achievements(
        &self,
        page: u32,
        limit: usize,
    ) -> Result<Vec<Achievement>, Error> {
        let mut tx = self.pool.begin().await?;
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
        tx.commit().await?;
        Ok(achievements)
    }

    async fn get_achievement(&self, achievement_id: Uuid)
        -> Result<Option<Achievement>, Error> {
        let mut tx = self.pool.begin().await?;
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
        tx.commit().await?;
        Ok(achievement)
    }

    async fn delete_achievement(&self, achievement_id: Uuid) -> Result<(), Error> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("DELETE FROM achievement WHERE id = $1")
            .bind(achievement_id)
            .execute(&mut *tx)
            .await.map_err(|e| {
            log::error!("ERROR: {}", e);
            e
        })?
        ;
        tx.commit().await?;
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
        let mut tx = self.pool.begin().await?;
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
        tx.commit().await?;
        Ok(user_achievement)
    }

    async fn earn_achievement(
        &self,
        user_id: Uuid,
        achievement_id: Uuid,
    ) -> Result<UserAchievement, Error> {
        let mut tx = self.pool.begin().await?;

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
        .await?;

        tx.commit().await?;
        Ok(user_achievement)
    }

    async fn get_user_achievements(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<UserAchievementDto>, Error> {
        let achievements = sqlx::query_as::<_, UserAchievementDto>(
            r#"
            SELECT
                a.id,
                a.name,
                a.description,
                a.icon,
                a.trigger_type,
                a.trigger_value,
                a.active,
                COALESCE(ua.earned, false) AS earned,
                ua.earned_at,
                a.created_at
            FROM achievement a
            LEFT JOIN user_achievement ua
                ON ua.achievement_id = a.id
                AND ua.user_id = $1
            WHERE a.active = true
            ORDER BY a.created_at ASC
            "#
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            log::error!("Error: {}", e);
            e
        })?;

        Ok(achievements)
    }

    async fn has_user_earned(
        &self,
        user_id: Uuid,
        achievement_id: Uuid,
    ) -> Result<bool, Error> {
        let mut tx = self.pool.begin().await?;
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM user_achievement WHERE user_id = $1 AND achievement_id = $2 AND earned = true)",
        )
        .bind(user_id)
        .bind(achievement_id)
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(exists)
    }

    async fn get_user_achievements_with_details(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<serde_json::Value>, Error> {
        let mut tx = self.pool.begin().await?;
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

        tx.commit().await?;
        Ok(result)
    }

    async fn check_and_award_achievements(
        &self,
        user_id: Uuid,
        action: &str,
        value: Option<i32>,
    ) -> Result<Vec<Achievement>, Error> {
        let mut tx = self.pool.begin().await?;
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

        tx.commit().await?;
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
    #[allow(dead_code)]
    async fn get_user_course_progress(
        &self,
        user_id: Uuid,
        course_id: Uuid,
    ) -> Result<Option<CourseProgress>, Error>;
    #[allow(dead_code)]
    async fn update_course_progress(
        &self,
        user_id: Uuid,
        course_id: Uuid,
        completed_lessons: i32,
        progress_percentage: f32,
    ) -> Result<(), Error>;

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
        let mut tx = self.pool.begin().await?;
        // Verificar que el curso existe
        let course_exists = query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM courses WHERE id = $1)",
            course_id
        )
        .fetch_one(&mut *tx)
        .await?;

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
        .await?;

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
            .await?;
            query!(
                r#"
                UPDATE courses
                SET students = students + 1
                WHERE id = $1
                "#,
                course_id
            )
            .execute(&mut *tx)
            .await?;
        }

        // Inicializar progreso del curso si no existe
        let progress_exists = query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM course_progress WHERE user_id = $1 AND course_id = $2)",
            user_id,
            course_id
        )
        .fetch_one(&mut *tx)
        .await?;

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
            .await?;

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
            .await?;
        }
        tx.commit().await?;

        // Verificar logros de cursos inscritos
        let _ = self.check_and_award_achievements(user_id, "courses_enrolled", None).await;

        Ok(())
    }

    async fn check_user_course_access(
        &self,
        user_id: Uuid,
        course_id: Uuid
    ) -> Result<Option<bool>, Error> {
        let mut tx = self.pool.begin().await?;
        // 1. Verificar si el usuario es admin
        let is_admin = query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM users WHERE id = $1 AND role = 'admin')",
            user_id
        )
        .fetch_one(&mut *tx)
        .await?;

        if is_admin.unwrap_or(false) {
            return Ok(Some(true));
        }

        // 2. Verificar si el usuario tiene una suscripción activa
        let has_active_subscription = query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM subscription WHERE user_id = $1 AND status = true AND end_time > NOW())",
            user_id
        )
        .fetch_one(&mut *tx)
        .await?;

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
        tx.commit().await?;
        return check
    }

    async fn get_user_purchased_courses(
        &self,
        user_id: Uuid
    ) -> Result<Vec<Uuid>, Error> {
        let mut tx = self.pool.begin().await?;
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
        tx.commit().await?;
        return purcha
    }

    async fn get_user_course_progress(
        &self,
        user_id: Uuid,
        course_id: Uuid
    ) -> Result<Option<CourseProgress>, Error> {
        let mut tx = self.pool.begin().await?;
        let progress = query_as::<_, CourseProgress>(
            r#"
            SELECT * FROM course_progress
            WHERE user_id = $1 AND course_id = $2
            "#
        )
        .bind(user_id)
        .bind(course_id)
        .fetch_optional(&mut *tx)
        .await.map_err(|e| {
            log::error!("ERROR: {}", e);
            e
        });
        tx.commit().await?;
        return progress
    }

    async fn update_course_progress(
        &self,
        user_id: Uuid,
        course_id: Uuid,
        completed_lessons: i32,
        progress_percentage: f32
    ) -> Result<(), Error> {
        let mut tx = self.pool.begin().await?;
        let _ = query!(
            r#"
            UPDATE course_progress
            SET completed_lessons = $1,
                progress_percentage = $2,
                last_accessed = NOW(),
                updated_at = NOW()
            WHERE user_id = $3 AND course_id = $4
            "#,
            completed_lessons,
            progress_percentage,
            user_id,
            course_id
        )
        .execute(&mut *tx)
        .await.map_err(|e|
            {
                log::error!("Error: {}", e);
                e
            }
        );
        tx.commit().await?;
        Ok(())
    }

    async fn update_lesson_progress(
        &self,
        user_id: Uuid,
        lesson_id: Uuid,
        is_completed: bool,
        progress: Option<f64>,
    ) -> Result<(), Error> {
        let mut tx = self.pool.begin().await?;

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
        .await?;

        let course_id = sqlx::query_scalar!(
            r#"
            SELECT course_id FROM modules WHERE id = $1
            "#,
            module_id
        )
        .fetch_one(&mut *tx)
        .await?;

        let total_lessons = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) FROM lessons WHERE module_id IN (SELECT id FROM modules WHERE course_id = $1)
            "#,
            course_id
        )
        .fetch_one(&mut *tx)
        .await?;

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
        .await?;

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
        .await?;
        tx.commit().await?;
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
    ) -> Result<SubscriptionPlan, Error> {
        let mut tx = self.pool.begin().await?;
        let id = Uuid::new_v4();
        let now = Utc::now();

        let plan = sqlx::query_as::<_, SubscriptionPlan>(
            r#"
            INSERT INTO subscription_plans (id, name, description, price, duration_months, features, paypal_plan_id, active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, true, $8, $9)
            RETURNING id, name, description, price, duration_months, features, paypal_plan_id, active, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(description)
        .bind(price)
        .bind(duration_months)
        .bind(features)
        .bind(paypal_plan_id)
        .bind(now)
        .bind(now)
        .fetch_one(&mut *tx)
        .await.map_err(|e| {
            log::error!("ERROR: {}", e);
            e
        })?;
        tx.commit().await?;
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
        active: Option<bool>,
    ) -> Result<SubscriptionPlan, Error> {
        let mut tx = self.pool.begin().await?;
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
                active = COALESCE($8, active),
                updated_at = $9
            WHERE id = $1
            RETURNING id, name, description, price, duration_months, features, paypal_plan_id, active, created_at, updated_at
            "#,
        )
        .bind(plan_id)
        .bind(name)
        .bind(description)
        .bind(price)
        .bind(duration_months)
        .bind(features)
        .bind(paypal_plan_id)
        .bind(active)
        .bind(now)
        .fetch_one(&mut *tx)
        .await.map_err(|e| {
            log::error!("ERROR: {}", e);
            e
        })?;
        tx.commit().await?;
        Ok(plan)
    }

    async fn delete_subscription_plan(&self, plan_id: Uuid) -> Result<(), Error> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("DELETE FROM subscription_plans WHERE id = $1")
            .bind(plan_id)
            .execute(&mut *tx)
            .await.map_err(|e| {
                log::error!("ERROR: {}", e);
                e
            })?;
        tx.commit().await?;
        Ok(())
    }

    async fn get_subscription_plans(&self) -> Result<Vec<SubscriptionPlan>, Error> {
        let mut tx = self.pool.begin().await?;
        let plans = sqlx::query_as::<_, SubscriptionPlan>(
            r#"
            SELECT id, name, description, price, duration_months, features, paypal_plan_id, active, created_at, updated_at
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
        tx.commit().await?;
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
        let mut tx = self.pool.begin().await?;
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
                created_at, 
                updated_at
            )
            VALUES ($1, $2, $3, true, $4, $5, NULL, $6, $7)
            RETURNING id, user_id, paypal_subscription_id, status, plan_id, start_time, end_time, created_at, updated_at
            "#,
        )
        .bind(id)        // $1
        .bind(user_id)   // $2
        .bind(paypal_id) // $3 (Ya no es '')
        .bind(plan_id)   // $4
        .bind(now)       // $5 (start_time)
        .bind(now)       // $6 (created_at)
        .bind(now)       // $7 (updated_at)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            log::error!("ERROR EN INSERT: {}", e);
            e
        })?;

        tx.commit().await?;
        Ok(subscription)
    }

    async fn get_user_subscriptions(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<Subscription>, Error> {
        let mut tx = self.pool.begin().await?;
        let subscriptions = sqlx::query_as::<_, Subscription>(
            r#"
            SELECT id, user_id, paypal_subscription_id, status, plan_id, start_time, end_time, created_at, updated_at
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
        tx.commit().await?;
        Ok(subscriptions)
    }

    async fn cancel_subscription(
        &self,
        subscription_id: Uuid,
    ) -> Result<(), Error> {
        let mut tx = self.pool.begin().await?;
        let now = Utc::now();

        sqlx::query(
            r#"
            UPDATE subscription
            SET status = false, updated_at = $2
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
        tx.commit().await?;
        Ok(())
    }

    async fn update_subscription_end_time(
        &self,
        paypal_subscription_id: &str,
    ) -> Result<(), Error> {
        let mut tx = self.pool.begin().await?;
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
            // Obtener duration_months del plan
            let duration_months: Option<i32> = sqlx::query_scalar(
                r#"
                SELECT duration_months FROM subscription_plans WHERE id = $1
                "#,
            )
            .bind(Uuid::parse_str(&plan_id).map_err(|_| Error::RowNotFound)?)
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

        tx.commit().await?;
        Ok(())
    }

    async fn update_subscription_status(
        &self,
        paypal_subscription_id: &str,
        status: bool,
    ) -> Result<(), Error> {
        let mut tx = self.pool.begin().await?;
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
        tx.commit().await?;
        Ok(())
    }

    async fn expire_subscription(
        &self,
        paypal_subscription_id: &str,
    ) -> Result<(), Error> {
        let mut tx = self.pool.begin().await?;
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
        tx.commit().await?;
        Ok(())
    }

    async fn check_user_has_active_subscription(
        &self,
        user_id: Uuid,
    ) -> Result<bool, Error> {
        let mut tx = self.pool.begin().await?;
        let has_active = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM subscription 
                WHERE user_id = $1 AND end_time > NOW()
            )
            "#,
        )
        .bind(user_id)
        .fetch_one(&mut *tx)
        .await.map_err(|e| {
            log::error!("ERROR: {}", e);
            e
        })?;
        tx.commit().await?;
        Ok(has_active)
    }
}

#[async_trait]
pub trait PasswordResetTokenExt {
    async fn create_password_reset_token(
        &self,
        user_id: Uuid,
        token_hash: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<PasswordResetToken, Error>;

    async fn get_password_reset_token(
        &self,
        token_hash: &str,
    ) -> Result<Option<PasswordResetToken>, Error>;

    async fn mark_token_used(
        &self,
        token_hash: &str,
    ) -> Result<(), Error>;

    async fn invalidate_user_tokens(
        &self,
        user_id: Uuid,
    ) -> Result<(), Error>;
}

#[async_trait]
impl PasswordResetTokenExt for DBClient {
    async fn create_password_reset_token(
        &self,
        user_id: Uuid,
        token_hash: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<PasswordResetToken, Error> {
        let mut tx = self.pool.begin().await?;
        let id = Uuid::new_v4();

        // Obtener la versión más alta para este usuario
        let max_version = sqlx::query_scalar!(
            "SELECT COALESCE(MAX(version), 0) FROM password_reset_tokens WHERE user_id = $1",
            user_id
        )
        .fetch_one(&mut *tx)
        .await?
        .unwrap_or(0);

        let new_version = max_version + 1;

        let token = sqlx::query_as::<_, PasswordResetToken>(
            r#"
            INSERT INTO password_reset_tokens (id, user_id, token_hash, version, expires_at, used, created_at)
            VALUES ($1, $2, $3, $4, $5, false, $6)
            RETURNING id, user_id, token_hash, version, expires_at, used, created_at
            "#,
        )
        .bind(id)
        .bind(user_id)
        .bind(token_hash)
        .bind(new_version)
        .bind(expires_at)
        .bind(Utc::now())
        .fetch_one(&mut *tx)
        .await.map_err(|e| {
            log::error!("ERROR: {}", e);
            e
        })?;
        tx.commit().await?;
        Ok(token)
    }

    async fn get_password_reset_token(
        &self,
        token_hash: &str,
    ) -> Result<Option<PasswordResetToken>, Error> {
        let mut tx = self.pool.begin().await?;
        let token = sqlx::query_as::<_, PasswordResetToken>(
            r#"
            SELECT id, user_id, token_hash, version, expires_at, used, created_at
            FROM password_reset_tokens
            WHERE token_hash = $1 AND used = false AND expires_at > $2
            "#,
        )
        .bind(token_hash)
        .bind(Utc::now())
        .fetch_optional(&mut *tx)
        .await.map_err(|e| {
            log::error!("ERROR: {}", e);
            e
        })?;
        tx.commit().await?;
        Ok(token)
    }

    async fn mark_token_used(
        &self,
        token_hash: &str,
    ) -> Result<(), Error> {
        let mut tx = self.pool.begin().await?;
        sqlx::query!(
            "UPDATE password_reset_tokens SET used = true WHERE token_hash = $1",
            token_hash
        )
        .execute(&mut *tx)
        .await.map_err(|e| {
            log::error!("ERROR: {}", e);
            e
        })?;
        tx.commit().await?;
        Ok(())
    }

    async fn invalidate_user_tokens(
        &self,
        user_id: Uuid,
    ) -> Result<(), Error> {
        let mut tx = self.pool.begin().await?;
        sqlx::query!(
            "UPDATE password_reset_tokens SET used = true WHERE user_id = $1",
            user_id
        )
        .execute(&mut *tx)
        .await.map_err(|e| {
            log::error!("ERROR: {}", e);
            e
        })?;
        tx.commit().await?;
        Ok(())
    }
}

#[async_trait]
impl NotificationExt for DBClient {
    async fn get_user_notifications(&self, user_id: Uuid) -> Result<Vec<Notification>, Error> {
        let mut tx = self.pool.begin().await?;
        let notifications = sqlx::query_as::<_, Notification>(
            r#"
            SELECT id, user_id, title, message, sent_via, sent_at, read
            FROM notification
            WHERE user_id = $1
            ORDER BY sent_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&mut *tx)
        .await.map_err(|e| {
            log::error!("ERROR: {}", e);
            e
        })?;
        tx.commit().await?;
        Ok(notifications)
    }

    async fn mark_notification_read(&self, notification_id: Uuid) -> Result<(), Error> {
        let mut tx = self.pool.begin().await?;
        sqlx::query!(
            "UPDATE notification SET read = true WHERE id = $1",
            notification_id
        )
        .execute(&mut *tx)
        .await.map_err(|e| {
            log::error!("ERROR: {}", e);
            e
        })?;
        tx.commit().await?;
        Ok(())
    }

    async fn create_notification(&self, user_id: Uuid, title: &str, message: &str, sent_via: &str) -> Result<Notification, Error> {
        let mut tx = self.pool.begin().await?;
        let id = Uuid::new_v4();
        let now = Utc::now();

        let notification = sqlx::query_as::<_, Notification>(
            r#"
            INSERT INTO notification (id, user_id, title, message, sent_via, sent_at, read)
            VALUES ($1, $2, $3, $4, $5, $6, false)
            RETURNING id, user_id, title, message, sent_via, sent_at, read
            "#,
        )
        .bind(id)
        .bind(user_id)
        .bind(title)
        .bind(message)
        .bind(sent_via)
        .bind(now)
        .fetch_one(&mut *tx)
        .await.map_err(|e| {
            log::error!("ERROR: {}", e);
            e
        })?;
        tx.commit().await?;
        Ok(notification)
    }
}