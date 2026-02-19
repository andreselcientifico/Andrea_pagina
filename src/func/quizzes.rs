use actix_web::{
    web::{Data, Json, Path},
    HttpResponse,
};
use std::sync::Arc;
use uuid::Uuid;
use crate::AppState;
use crate::db::db::{QuizExt, CertificateExt};
use crate::errors::error::HttpError;
use crate::middleware::middleware::JWTAuthMiddleware;
use actix_web::HttpMessage;
use crate::config::dtos::{SubmitQuizDto, QuizSubmissionResponseDto, QuizResultDetailDto, CreateQuizDto};

pub async fn get_quiz_by_lesson_handler(
    app_state: Data<Arc<AppState>>,
    path: Path<Uuid>,
) -> Result<HttpResponse, HttpError> {
    let lesson_id = path.into_inner();
    let quiz = app_state.db_client.get_quiz_by_lesson(lesson_id).await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    match quiz {
        Some(q) => Ok(HttpResponse::Ok().json(q)),
        None => Err(HttpError::not_found("Quiz no encontrado para esta lección".to_string())),
    }
}

pub async fn get_quiz_questions_handler(
    app_state: Data<Arc<AppState>>,
    path: Path<Uuid>,
) -> Result<HttpResponse, HttpError> {
    let quiz_id = path.into_inner();
    
    // Check if quiz exists
    let _ = app_state.db_client.get_quiz(quiz_id).await
        .map_err(|e| HttpError::server_error(e.to_string()))?
        .ok_or(HttpError::not_found("Quiz no encontrado".to_string()))?;

    let questions = app_state.db_client.get_quiz_questions(quiz_id).await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    Ok(HttpResponse::Ok().json(questions))
}

pub async fn submit_quiz_handler(
    req: actix_web::HttpRequest,
    app_state: Data<Arc<AppState>>,
    path: Path<Uuid>,
    body: Json<SubmitQuizDto>,
) -> Result<HttpResponse, HttpError> {
    let quiz_id = path.into_inner();
    let user_data = req.extensions().get::<JWTAuthMiddleware>()
        .ok_or_else(|| HttpError::unauthorized("Usuario no autenticado".to_string()))?.clone();
    let user_id = user_data.claims.sub;

    // 1. Fetch Quiz details (pass percentage)
    let quiz = app_state.db_client.get_quiz(quiz_id).await
        .map_err(|e| HttpError::server_error(e.to_string()))?
        .ok_or(HttpError::not_found("Quiz no encontrado".to_string()))?;

    // 2. Fetch Questions (with correct answers)
    let questions = app_state.db_client.get_quiz_questions(quiz_id).await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    // 3. Grade the submission
    let mut score = 0;
    let total_questions = questions.len() as i32; // Assuming all questions have 1 point
    let mut results: Vec<QuizResultDetailDto> = Vec::new();

    for question in &questions {
        // Find user answer
        let user_answer = body.answers.iter().find(|a| a.question_id == question.id);
        
        let correct_option_id = question.correct_option_id.clone().unwrap_or_default();
        let mut is_correct = false;
        let mut selected_option_id = "".to_string();

        if let Some(answer) = user_answer {
             selected_option_id = answer.selected_option_id.clone();
             if selected_option_id == correct_option_id {
                 score += 1;
                 is_correct = true;
             }
        }

        results.push(QuizResultDetailDto {
            question_id: question.id.clone(),
            question: question.question.clone(),
            selected_option_id,
            correct_option_id,
            is_correct,
            explanation: question.explanation.clone(),
        });
    }

    let percentage = if total_questions > 0 {
        (score as f64 / total_questions as f64) * 100.0
    } else {
        0.0
    };

    let passed = percentage >= quiz.pass_percentage.unwrap_or(0.0);

    // 4. Save Attempt
    let answers_json = serde_json::to_value(&results).unwrap_or(serde_json::json!([]));
    let attempt = app_state.db_client.submit_quiz_attempt(
        user_id, 
        quiz_id, 
        score, 
        total_questions, 
        percentage, 
        passed, 
        answers_json
    ).await.map_err(|e| HttpError::server_error(e.to_string()))?;

    // 5. Generate Certificate if passed
    let mut certificate_dto: Option<crate::config::dtos::CertificateDto> = None;
    if passed {
        if let Some(course_id) = app_state.db_client.get_course_id_by_quiz(quiz_id).await.map_err(|e| HttpError::server_error(e.to_string()))? {
            let total_quizzes = app_state.db_client.get_total_quizzes_in_course(course_id).await.map_err(|e| HttpError::server_error(e.to_string()))?;
            let passed_quizzes = app_state.db_client.get_user_passed_quizzes_count(user_id, course_id).await.map_err(|e| HttpError::server_error(e.to_string()))?;

            // Completion percentage
            let completion_percentage = if total_quizzes > 0 {
                (passed_quizzes / total_quizzes) * 100
            } else { 0 };

            // If user passed all quizzes in the course, generate or update certificate
            if total_quizzes > 0 && passed_quizzes >= total_quizzes {
                let cert = app_state.db_client.generate_certificate(user_id, course_id, completion_percentage).await.map_err(|e| HttpError::server_error(e.to_string()))?;
                certificate_dto = app_state.db_client.get_certificate(cert.id).await.map_err(|e| HttpError::server_error(e.to_string()))?;
            }
        }
    }

    // 6. Return response
    Ok(HttpResponse::Ok().json(QuizSubmissionResponseDto {
        submission_id: attempt.id.to_string(),
        quiz_id: quiz_id.to_string(),
        score,
        total_score: total_questions,
        percentage,
        passed,
        submitted_at: attempt.submitted_at,
        results,
        certificate: certificate_dto,
    }))
}

pub async fn get_user_attempts_handler(
     req: actix_web::HttpRequest,
    app_state: Data<Arc<AppState>>,
    path: Path<Uuid>,
) -> Result<HttpResponse, HttpError> {
    let quiz_id = path.into_inner();
    let user_data = req.extensions().get::<JWTAuthMiddleware>()
        .ok_or_else(|| HttpError::unauthorized("Usuario no autenticado".to_string()))?.clone();
    
    let attempts = app_state.db_client.get_user_attempts(user_data.claims.sub, quiz_id).await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    Ok(HttpResponse::Ok().json(attempts))
}

pub async fn get_attempt_detail_handler(
    req: actix_web::HttpRequest,
    app_state: Data<Arc<AppState>>,
    path: Path<Uuid>,
) -> Result<HttpResponse, HttpError> {
    let attempt_id = path.into_inner();
    let user_data = req.extensions().get::<JWTAuthMiddleware>()
        .ok_or_else(|| HttpError::unauthorized("Usuario no autenticado".to_string()))?.clone();

    let attempt = app_state.db_client.get_attempt(attempt_id).await
        .map_err(|e| HttpError::server_error(e.to_string()))?
        .ok_or(HttpError::not_found("Intento no encontrado".to_string()))?;

    if attempt.user_id != user_data.claims.sub {
         return Err(HttpError::forbidden("No tienes permiso para ver este intento".to_string()));
    }

    // Reconstruct response from saved answers
    // attempt.answers is JSONB.
    let results: Vec<QuizResultDetailDto> = serde_json::from_value(attempt.answers.unwrap_or(serde_json::json!([])))
        .unwrap_or_default();

    Ok(HttpResponse::Ok().json(QuizSubmissionResponseDto {
        submission_id: attempt.id.to_string(),
        quiz_id: attempt.quiz_id.to_string(),
        score: attempt.score,
        total_score: attempt.total_score,
        percentage: attempt.percentage,
        passed: attempt.passed,
        submitted_at: attempt.submitted_at,
        results,
        certificate: None,
    }))
}

// ------------------------------
// Admin quiz management handlers
// ------------------------------

pub async fn create_quiz_handler(
    app_state: Data<Arc<AppState>>,
    body: Json<CreateQuizDto>,
) -> Result<HttpResponse, HttpError> {
    let dto = body.into_inner();
    let created = app_state.db_client.create_quiz_with_questions(dto).await
        .map_err(|e| HttpError::server_error(e.to_string()))?;
    Ok(HttpResponse::Created().json(created))
}

pub async fn update_quiz_handler(
    app_state: Data<Arc<AppState>>,
    path: Path<Uuid>,
    body: Json<CreateQuizDto>,
) -> Result<HttpResponse, HttpError> {
    let quiz_id = path.into_inner();
    let dto = body.into_inner();
    let updated = app_state.db_client.update_quiz_with_questions(quiz_id, dto).await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    Ok(HttpResponse::Ok().json(updated))
}

pub async fn delete_quiz_handler(
    app_state: Data<Arc<AppState>>,
    path: Path<Uuid>,
) -> Result<HttpResponse, HttpError> {
    let quiz_id = path.into_inner();
    app_state.db_client.delete_quiz(quiz_id).await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    Ok(HttpResponse::NoContent().finish())
}

