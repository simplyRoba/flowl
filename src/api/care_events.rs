#![allow(clippy::missing_errors_doc)]

use std::fmt::Write;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use super::error::{ApiError, JsonBody};

const VALID_EVENT_TYPES: &[&str] = &["watered", "fertilized", "repotted", "pruned", "custom"];

#[derive(Serialize, sqlx::FromRow)]
pub struct CareEvent {
    pub id: i64,
    pub plant_id: i64,
    pub plant_name: String,
    pub event_type: String,
    pub notes: Option<String>,
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
    ce.event_type, ce.notes, ce.occurred_at, ce.created_at \
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

pub async fn create_care_event(
    State(pool): State<SqlitePool>,
    Path(plant_id): Path<i64>,
    JsonBody(body): JsonBody<CreateCareEvent>,
) -> Result<(StatusCode, Json<CareEvent>), ApiError> {
    plant_exists(&pool, plant_id).await?;

    let event_type = body
        .event_type
        .filter(|t| !t.trim().is_empty())
        .ok_or_else(|| ApiError::Validation("event_type is required".to_string()))?;

    validate_event_type(&event_type)?;

    let occurred_at = body.occurred_at.unwrap_or_else(|| {
        chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
    });

    let id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO care_events (plant_id, event_type, notes, occurred_at) \
         VALUES (?, ?, ?, ?) RETURNING id",
    )
    .bind(plant_id)
    .bind(&event_type)
    .bind(&body.notes)
    .bind(&occurred_at)
    .fetch_one(&pool)
    .await
    .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    let query = format!("{CARE_EVENT_SELECT} WHERE ce.id = ?");
    let event = sqlx::query_as::<_, CareEvent>(&query)
        .bind(id)
        .fetch_one(&pool)
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    Ok((StatusCode::CREATED, Json(event)))
}

pub async fn delete_care_event(
    State(pool): State<SqlitePool>,
    Path((plant_id, event_id)): Path<(i64, i64)>,
) -> Result<StatusCode, ApiError> {
    let result = sqlx::query("DELETE FROM care_events WHERE id = ? AND plant_id = ?")
        .bind(event_id)
        .bind(plant_id)
        .execute(&pool)
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound("Care event not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

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
    let mut conditions: Vec<String> = Vec::new();

    if let Some(before) = params.before {
        conditions.push(format!("ce.id < {before}"));
    }
    if let Some(ref event_type) = params.event_type {
        conditions.push(format!("ce.event_type = '{event_type}'"));
    }

    if !conditions.is_empty() {
        query.push_str(" WHERE ");
        query.push_str(&conditions.join(" AND "));
    }

    let _ = write!(
        query,
        " ORDER BY ce.occurred_at DESC, ce.id DESC LIMIT {fetch_count}"
    );

    let mut events = sqlx::query_as::<_, CareEvent>(&query)
        .fetch_all(&pool)
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    let has_more = events.len() > usize::try_from(limit).unwrap_or(0);
    if has_more {
        events.pop();
    }

    Ok(Json(CareEventsPage { events, has_more }))
}
