pub mod care_events;
pub mod error;
pub mod locations;
pub mod mqtt_repair;
pub mod mqtt_status;
pub mod photos;
pub mod plants;
pub mod stats;

use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::routing::{delete, get, post, put};

use crate::state::AppState;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route(
            "/plants",
            get(plants::list_plants).post(plants::create_plant),
        )
        .route(
            "/plants/{id}",
            get(plants::get_plant)
                .put(plants::update_plant)
                .delete(plants::delete_plant),
        )
        .route("/plants/{id}/water", post(plants::water_plant))
        .route(
            "/plants/{id}/care",
            get(care_events::list_care_events).post(care_events::create_care_event),
        )
        .route(
            "/plants/{id}/care/{event_id}",
            delete(care_events::delete_care_event),
        )
        .route("/care", get(care_events::list_all_care_events))
        .route("/stats", get(stats::get_stats))
        .route("/mqtt/status", get(mqtt_status::get_mqtt_status))
        .route("/mqtt/repair", post(mqtt_repair::post_mqtt_repair))
        .route(
            "/plants/{id}/photo",
            axum::routing::post(photos::upload_photo)
                .delete(photos::delete_photo)
                .layer(DefaultBodyLimit::max(10 * 1024 * 1024)),
        )
        .route(
            "/locations",
            get(locations::list_locations).post(locations::create_location),
        )
        .route(
            "/locations/{id}",
            put(locations::update_location).delete(locations::delete_location),
        )
        .with_state(state)
}
