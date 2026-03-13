use std::sync::Arc;
use actix_web::{HttpResponse, web::{Data, Json, Path}};
use uuid::Uuid;
use crate::{AppState, config, db::db::{PresentacionVideosExt}};
use config::dtos::{CreatePresentacionVideoDTO, UpdatePresentacionVideoDTO};


// Get all presentation videos
pub async fn get_presentacion_videos(app_state: Data<Arc<AppState>>) -> HttpResponse {
    match app_state.db_client.get_presentacion_videos().await.map_err( |e|
        {
            log::error!("Error: {}", e);
            e
        }
    )
    {
        Ok(videos) => HttpResponse::Ok().json(videos),
        Err(e) => {
            log::error!("Error fetching Presentaciopn videos: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch Presentaciopn videos"
            }))
        }
    }
}

// Get Presentaciopn video by ID
pub async fn get_presentacion_video(
    app_state: Data<Arc<AppState>>,
    video_url: Path<Uuid>,
) -> HttpResponse {
    match app_state.db_client.get_presentacion_video(video_url.into_inner())
        .await
        .map_err(
                |e|
            {
                log::error!("Error: {}", e);
                e
            }
        )
    {
        Ok(Some(video)) => HttpResponse::Ok().json(video),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Video not found"
        })),
        Err(e) => {
            log::error!("Error fetching Presentaciopn video: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch Presentaciopn video"
            }))
        }
    }
}

// Create Presentaciopn video (admin only)
pub async fn create_presentacion_video(
    app_state: Data<Arc<AppState>>,
    Json(body): Json<CreatePresentacionVideoDTO>,
) -> HttpResponse {
    let id = Uuid::new_v4();
    match app_state.db_client.create_presentacion_video(id,  &body.title,  &body.source,  &body.video_url, &body.embed_url, &body.description)
    .await
    .map_err(|e| {
        log::error!("Error: {}", e);
        e
    })
    {
        Ok(video) => HttpResponse::Created().json(video),
        Err(e) => {
            log::error!("Error creating Presentaciopn video: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to create Presentaciopn video"
            }))
        }
    }
}

// Update Presentaciopn video (admin only)
pub async fn update_presentacion_video(
    app_state: Data<Arc<AppState>>,
    id: Path<Uuid>,
    dto: Json<UpdatePresentacionVideoDTO>,
) -> HttpResponse {
    // Update fields
    let title = dto.title.as_ref();
    let description = dto.description.as_ref();
    let embed_url = dto.embed_url.as_ref();

    match app_state.db_client.update_presentacion_video(id.into_inner(), title, description, embed_url)
        .await
        .map_err(
            |e|
            {
                log::error!("Error: {}", e);
                e
            }
        )
    {
        Ok(video) => HttpResponse::Ok().json(video),
        Err(e) => {
            log::error!("Error updating Presentaciopn video: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to update Presentaciopn video"
            }))
        }
    }
}

// Delete Presentaciopn video (admin only)
pub async fn delete_presentacion_video(
    app_state: Data<Arc<AppState>>,
    id: Path<Uuid>,
) -> HttpResponse {
    match app_state.db_client.delete_presentacion_video(id.into_inner())
        .await
        .map_err(
            |e|
            {
                log::error!("Error: {}", e);
                e
            }
        )
    {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => {
            log::error!("Error deleting Presentaciopn video: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to delete Presentaciopn video"
            }))
        }
    }
}
