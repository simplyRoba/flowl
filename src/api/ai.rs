use axum::Json;
use axum::extract::State;
use serde::Serialize;

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
