use std::collections::HashMap;

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use sqlx::{Pool, Postgres, query_scalar, query_as, query, Error};
use uuid::Uuid;

use crate::{config::dtos::{CourseWithModulesDto, CreateCourseDTO, CreateLessonDTO, CreateModuleDTO, LessonDto, ModuleWithLessonsDto, UpdateCourseDTO},  models::models::{Achievement, Course, CourseProgress, Lesson, Module, Payment, User, UserAchievement, UserCourse, UserRole}};

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
        ).fetch_optional(&self.pool).await.map_err(|e| {
            eprintln!("ERROR: {}", e);
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
        ).fetch_optional(&self.pool).await.map_err(|e| {
            eprintln!("ERROR: {}", e);
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
        ).fetch_optional(&self.pool).await.map_err(|e| {
            eprintln!("ERROR: {}", e);
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
        ).fetch_optional(&self.pool).await.map_err(|e| {
            eprintln!("ERROR: {}", e);
            e
        })?
        ;
    }

    Ok(user)
}

    async fn get_users(
        &self,
        page: u32,
        limit: usize,
    ) -> Result<Vec<User>, Error> {
        let offset = (page - 1) * limit as u32;

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
        ).fetch_all(&self.pool)
        .await.map_err(|e| {
            eprintln!("ERROR: {}", e);
            e
        })?
        ;

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
        .fetch_one(&self.pool)
        .await.map_err(|e| {
            eprintln!("ERROR: {}", e);
            e
        })?
        ;
        Ok(user)
    }

    async fn get_user_count(&self) -> Result<i64, Error> {
        let count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM users"#
        )
       .fetch_one(&self.pool)
       .await.map_err(|e| {
            eprintln!("ERROR: {}", e);
            e
        })?
        ;

        Ok(count.unwrap_or(0))
    }

    async fn update_user_name<T: Into<String> + Send>(
        &self,
        user_id: Uuid,
        new_name: T
    ) -> Result<User, Error> {
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
        ).fetch_one(&self.pool)
        .await.map_err(|e| {
            eprintln!("ERROR: {}", e);
            e
        })?
        ;

        Ok(user)
    }

    async fn update_user_role(
        &self,
        user_id: Uuid,
        new_role: UserRole
    ) -> Result<User, Error> {
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
        ).fetch_one(&self.pool)
       .await.map_err(|e| {
            eprintln!("ERROR: {}", e);
            e
        })?
        ;

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
        .fetch_one(&self.pool)
        .await.map_err(|e| {
            eprintln!("ERROR: {}", e);
            e
        })?
        ;

        Ok(user)
    }

    async fn update_user_password(
        &self,
        user_id: Uuid,
        new_password: String
    ) -> Result<User, Error> {
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
        ).fetch_one(&self.pool)
        .await.map_err(|e| {
            eprintln!("ERROR: {}", e);
            e
        })?
        ;

        Ok(user)
    }

    async fn verifed_token(
        &self,
        token: &str,
    ) -> Result<(), Error> {
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
    ) -> Result<(), Error> {
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
       .await.map_err(|e| {
            eprintln!("ERROR: {}", e);
            e
        })?
        ;

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

    async fn get_user_courses(&self, user_id: Uuid) -> Result<Vec<Course>, Error>;

    async fn get_courses(
        &self,
        page: u32,
        limit: usize,
    ) -> Result<Vec<Course>, Error>;

    async fn get_all_courses_with_modules(
        &self,
    ) -> Result<Vec<CourseWithModulesDto>, Error> ;

    async fn get_course_with_videos(
    &self,
    course_id: Uuid,
) -> Result<Option<CourseWithModulesDto>, Error>;

    async fn update_course(
        &self,
        course_id: Uuid,
        dto: UpdateCourseDTO,
    ) -> Result<CourseWithModulesDto, Error>;

    async fn delete_course(&self, course_id: Uuid) -> Result<(), Error>;

    #[allow(dead_code)]
    async fn get_course_count(&self) -> Result<i64, Error>;
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
                (id, title, description, long_description, level, price, duration, students, rating, image, category, features, paypal_product_id, created_at, updated_at)
            VALUES
                ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
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
        .bind(dto.rating.unwrap_or(5.0))
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
            let module_id = Uuid::new_v4();
            // Forzamos el orden basado en el índice para evitar error de UNIQUE constraint
            let module_order = (module_idx + 1) as i32; 

            let module_insert = sqlx::query_as::<_, Module>(
                r#"
                INSERT INTO modules (id, course_id, title, "order")
                VALUES ($1, $2, $3, $4)
                RETURNING *
                "#
            )
            .bind(module_id)
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
                .bind(module_id)
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
            rating: Some(course.rating),
            image: course.image,
            category: course.category,
            features: course.features.and_then(|f| serde_json::from_value(f).ok()),
            paypal_product_id: None,
            modules: modules_dtos,
        })
    }

    async fn get_course(&self, course_id: Uuid) -> Result<Option<Course>, Error> {
        let course = sqlx::query_as::<_, Course>(
            r#"SELECT * FROM courses WHERE id = $1"#,
        )
        .bind(course_id)
        .fetch_optional(&self.pool)
        .await.map_err(|e| {
            eprintln!("ERROR: {}", e);
            e
        })?
        ;
        
        Ok(course)
    }

    async fn get_user_courses(&self, user_id: Uuid) -> Result<Vec<Course>, Error> {
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
        .await.map_err(|e| {
            eprintln!("ERROR: {}", e);
            e
        })?
        ;
        
        Ok(courses)
    }

    async fn get_courses(
        &self,
        page: u32,
        limit: usize,
    ) -> Result<Vec<Course>, Error> {
        let offset = ((page - 1) * limit as u32) as i64;
        let courses = sqlx::query_as::<_, Course>(
            r#"SELECT * FROM courses 
               ORDER BY created_at DESC LIMIT $1 OFFSET $2"#,
        )
        .bind(limit as i64)
        .bind(offset)
        .fetch_all(&self.pool)
        .await.map_err(|e| {
            eprintln!("ERROR: {}", e);
            e
        })?;
        Ok(courses)
    }

    /// Mucho más eficiente: 3 queries en vez de un JOIN enorme.
    async fn get_all_courses_with_modules(
        &self,
    ) -> Result<Vec<CourseWithModulesDto>, Error> {
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
                c.rating,
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
        .fetch_all(&self.pool)
        .await.map_err(|e| {
            eprintln!("ERROR: {}", e);
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
                    rating: row.rating,
                    image: row.image.clone(),
                    category: row.category.unwrap(),
                    features: row.features
                        .as_ref()
                        .and_then(|v| serde_json::from_value(v.clone()).ok()),
                    created_at: row.created_at.unwrap(),
                    updated_at: row.updated_at.unwrap(),
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

        Ok(courses_map.into_values().collect())
    }

    async fn get_course_with_videos(
        &self,
        course_id: Uuid,
    ) -> Result<Option<CourseWithModulesDto>, Error> {

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
                c.rating,
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
            WHERE c.id = $1
            ORDER BY m."order" ASC, l."order" ASC
            "#,
            course_id
        )
        .fetch_all(&self.pool)
        .await?;

        if rows.is_empty() {
            return Ok(None);
        }

        // -------------- AGRUPACIÓN ---------------
        let mut course_opt: Option<CourseWithModulesDto> = None;

        for row in rows {
            // 1️⃣ Crear el curso si aún no existe
            let course = course_opt.get_or_insert_with(|| CourseWithModulesDto {
                id: row.course_id,
                title: row.course_title.clone(),
                description: row.description.clone(),
                long_description: row.long_description.clone(),
                price: row.price,
                level: row.level.clone().unwrap(),
                duration: row.duration,
                students: row.students.unwrap_or(0),
                rating: row.rating,
                image: row.image.clone(),
                category: row.category.clone().unwrap(),
                features: row.features
                    .as_ref()
                    .and_then(|v| serde_json::from_value(v.clone()).ok()),
                created_at: row.created_at.unwrap(),
                updated_at: row.updated_at.unwrap(),
                modules: vec![],
            });

            // 2️⃣ Agregar módulo si existe
            if let Some(module_id) = row.module_id {
                // Buscar módulo existente
                let module = course.modules
                    .iter_mut()
                    .find(|m| m.id == module_id);

                let module_ref = match module {
                    Some(m) => m,
                    None => {
                        course.modules.push(ModuleWithLessonsDto {
                            id: module_id,
                            title: row.module_title.clone().unwrap_or("Title".to_string()),
                            order: row.module_order.unwrap_or(1),
                            lessons: vec![],
                        });

                        course.modules.last_mut().unwrap()
                    }
                };

                // 3️⃣ Agregar lección si existe
                if let Some(lesson_id) = row.lesson_id {
                    module_ref.lessons.push(LessonDto {
                        id: lesson_id,
                        title: row.lesson_title.clone().unwrap_or("Title".to_string()),
                        duration: row.lesson_duration,
                        completed: None,
                        r#type: row.lesson_type.clone().unwrap_or("video".to_string()),
                        content_url: row.content_url.clone(),
                        description: row.lesson_description.clone(),
                        order: row.lesson_order.unwrap_or(1),
                    });
                }
            }
        }

        Ok(course_opt)
    }

    async fn update_course(
        &self,
        course_id: Uuid,
        dto: UpdateCourseDTO, // O el DTO que uses
    ) -> Result<CourseWithModulesDto, Error> {
        let now = Utc::now();

        // Serializamos módulos y lecciones a JSON y los mandamos como 2 arrays.
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
                                    "module_id": m.id,
                                    "title": l.title.clone(),        // clonar String
                                    "duration": l.duration.clone(),
                                    "type": l.r#type.clone(),
                                    "content_url": l.content_url.clone(),
                                    "description": l.description.clone(),
                                    "order": l.order
                                })
                            }).collect::<Vec<_>>())
                            .unwrap_or_default() // esto devuelve un Vec<_> vacío
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
                rating = COALESCE($9, rating),
                image = COALESCE($10, image),
                category = COALESCE($11, category),
                features = COALESCE($12::jsonb, features),
                updated_at = $13
            WHERE id = $1
            RETURNING *
        ),
        module_input AS (
            SELECT
                (m->>'id')::uuid AS id,
                (m->>'title') AS title,
                (m->>'order')::int AS module_order,
                $1 AS course_id
            FROM jsonb_array_elements($14::jsonb) AS m
        ),
        module_upsert AS (
            INSERT INTO modules (id, course_id, title, "order")
            SELECT COALESCE(id, gen_random_uuid()), course_id, title, module_order FROM module_input
            ON CONFLICT (id) DO UPDATE SET
                title = EXCLUDED.title,
                "order" = EXCLUDED."order"
            RETURNING id
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
            FROM jsonb_array_elements($15::jsonb) AS l
        ),
        lesson_upsert AS (
            INSERT INTO lessons (id, module_id, title, duration, "type", content_url, description, "order")
            SELECT COALESCE(id, gen_random_uuid()), module_id, title, duration, type, content_url, description, lesson_order
            FROM lesson_input
            ON CONFLICT (id) DO UPDATE SET
                title = EXCLUDED.title,
                duration = EXCLUDED.duration,
                "type" = EXCLUDED."type",
                content_url = EXCLUDED.content_url,
                description = EXCLUDED.description,
                "order" = EXCLUDED."order"
            RETURNING id
        ),
        lesson_deleted AS (
            DELETE FROM lessons
            WHERE module_id IN (SELECT id FROM module_upsert)
            AND id NOT IN (SELECT id FROM lesson_input)
            RETURNING id
        )
        SELECT * FROM course_update;
        "#;

        let _ = sqlx::query(
            sql // el SQL de arriba
        )
        .bind(course_id)
        .bind(dto.title)
        .bind(dto.description)
        .bind(dto.long_description)
        .bind(dto.level)
        .bind(dto.price)
        .bind(dto.duration)
        .bind(dto.students)
        .bind(dto.rating)
        .bind(dto.image)
        .bind(dto.category)
        .bind(dto.features.map(|f| serde_json::to_value(f).unwrap()))
        .bind(now)
        .bind(modules_json) // $14
        .bind(lessons_json) // $15
        .execute(&self.pool)
        .await.map_err(|e|{
            eprintln!("ERROR: {}", e);
            e
        });

        Ok(
            self.get_all_courses_with_modules()
            .await.map_err(|e| { eprintln!("ERROR: {}", e); e })? 
            .into_iter() 
            .find(|c| c.id == course_id) 
            .expect("Curso debería existir después de la actualización")
        )
    }


    async fn delete_course(&self, course_id: Uuid) -> Result<(), Error> {
        sqlx::query("DELETE FROM courses WHERE id = $1")
            .bind(course_id)
            .execute(&self.pool)
            .await.map_err(|e| {
            eprintln!("ERROR: {}", e);
            e
        })?
        ;
        
        Ok(())
    }

    async fn get_course_count(&self) -> Result<i64, Error> {
        let result = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM courses")
            .fetch_one(&self.pool)
            .await.map_err(|e| {
            eprintln!("ERROR: {}", e);
            e
        })?
        ;
        
        Ok(result)
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
    ) -> Result<Vec<Achievement>, Error>;

    /// Verifica si un usuario ya ha ganado un logro específico.
    #[allow(dead_code)]
    async fn has_user_earned(
        &self,
        user_id: Uuid,
        achievement_id: Uuid,
    ) -> Result<bool, Error>;
}

/// Implementación para la conexión principal del sistema (`DBClient`).
#[async_trait]
impl AchievementExt for DBClient {
    async fn create_achievement<T: Into<String> + Send>(
        &self,
        name: T,
        description: Option<T>,
        icon: Option<T>,
    ) -> Result<Achievement, Error> {
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
        .await.map_err(|e| {
            eprintln!("ERROR: {}", e);
            e
        })?
        ;

        Ok(achievement)
    }

    async fn get_achievements(
        &self,
        page: u32,
        limit: usize,
    ) -> Result<Vec<Achievement>, Error> {
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
        .await.map_err(|e| {
            eprintln!("ERROR: {}", e);
            e
        })?
        ;

        Ok(achievements)
    }

    async fn get_achievement(&self, achievement_id: Uuid)
        -> Result<Option<Achievement>, Error> {
        let achievement = sqlx::query_as::<_, Achievement>(
            r#"
            SELECT id, name, description, icon, created_at
            FROM achievements
            WHERE id = $1
            "#
        )
        .bind(achievement_id)
        .fetch_optional(&self.pool)
        .await.map_err(|e| {
            eprintln!("ERROR: {}", e);
            e
        })?
        ;

        Ok(achievement)
    }

    async fn delete_achievement(&self, achievement_id: Uuid) -> Result<(), Error> {
        sqlx::query("DELETE FROM achievements WHERE id = $1")
            .bind(achievement_id)
            .execute(&self.pool)
            .await.map_err(|e| {
            eprintln!("ERROR: {}", e);
            e
        })?
        ;
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
        .await.map_err(|e| {
            eprintln!("ERROR: {}", e);
            e
        })?
        ;

        Ok(user_achievement)
    }

    async fn earn_achievement(
        &self,
        user_id: Uuid,
        achievement_id: Uuid,
    ) -> Result<UserAchievement, Error> {
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
        .await.map_err(|e| {
            eprintln!("ERROR: {}", e);
            e
        })?
        ;

        Ok(user_achievement)
    }

    async fn get_user_achievements(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<Achievement>, Error> {
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
        .await.map_err(|e| {
            eprintln!("ERROR: {}", e);
            e
        })?
        ;

        Ok(achievements)
    }

    async fn has_user_earned(
        &self,
        user_id: Uuid,
        achievement_id: Uuid,
    ) -> Result<bool, Error> {
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
        .await.map_err(|e| {
            eprintln!("ERROR: {}", e);
            e
        })?
        ;

        Ok(exists)
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
        // Verificar que el curso existe
        let course_exists = query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM courses WHERE id = $1)",
            course_id
        )
        .fetch_one(&self.pool)
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
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            eprintln!("ERROR: {}", e);
            e
        })?;

        // Registrar en user_courses si no existe
        let user_course_exists = query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM user_courses WHERE user_id = $1 AND course_id = $2)",
            user_id,
            course_id
        )
        .fetch_one(&self.pool)
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
            .execute(&self.pool)
            .await?;
            query!(
                r#"
                UPDATE courses
                SET students = students + 1
                WHERE id = $1
                "#,
                course_id
            )
            .execute(&self.pool)
            .await?;
        }

        // Inicializar progreso del curso si no existe
        let progress_exists = query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM course_progress WHERE user_id = $1 AND course_id = $2)",
            user_id,
            course_id
        )
        .fetch_one(&self.pool)
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
            .fetch_one(&self.pool)
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
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    async fn check_user_course_access(
        &self,
        user_id: Uuid,
        course_id: Uuid
    ) -> Result<Option<bool>, Error> {
        // 1. Verificar si el usuario es admin
        let is_admin = query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM users WHERE id = $1 AND role = 'admin')",
            user_id
        )
        .fetch_one(&self.pool)
        .await?;

        if is_admin.unwrap_or(false) {
            return Ok(Some(true));
        }

        // 2. Verificar si el usuario tiene una suscripción activa
        let has_active_subscription = query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM subscription WHERE user_id = $1 AND status = true AND end_time > NOW())",
            user_id
        )
        .fetch_one(&self.pool)
        .await?;

        if has_active_subscription.unwrap_or(false) {
            return Ok(Some(true));
        }

        // 3. Verificar si el usuario ha comprado este curso específico
        query_scalar!(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM user_courses
                WHERE user_id = $1 AND course_id = $2
            )
            "#,
            user_id,
            course_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            eprint!("Error: {}", e);
            e
        })
    }

    async fn get_user_purchased_courses(
        &self,
        user_id: Uuid
    ) -> Result<Vec<Uuid>, Error> {
        query_as::<_, UserCourse>(
            r#"
            SELECT id, user_id, course_id, purchased_at, created_at, updated_at
            FROM user_courses
            WHERE user_id = $1
            "#
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            eprint!("Error: {}", e);
            e
        })
        .map(|user_courses| {
            user_courses.into_iter().map(|uc| uc.course_id).collect()
        })
    }

    async fn get_user_course_progress(
        &self,
        user_id: Uuid,
        course_id: Uuid
    ) -> Result<Option<CourseProgress>, Error> {
        query_as::<_, CourseProgress>(
            r#"
            SELECT * FROM course_progress
            WHERE user_id = $1 AND course_id = $2
            "#
        )
        .bind(user_id)
        .bind(course_id)
        .fetch_optional(&self.pool)
        .await.map_err(|e| {
            eprintln!("ERROR: {}", e);
            e
        })
    }

    async fn update_course_progress(
        &self,
        user_id: Uuid,
        course_id: Uuid,
        completed_lessons: i32,
        progress_percentage: f32
    ) -> Result<(), Error> {
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
        .execute(&self.pool)
        .await.map_err(|e|
            {
                eprint!("Error: {}", e);
                e
            }
        );

        Ok(())
    }

}