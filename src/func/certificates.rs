use actix_web::{
    web::{Data, Path},
    HttpResponse,
};
use std::sync::Arc;
use uuid::Uuid;
use crate::AppState;
use crate::db::db::CertificateExt;
use crate::errors::error::HttpError;
use crate::middleware::middleware::JWTAuthMiddleware;
use actix_web::HttpMessage;

pub async fn get_user_certificates_handler(
    req: actix_web::HttpRequest,
    app_state: Data<Arc<AppState>>,
) -> Result<HttpResponse, HttpError> {
    let user_data = req.extensions().get::<JWTAuthMiddleware>()
        .ok_or_else(|| HttpError::unauthorized("Usuario no autenticado".to_string()))?.clone();

    let certificates = app_state.db_client.get_user_certificates(user_data.claims.sub).await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    Ok(HttpResponse::Ok().json(certificates))
}

pub async fn get_certificate_detail_handler(
    req: actix_web::HttpRequest,
    app_state: Data<Arc<AppState>>,
    path: Path<Uuid>,
) -> Result<HttpResponse, HttpError> {
    let certificate_id = path.into_inner();
    let user_data = req.extensions().get::<JWTAuthMiddleware>()
        .ok_or_else(|| HttpError::unauthorized("Usuario no autenticado".to_string()))?.clone();

    let certificate = app_state.db_client.get_certificate(certificate_id).await
        .map_err(|e| HttpError::server_error(e.to_string()))?
        .ok_or(HttpError::not_found("Certificado no encontrado".to_string()))?;

    if certificate.user_id != user_data.claims.sub {
        return Err(HttpError::forbidden("No tienes permiso para ver este certificado".to_string()));
    }

    Ok(HttpResponse::Ok().json(certificate))
}

pub async fn download_certificate_handler(
    req: actix_web::HttpRequest,
    app_state: Data<Arc<AppState>>,
    path: Path<Uuid>,
) -> Result<HttpResponse, HttpError> {
    let certificate_id = path.into_inner();
    let user_data = req.extensions().get::<JWTAuthMiddleware>()
        .ok_or_else(|| HttpError::unauthorized("Usuario no autenticado".to_string()))?.clone();

    let certificate = app_state.db_client.get_certificate(certificate_id).await
        .map_err(|e| {
             log::error!("Error fetching certificate: {}", e);
            HttpError::server_error(e.to_string())
        })?
        .ok_or(HttpError::not_found("Certificado no encontrado".to_string()))?;

    if certificate.user_id != user_data.claims.sub {
        return Err(HttpError::forbidden("No tienes permiso para descargar este certificado".to_string()));
    }

    // Mock PDF generation
    // In a real app, we would generate a PDF here using a library like `genpdf` or `printpdf`.
    // For now, we'll return a dummy PDF content or just a text file.
    let pdf_content = format!("Certificado para {} \n Curso: {} \n Fecha: {}", certificate.user_name, certificate.course_title, certificate.issue_date);
    
    Ok(HttpResponse::Ok()
        .content_type("application/pdf")
        .insert_header(("Content-Disposition", format!("attachment; filename=\"certificate-{}.pdf\"", certificate.certificate_number)))
        .body(pdf_content))
}
