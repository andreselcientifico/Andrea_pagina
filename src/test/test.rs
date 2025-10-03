#[cfg(test)]
mod tests {
    use mockall::*;
    use mockall::predicate::*;
    use crate::models::models::{User, UserRole};

    trait CustomUserTrait {
        fn get_user(&self, user_id: &uuid::Uuid) -> Result<User, String>;
    }

    mock! {
        CustomUser {}
        impl CustomUserTrait for CustomUser {
            fn get_user(&self, user_id: &uuid::Uuid) -> Result<User, String>;
        }
    }

    #[test]
    fn test_user_creation() {
        let mut mock = MockCustomUser::new();
        let user_id = uuid::Uuid::new_v4();
        // Construct User manually if User::new does not exist
        let user = User {
            id: user_id,
            email: "test@example.com".to_string(),
            name: "Test Name".to_string(),
            verified: false,
            password: "password123".to_string(),
            role: UserRole::User,
            verification_token: Some("token123".to_string()),
            token_expiry: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        mock.expect_get_user()
            .with(eq(user_id))
            .returning(move |_| Ok(user.clone()));

        // Aquí puedes usar el mock en tus pruebas
        // Use the trait to avoid unused trait warning
        let result = CustomUserTrait::get_user(&mock, &user_id);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().email, "test@example.com");
    }
}