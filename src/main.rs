mod api;
mod config;
mod db;
mod embedded;
mod mqtt;
mod server;
mod state;

use std::path::PathBuf;

use state::AppState;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    let config = config::Config::from_env();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_new(&config.log_level).unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    info!("Starting flowl");

    let pool = db::create_pool(&config.db_path)
        .await
        .expect("Failed to create database pool");

    db::run_migrations(&pool)
        .await
        .expect("Failed to run database migrations");
    info!("Database ready at {}", config.db_path);

    let mqtt_handle = mqtt::connect(&config);
    let mqtt_client = Some(mqtt_handle.client.clone());
    info!(
        "MQTT client connecting to {}:{}",
        config.mqtt_host, config.mqtt_port
    );

    let upload_dir = PathBuf::from(&config.db_path)
        .parent()
        .map_or_else(|| PathBuf::from("uploads"), |p| p.join("uploads"));

    tokio::fs::create_dir_all(&upload_dir)
        .await
        .expect("Failed to create upload directory");
    info!("Upload directory at {}", upload_dir.display());

    let state = AppState {
        pool: pool.clone(),
        upload_dir,
        mqtt_client: mqtt_client.clone(),
        mqtt_prefix: config.mqtt_topic_prefix.clone(),
    };
    let router = server::router(state);

    let checker_handle = mqtt::spawn_state_checker(pool, mqtt_client, config.mqtt_topic_prefix);

    if let Err(e) = server::serve(router, config.port).await {
        error!("Server error: {e}");
    }

    info!("Shutting down");
    checker_handle.abort();
    mqtt_handle.disconnect().await;
}
