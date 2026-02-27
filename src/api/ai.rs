#![allow(clippy::missing_errors_doc)]

use axum::Json;
use axum::extract::{Multipart, State};
use axum::response::sse::{Event, Sse};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tokio_stream::StreamExt;
use tracing::{debug, warn};

use super::error::ApiError;
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

// --- Plant context builder ---

#[derive(Serialize)]
struct PlantContext {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    species: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    location_name: Option<String>,
    light_needs: String,
    watering_interval_days: i64,
    watering_status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_watered: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    difficulty: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pet_safety: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    growth_speed: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    soil_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    soil_moisture: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    notes: Option<String>,
    recent_care_events: Vec<CareEventContext>,
}

#[derive(Serialize)]
struct CareEventContext {
    event_type: String,
    date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    notes: Option<String>,
}

#[derive(sqlx::FromRow)]
struct PlantContextRow {
    name: String,
    species: Option<String>,
    location_name: Option<String>,
    light_needs: String,
    watering_interval_days: i64,
    last_watered: Option<String>,
    difficulty: Option<String>,
    pet_safety: Option<String>,
    growth_speed: Option<String>,
    soil_type: Option<String>,
    soil_moisture: Option<String>,
    notes: Option<String>,
}

#[derive(sqlx::FromRow)]
struct CareEventRow {
    event_type: String,
    occurred_at: String,
    notes: Option<String>,
}

async fn build_plant_context(pool: &SqlitePool, plant_id: i64) -> Result<PlantContext, ApiError> {
    let row = sqlx::query_as::<_, PlantContextRow>(
        "SELECT p.name, p.species, l.name AS location_name, p.light_needs, \
         p.watering_interval_days, \
         (SELECT MAX(occurred_at) FROM care_events WHERE plant_id = p.id AND event_type = 'watered') AS last_watered, \
         p.difficulty, p.pet_safety, p.growth_speed, p.soil_type, p.soil_moisture, p.notes \
         FROM plants p LEFT JOIN locations l ON p.location_id = l.id \
         WHERE p.id = ?",
    )
    .bind(plant_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| ApiError::InternalError(e.to_string()))?
    .ok_or_else(|| ApiError::NotFound("Plant not found".to_string()))?;

    let (watering_status, _) = crate::api::plants::compute_watering_status(
        row.last_watered.as_deref(),
        row.watering_interval_days,
    );

    let events = sqlx::query_as::<_, CareEventRow>(
        "SELECT event_type, occurred_at, notes FROM care_events \
         WHERE plant_id = ? ORDER BY occurred_at DESC LIMIT 20",
    )
    .bind(plant_id)
    .fetch_all(pool)
    .await
    .map_err(|e| ApiError::InternalError(e.to_string()))?;

    let recent_care_events = events
        .into_iter()
        .map(|e| CareEventContext {
            event_type: e.event_type,
            date: e
                .occurred_at
                .get(..10)
                .unwrap_or(&e.occurred_at)
                .to_string(),
            notes: e.notes,
        })
        .collect();

    Ok(PlantContext {
        name: row.name,
        species: row.species,
        location_name: row.location_name,
        light_needs: row.light_needs,
        watering_interval_days: row.watering_interval_days,
        watering_status,
        last_watered: row
            .last_watered
            .as_deref()
            .map(|d| d.get(..10).unwrap_or(d).to_string()),
        difficulty: row.difficulty,
        pet_safety: row.pet_safety,
        growth_speed: row.growth_speed,
        soil_type: row.soil_type,
        soil_moisture: row.soil_moisture,
        notes: row.notes,
        recent_care_events,
    })
}

// --- System prompt builders ---

fn build_chat_system_prompt(context: &PlantContext, locale: &str) -> String {
    let context_json = serde_json::to_string_pretty(context).unwrap_or_else(|_| "{}".to_string());

    let lang_instruction = locale_instruction(locale);

    format!(
        "You are flowl, a plant care assistant embedded in a self-hosted plant management app. \
         You help users with plant health diagnosis, watering advice, and general care questions.\n\n\
         You have access to the user's plant data and recent care history (provided below as JSON). \
         Use this context to give specific, personalized advice rather than generic answers.\n\n\
         Be friendly and casual — use informal language (e.g. \"du\" in German, \"tú\" in Spanish). \
         Be concise and practical — 2-4 short paragraphs max. \
         Use plain text only — no markdown, no bold, no headers, no code blocks. \
         Use simple dashes (- ) for lists. \
         If you're unsure about a diagnosis, say so and suggest what to look for.\n\n\
         Do not answer questions unrelated to plant care.\n\n\
         Plant context:\n{context_json}\n\n\
         {lang_instruction}"
    )
}

fn build_summarize_system_prompt(plant_name: &str, species: Option<&str>, locale: &str) -> String {
    let species_part = species.map(|s| format!(" ({s})")).unwrap_or_default();

    let lang_instruction = locale_instruction(locale);

    format!(
        "Summarize the following conversation about the plant \"{plant_name}\"{species_part} \
         into a 1-3 sentence care journal note. Focus on diagnoses, advice given, \
         and actions recommended. Return your answer as JSON: {{\"summary\": \"...\"}}\n\n\
         {lang_instruction}"
    )
}

fn locale_instruction(locale: &str) -> String {
    match locale {
        "en" => "Respond in English.".to_string(),
        "de" => "Respond in German.".to_string(),
        "es" => "Respond in Spanish.".to_string(),
        _ => format!("Respond in the language with locale code \"{locale}\"."),
    }
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

pub async fn chat(
    State(state): State<AppState>,
    Json(body): Json<ChatRequest>,
) -> Result<Sse<impl tokio_stream::Stream<Item = Result<Event, std::convert::Infallible>>>, ApiError>
{
    let provider = state
        .ai_provider
        .as_ref()
        .ok_or_else(|| ApiError::ServiceUnavailable("AI provider is not configured".to_string()))?;

    let context = build_plant_context(&state.pool, body.plant_id).await?;
    let locale = get_locale(&state.pool).await;
    let system_prompt = build_chat_system_prompt(&context, &locale);

    let image_bytes = body
        .image
        .as_ref()
        .map(|b64| STANDARD.decode(b64))
        .transpose()
        .map_err(|e| ApiError::BadRequest(format!("Invalid base64 image: {e}")))?;

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
            ApiError::InternalError(format!("AI chat failed: {e}"))
        })?;

    let sse_stream = stream.map(|result| {
        let event = match result {
            Ok(delta) => Event::default().data(serde_json::json!({"delta": delta}).to_string()),
            Err(err) => Event::default().data(serde_json::json!({"error": err}).to_string()),
        };
        Ok(event)
    });

    let done_event = tokio_stream::once(Ok(
        Event::default().data(serde_json::json!({"done": true}).to_string())
    ));

    let full_stream = sse_stream.chain(done_event);

    Ok(Sse::new(full_stream))
}

pub async fn summarize(
    State(state): State<AppState>,
    Json(body): Json<SummarizeRequest>,
) -> Result<Json<SummarizeResponse>, ApiError> {
    let provider = state
        .ai_provider
        .as_ref()
        .ok_or_else(|| ApiError::ServiceUnavailable("AI provider is not configured".to_string()))?;

    if body.history.is_empty() {
        return Err(ApiError::Validation(
            "History must contain at least one message".to_string(),
        ));
    }

    let context = build_plant_context(&state.pool, body.plant_id).await?;
    let locale = get_locale(&state.pool).await;
    let system_prompt =
        build_summarize_system_prompt(&context.name, context.species.as_deref(), &locale);

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
            ApiError::InternalError(format!("AI summarize failed: {e}"))
        })?;

    Ok(Json(SummarizeResponse { summary }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_system_prompt_contains_plant_context() {
        let context = PlantContext {
            name: "Monstera".to_string(),
            species: Some("Monstera deliciosa".to_string()),
            location_name: Some("Living Room".to_string()),
            light_needs: "indirect".to_string(),
            watering_interval_days: 10,
            watering_status: "ok".to_string(),
            last_watered: Some("2026-02-20".to_string()),
            difficulty: Some("easy".to_string()),
            pet_safety: Some("toxic".to_string()),
            growth_speed: Some("moderate".to_string()),
            soil_type: Some("standard".to_string()),
            soil_moisture: Some("moderate".to_string()),
            notes: Some("Bought from nursery".to_string()),
            recent_care_events: vec![CareEventContext {
                event_type: "watered".to_string(),
                date: "2026-02-20".to_string(),
                notes: None,
            }],
        };
        let prompt = build_chat_system_prompt(&context, "en");
        assert!(prompt.contains("flowl"));
        assert!(prompt.contains("Monstera"));
        assert!(prompt.contains("Monstera deliciosa"));
        assert!(prompt.contains("Living Room"));
        assert!(prompt.contains("watered"));
        assert!(prompt.contains("Respond in English"));
    }

    #[test]
    fn chat_system_prompt_locale_german() {
        let context = PlantContext {
            name: "Fern".to_string(),
            species: None,
            location_name: None,
            light_needs: "low".to_string(),
            watering_interval_days: 3,
            watering_status: "due".to_string(),
            last_watered: None,
            difficulty: None,
            pet_safety: None,
            growth_speed: None,
            soil_type: None,
            soil_moisture: None,
            notes: None,
            recent_care_events: vec![],
        };
        let prompt = build_chat_system_prompt(&context, "de");
        assert!(prompt.contains("Respond in German"));
    }

    #[test]
    fn chat_system_prompt_no_optional_fields() {
        let context = PlantContext {
            name: "Unknown".to_string(),
            species: None,
            location_name: None,
            light_needs: "indirect".to_string(),
            watering_interval_days: 7,
            watering_status: "ok".to_string(),
            last_watered: None,
            difficulty: None,
            pet_safety: None,
            growth_speed: None,
            soil_type: None,
            soil_moisture: None,
            notes: None,
            recent_care_events: vec![],
        };
        let prompt = build_chat_system_prompt(&context, "en");
        assert!(prompt.contains("Unknown"));
        assert!(prompt.contains("recent_care_events"));
    }

    #[test]
    fn summarize_system_prompt_with_species() {
        let prompt = build_summarize_system_prompt("Monstera", Some("Monstera deliciosa"), "en");
        assert!(prompt.contains("Monstera"));
        assert!(prompt.contains("Monstera deliciosa"));
        assert!(prompt.contains("1-3 sentence"));
        assert!(prompt.contains("Respond in English"));
    }

    #[test]
    fn summarize_system_prompt_without_species() {
        let prompt = build_summarize_system_prompt("My Plant", None, "es");
        assert!(prompt.contains("My Plant"));
        assert!(!prompt.contains("()"));
        assert!(prompt.contains("Respond in Spanish"));
    }

    #[test]
    fn locale_instruction_known_locales() {
        assert_eq!(locale_instruction("en"), "Respond in English.");
        assert_eq!(locale_instruction("de"), "Respond in German.");
        assert_eq!(locale_instruction("es"), "Respond in Spanish.");
    }

    #[test]
    fn locale_instruction_unknown_locale() {
        let result = locale_instruction("fr");
        assert!(result.contains("fr"));
    }
}
