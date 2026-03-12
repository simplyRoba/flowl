use axum::extract::{FromRequest, Request};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

pub struct JsonBody<T>(pub T);

impl<S, T> FromRequest<S> for JsonBody<T>
where
    T: serde::de::DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match axum::Json::<T>::from_request(req, state).await {
            Ok(axum::Json(value)) => Ok(Self(value)),
            Err(_) => Err(ApiError::BadRequest("INVALID_REQUEST_BODY")),
        }
    }
}

pub enum ApiError {
    NotFound(&'static str),
    Validation(&'static str),
    Conflict(&'static str),
    BadRequest(&'static str),
    ServiceUnavailable(&'static str),
    InternalError(&'static str),
}

#[allow(clippy::needless_pass_by_value)]
pub fn db_error(e: sqlx::Error) -> ApiError {
    tracing::error!("Database error: {e}");
    ApiError::InternalError("INTERNAL_ERROR")
}

pub fn default_message(code: &str) -> &'static str {
    match code {
        // Generic
        "INTERNAL_ERROR" => "An internal error occurred",
        "INVALID_REQUEST_BODY" => "Invalid request body",

        // Plants
        "PLANT_NOT_FOUND" => "Plant not found",
        "PLANT_NAME_REQUIRED" => "Plant name is required",
        "PLANT_INVALID_LIGHT_NEEDS" => "Invalid value for light_needs",
        "PLANT_INVALID_DIFFICULTY" => "Invalid value for difficulty",
        "PLANT_INVALID_PET_SAFETY" => "Invalid value for pet_safety",
        "PLANT_INVALID_GROWTH_SPEED" => "Invalid value for growth_speed",
        "PLANT_INVALID_SOIL_TYPE" => "Invalid value for soil_type",
        "PLANT_INVALID_SOIL_MOISTURE" => "Invalid value for soil_moisture",
        "PLANT_INVALID_WATERING_INTERVAL" => "Watering interval must be between 1 and 365 days",

        // Care events
        "CARE_EVENT_NOT_FOUND" => "Care event not found",
        "CARE_EVENT_TYPE_REQUIRED" => "Event type is required",
        "CARE_EVENT_INVALID_TYPE" => "Invalid event type",

        // Locations
        "LOCATION_NOT_FOUND" => "Location not found",
        "LOCATION_NAME_REQUIRED" => "Location name is required",
        "LOCATION_ALREADY_EXISTS" => "A location with this name already exists",

        // Photos
        "PHOTO_NOT_FOUND" => "Photo not found",
        "PHOTO_NO_FILE" | "IMPORT_NO_FILE" => "No file provided",
        "PHOTO_INVALID_TYPE" => "Invalid image type",
        "PHOTO_TOO_LARGE" => "File is too large",
        "PHOTO_SAVE_FAILED" => "Failed to save photo",

        // Settings
        "SETTINGS_INVALID_THEME" => "Invalid theme value",
        "SETTINGS_INVALID_LOCALE" => "Invalid locale value",

        // Import
        "IMPORT_INVALID_ARCHIVE" => "Invalid ZIP archive",
        "IMPORT_INVALID_DATA" => "Invalid import data",
        "IMPORT_VERSION_MISMATCH" => "Incompatible export version",
        "IMPORT_INVALID_FILENAME" => "Invalid filename in archive",
        "IMPORT_FILE_TOO_LARGE" => "File in archive is too large",
        "IMPORT_VALIDATION_FAILED" => "Import data validation failed",

        // AI
        "AI_NOT_CONFIGURED" => "AI provider is not configured",
        "AI_PROVIDER_FAILED" => "AI provider request failed",
        "AI_STREAM_ERROR" => "AI response interrupted",
        "AI_INVALID_IMAGE" => "Invalid image data",
        "AI_HISTORY_EMPTY" => "Chat history is empty",

        // MQTT
        "MQTT_DISABLED" => "MQTT is disabled",
        "MQTT_UNAVAILABLE" => "MQTT is not connected",

        _ => "An unexpected error occurred",
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code) = match self {
            Self::NotFound(c) => (StatusCode::NOT_FOUND, c),
            Self::Validation(c) => (StatusCode::UNPROCESSABLE_ENTITY, c),
            Self::Conflict(c) => (StatusCode::CONFLICT, c),
            Self::BadRequest(c) => (StatusCode::BAD_REQUEST, c),
            Self::ServiceUnavailable(c) => (StatusCode::SERVICE_UNAVAILABLE, c),
            Self::InternalError(c) => (StatusCode::INTERNAL_SERVER_ERROR, c),
        };

        let message = default_message(code);
        let body = axum::Json(json!({ "code": code, "message": message }));
        (status, body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    #[test]
    fn default_message_returns_known_messages() {
        assert_eq!(default_message("PLANT_NOT_FOUND"), "Plant not found");
        assert_eq!(
            default_message("INTERNAL_ERROR"),
            "An internal error occurred"
        );
        assert_eq!(default_message("MQTT_DISABLED"), "MQTT is disabled");
    }

    #[test]
    fn default_message_returns_fallback_for_unknown_code() {
        assert_eq!(
            default_message("DOES_NOT_EXIST"),
            "An unexpected error occurred"
        );
    }

    #[test]
    fn into_response_sets_correct_status_and_body() {
        let response = ApiError::NotFound("PLANT_NOT_FOUND").into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let response = ApiError::Validation("PLANT_INVALID_DIFFICULTY").into_response();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

        let response = ApiError::Conflict("LOCATION_ALREADY_EXISTS").into_response();
        assert_eq!(response.status(), StatusCode::CONFLICT);

        let response = ApiError::BadRequest("INVALID_REQUEST_BODY").into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let response = ApiError::ServiceUnavailable("AI_NOT_CONFIGURED").into_response();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);

        let response = ApiError::InternalError("INTERNAL_ERROR").into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
