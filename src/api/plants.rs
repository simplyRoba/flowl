#![allow(clippy::missing_errors_doc)]

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use super::error::{ApiError, JsonBody};
use crate::mqtt;
use crate::state::AppState;

#[allow(clippy::option_option)]
fn deserialize_nullable<'de, T, D>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    T: serde::Deserialize<'de>,
    D: serde::Deserializer<'de>,
{
    let value = Option::<T>::deserialize(deserializer)?;
    Ok(Some(value))
}

#[derive(Serialize)]
pub struct Plant {
    pub id: i64,
    pub name: String,
    pub species: Option<String>,
    pub icon: String,
    pub photo_url: Option<String>,
    pub location_id: Option<i64>,
    pub location_name: Option<String>,
    pub watering_interval_days: i64,
    pub watering_status: String,
    pub last_watered: Option<String>,
    pub next_due: Option<String>,
    pub light_needs: String,
    pub difficulty: Option<String>,
    pub pet_safety: Option<String>,
    pub growth_speed: Option<String>,
    pub soil_type: Option<String>,
    pub soil_moisture: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(sqlx::FromRow)]
pub(crate) struct PlantRow {
    pub(crate) id: i64,
    pub(crate) name: String,
    pub(crate) species: Option<String>,
    pub(crate) icon: String,
    pub(crate) photo_path: Option<String>,
    pub(crate) location_id: Option<i64>,
    pub(crate) location_name: Option<String>,
    pub(crate) watering_interval_days: i64,
    pub(crate) last_watered: Option<String>,
    pub(crate) light_needs: String,
    pub(crate) difficulty: Option<String>,
    pub(crate) pet_safety: Option<String>,
    pub(crate) growth_speed: Option<String>,
    pub(crate) soil_type: Option<String>,
    pub(crate) soil_moisture: Option<String>,
    pub(crate) notes: Option<String>,
    pub(crate) created_at: String,
    pub(crate) updated_at: String,
}

/// Compute watering status and next-due date from `last_watered` and interval.
///
/// Returns `(status, next_due)`. Status is one of `"ok"`, `"due"`, `"overdue"`.
pub fn compute_watering_status(
    last_watered: Option<&str>,
    interval_days: i64,
) -> (String, Option<String>) {
    let Some(lw) = last_watered else {
        return ("due".to_string(), None);
    };

    // Parse the date portion of the ISO 8601 datetime
    let Ok(lw_date) = lw.get(..10).unwrap_or(lw).parse::<NaiveDate>() else {
        return ("due".to_string(), None);
    };

    let next_due = lw_date + chrono::Days::new(interval_days.max(0).cast_unsigned());
    let today = chrono::Utc::now().date_naive();

    let status = if today > next_due {
        "overdue"
    } else if today >= next_due {
        "due"
    } else {
        "ok"
    };

    (status.to_string(), Some(next_due.to_string()))
}

const VALID_DIFFICULTY: &[&str] = &["easy", "moderate", "demanding"];
const VALID_PET_SAFETY: &[&str] = &["safe", "caution", "toxic"];
const VALID_GROWTH_SPEED: &[&str] = &["slow", "moderate", "fast"];
const VALID_SOIL_TYPE: &[&str] = &["standard", "cactus-mix", "orchid-bark", "peat-moss"];
const VALID_SOIL_MOISTURE: &[&str] = &["dry", "moderate", "moist"];

pub fn validate_care_info(
    field: &str,
    value: Option<&str>,
    allowed: &[&str],
) -> Result<(), ApiError> {
    if let Some(v) = value
        && !allowed.contains(&v)
    {
        return Err(ApiError::Validation(format!(
            "Invalid value for {field}: \"{v}\""
        )));
    }
    Ok(())
}

fn validate_all_care_info(
    difficulty: Option<&str>,
    pet_safety: Option<&str>,
    growth_speed: Option<&str>,
    soil_type: Option<&str>,
    soil_moisture: Option<&str>,
) -> Result<(), ApiError> {
    validate_care_info("difficulty", difficulty, VALID_DIFFICULTY)?;
    validate_care_info("pet_safety", pet_safety, VALID_PET_SAFETY)?;
    validate_care_info("growth_speed", growth_speed, VALID_GROWTH_SPEED)?;
    validate_care_info("soil_type", soil_type, VALID_SOIL_TYPE)?;
    validate_care_info("soil_moisture", soil_moisture, VALID_SOIL_MOISTURE)?;
    Ok(())
}

impl From<PlantRow> for Plant {
    fn from(row: PlantRow) -> Self {
        let (watering_status, next_due) =
            compute_watering_status(row.last_watered.as_deref(), row.watering_interval_days);

        Self {
            id: row.id,
            name: row.name,
            species: row.species,
            icon: row.icon,
            photo_url: row.photo_path.map(|p| format!("/uploads/{p}")),
            location_id: row.location_id,
            location_name: row.location_name,
            watering_interval_days: row.watering_interval_days,
            watering_status,
            last_watered: row.last_watered,
            next_due,
            light_needs: row.light_needs,
            difficulty: row.difficulty,
            pet_safety: row.pet_safety,
            growth_speed: row.growth_speed,
            soil_type: row.soil_type,
            soil_moisture: row.soil_moisture,
            notes: row.notes,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

pub(crate) const PLANT_SELECT: &str = "SELECT p.id, p.name, p.species, p.icon, p.photo_path, \
    p.location_id, l.name AS location_name, p.watering_interval_days, \
    (SELECT MAX(occurred_at) FROM care_events WHERE plant_id = p.id AND event_type = 'watered') AS last_watered, \
    p.light_needs, p.difficulty, p.pet_safety, p.growth_speed, p.soil_type, p.soil_moisture, \
    p.notes, p.created_at, p.updated_at \
    FROM plants p LEFT JOIN locations l ON p.location_id = l.id";

#[derive(Deserialize)]
pub struct CreatePlant {
    pub name: Option<String>,
    pub species: Option<String>,
    pub icon: Option<String>,
    pub location_id: Option<i64>,
    pub watering_interval_days: Option<i64>,
    pub light_needs: Option<String>,
    pub difficulty: Option<String>,
    pub pet_safety: Option<String>,
    pub growth_speed: Option<String>,
    pub soil_type: Option<String>,
    pub soil_moisture: Option<String>,
    pub notes: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdatePlant {
    pub name: Option<String>,
    #[allow(clippy::option_option)]
    #[serde(default, deserialize_with = "deserialize_nullable")]
    pub species: Option<Option<String>>,
    pub icon: Option<String>,
    #[allow(clippy::option_option)]
    #[serde(default, deserialize_with = "deserialize_nullable")]
    pub location_id: Option<Option<i64>>,
    pub watering_interval_days: Option<i64>,
    pub light_needs: Option<String>,
    #[allow(clippy::option_option)]
    #[serde(default, deserialize_with = "deserialize_nullable")]
    pub difficulty: Option<Option<String>>,
    #[allow(clippy::option_option)]
    #[serde(default, deserialize_with = "deserialize_nullable")]
    pub pet_safety: Option<Option<String>>,
    #[allow(clippy::option_option)]
    #[serde(default, deserialize_with = "deserialize_nullable")]
    pub growth_speed: Option<Option<String>>,
    #[allow(clippy::option_option)]
    #[serde(default, deserialize_with = "deserialize_nullable")]
    pub soil_type: Option<Option<String>>,
    #[allow(clippy::option_option)]
    #[serde(default, deserialize_with = "deserialize_nullable")]
    pub soil_moisture: Option<Option<String>>,
    #[allow(clippy::option_option)]
    #[serde(default, deserialize_with = "deserialize_nullable")]
    pub notes: Option<Option<String>>,
}

pub async fn list_plants(State(pool): State<SqlitePool>) -> Result<Json<Vec<Plant>>, ApiError> {
    let query = format!("{PLANT_SELECT} ORDER BY p.name");
    let rows = sqlx::query_as::<_, PlantRow>(&query)
        .fetch_all(&pool)
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    Ok(Json(rows.into_iter().map(Plant::from).collect()))
}

pub async fn get_plant(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<Plant>, ApiError> {
    let query = format!("{PLANT_SELECT} WHERE p.id = ?");
    let row = sqlx::query_as::<_, PlantRow>(&query)
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?
        .ok_or_else(|| ApiError::NotFound("Plant not found".to_string()))?;

    Ok(Json(Plant::from(row)))
}

pub async fn create_plant(
    State(state): State<AppState>,
    JsonBody(body): JsonBody<CreatePlant>,
) -> Result<(StatusCode, Json<Plant>), ApiError> {
    let name = body
        .name
        .filter(|n| !n.trim().is_empty())
        .ok_or_else(|| ApiError::Validation("Name is required".to_string()))?;
    let name = name.trim().to_string();

    let icon = body
        .icon
        .filter(|i| !i.trim().is_empty())
        .unwrap_or_else(|| "\u{1fab4}".to_string());
    let watering_interval_days = body.watering_interval_days.unwrap_or(7);
    let light_needs = body
        .light_needs
        .filter(|l| !l.trim().is_empty())
        .unwrap_or_else(|| "indirect".to_string());

    validate_all_care_info(
        body.difficulty.as_deref(),
        body.pet_safety.as_deref(),
        body.growth_speed.as_deref(),
        body.soil_type.as_deref(),
        body.soil_moisture.as_deref(),
    )?;

    let id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO plants (name, species, icon, location_id, watering_interval_days, light_needs, \
         difficulty, pet_safety, growth_speed, soil_type, soil_moisture, notes) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING id",
    )
    .bind(&name)
    .bind(&body.species)
    .bind(&icon)
    .bind(body.location_id)
    .bind(watering_interval_days)
    .bind(&light_needs)
    .bind(&body.difficulty)
    .bind(&body.pet_safety)
    .bind(&body.growth_speed)
    .bind(&body.soil_type)
    .bind(&body.soil_moisture)
    .bind(&body.notes)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    let query = format!("{PLANT_SELECT} WHERE p.id = ?");
    let row = sqlx::query_as::<_, PlantRow>(&query)
        .bind(id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    let plant = Plant::from(row);

    mqtt::publish_discovery(
        state.mqtt_client.as_ref(),
        &state.mqtt_prefix,
        plant.id,
        &plant.name,
    )
    .await;
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

    Ok((StatusCode::CREATED, Json(plant)))
}

pub async fn update_plant(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    JsonBody(body): JsonBody<UpdatePlant>,
) -> Result<Json<Plant>, ApiError> {
    // Fetch current plant
    let current = sqlx::query_as::<_, PlantRow>(&format!("{PLANT_SELECT} WHERE p.id = ?"))
        .bind(id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?
        .ok_or_else(|| ApiError::NotFound("Plant not found".to_string()))?;

    let name = body.name.unwrap_or(current.name);
    let species = body.species.unwrap_or(current.species);
    let icon = body.icon.unwrap_or(current.icon);
    let location_id = body.location_id.unwrap_or(current.location_id);
    let watering_interval_days = body
        .watering_interval_days
        .unwrap_or(current.watering_interval_days);
    let light_needs = body.light_needs.unwrap_or(current.light_needs);
    let difficulty = body.difficulty.unwrap_or(current.difficulty);
    let pet_safety = body.pet_safety.unwrap_or(current.pet_safety);
    let growth_speed = body.growth_speed.unwrap_or(current.growth_speed);
    let soil_type = body.soil_type.unwrap_or(current.soil_type);
    let soil_moisture = body.soil_moisture.unwrap_or(current.soil_moisture);
    let notes = body.notes.unwrap_or(current.notes);

    validate_all_care_info(
        difficulty.as_deref(),
        pet_safety.as_deref(),
        growth_speed.as_deref(),
        soil_type.as_deref(),
        soil_moisture.as_deref(),
    )?;

    sqlx::query(
        "UPDATE plants SET name = ?, species = ?, icon = ?, location_id = ?, \
         watering_interval_days = ?, light_needs = ?, \
         difficulty = ?, pet_safety = ?, growth_speed = ?, soil_type = ?, \
         soil_moisture = ?, notes = ?, updated_at = datetime('now') WHERE id = ?",
    )
    .bind(&name)
    .bind(&species)
    .bind(&icon)
    .bind(location_id)
    .bind(watering_interval_days)
    .bind(&light_needs)
    .bind(&difficulty)
    .bind(&pet_safety)
    .bind(&growth_speed)
    .bind(&soil_type)
    .bind(&soil_moisture)
    .bind(&notes)
    .bind(id)
    .execute(&state.pool)
    .await
    .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    let row = sqlx::query_as::<_, PlantRow>(&format!("{PLANT_SELECT} WHERE p.id = ?"))
        .bind(id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    let plant = Plant::from(row);

    mqtt::publish_discovery(
        state.mqtt_client.as_ref(),
        &state.mqtt_prefix,
        plant.id,
        &plant.name,
    )
    .await;
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

    Ok(Json(plant))
}

pub async fn water_plant(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Plant>, ApiError> {
    // Verify the plant exists
    let result = sqlx::query("UPDATE plants SET updated_at = datetime('now') WHERE id = ?")
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound("Plant not found".to_string()));
    }

    // Record the watering care event — last_watered is computed from this
    sqlx::query(
        "INSERT INTO care_events (plant_id, event_type, occurred_at) VALUES (?, 'watered', strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))",
    )
    .bind(id)
    .execute(&state.pool)
    .await
    .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    let row = sqlx::query_as::<_, PlantRow>(&format!("{PLANT_SELECT} WHERE p.id = ?"))
        .bind(id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

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

    Ok(Json(plant))
}

pub async fn delete_plant(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    // Check for photo to clean up
    let photo_path =
        sqlx::query_scalar::<_, Option<String>>("SELECT photo_path FROM plants WHERE id = ?")
            .bind(id)
            .fetch_optional(&state.pool)
            .await
            .map_err(|e| ApiError::BadRequest(e.to_string()))?
            .ok_or_else(|| ApiError::NotFound("Plant not found".to_string()))?;

    let result = sqlx::query("DELETE FROM plants WHERE id = ?")
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound("Plant not found".to_string()));
    }

    // Delete photo file if exists
    if let Some(filename) = photo_path {
        let file_path = state.upload_dir.join(&filename);
        let _ = tokio::fs::remove_file(&file_path).await;
    }

    mqtt::remove_plant(state.mqtt_client.as_ref(), &state.mqtt_prefix, id).await;

    Ok(StatusCode::NO_CONTENT)
}
