pub mod error;
pub mod locations;
pub mod photos;
pub mod plants;

use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::routing::{get, post, put};

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
