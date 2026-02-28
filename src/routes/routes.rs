use actix_web::{
    dev::HttpServiceFactory,
    web::{delete, get, post, put, resource, scope},
};
use uuid::Uuid;

use crate::func::payments;
use crate::func::{
    achievements::{
        assign_achievement_to_user, check_and_award_achievements, create_achievement,
        debug_user_achievements, delete_achievement, earn_achievement, get_achievement,
        get_achievements, get_user_achievements_with_details, update_achievement,
    },
    certificates::{
        download_certificate_handler, get_certificate_detail_handler, get_user_certificates_handler,
    },
    courses::{
        create_course, create_lesson_comment, create_or_update_rating, delete_comment,
        delete_course, get_course_with_modules, get_course_with_modules_preview,
        get_courses_with_modules, get_lesson_comments, get_rating, update_course,
        update_lesson_progress,
    },
    payments::created_order,
    quizzes::{
        get_attempt_detail_handler, get_quiz_by_lesson_handler, get_quiz_questions_handler,
        get_user_attempts_handler, submit_quiz_handler,
    },
    subscriptions::{
        cancel_subscription, create_subscription_plan, delete_subscription_plan,
        get_subscription_plans, get_user_subscriptions, update_subscription_plan,
    },
    users::{get_me, get_users, update_user_name, update_user_password, update_user_role},
};
use crate::func::{courses};
use crate::middleware::middleware::{AccessCheck, RequiredAccess, RoleCheck};
use crate::models::models::UserRole;
use crate::{
    func::{
        handlers,
        notifications::{get_notification_recipients_count, send_bulk_email},
        quizzes::{create_quiz_handler, delete_quiz_handler, update_quiz_handler},
        users::update_notification_settings,
    },
    middleware::middleware::AuthMiddlewareFactory,
};

pub fn auth_scope() -> impl HttpServiceFactory {
    scope("/auth")
        .service(handlers::register_user)
        .service(handlers::login_user)
        .service(handlers::verify_email)
        .service(handlers::logout_user)
        .service(handlers::forgot_password)
        .service(handlers::reset_password)
        .service(handlers::resend_verification)
        .service(resource("/plans/subscriptions").route(get().to(get_subscription_plans)))
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
                    resource("/all")
                        .route(get().to(get_users))
                        .wrap(RoleCheck::new(vec![UserRole::Admin])),
                )
                .service(resource("/name").route(put().to(update_user_name)))
                .service(
                    resource("/role")
                        .route(put().to(update_user_role))
                        .wrap(RoleCheck::new(vec![UserRole::Admin])),
                )
                .service(resource("/change-password").route(put().to(update_user_password)))
                .service(resource("/notifications").route(put().to(update_notification_settings))),
        )
        .service(
            scope("/courses")
                .service(resource("/edit").route(get().to(create_course)))
                .service(
                    resource("/edit/{id}")
                        .wrap(RoleCheck::new(vec![UserRole::Admin]))
                        .route(put().to(update_course))
                        .route(delete().to(delete_course)),
                )
                .service(
                    resource("/videos")
                        .wrap(RoleCheck::new(vec![UserRole::Admin]))
                        .route(get().to(get_courses_with_modules)),
                )
                .service(
                    scope("/{id}")
                        .service(
                            resource("/videos/preview")
                                .route(get().to(get_course_with_modules_preview)),
                        )
                        .service(resource("/createorder").route(post().to(created_order)))
                        .service(
                            resource("/videos")
                                .wrap(AccessCheck::new(vec![
                                    RequiredAccess::Role(UserRole::Admin),
                                    RequiredAccess::PremiumAccess,
                                    RequiredAccess::OwnedCourse(Uuid::nil()),
                                    RequiredAccess::AnyCourseAccess,
                                ]))
                                .route(get().to(get_course_with_modules)),
                        )
                        .service(
                            resource("/rating")
                                .route(post().to(create_or_update_rating))
                                .route(get().to(get_rating)),
                        )
                        .service(
                            resource("/comments")
                                .route(post().to(create_lesson_comment))
                                .route(get().to(get_lesson_comments)),
                        )
                        .service(
                            resource("/comments/{commentId}").route(delete().to(delete_comment)),
                        )
                        .service(
                            resource("/lessons/{lesson_id}/progress")
                                .route(put().to(update_lesson_progress)),
                        ),
                ),
        )
        .service(
            scope("/achievements")
                .service(resource("/all").route(get().to(get_achievements)))
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
                .service(resource("/debug").route(post().to(debug_user_achievements))),
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
                ),
        )
        .service(
            scope("/quiz")
                .service(
                    resource("/lesson/{lesson_id}").route(get().to(get_quiz_by_lesson_handler)),
                )
                .service(
                    resource("/{quiz_id}/questions").route(get().to(get_quiz_questions_handler)),
                )
                .service(resource("/{quiz_id}/submit").route(post().to(submit_quiz_handler)))
                .service(resource("/{quiz_id}/attempts").route(get().to(get_user_attempts_handler)))
                .service(
                    resource("/{quiz_id}")
                        .route(put().to(update_quiz_handler))
                        .route(delete().to(delete_quiz_handler))
                        .wrap(RoleCheck::new(vec![UserRole::Admin])),
                )
                .service(
                    resource("/edit/create")
                        .route(post().to(create_quiz_handler))
                        .wrap(RoleCheck::new(vec![UserRole::Admin])),
                )
                .service(
                    resource("/attempt/{attempt_id}").route(get().to(get_attempt_detail_handler)),
                ),
        )
        .service(
            scope("/certificates")
                .service(resource("get").route(get().to(get_user_certificates_handler)))
                .service(
                    resource("/{certificate_id}").route(get().to(get_certificate_detail_handler)),
                )
                .service(
                    resource("/{certificate_id}/download")
                        .route(get().to(download_certificate_handler)),
                ),
        )
        .service(
            scope("/admin/notifications")
                .wrap(RoleCheck::new(vec![UserRole::Admin]))
                .service(resource("/send").route(post().to(send_bulk_email)))
                .service(
                    resource("/count/{notification_type}")
                        .route(get().to(get_notification_recipients_count)),
                ),
        )
        .service(handlers::get_user_profile)
        .service(handlers::update_user_profile)
        .service(handlers::get_user_courses_api)
        .service(payments::capture_order)
        .service(payments::verify_subscription)
}
