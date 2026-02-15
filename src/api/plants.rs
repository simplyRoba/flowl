#![allow(clippy::missing_errors_doc)]

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use super::error::{ApiError, JsonBody};
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
    pub light_needs: String,
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
    pub(crate) light_needs: String,
    pub(crate) notes: Option<String>,
    pub(crate) created_at: String,
    pub(crate) updated_at: String,
}

impl From<PlantRow> for Plant {
    fn from(row: PlantRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            species: row.species,
            icon: row.icon,
            photo_url: row.photo_path.map(|p| format!("/uploads/{p}")),
            location_id: row.location_id,
            location_name: row.location_name,
            watering_interval_days: row.watering_interval_days,
            light_needs: row.light_needs,
            notes: row.notes,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

pub(crate) const PLANT_SELECT: &str = "SELECT p.id, p.name, p.species, p.icon, p.photo_path, \
    p.location_id, l.name AS location_name, p.watering_interval_days, p.light_needs, \
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
    State(pool): State<SqlitePool>,
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

    let id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO plants (name, species, icon, location_id, watering_interval_days, light_needs, notes) \
         VALUES (?, ?, ?, ?, ?, ?, ?) RETURNING id",
    )
    .bind(&name)
    .bind(&body.species)
    .bind(&icon)
    .bind(body.location_id)
    .bind(watering_interval_days)
    .bind(&light_needs)
    .bind(&body.notes)
    .fetch_one(&pool)
    .await
    .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    let query = format!("{PLANT_SELECT} WHERE p.id = ?");
    let row = sqlx::query_as::<_, PlantRow>(&query)
        .bind(id)
        .fetch_one(&pool)
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    Ok((StatusCode::CREATED, Json(Plant::from(row))))
}

pub async fn update_plant(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    JsonBody(body): JsonBody<UpdatePlant>,
) -> Result<Json<Plant>, ApiError> {
    // Fetch current plant
    let current = sqlx::query_as::<_, PlantRow>(&format!("{PLANT_SELECT} WHERE p.id = ?"))
        .bind(id)
        .fetch_optional(&pool)
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
    let notes = body.notes.unwrap_or(current.notes);

    sqlx::query(
        "UPDATE plants SET name = ?, species = ?, icon = ?, location_id = ?, \
         watering_interval_days = ?, light_needs = ?, notes = ?, \
         updated_at = datetime('now') WHERE id = ?",
    )
    .bind(&name)
    .bind(&species)
    .bind(&icon)
    .bind(location_id)
    .bind(watering_interval_days)
    .bind(&light_needs)
    .bind(&notes)
    .bind(id)
    .execute(&pool)
    .await
    .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    let row = sqlx::query_as::<_, PlantRow>(&format!("{PLANT_SELECT} WHERE p.id = ?"))
        .bind(id)
        .fetch_one(&pool)
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    Ok(Json(Plant::from(row)))
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

    Ok(StatusCode::NO_CONTENT)
}
