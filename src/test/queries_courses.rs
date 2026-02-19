#[cfg(test)]
mod tests {
    use crate::config::dtos::{CreateCourseDTO, CreateLessonDTO, CreateModuleDTO};
    use crate::db::db::{CourseExt, DBClient};
    use crate::test::db_utils::get_test_pool;
    // UpdateCourseDTO might be needed later

    #[tokio::test]
    async fn test_create_and_fetch_course() {
        let (pool, _db_name) = get_test_pool().await;
        let client = DBClient::new(pool);

        // Define course data
        let lesson_dto = CreateLessonDTO {
            title: "Lesson 1".to_string(),
            duration: Some("10 min".to_string()),
            completed: false,
            r#type: "video".to_string(),
            content_url: Some("http://example.com/video".to_string()),
            description: Some("Intro lesson".to_string()),
            order: None,
        };

        let module_dto = CreateModuleDTO {
            title: "Module 1".to_string(),
            order: None,
            lessons: vec![lesson_dto],
        };

        let course_dto = CreateCourseDTO {
            title: "Test Course".to_string(),
            description: "Short desc".to_string(),
            long_description: Some("Long desc".to_string()),
            level: "básico".to_string(),
            price: 9.99,
            duration: Some("1h".to_string()),
            students: Some(0),
            image: Some("http://example.com/img.jpg".to_string()),
            category: "premium".to_string(),
            features: Some(vec!["Feature 1".to_string()]),
            paypal_product_id: None,
            modules: vec![module_dto],
        };

        // 1. Create Course
        let _created = client
            .create_course(course_dto.clone())
            .await
            .expect("Failed to create course");

        // 2. Fetch all courses (to get the ID, since create_course doesn't return it)
        let courses = client
            .get_courses(1, 10)
            .await
            .expect("Failed to get courses");
        assert!(!courses.is_empty());
        let fetched_course_summary = &courses[0];

        assert_eq!(fetched_course_summary.title, course_dto.title);
        assert_eq!(fetched_course_summary.price, course_dto.price);

        // 3. Get Course Detail
        let course_detail = client
            .get_course(fetched_course_summary.id)
            .await
            .expect("Failed to get course detail");
        assert!(course_detail.is_some());
        let c = course_detail.unwrap();
        assert_eq!(c.title, course_dto.title);

        // 4. Get Course with Modules
        let course_with_modules = client
            .get_course_with_videos(fetched_course_summary.id, None)
            .await
            .expect("Failed to get with modules");

        assert!(course_with_modules.is_some());
        let cwm = course_with_modules.unwrap();
        assert_eq!(cwm.modules.len(), 1);
        assert_eq!(cwm.modules[0].lessons.len(), 1);
    }
}
