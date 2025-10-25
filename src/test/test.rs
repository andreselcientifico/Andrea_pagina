#[cfg(test)]
mod tests {
    use mockall::*;
    use mockall::predicate::*;
    use crate::models::models::{User, UserRole};
    use chrono::Utc;

    trait CustomUserTrait {
        fn get_user(&self, user_id: &uuid::Uuid) -> Result<User, String>;
    }

    mock! {
        CustomUser {}
        impl CustomUserTrait for CustomUser {
            fn get_user(&self, user_id: &uuid::Uuid) -> Result<User, String>;
        }
    }

    fn build_test_user(user_id: uuid::Uuid) -> User {
        User {
            id: user_id,
            email: "test@example.com".to_string(),
            name: "Test Name".to_string(),
            verified: false,
            password: "password123".to_string(),
            role: Some(UserRole::User),
            verification_token: Some("token123".to_string()),
            token_expiry: None,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        }
    }

    #[test]
    fn test_user_creation_success() {
        let mut mock = MockCustomUser::new();
        let user_id = uuid::Uuid::new_v4();
        let user = build_test_user(user_id);

        mock.expect_get_user()
            .with(eq(user_id))
            .returning(move |_| Ok(user.clone()));

        let result = CustomUserTrait::get_user(&mock, &user_id);
        assert!(result.is_ok());
        let retrieved_user = result.unwrap();
        assert_eq!(retrieved_user.email, "test@example.com");
        assert_eq!(retrieved_user.role, Some(UserRole::User));
    }

    #[test]
    fn test_user_creation_error() {
        let mut mock = MockCustomUser::new();
        let user_id = uuid::Uuid::new_v4();

        mock.expect_get_user()
            .with(eq(user_id))
            .returning(|_| Err("User not found".to_string()));

        let result = CustomUserTrait::get_user(&mock, &user_id);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "User not found");
    }

    #[test]
    fn test_user_role_admin() {
        let mut mock = MockCustomUser::new();
        let user_id = uuid::Uuid::new_v4();

        let mut user = build_test_user(user_id);
        user.role = Some(UserRole::Admin);

        mock.expect_get_user()
            .with(eq(user_id))
            .returning(move |_| Ok(user.clone()));

        let result = CustomUserTrait::get_user(&mock, &user_id).unwrap();
        assert_eq!(result.role, Some(UserRole::Admin));
    }
}
