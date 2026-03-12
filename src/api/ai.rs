use axum::Json;
use axum::extract::{Multipart, State};
use axum::response::sse::{Event, Sse};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tokio_stream::StreamExt;
use tracing::{debug, warn};

use super::error::{ApiError, default_message};
use crate::ai::prompts;
use crate::ai::types::ChatMessage;
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

/// # Errors
/// Returns `ApiError::ServiceUnavailable` if the AI provider is not configured,
/// `ApiError::Validation` for invalid file types or missing photos,
/// `ApiError::BadRequest` for malformed multipart data, or
/// `ApiError::InternalError` if the AI provider call fails.
pub async fn identify_plant(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<crate::ai::types::IdentifyResponse>, ApiError> {
    let provider = state
        .ai_provider
        .as_ref()
        .ok_or(ApiError::ServiceUnavailable("AI_NOT_CONFIGURED"))?;

    let mut photos: Vec<Vec<u8>> = Vec::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| ApiError::BadRequest("INVALID_REQUEST_BODY"))?
    {
        let name = field.name().unwrap_or("").to_string();
        if name != "photos" && name != "photo" {
            continue;
        }

        let content_type = field.content_type().unwrap_or("").to_string();
        match content_type.as_str() {
            "image/jpeg" | "image/png" | "image/webp" => {}
            _ => {
                return Err(ApiError::Validation("PHOTO_INVALID_TYPE"));
            }
        }

        let data = field
            .bytes()
            .await
            .map_err(|_| ApiError::BadRequest("INVALID_REQUEST_BODY"))?;

        photos.push(data.to_vec());
    }

    if photos.is_empty() {
        return Err(ApiError::Validation("PHOTO_NO_FILE"));
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
        ApiError::InternalError("AI_PROVIDER_FAILED")
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

// --- Request / response types ---

#[derive(Deserialize)]
pub struct ChatRequest {
    pub plant_id: i64,
    pub message: String,
    #[serde(default)]
    pub image: Option<String>,
    #[serde(default)]
    pub history: Vec<ChatMessage>,
}

#[derive(Deserialize)]
pub struct SummarizeRequest {
    pub plant_id: i64,
    pub history: Vec<ChatMessage>,
}

#[derive(Serialize)]
pub struct SummarizeResponse {
    pub summary: String,
}

// --- Handlers ---

async fn get_locale(pool: &SqlitePool) -> String {
    sqlx::query_scalar::<_, String>("SELECT locale FROM user_settings WHERE id = 1")
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| "en".to_string())
}

/// # Errors
/// Returns `ApiError::ServiceUnavailable` if the AI provider is not configured,
/// `ApiError::BadRequest` for invalid base64 image data,
/// `ApiError::NotFound` if the plant does not exist, or
/// `ApiError::InternalError` if the AI provider call fails.
pub async fn chat(
    State(state): State<AppState>,
    Json(body): Json<ChatRequest>,
) -> Result<Sse<impl tokio_stream::Stream<Item = Result<Event, std::convert::Infallible>>>, ApiError>
{
    let provider = state
        .ai_provider
        .as_ref()
        .ok_or(ApiError::ServiceUnavailable("AI_NOT_CONFIGURED"))?;

    let context = prompts::build_plant_context(&state.pool, body.plant_id).await?;
    let locale = get_locale(&state.pool).await;
    let system_prompt = prompts::build_chat_system_prompt(&context, &locale);

    let image_bytes = body
        .image
        .as_ref()
        .map(|b64| STANDARD.decode(b64))
        .transpose()
        .map_err(|_| ApiError::BadRequest("AI_INVALID_IMAGE"))?;

    let image_ref = image_bytes.as_deref();

    // Build full message list: history + current message
    let mut messages = body.history;
    messages.push(ChatMessage {
        role: "user".to_string(),
        content: body.message,
        image: None, // Current image is passed separately as raw bytes
    });

    debug!(
        plant_id = body.plant_id,
        has_image = image_ref.is_some(),
        message_count = messages.len(),
        "AI chat request"
    );

    let stream = provider
        .chat(&system_prompt, &messages, image_ref, &locale)
        .await
        .map_err(|e| {
            warn!(error = %e, "AI chat failed");
            ApiError::InternalError("AI_PROVIDER_FAILED")
        })?;

    let sse_stream = stream.map(|result| {
        let event = match result {
            Ok(delta) => Event::default().data(serde_json::json!({"delta": delta}).to_string()),
            Err(err) => {
                warn!("AI stream error: {err}");
                let code = "AI_STREAM_ERROR";
                Event::default().data(
                    serde_json::json!({
                        "error": { "code": code, "message": default_message(code) }
                    })
                    .to_string(),
                )
            }
        };
        Ok(event)
    });

    let done_event = tokio_stream::once(Ok(
        Event::default().data(serde_json::json!({"done": true}).to_string())
    ));

    let full_stream = sse_stream.chain(done_event);

    Ok(Sse::new(full_stream))
}

/// # Errors
/// Returns `ApiError::ServiceUnavailable` if the AI provider is not configured,
/// `ApiError::Validation` if the history is empty,
/// `ApiError::NotFound` if the plant does not exist, or
/// `ApiError::InternalError` if the AI provider call fails.
pub async fn summarize(
    State(state): State<AppState>,
    Json(body): Json<SummarizeRequest>,
) -> Result<Json<SummarizeResponse>, ApiError> {
    let provider = state
        .ai_provider
        .as_ref()
        .ok_or(ApiError::ServiceUnavailable("AI_NOT_CONFIGURED"))?;

    if body.history.is_empty() {
        return Err(ApiError::Validation("AI_HISTORY_EMPTY"));
    }

    let context = prompts::build_plant_context(&state.pool, body.plant_id).await?;
    let locale = get_locale(&state.pool).await;
    let system_prompt =
        prompts::build_summarize_system_prompt(&context.name, context.species.as_deref(), &locale);

    debug!(
        plant_id = body.plant_id,
        history_len = body.history.len(),
        "AI summarize request"
    );

    let summary = provider
        .summarize(&system_prompt, &body.history, &locale)
        .await
        .map_err(|e| {
            warn!(error = %e, "AI summarize failed");
            ApiError::InternalError("AI_PROVIDER_FAILED")
        })?;

    Ok(Json(SummarizeResponse { summary }))
}
