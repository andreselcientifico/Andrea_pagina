use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::models::models::{User, UserRole, Course, Payment};

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
                password, 
                verified, 
                created_at, 
                updated_at, 
                verification_token, 
                token_expiry, 
                role as "role: UserRole" 
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
                password, 
                verified, 
                created_at, 
                updated_at, 
                verification_token, 
                token_expiry, 
                role as "role: UserRole" 
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
                password, 
                verified, 
                created_at, 
                updated_at, 
                verification_token, 
                token_expiry, 
                role as "role: UserRole" 
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
                password, 
                verified, 
                created_at, 
                updated_at, 
                verification_token, 
                token_expiry, 
                role as "role: UserRole" 
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
            r#"SELECT id, name, email, password, verified, created_at, updated_at, verification_token, token_expiry, role as "role: UserRole" FROM users 
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
    ) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (name, email, password, verification_token, token_expiry) 
            VALUES ($1, $2, $3, $4, $5) 
            RETURNING id, name, email, password, verified, created_at, updated_at, verification_token, token_expiry, role as "role: UserRole"
            "#,
            name.into(),
            email.into(),
            password.into(),
            verification_token.into(),
            token_expiry // This can be `None` if no expiry is provided.
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
            RETURNING id, name, email, password, verified, created_at, updated_at, verification_token, token_expiry, role as "role: UserRole"
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
            RETURNING id, name, email, password, verified, created_at, updated_at, verification_token, token_expiry, role as "role: UserRole"
            "#,
            new_role as UserRole,
            user_id
        ).fetch_one(&self.pool)
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
            RETURNING id, name, email, password, verified, created_at, updated_at, verification_token, token_expiry, role as "role: UserRole"
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
    async fn create_course<T: Into<String> + Send>(
        &self,
        name: T,
        description: T,
        price: f64,
    ) -> Result<Course, sqlx::Error>;

    async fn get_course(&self, course_id: Uuid) -> Result<Option<Course>, sqlx::Error>;

    async fn get_courses(
        &self,
        page: u32,
        limit: usize,
    ) -> Result<Vec<Course>, sqlx::Error>;

    async fn update_course(
        &self,
        course_id: Uuid,
        name: Option<String>,
        description: Option<String>,
        price: Option<f64>,
    ) -> Result<Course, sqlx::Error>;

    async fn delete_course(&self, course_id: Uuid) -> Result<(), sqlx::Error>;

    async fn get_course_count(&self) -> Result<i64, sqlx::Error>;
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

    async fn get_payment_count(&self) -> Result<i64, sqlx::Error>;
}

#[async_trait]
impl CourseExt for DBClient {
    async fn create_course<T: Into<String> + Send>(
        &self,
        name: T,
        description: T,
        price: f64,
    ) -> Result<Course, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        
        let course = sqlx::query_as::<_, Course>(
            r#"INSERT INTO courses (id, name, description, price, created_at) 
               VALUES ($1, $2, $3, $4, $5) RETURNING id, name, description, price, created_at, updated_at"#,
        )
        .bind(id)
        .bind(name.into())
        .bind(description.into())
        .bind(price)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(course)
    }

    async fn get_course(&self, course_id: Uuid) -> Result<Option<Course>, sqlx::Error> {
        let course = sqlx::query_as::<_, Course>(
            r#"SELECT id, name, description, price, created_at, updated_at FROM courses WHERE id = $1"#,
        )
        .bind(course_id)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(course)
    }

    async fn get_courses(
        &self,
        page: u32,
        limit: usize,
    ) -> Result<Vec<Course>, sqlx::Error> {
        let offset = ((page - 1) * limit as u32) as i64;
        
        let courses = sqlx::query_as::<_, Course>(
            r#"SELECT id, name, description, price, created_at, updated_at FROM courses 
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
        name: Option<String>,
        description: Option<String>,
        price: Option<f64>,
    ) -> Result<Course, sqlx::Error> {
        let course = sqlx::query_as::<_, Course>(
            r#"UPDATE courses 
               SET name = COALESCE($2, name), 
                   description = COALESCE($3, description), 
                   price = COALESCE($4, price), 
                   updated_at = $5 
               WHERE id = $1 
               RETURNING id, name, description, price, created_at, updated_at"#,
        )
        .bind(course_id)
        .bind(name)
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