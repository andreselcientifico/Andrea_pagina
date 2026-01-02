use actix_web::{dev::HttpServiceFactory, web::{resource, scope, get, put, post, delete}};
use uuid::Uuid;

use crate::{func::{self, courses::{create_course, create_lesson_comment, create_or_update_rating, delete_comment, delete_course, get_course_with_modules, get_courses, get_courses_with_modules, get_lesson_comments, get_rating, update_course, update_lesson_progress}, payments::{created_order, paypal_webhook}, users::{get_me, get_users, update_user_name, update_user_password, update_user_role}}, middleware::middleware::{AccessCheck, RequiredAccess, RoleCheck}, models::models::UserRole};

pub fn auth_scope() -> impl HttpServiceFactory {
    scope("/auth")
        .service(func::handlers::register_user)
        .service(func::handlers::login_user)
        .service(func::handlers::verify_email)
        .service(func::handlers::logout_user)
}


pub fn global_scope() -> impl HttpServiceFactory {
    scope("/api")
        .service(
            scope("/users")
                .service(
                    resource("/me")
                        .route(get().to(get_me))
                        .wrap(RoleCheck::new(vec![UserRole::User, UserRole::Admin])),
                )
                .service(
                    resource("")
                        .route(get().to(get_users))
                        .wrap(RoleCheck::new(vec![UserRole::Admin])),
                )
                .service(resource("/name").route(put().to(update_user_name)))
                .service(resource("/role").route(put().to(update_user_role)))
                .service(resource("/password").route(put().to(update_user_password)))
        )
        .service(
            scope("/payments")
                .route("/webhooks/paypal", post().to(paypal_webhook))
        )
        .service(
            scope("/courses")
                .wrap(RoleCheck::new(vec![UserRole::Admin]))
                .route("", get().to(get_courses))
                .route("", post().to(create_course))
                .route("/{id}", put().to(update_course))
                .route("/{id}", delete().to(delete_course))
                .service(
                    scope("/videos")
                        .wrap(RoleCheck::new(vec![UserRole::Admin]))
                        .route("", get().to(get_courses_with_modules))
                )
                .service(
                    scope("/{id}/videos")
                        .wrap(AccessCheck::new(vec![
                            RequiredAccess::Role(UserRole::Admin),
                            RequiredAccess::PremiumAccess,
                            RequiredAccess::OwnedCourse(Uuid::nil()),
                            RequiredAccess::AnyCourseAccess
                        ]))
                        .route("", get().to(get_course_with_modules))
                )
                .service(created_order)
                .service(update_lesson_progress)
                .service(
                    scope("/{lessonId}/comments")
                        .route("", post().to(create_lesson_comment))
                        .route("", get().to(get_lesson_comments))
                        .route("/{commentId}", delete().to(delete_comment))
                )
                .service(
                    scope("/{courseId}/rating")
                        .route("", post().to(create_or_update_rating))
                        .route("", get().to(get_rating))
                )
        )
        .service(func::handlers::get_user_profile)
        .service(func::handlers::update_user_profile)
        .service(func::payments::capture_order)
        .service(func::handlers::get_user_courses_api)
}
