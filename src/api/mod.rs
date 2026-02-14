pub mod error;
pub mod locations;
pub mod plants;

use axum::Router;
use axum::routing::{get, put};
use sqlx::SqlitePool;

pub fn router(pool: SqlitePool) -> Router {
    Router::new()
        .route("/plants", get(plants::list_plants).post(plants::create_plant))
        .route(
            "/plants/{id}",
            get(plants::get_plant)
                .put(plants::update_plant)
                .delete(plants::delete_plant),
        )
        .route(
            "/locations",
            get(locations::list_locations).post(locations::create_location),
        )
        .route(
            "/locations/{id}",
            put(locations::update_location).delete(locations::delete_location),
        )
        .with_state(pool)
}
