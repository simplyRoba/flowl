#![allow(clippy::missing_errors_doc)]

use std::sync::atomic::Ordering;

use axum::Json;
use axum::extract::State;
use serde::Serialize;

use crate::state::AppState;

#[derive(Serialize)]
pub struct MqttStatus {
    pub status: &'static str,
    pub broker: Option<String>,
    pub topic_prefix: Option<String>,
}

pub async fn get_mqtt_status(State(state): State<AppState>) -> Json<MqttStatus> {
    if state.mqtt_disabled {
        return Json(MqttStatus {
            status: "disabled",
            broker: None,
            topic_prefix: None,
        });
    }

    let connected = state
        .mqtt_connected
        .as_ref()
        .is_some_and(|b| b.load(Ordering::Relaxed));

    Json(MqttStatus {
        status: if connected {
            "connected"
        } else {
            "disconnected"
        },
        broker: Some(format!("{}:{}", state.mqtt_host, state.mqtt_port)),
        topic_prefix: Some(state.mqtt_prefix.clone()),
    })
}
