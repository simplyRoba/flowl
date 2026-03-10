use axum::Json;
use axum::extract::State;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use super::error::{ApiError, JsonBody, db_error};

const VALID_THEMES: &[&str] = &["light", "dark", "system"];
const VALID_LOCALES: &[&str] = &["en", "de", "es"];

#[derive(sqlx::FromRow, Serialize)]
pub struct UserSettings {
    pub theme: String,
    pub locale: String,
}

#[derive(Deserialize)]
pub struct UpdateSettings {
    pub theme: Option<String>,
    pub locale: Option<String>,
}

/// # Errors
/// Returns `ApiError::InternalError` on database failures.
pub async fn get_settings(State(pool): State<SqlitePool>) -> Result<Json<UserSettings>, ApiError> {
    let row =
        sqlx::query_as::<_, UserSettings>("SELECT theme, locale FROM user_settings WHERE id = 1")
            .fetch_one(&pool)
            .await
            .map_err(db_error)?;

    Ok(Json(row))
}

/// # Errors
/// Returns `ApiError::Validation` for invalid theme or locale values, or
/// `ApiError::InternalError` on database failures.
pub async fn update_settings(
    State(pool): State<SqlitePool>,
    JsonBody(body): JsonBody<UpdateSettings>,
) -> Result<Json<UserSettings>, ApiError> {
    if let Some(ref theme) = body.theme
        && !VALID_THEMES.contains(&theme.as_str())
    {
        return Err(ApiError::Validation("SETTINGS_INVALID_THEME"));
    }

    if let Some(ref locale) = body.locale
        && !VALID_LOCALES.contains(&locale.as_str())
    {
        return Err(ApiError::Validation("SETTINGS_INVALID_LOCALE"));
    }

    sqlx::query(
        "UPDATE user_settings SET theme = COALESCE(?, theme), locale = COALESCE(?, locale) WHERE id = 1",
    )
    .bind(&body.theme)
    .bind(&body.locale)
    .execute(&pool)
    .await
    .map_err(db_error)?;

    let row =
        sqlx::query_as::<_, UserSettings>("SELECT theme, locale FROM user_settings WHERE id = 1")
            .fetch_one(&pool)
            .await
            .map_err(db_error)?;

    Ok(Json(row))
}
