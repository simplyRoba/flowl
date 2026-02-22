#![allow(clippy::missing_errors_doc)]

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use tracing::debug;

use super::error::{ApiError, JsonBody};

#[derive(Serialize)]
pub struct Location {
    pub id: i64,
    pub name: String,
    pub plant_count: i64,
}

#[derive(sqlx::FromRow)]
struct LocationRow {
    id: i64,
    name: String,
    plant_count: i64,
}

#[derive(Deserialize)]
pub struct CreateLocation {
    pub name: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateLocation {
    pub name: Option<String>,
}

pub async fn list_locations(
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<Location>>, ApiError> {
    let rows = sqlx::query_as::<_, LocationRow>(
        "SELECT l.id, l.name, COUNT(p.id) AS plant_count \
         FROM locations l LEFT JOIN plants p ON p.location_id = l.id \
         GROUP BY l.id, l.name ORDER BY l.name",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    Ok(Json(
        rows.into_iter()
            .map(|r| Location {
                id: r.id,
                name: r.name,
                plant_count: r.plant_count,
            })
            .collect(),
    ))
}

pub async fn create_location(
    State(pool): State<SqlitePool>,
    JsonBody(body): JsonBody<CreateLocation>,
) -> Result<(StatusCode, Json<Location>), ApiError> {
    let name = body
        .name
        .filter(|n| !n.trim().is_empty())
        .ok_or_else(|| ApiError::Validation("Name is required".to_string()))?;
    let name = name.trim().to_string();

    // Check for duplicate
    let existing = sqlx::query_scalar::<_, i64>("SELECT id FROM locations WHERE name = ?")
        .bind(&name)
        .fetch_optional(&pool)
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    if existing.is_some() {
        return Err(ApiError::Conflict(format!(
            "Location '{name}' already exists"
        )));
    }

    let row = sqlx::query_as::<_, LocationRow>(
        "INSERT INTO locations (name) VALUES (?) RETURNING id, name, 0 AS plant_count",
    )
    .bind(&name)
    .fetch_one(&pool)
    .await
    .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    debug!(location_id = row.id, name = %row.name, "Location created");
    Ok((
        StatusCode::CREATED,
        Json(Location {
            id: row.id,
            name: row.name,
            plant_count: row.plant_count,
        }),
    ))
}

pub async fn update_location(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    JsonBody(body): JsonBody<UpdateLocation>,
) -> Result<Json<Location>, ApiError> {
    // Check existence
    let exists = sqlx::query_scalar::<_, i64>("SELECT id FROM locations WHERE id = ?")
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    if exists.is_none() {
        return Err(ApiError::NotFound("Location not found".to_string()));
    }

    let name = body
        .name
        .filter(|n| !n.trim().is_empty())
        .ok_or_else(|| ApiError::Validation("Name is required".to_string()))?;
    let name = name.trim().to_string();

    // Check for duplicate (different id)
    let duplicate =
        sqlx::query_scalar::<_, i64>("SELECT id FROM locations WHERE name = ? AND id != ?")
            .bind(&name)
            .bind(id)
            .fetch_optional(&pool)
            .await
            .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    if duplicate.is_some() {
        return Err(ApiError::Conflict(format!(
            "Location '{name}' already exists"
        )));
    }

    sqlx::query("UPDATE locations SET name = ? WHERE id = ?")
        .bind(&name)
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    // Get plant count for response
    let plant_count =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM plants WHERE location_id = ?")
            .bind(id)
            .fetch_one(&pool)
            .await
            .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    Ok(Json(Location {
        id,
        name,
        plant_count,
    }))
}

pub async fn delete_location(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    // Nullify plant references
    sqlx::query("UPDATE plants SET location_id = NULL WHERE location_id = ?")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    let result = sqlx::query("DELETE FROM locations WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound("Location not found".to_string()));
    }

    debug!(location_id = id, "Location deleted");
    Ok(StatusCode::NO_CONTENT)
}
