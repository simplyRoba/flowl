#![allow(clippy::missing_errors_doc)]

use axum::Json;
use axum::extract::{Multipart, State};
use serde::Serialize;
use tracing::{debug, warn};

use super::error::ApiError;
use crate::state::AppState;

#[derive(Serialize)]
pub struct AiStatus {
    pub enabled: bool,
    pub base_url: Option<String>,
    pub model: Option<String>,
}

pub async fn get_ai_status(State(state): State<AppState>) -> Json<AiStatus> {
    if state.ai_provider.is_some() {
        Json(AiStatus {
            enabled: true,
            base_url: Some(state.ai_base_url.clone()),
            model: Some(state.ai_model.clone()),
        })
    } else {
        Json(AiStatus {
            enabled: false,
            base_url: None,
            model: None,
        })
    }
}

pub async fn identify_plant(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<crate::ai::types::IdentifyResponse>, ApiError> {
    let provider = state
        .ai_provider
        .as_ref()
        .ok_or_else(|| ApiError::ServiceUnavailable("AI provider is not configured".to_string()))?;

    let mut photos: Vec<Vec<u8>> = Vec::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?
    {
        let name = field.name().unwrap_or("").to_string();
        if name != "photos" && name != "photo" {
            continue;
        }

        let content_type = field.content_type().unwrap_or("").to_string();
        match content_type.as_str() {
            "image/jpeg" | "image/png" | "image/webp" => {}
            _ => {
                return Err(ApiError::Validation(
                    "Invalid file type. Allowed: JPEG, PNG, WebP".to_string(),
                ));
            }
        }

        let data = field
            .bytes()
            .await
            .map_err(|e| ApiError::BadRequest(e.to_string()))?;

        photos.push(data.to_vec());
    }

    if photos.is_empty() {
        return Err(ApiError::Validation(
            "At least one photo is required".to_string(),
        ));
    }

    let locale = sqlx::query_scalar::<_, String>("SELECT locale FROM user_settings WHERE id = 1")
        .fetch_optional(&state.pool)
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| "en".to_string());

    debug!(photo_count = photos.len(), locale = %locale, "sending photos to AI provider");
    let image_refs: Vec<&[u8]> = photos.iter().map(Vec::as_slice).collect();

    let result = provider.identify(&image_refs, &locale).await.map_err(|e| {
        warn!(error = %e, "AI identify failed");
        ApiError::InternalError(format!("AI identification failed: {e}"))
    })?;

    debug!(
        suggestion_count = result.suggestions.len(),
        first_name = result.suggestions.first().map_or("—", |s| s.common_name.as_str()),
        first_scientific = result.suggestions.first().map_or("—", |s| s.scientific_name.as_str()),
        first_confidence = ?result.suggestions.first().and_then(|s| s.confidence),
        "AI identify result"
    );

    Ok(Json(result))
}
