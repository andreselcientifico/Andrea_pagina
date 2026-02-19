#[cfg(test)]
mod tests {
    use crate::db::db::{DBClient, UserExt};
    use crate::models::models::UserRole;
    use crate::test::db_utils::get_test_pool;

    #[tokio::test]
    async fn test_create_and_get_user() {
        // Setup
        let (pool, _db_name) = get_test_pool().await;
        let client = DBClient::new(pool);

        // Define user data
        let name = "Test User";
        let email = "test@example.com";
        let password = "hashedpassword";
        let verification_token = "token123";

        // 1. Test create user
        let user = client
            .save_user(
                name,
                email,
                password,
                verification_token,
                None,
                Some(UserRole::User),
            )
            .await
            .expect("Failed to create user");

        assert_eq!(user.name, name);
        assert_eq!(user.email, email);
        assert_eq!(user.role, UserRole::User);

        // 2. Test get user by ID
        let fetched_user = client
            .get_user(Some(user.id), None, None, None)
            .await
            .expect("Failed to get user")
            .expect("User not found");

        assert_eq!(fetched_user.id, user.id);
        assert_eq!(fetched_user.email, email);

        // 3. Test get user by Email
        let fetched_user_email = client
            .get_user(None, None, Some(email), None)
            .await
            .expect("Failed to get user by email")
            .expect("User not found by email");

        assert_eq!(fetched_user_email.id, user.id);

        // Teardown (optional, as the test DB is separate anyway, but good practice if we had a drop mechanism)
    }

    #[tokio::test]
    async fn test_update_user_name() {
        let (pool, _db_name) = get_test_pool().await;
        let client = DBClient::new(pool);

        let user = client
            .save_user(
                "Old Name",
                "update_name@example.com",
                "pass",
                "token_update",
                None,
                None,
            )
            .await
            .expect("Failed to create user");

        let updated_user = client
            .update_user_name(user.id, "New Name")
            .await
            .expect("Failed to update name");

        assert_eq!(updated_user.name, "New Name");
        assert_eq!(updated_user.id, user.id);
    }

    #[tokio::test]
    async fn test_user_count() {
        let (pool, _db_name) = get_test_pool().await;
        let client = DBClient::new(pool);

        let initial_count = client.get_user_count().await.expect("Failed to get count");

        client
            .save_user("U1", "u1@e.com", "p", "t1", None, None)
            .await
            .expect("Failed to save 1");
        client
            .save_user("U2", "u2@e.com", "p", "t2", None, None)
            .await
            .expect("Failed to save 2");

        let final_count = client.get_user_count().await.expect("Failed to get count");
        assert_eq!(final_count, initial_count + 2);
    }
}
