use actix_web::{dev::HttpServiceFactory, web::{resource, scope, get, put, post, delete}};
use uuid::Uuid;

use crate::{func::{handlers, quizzes::{create_quiz_handler, delete_quiz_handler, update_quiz_handler}}, middleware::middleware::AuthMiddlewareFactory};
use crate::func::courses;
use crate::func::payments;
use crate::func::{
    achievements::{
        create_achievement,
        get_achievements,
        assign_achievement_to_user,
        earn_achievement,
        get_achievement,
        update_achievement,
        delete_achievement,
        get_user_achievements_with_details,
        check_and_award_achievements,
        debug_user_achievements
    },
    subscriptions::{
        create_subscription_plan,
        get_subscription_plans,
        get_user_subscriptions,
        update_subscription_plan,
        delete_subscription_plan,
        cancel_subscription
    },
    courses::{
        create_course,
        create_lesson_comment,
        create_or_update_rating,
        delete_comment,
        delete_course,
        get_course_with_modules,
        get_course_with_modules_preview,
        get_courses_with_modules,
        get_lesson_comments,
        get_rating,
        update_course,
        update_lesson_progress
    },
    payments::{
        created_order,
        paypal_webhook
    },
    users::{
        get_me,
        get_users,
        update_user_name,
        update_user_password,
        update_user_role
    },
    quizzes::{
        get_quiz_by_lesson_handler,
        get_quiz_questions_handler,
        submit_quiz_handler,
        get_user_attempts_handler,
        get_attempt_detail_handler
    },
    certificates::{
        get_user_certificates_handler,
        get_certificate_detail_handler,
        download_certificate_handler
    }
};
use crate::middleware::middleware::{AccessCheck, RequiredAccess, RoleCheck};
use crate::models::models::UserRole;

pub fn auth_scope() -> impl HttpServiceFactory {
    scope("/auth")
        .service(handlers::register_user)
        .service(handlers::login_user)
        .service(handlers::verify_email)
        .service(handlers::logout_user)
        .service(
                    resource("/plans/subscriptions")
                        .route(get().to(get_subscription_plans))
        )
}

pub fn course_scope() -> impl HttpServiceFactory {
    scope("/courses")
        .route("", get().to(courses::get_courses))
        .service(handlers::get_courses_page)
}


pub fn global_scope() -> impl HttpServiceFactory {
    scope("/api")
        .wrap(AuthMiddlewareFactory::new())
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
                .service(resource("/change-password").route(put().to(update_user_password)))
                .service(resource("/notifications").route(put().to(crate::func::users::update_notification_settings)))
        )
        .service(
            scope("/payments")
                .route("/webhooks/paypal", post().to(paypal_webhook))
        )
        .service(
            scope("/courses")
                .service(
                    scope("/edit")
                    .wrap(RoleCheck::new(vec![UserRole::Admin]))
                    .route("", post().to(create_course))
                    .route("/{id}", put().to(update_course))
                    .route("/{id}", delete().to(delete_course))
                )
                .service(
                    scope("/videos")
                        .wrap(RoleCheck::new(vec![UserRole::Admin]))
                        .route("", get().to(get_courses_with_modules))
                )
                .service(
                    scope("/{id}")
                        .route("/videos/preview", get().to(get_course_with_modules_preview))
                        .route("/createorder", post().to(created_order))
                        .service(
                            scope("/videos")
                            .wrap(AccessCheck::new(vec![
                                RequiredAccess::Role(UserRole::Admin),
                                RequiredAccess::PremiumAccess,
                                RequiredAccess::OwnedCourse(Uuid::nil()),
                                RequiredAccess::AnyCourseAccess
                            ]))
                            .route("", get().to(get_course_with_modules))
                        )
                        .service(
                            scope("/rating")
                            .route("", post().to(create_or_update_rating))
                            .route("", get().to(get_rating))
                        )
                        .service(
                            scope("/comments")
                                .route("", post().to(create_lesson_comment))
                                .route("", get().to(get_lesson_comments))
                                .route("/{commentId}", delete().to(delete_comment))
                        )
                        .service(
                            scope("/lessons")
                                .route("/{lesson_id}/progress", put().to(update_lesson_progress))
                        )
                )
        )
        .service(
            scope("/achievements")
                .service(
                    resource("/all")
                        .route(get().to(get_achievements))
                )
                .service(
                    resource("create")
                        .route(post().to(create_achievement))
                        .wrap(RoleCheck::new(vec![UserRole::Admin])),
                )
                .service(
                    resource("/{achievement_id}")
                        .route(get().to(get_achievement))
                        .route(put().to(update_achievement))
                        .route(delete().to(delete_achievement))
                        .wrap(RoleCheck::new(vec![UserRole::Admin])),
                )
                .service(
                    resource("/assign")
                        .route(post().to(assign_achievement_to_user))
                        .wrap(RoleCheck::new(vec![UserRole::Admin])),
                )
                .service(
                    resource("/earn")
                        .route(post().to(earn_achievement))
                        .wrap(RoleCheck::new(vec![UserRole::Admin])),
                )
                .service(
                    resource("/users/{user_id}")
                        .route(get().to(get_user_achievements_with_details))
                        .wrap(RoleCheck::new(vec![UserRole::User, UserRole::Admin])),
                )
                .service(
                    resource("/users/{user_id}/check")
                        .route(post().to(check_and_award_achievements))
                        .wrap(RoleCheck::new(vec![UserRole::User, UserRole::Admin])),
                )
                .service(
                    resource("/debug")
                        .route(post().to(debug_user_achievements)),
                )
        )
        .service(
            scope("/subscriptions")
                .service(
                    resource("/plans")
                        .route(post().to(create_subscription_plan))
                        .wrap(RoleCheck::new(vec![UserRole::User, UserRole::Admin])),
                )
                .service(
                    resource("/plans/{plan_id}")
                        .route(put().to(update_subscription_plan))
                        .route(delete().to(delete_subscription_plan))
                        .wrap(RoleCheck::new(vec![UserRole::Admin])),
                )
                .service(
                    resource("/user")
                        .route(get().to(get_user_subscriptions))
                        .wrap(RoleCheck::new(vec![UserRole::User, UserRole::Admin])),
                )
                .service(
                    resource("/{subscription_id}/cancel")
                        .route(post().to(cancel_subscription))
                        .wrap(RoleCheck::new(vec![UserRole::User, UserRole::Admin])),
                )
        )
        .service(
            scope("/quiz")
                .service(
                    resource("/lesson/{lesson_id}")
                        .route(get().to(get_quiz_by_lesson_handler))
                )
                .service(
                    resource("/{quiz_id}/questions")
                        .route(get().to(get_quiz_questions_handler))
                )
                .service(
                    resource("/{quiz_id}/submit")
                        .route(post().to(submit_quiz_handler))
                )
                .service(
                    resource("/{quiz_id}/attempts")
                        .route(get().to(get_user_attempts_handler))
                )
                .service(
                    resource("/{quiz_id}")
                        .route(put().to(update_quiz_handler))
                        .route(delete().to(delete_quiz_handler))
                        .wrap(RoleCheck::new(vec![UserRole::Admin]))
                )
                .service(
                    resource("/edit/create")
                        .route(post().to(create_quiz_handler))
                        .wrap(RoleCheck::new(vec![UserRole::Admin]))
                )
                .service(
                    resource("/attempt/{attempt_id}")
                        .route(get().to(get_attempt_detail_handler))
                )
        )
        .service(
            scope("/certificates")
                .service(
                    resource("get")
                        .route(get().to(get_user_certificates_handler))
                )
                .service(
                    resource("/{certificate_id}")
                        .route(get().to(get_certificate_detail_handler))
                )
                .service(
                    resource("/{certificate_id}/download")
                        .route(get().to(download_certificate_handler))
                )
        )
        .service(
            scope("/admin/notifications")
                .wrap(RoleCheck::new(vec![UserRole::Admin]))
                .route("/send", post().to(crate::func::notifications::send_bulk_email))
                .route("/count/{notification_type}", get().to(crate::func::notifications::get_notification_recipients_count))
        )
        .service(handlers::get_user_profile)
        .service(handlers::update_user_profile)
        .service(handlers::get_user_courses_api)
        .service(payments::capture_order)
        .service(payments::verify_subscription)
}
