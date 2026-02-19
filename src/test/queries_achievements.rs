#[cfg(test)]
mod tests {
    use crate::db::db::{AchievementExt, DBClient};
    use crate::test::db_utils::get_test_pool;

    #[tokio::test]
    async fn test_create_and_manage_achievement() {
        let (pool, _db_name) = get_test_pool().await;
        let client = DBClient::new(pool);

        // 1. Create Achievement
        let name = "First Login";
        let description = Some("Log in for the first time");
        let icon = Some("login.png");
        let trigger_type = "login_count";
        let trigger_value = 1;
        let active = true;

        let achievement = client
            .create_achievement(name, description, icon, trigger_type, trigger_value, active)
            .await
            .expect("Failed to create achievement");

        assert_eq!(achievement.name, name);
        assert_eq!(achievement.trigger_value, trigger_value);

        // 2. Get Achievement by ID
        let fetched = client
            .get_achievement(achievement.id)
            .await
            .expect("Failed to get achievement")
            .expect("Achievement not found");

        assert_eq!(fetched.id, achievement.id);
        assert_eq!(fetched.name, name);

        // 3. Update Achievement
        let new_name = "First Login Updated";
        let updated = client
            .update_achievement(
                achievement.id,
                Some(new_name),
                None,
                None,
                None,
                Some(5),
                None,
            )
            .await
            .expect("Failed to update achievement");

        assert_eq!(updated.name, new_name);
        assert_eq!(updated.trigger_value, 5);

        // 4. Delete Achievement
        client
            .delete_achievement(achievement.id)
            .await
            .expect("Failed to delete achievement");

        let deleted = client
            .get_achievement(achievement.id)
            .await
            .expect("Failed to get after delete");
        assert!(deleted.is_none());
    }
}
