use axum::Json;
use axum::extract::{Multipart, Path, Query, State};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use tracing::{debug, info};

use super::error::{ApiError, JsonBody};
use super::plants::{PLANT_SELECT, Plant, PlantRow};
use crate::images::ImageError;
use crate::mqtt;
use crate::state::AppState;

const VALID_EVENT_TYPES: &[&str] = &[
    "watered",
    "fertilized",
    "repotted",
    "pruned",
    "custom",
    "ai-consultation",
];

#[derive(Serialize, sqlx::FromRow)]
pub struct CareEvent {
    pub id: i64,
    pub plant_id: i64,
    pub plant_name: String,
    pub event_type: String,
    pub notes: Option<String>,
    pub photo_url: Option<String>,
    pub occurred_at: String,
    pub created_at: String,
}

#[derive(Deserialize)]
pub struct CreateCareEvent {
    pub event_type: Option<String>,
    pub notes: Option<String>,
    pub occurred_at: Option<String>,
}

#[derive(Deserialize)]
pub struct GlobalCareQuery {
    pub limit: Option<i64>,
    pub before: Option<i64>,
    #[serde(rename = "type")]
    pub event_type: Option<String>,
}

#[derive(Serialize)]
pub struct CareEventsPage {
    pub events: Vec<CareEvent>,
    pub has_more: bool,
}

const CARE_EVENT_SELECT: &str = "SELECT ce.id, ce.plant_id, p.name AS plant_name, \
    ce.event_type, ce.notes, \
    CASE WHEN ce.photo_path IS NOT NULL THEN '/uploads/' || ce.photo_path END AS photo_url, \
    ce.occurred_at, ce.created_at \
    FROM care_events ce JOIN plants p ON ce.plant_id = p.id";

fn validate_event_type(event_type: &str) -> Result<(), ApiError> {
    if VALID_EVENT_TYPES.contains(&event_type) {
        Ok(())
    } else {
        Err(ApiError::Validation(format!(
            "Invalid event_type '{}'. Must be one of: {}",
            event_type,
            VALID_EVENT_TYPES.join(", ")
        )))
    }
}

async fn plant_exists(pool: &SqlitePool, id: i64) -> Result<(), ApiError> {
    let exists = sqlx::query_scalar::<_, i64>("SELECT id FROM plants WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    if exists.is_none() {
        return Err(ApiError::NotFound("Plant not found".to_string()));
    }
    Ok(())
}

async fn publish_plant_watering_mqtt(state: &AppState, plant_id: i64) {
    let Ok(row) = sqlx::query_as::<_, PlantRow>(&format!("{PLANT_SELECT} WHERE p.id = ?"))
        .bind(plant_id)
        .fetch_one(&state.pool)
        .await
    else {
        return;
    };
    let plant = Plant::from(row);
    mqtt::publish_state(
        state.mqtt_client.as_ref(),
        &state.mqtt_prefix,
        plant.id,
        &plant.watering_status,
    )
    .await;
    mqtt::publish_attributes(
        state.mqtt_client.as_ref(),
        &state.mqtt_prefix,
        plant.id,
        plant.last_watered.as_deref(),
        plant.next_due.as_deref(),
        plant.watering_interval_days,
    )
    .await;
}

/// # Errors
/// Returns `ApiError::NotFound` if the plant does not exist, or
/// `ApiError::BadRequest` on database failures.
pub async fn list_care_events(
    State(pool): State<SqlitePool>,
    Path(plant_id): Path<i64>,
) -> Result<Json<Vec<CareEvent>>, ApiError> {
    plant_exists(&pool, plant_id).await?;

    let query = format!("{CARE_EVENT_SELECT} WHERE ce.plant_id = ? ORDER BY ce.occurred_at DESC");
    let events = sqlx::query_as::<_, CareEvent>(&query)
        .bind(plant_id)
        .fetch_all(&pool)
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    Ok(Json(events))
}

/// # Errors
/// Returns `ApiError::NotFound` if the plant does not exist,
/// `ApiError::Validation` if `event_type` is missing or invalid, or
/// `ApiError::BadRequest` on database failures.
pub async fn create_care_event(
    State(state): State<AppState>,
    Path(plant_id): Path<i64>,
    JsonBody(body): JsonBody<CreateCareEvent>,
) -> Result<(StatusCode, Json<CareEvent>), ApiError> {
    plant_exists(&state.pool, plant_id).await?;

    let event_type = body
        .event_type
        .filter(|t| !t.trim().is_empty())
        .ok_or_else(|| ApiError::Validation("event_type is required".to_string()))?;

    validate_event_type(&event_type)?;

    let occurred_at = body
        .occurred_at
        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true));

    let id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO care_events (plant_id, event_type, notes, occurred_at) \
         VALUES (?, ?, ?, ?) RETURNING id",
    )
    .bind(plant_id)
    .bind(&event_type)
    .bind(&body.notes)
    .bind(&occurred_at)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    let query = format!("{CARE_EVENT_SELECT} WHERE ce.id = ?");
    let event = sqlx::query_as::<_, CareEvent>(&query)
        .bind(id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    if event_type == "watered" {
        publish_plant_watering_mqtt(&state, plant_id).await;
    }

    debug!(plant_id, event_type = %event_type, "Care event created");
    Ok((StatusCode::CREATED, Json(event)))
}

/// # Errors
/// Returns `ApiError::NotFound` if the care event does not exist, or
/// `ApiError::BadRequest` on database failures.
pub async fn delete_care_event(
    State(state): State<AppState>,
    Path((plant_id, event_id)): Path<(i64, i64)>,
) -> Result<StatusCode, ApiError> {
    // Read event type + photo before deleting
    let row = sqlx::query_as::<_, (String, Option<String>)>(
        "SELECT event_type, photo_path FROM care_events WHERE id = ? AND plant_id = ?",
    )
    .bind(event_id)
    .bind(plant_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| ApiError::BadRequest(e.to_string()))?
    .ok_or_else(|| ApiError::NotFound("Care event not found".to_string()))?;

    let (event_type, photo_path) = row;

    sqlx::query("DELETE FROM care_events WHERE id = ? AND plant_id = ?")
        .bind(event_id)
        .bind(plant_id)
        .execute(&state.pool)
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    if let Some(filename) = photo_path {
        state.image_store.delete(&filename).await;
    }

    if event_type == "watered" {
        publish_plant_watering_mqtt(&state, plant_id).await;
    }

    debug!(plant_id, event_id, event_type = %event_type, "Care event deleted");
    Ok(StatusCode::NO_CONTENT)
}

/// # Errors
/// Returns `ApiError::Validation` if event type filter is invalid, or
/// `ApiError::BadRequest` on database failures.
pub async fn list_all_care_events(
    State(pool): State<SqlitePool>,
    Query(params): Query<GlobalCareQuery>,
) -> Result<Json<CareEventsPage>, ApiError> {
    let limit = params.limit.unwrap_or(20).clamp(1, 100);
    let fetch_count = limit + 1;

    if let Some(ref event_type) = params.event_type {
        validate_event_type(event_type)?;
    }

    let mut query = String::from(CARE_EVENT_SELECT);
    let mut conditions: Vec<&str> = Vec::new();

    if params.before.is_some() {
        conditions.push("ce.id < ?");
    }
    if params.event_type.is_some() {
        conditions.push("ce.event_type = ?");
    }

    if !conditions.is_empty() {
        query.push_str(" WHERE ");
        query.push_str(&conditions.join(" AND "));
    }

    query.push_str(" ORDER BY ce.occurred_at DESC, ce.id DESC LIMIT ?");

    let mut q = sqlx::query_as::<_, CareEvent>(&query);
    if let Some(before) = params.before {
        q = q.bind(before);
    }
    if let Some(ref event_type) = params.event_type {
        q = q.bind(event_type);
    }
    q = q.bind(fetch_count);

    let mut events = q
        .fetch_all(&pool)
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    let has_more = events.len() > usize::try_from(limit).unwrap_or(0);
    if has_more {
        events.pop();
    }

    Ok(Json(CareEventsPage { events, has_more }))
}

// --- Care event photo handlers ---

/// # Errors
/// Returns `ApiError::NotFound` if the care event does not exist,
/// `ApiError::Validation` for invalid file types or oversized files, or
/// `ApiError::BadRequest` on multipart parsing or database failures.
pub async fn upload_care_event_photo(
    State(state): State<AppState>,
    Path((plant_id, event_id)): Path<(i64, i64)>,
    mut multipart: Multipart,
) -> Result<Json<CareEvent>, ApiError> {
    let current_photo = sqlx::query_scalar::<_, Option<String>>(
        "SELECT photo_path FROM care_events WHERE id = ? AND plant_id = ?",
    )
    .bind(event_id)
    .bind(plant_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| ApiError::BadRequest(e.to_string()))?
    .ok_or_else(|| ApiError::NotFound("Care event not found".to_string()))?;

    let field = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?
        .ok_or_else(|| ApiError::Validation("No file provided".to_string()))?;

    let content_type = field.content_type().unwrap_or("").to_string();
    let data = field
        .bytes()
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    let filename = state
        .image_store
        .save(&data, &content_type)
        .await
        .map_err(|e| match e {
            ImageError::InvalidContentType | ImageError::TooLarge => {
                ApiError::Validation(e.to_string())
            }
            ImageError::Io(_) => ApiError::BadRequest(format!("Failed to save file: {e}")),
        })?;

    if let Some(ref old_filename) = current_photo {
        state.image_store.delete(old_filename).await;
    }

    sqlx::query("UPDATE care_events SET photo_path = ? WHERE id = ? AND plant_id = ?")
        .bind(&filename)
        .bind(event_id)
        .bind(plant_id)
        .execute(&state.pool)
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    info!(plant_id, event_id, filename = %filename, "Care event photo uploaded");

    let query = format!("{CARE_EVENT_SELECT} WHERE ce.id = ?");
    let event = sqlx::query_as::<_, CareEvent>(&query)
        .bind(event_id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    Ok(Json(event))
}

/// # Errors
/// Returns `ApiError::NotFound` if the care event does not exist or has no photo, or
/// `ApiError::BadRequest` on database failures.
pub async fn delete_care_event_photo(
    State(state): State<AppState>,
    Path((plant_id, event_id)): Path<(i64, i64)>,
) -> Result<StatusCode, ApiError> {
    let photo_path = sqlx::query_scalar::<_, Option<String>>(
        "SELECT photo_path FROM care_events WHERE id = ? AND plant_id = ?",
    )
    .bind(event_id)
    .bind(plant_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| ApiError::BadRequest(e.to_string()))?
    .ok_or_else(|| ApiError::NotFound("Care event not found".to_string()))?;

    let filename =
        photo_path.ok_or_else(|| ApiError::NotFound("Care event has no photo".to_string()))?;

    sqlx::query("UPDATE care_events SET photo_path = NULL WHERE id = ? AND plant_id = ?")
        .bind(event_id)
        .bind(plant_id)
        .execute(&state.pool)
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    state.image_store.delete(&filename).await;

    info!(plant_id, event_id, "Care event photo deleted");
    Ok(StatusCode::NO_CONTENT)
}
