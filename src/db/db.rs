use std::collections::HashMap;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::{config::dtos::{CourseWithModulesDto, CreateCourseDTO, CreateLessonDTO, CreateModuleDTO, LessonDto, ModuleWithLessonsDto, UpdateCourseDTO},  models::models::{Achievement, Course, Lesson, Module, Payment, User, UserAchievement, UserRole}};

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
                profile_image_url,
                subscription_expires_at
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
                profile_image_url,
                subscription_expires_at
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
                profile_image_url,
                subscription_expires_at
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
                profile_image_url,
                subscription_expires_at
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
                profile_image_url,
                subscription_expires_at
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
                profile_image_url,
                subscription_expires_at
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
                profile_image_url,
                subscription_expires_at
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
                profile_image_url,
                subscription_expires_at
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

// ===================== //
//      COURSES EXT 
// ===================== //

#[async_trait]
pub trait CourseExt {
    async fn create_course(
        &self,
        dto: CreateCourseDTO,
    ) -> Result<CreateCourseDTO, sqlx::Error>;

    async fn get_course(&self, course_id: Uuid) -> Result<Option<Course>, sqlx::Error>;

    async fn get_user_courses(&self, user_id: Uuid) -> Result<Vec<Course>, sqlx::Error>;

    async fn get_courses(
        &self,
        page: u32,
        limit: usize,
    ) -> Result<Vec<Course>, sqlx::Error>;

    async fn get_all_courses_with_modules(
        &self,
    ) -> Result<Vec<CourseWithModulesDto>, sqlx::Error> ;

    async fn get_course_with_videos(
    &self,
    course_id: Uuid,
) -> Result<Option<CourseWithModulesDto>, sqlx::Error>;

    async fn update_course(
        &self,
        course_id: Uuid,
        dto: UpdateCourseDTO,
    ) -> Result<CourseWithModulesDto, sqlx::Error>;

    async fn delete_course(&self, course_id: Uuid) -> Result<(), sqlx::Error>;

    #[allow(dead_code)]
    async fn get_course_count(&self) -> Result<i64, sqlx::Error>;
}

// ===================== //
//   IMPLEMENTATION COURSES EXT
// ===================== //
#[async_trait]
impl CourseExt for DBClient {
    async fn create_course(
        &self,
        dto: CreateCourseDTO,
    ) -> Result<CreateCourseDTO, sqlx::Error> {
        println!("--> INICIO create_course: Recibido título: {}", dto.title);

        let course_id = Uuid::new_v4();
        let now = Utc::now();

        // 1. INICIAR TRANSACCIÓN
        let mut tx = match self.pool.begin().await {
            Ok(t) => t,
            Err(e) => {
                println!("ERROR: Falló al iniciar transacción: {:?}", e);
                return Err(e);
            }
        };

        println!("--> Transacción iniciada. Insertando curso...");

        // 2. INSERTAR CURSO
        // Nota: Manejo seguro de features
        let features_json = match &dto.features {
            Some(f) => serde_json::to_value(f).unwrap_or(serde_json::Value::Array(vec![])),
            None => serde_json::Value::Array(vec![]),
        };

        let course_insert_result = sqlx::query_as::<_, Course>(
            r#"
            INSERT INTO courses
                (id, title, description, long_description, level, price, duration, students, rating, image, category, features, created_at, updated_at)
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
        .bind(dto.rating.unwrap_or(5.0))
        .bind(&dto.image)
        .bind(&dto.category)
        .bind(features_json)
        .bind(now)
        .bind(now)
        .fetch_one(&mut *tx)
        .await;

        let course = match course_insert_result {
            Ok(c) => c,
            Err(e) => {
                println!("ERROR SQL al insertar CURSO: {:?}", e);
                // Hacemos rollback manual aunque sqlx lo suele hacer al caer el scope
                let _ = tx.rollback().await; 
                return Err(e);
            }
        };

        println!("--> Curso insertado. Procesando {} módulos...", dto.modules.len());

        // 3. INSERTAR MÓDULOS Y LECCIONES
        let mut modules_dtos: Vec<CreateModuleDTO> = Vec::new();

        for (module_idx, module_dto) in dto.modules.into_iter().enumerate() {
            let module_id = Uuid::new_v4();
            // Forzamos el orden basado en el índice para evitar error de UNIQUE constraint
            let module_order = (module_idx + 1) as i32; 

            println!("----> Insertando Módulo {}: {}", module_order, module_dto.title);

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
                    println!("ERROR SQL al insertar MÓDULO '{}': {:?}", module_dto.title, e);
                    let _ = tx.rollback().await;
                    return Err(e);
                }
            };

            let mut lessons_dtos: Vec<CreateLessonDTO> = Vec::new();

            for (lesson_idx, lesson) in module_dto.lessons.into_iter().enumerate() {
                // Forzamos el orden también aquí
                let lesson_order = (lesson_idx + 1) as i32;

                println!("------> Insertando Lección {}: {}", lesson_order, lesson.title);

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
                        println!("ERROR SQL al insertar LECCIÓN '{}': {:?}", lesson.title, e);
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
        println!("--> Confirmando transacción (COMMIT)...");
        if let Err(e) = tx.commit().await {
            println!("ERROR CRÍTICO: Falló el COMMIT: {:?}", e);
            return Err(e);
        }

        println!("--> ÉXITO: Curso creado correctamente.");

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
            modules: modules_dtos,
        })
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

    /// Mucho más eficiente: 3 queries en vez de un JOIN enorme.
    async fn get_all_courses_with_modules(
        &self,
    ) -> Result<Vec<CourseWithModulesDto>, sqlx::Error> {
        // 1️⃣ Traer cursos
        let courses = sqlx::query_as::<_, Course>(
            r#"
            SELECT id, title, description, long_description,
                level, price, duration, students, rating, image,
                category, features, created_at, updated_at
            FROM courses
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        // 2️⃣ Traer módulos
        let modules = sqlx::query!(
            r#"
            SELECT id, course_id, title, "order"
            FROM modules
            ORDER BY "order" ASC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        // 3️⃣ Traer lecciones
        let lessons = sqlx::query!(
            r#"
            SELECT id, module_id, title, duration, "type",
                content_url, description, "order"
            FROM lessons
            ORDER BY "order" ASC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        // -------------- AGRUPACIÓN EFICIENTE ---------------

        use std::collections::HashMap;

        // Agrupar lecciones por módulo
        let mut lessons_by_module: HashMap<Uuid, Vec<LessonDto>> = HashMap::new();
        for row in lessons {
            lessons_by_module
                .entry(row.module_id)
                .or_default()
                .push(LessonDto {
                    id: row.id,
                    title: row.title,
                    duration: row.duration,
                    completed: None, // No tenemos info de completado aquí
                    r#type: row.r#type,
                    content_url: row.content_url,
                    description: row.description,
                    order: row.order,
                });
        }

        // Agrupar módulos por curso
        let mut modules_by_course: HashMap<Uuid, Vec<ModuleWithLessonsDto>> = HashMap::new();
        for m in modules {
            modules_by_course
                .entry(m.course_id)
                .or_default()
                .push(ModuleWithLessonsDto {
                    id: m.id,
                    title: m.title,
                    order: m.order,
                    lessons: lessons_by_module.remove(&m.id).unwrap_or_default(),
                });
        }

        // Construir resultado final
        let mut result = Vec::new();
        for c in courses {
            result.push(CourseWithModulesDto {
                id: c.id,
                title: c.title,
                description: c.description,
                long_description: c.long_description,
                price: c.price,
                level: c.level,
                duration: c.duration,
                students: c.students,
                rating: c.rating,
                image: c.image,
                category: c.category,
                features: c.features
                        .as_ref()
                        .and_then(|v| serde_json::from_value(v.clone()).ok()),
                created_at: c.created_at,
                updated_at: c.updated_at,
                modules: modules_by_course.remove(&c.id).unwrap_or_default(),
            });
        }

        Ok(result)
    }

    async fn get_course_with_videos(
        &self,
        course_id: Uuid,
    ) -> Result<Option<CourseWithModulesDto>, sqlx::Error> {
        
        println!("DEBUG 1. Buscando curso con ID: {}", course_id);

        // 1. OBTENER CURSO BASE
        let course = sqlx::query_as::<_, Course>(
            r#"
            SELECT id, title, description, long_description,
                level, price, duration, students, rating, image,
                category, features, created_at, updated_at
            FROM courses
            WHERE id = $1
            "#,
        )
        .bind(course_id)
        .fetch_optional(&self.pool)
        .await?;

        let course = match course {
            Some(c) => c,
            None => {
                println!("DEBUG 1a. Curso no encontrado, retornando None.");
                return Ok(None)
            },
        };
        println!("DEBUG 1b. Curso base encontrado: {}", course.title);
        
        // 2. OBTENER MÓDULOS
        let db_modules: Vec<Module> = sqlx::query_as!(
            Module,
            r#"
            SELECT id, course_id, title, "order"
            FROM modules
            WHERE course_id = $1
            ORDER BY "order" ASC
            "#,
            course_id
        )
        .fetch_all(&self.pool)
        .await?;
        
        println!("DEBUG 2. Módulos encontrados: {}", db_modules.len());


        // 3. OBTENER TODAS LAS LECCIONES DE LOS MÓDULOS ENCONTRADOS
        let module_ids: Vec<Uuid> = db_modules.iter().map(|m| m.id).collect();
        println!("DEBUG 3a. IDs de módulos a buscar lecciones: {:?}", module_ids);
        
        let db_lessons: Vec<Lesson> = if module_ids.is_empty() {
            println!("DEBUG 3b. No hay módulos, saltando la búsqueda de lecciones.");
            vec![]
        } else {
            println!("DEBUG 3c. Ejecutando consulta de lecciones con {} módulos.", module_ids.len());
            
            // 🚨 VERIFICACIÓN CRÍTICA: La consulta asume que Lesson tiene #[sqlx(rename = "type")]
            let lessons = sqlx::query_as::<_, Lesson>(
                r#"
                SELECT id, module_id, title, duration, "type",  
                    content_url, description, "order", completed
                FROM lessons
                WHERE module_id = ANY($1) 
                ORDER BY "order" ASC
                "#
            )
            .bind(&module_ids as &[Uuid]) // Usamos el bind correcto para &[Uuid]
            .fetch_all(&self.pool)
            .await?; // ⬅️ Si falla, el pánico ocurre aquí.

            println!("DEBUG 3d. Lecciones retornadas por DB: {}", lessons.len());
            lessons
        };

        // 4. AGRUPAR LECCIONES EN MÓDULOS (Lógica de agrupamiento limpia)
        println!("DEBUG 4. Iniciando agrupamiento de lecciones...");
        let mut lessons_by_module: HashMap<Uuid, Vec<LessonDto>> = HashMap::new();
        let mut total_lessons = 0;

        for (index, lesson) in db_lessons.into_iter().enumerate() {
            println!("DEBUG 4a. Procesando lección #{}: ID={}, Módulo={}", index + 1, lesson.id, lesson.module_id);
            
            total_lessons += 1;
            
            // Asumiendo que Lesson::r#type existe (ya sea por FromRow o #[sqlx(rename)])
            let lesson_dto = LessonDto {
                id: lesson.id,
                title: lesson.title,
                duration: lesson.duration,
                completed: None, // Seguro contra la falta de columna
                r#type: lesson.r#type,
                content_url: lesson.content_url,
                description: lesson.description,
                order: lesson.order,
            };
            lessons_by_module.entry(lesson.module_id).or_insert_with(Vec::new).push(lesson_dto);
        }
        
        println!("DEBUG 4b. Agrupamiento finalizado. Total de lecciones procesadas: {}", total_lessons);

        // 5. CONSTRUIR DTO DE MÓDULOS
        println!("DEBUG 5. Construyendo DTO de Módulos...");
        let modules: Vec<ModuleWithLessonsDto> = db_modules.into_iter().map(|m| {
            let lessons = lessons_by_module.remove(&m.id).unwrap_or_default();
            
            println!("DEBUG 5a. Módulo ID {} tiene {} lecciones.", m.id, lessons.len());

            ModuleWithLessonsDto {
                id: m.id,
                title: m.title,
                order: m.order,
                lessons, 
                // Añade otros campos si ModuleWithLessonsDto los tiene y los necesitas
            }
        }).collect();
        
        println!("DEBUG 5b. Módulos construidos: {}", modules.len());

        // 6. ARMAR RESPUESTA FINAL
        
        // Deserialización de features
        let features_result: Option<Vec<String>> = course.features.as_ref()
            .and_then(|v| {
                match serde_json::from_value(v.clone()) {
                    Ok(f) => {
                        println!("DEBUG 6a. Deserialización de features exitosa.");
                        Some(f)
                    },
                    Err(e) => {
                        eprintln!("ERROR: Falló la deserialización de features: {}", e);
                        None
                    }
                }
            });
        
        // Se asume la existencia de campos de resumen en CourseWithModulesDto
        let result = CourseWithModulesDto {
            id: course.id,
            title: course.title.clone(),
            description: course.description.clone(),
            long_description: course.long_description.clone(),
            price: course.price,
            level: course.level.clone(),
            duration: course.duration.clone(),
            students: course.students,
            rating: course.rating,
            image: course.image.clone(),
            category: course.category.clone(),
            features: features_result,
            created_at: course.created_at,
            updated_at: course.updated_at,
            modules,
        };
        
        println!("DEBUG 7. Construcción final del DTO exitosa.");
        
        Ok(Some(result))
    }

    async fn update_course(
        &self,
        course_id: Uuid,
        dto: UpdateCourseDTO, // O el DTO que uses
    ) -> Result<CourseWithModulesDto, sqlx::Error> {

        // 1. VERIFICACIÓN DE ENTRADA Y BINDING DEL ID
        // Este mensaje te dice qué ID estás intentando modificar.
        println!("DEBUG: Intentando actualizar el curso con ID: {}", course_id);
        // Puedes imprimir el DTO completo si es necesario, si usa #[derive(Debug)]
        // println!("DEBUG: DTO recibido para actualización: {:?}", dto);

        let now = Utc::now();
        let features_json = dto.features
            .as_ref() // Usamos .as_ref() ya que .features podría ser un Option
            .and_then(|f| serde_json::to_value(f).ok());
        
        // --- QUERY DE ACTUALIZACIÓN ---
        let result = sqlx::query_as::<_, Course>(
            r#"
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
                features = COALESCE($12::JSONB, features),
                updated_at = $13
            WHERE id = $1
            RETURNING *
            "#
        )
        .bind(course_id) // $1
        .bind(&dto.title) // $2
        .bind(&dto.description) // $3
        .bind(&dto.long_description) // $4
        .bind(&dto.level) // $5
        .bind(dto.price) // $6 (Asegúrate que el tipo es f64/f32 o BigDecimal)
        .bind(&dto.duration) // $7
        .bind(dto.students) // $8
        .bind(dto.rating) // $9 (Asegúrate que el tipo es f32/f64)
        .bind(&dto.image) // $10
        .bind(&dto.category) // $11
        .bind(features_json) // $12
        .bind(now) // $13
        .fetch_optional(&self.pool) // Usamos fetch_optional para manejar 0 filas
        .await;
        // --- FIN DEL QUERY DE ACTUALIZACIÓN ---
        // ==============================
        // 2. ACTUALIZAR / CREAR MÓDULOS Y LECCIONES
        // ==============================
        if let Some(modules_dto) = dto.modules {
            for (i, module_dto) in modules_dto.into_iter().enumerate() {
                let module_id = module_dto.id.unwrap_or_else(Uuid::new_v4);  // Si el módulo no tiene ID, generamos uno nuevo
                let order = module_dto.order.unwrap_or_else(|| (i + 1) as i32);  // Orden de módulo por defecto
                let title = module_dto.title.unwrap_or_else(|| "Módulo sin título".to_string());  // Título por defecto

                // Intentamos actualizar el módulo, si no existe lo insertamos
                let module = if let Some(existing_module) =
                    sqlx::query_as::<_, Module>(
                        r#"
                        UPDATE modules SET
                            title = COALESCE($2, title),
                            "order" = COALESCE($3, "order")
                        WHERE id = $1
                        RETURNING *
                        "#,
                    )
                    .bind(module_id)
                    .bind(&title)
                    .bind(order)
                    .fetch_optional(&self.pool)
                    .await?
                {
                    existing_module
                } else {
                    // Insert módulo nuevo si no existía
                    sqlx::query_as::<_, Module>(
                        r#"
                        INSERT INTO modules (id, course_id, title, "order")
                        VALUES ($1, $2, $3, $4)
                        RETURNING *
                        "#,
                    )
                    .bind(module_id)
                    .bind(course_id)
                    .bind(&title)
                    .bind(order)
                    .fetch_one(&self.pool)
                    .await?
                };

                // --- LECCIONES ---
                if let Some(lessons_dto) = module_dto.lessons {
                    for (j, lesson_dto) in lessons_dto.into_iter().enumerate() {
                        let lesson_id = lesson_dto.id.unwrap_or_else(Uuid::new_v4);  // Si la lección no tiene ID, generamos uno nuevo
                        let lesson_order = lesson_dto.order.unwrap_or_else(|| (j + 1) as i32);  // Orden por defecto

                        if lesson_dto.id.is_some() {
                            // UPDATE lección
                            sqlx::query(
                                r#"
                                UPDATE lessons SET
                                    title = COALESCE($2, title),
                                    duration = COALESCE($3, duration),
                                    "type" = COALESCE($4, "type"),
                                    content_url = COALESCE($5, content_url),
                                    description = COALESCE($6, description),
                                    "order" = COALESCE($7, "order")
                                WHERE id = $1
                                "#,
                            )
                            .bind(lesson_id)
                            .bind(lesson_dto.title.unwrap_or_else(|| "Título por defecto".to_string()))  // Título por defecto
                            .bind(lesson_dto.duration.unwrap_or_else(|| "Duración por defecto".to_string()))  // Duración por defecto
                            .bind(lesson_dto.r#type.unwrap_or_else(|| "Tipo por defecto".to_string()))  // Tipo por defecto
                            .bind(lesson_dto.content_url.unwrap_or_else(|| "https://example.com/default-url".to_string()))  // Default URL
                            .bind(lesson_dto.description.unwrap_or_else(|| "Descripción por defecto".to_string()))  // Descripción por defecto
                            .bind(lesson_order)
                            .execute(&self.pool)
                            .await?;
                        } else {
                            // INSERT lección nueva
                            sqlx::query(
                                r#"
                                INSERT INTO lessons (id, module_id, title, duration, "type", content_url, description, "order")
                                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                                "#,
                            )
                            .bind(lesson_id)
                            .bind(module.id)
                            .bind(lesson_dto.title.unwrap_or_else(|| "Título por defecto".to_string()))  // Título por defecto
                            .bind(lesson_dto.duration.unwrap_or_else(|| "Duración por defecto".to_string()))  // Duración por defecto
                            .bind(lesson_dto.r#type.unwrap_or_else(|| "Tipo por defecto".to_string()))  // Tipo por defecto
                            .bind(lesson_dto.content_url.unwrap_or_else(|| "https://example.com/default-url".to_string()))  // Default URL
                            .bind(lesson_dto.description.unwrap_or_else(|| "Descripción por defecto".to_string()))  // Descripción por defecto
                            .bind(lesson_order)
                            .execute(&self.pool)
                            .await?;
                        }
                    }
                }
            }
        }

        // ============================================================
        // 3. REUTILIZAMOS LA FUNCIÓN get_all_courses_with_modules
        // ============================================================
        let updated_full_course = self.get_all_courses_with_modules().await?;

        // 2. VERIFICACIÓN DEL RESULTADO
        match result {
            Ok(Some(course)) => {
                println!("DEBUG: ÉXITO. Curso ID {} actualizado. Filas afectadas: 1", course_id);
                Ok(updated_full_course.into_iter()
                    .find(|c| c.id == course_id)
                    .expect("Curso debería existir después de la actualización"))
            },
            Ok(None) => {
                // Este es el error que estás viendo: rows_affected=0
                println!("ERROR LÓGICO: No se encontró el curso con ID {} para actualizar. Filas afectadas: 0", course_id);
                // Aquí puedes retornar un error de "No encontrado" (ej: sqlx::Error::RowNotFound)
                // o un error personalizado de tu aplicación.
                Err(sqlx::Error::RowNotFound) 
            },
            Err(e) => {
                // Este es el error si la DB falla por restricción, tipo, o conexión
                println!("ERROR SQL: Fallo en la ejecución del UPDATE para el curso ID {}: {:?}", course_id, e);
                Err(e)
            }
        }
        
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