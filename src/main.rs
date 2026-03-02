mod ai;
mod api;
mod config;
mod db;
mod embedded;
mod images;
mod mqtt;
mod server;
mod state;

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use ai::openai::OpenAiProvider;
use ai::provider::AiProvider;
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

    let mqtt_prefix = config.mqtt_topic_prefix.clone();
    let (mqtt_handle, mqtt_connected, mqtt_needs_republish) = if config.mqtt_disabled {
        info!("FLOWL_MQTT_DISABLED set, skipping MQTT initialization");
        (None, None, None)
    } else {
        let connected = Arc::new(AtomicBool::new(false));
        let needs_republish = Arc::new(AtomicBool::new(false));
        let handle = mqtt::connect(&config, connected.clone(), needs_republish.clone());
        if handle.is_some() {
            info!(
                "MQTT client connecting to {}:{}",
                config.mqtt_host, config.mqtt_port
            );
        }
        (handle, Some(connected), Some(needs_republish))
    };
    let mqtt_client = mqtt_handle.as_ref().map(|h| h.client.clone());

    let upload_dir = PathBuf::from(&config.db_path)
        .parent()
        .map_or_else(|| PathBuf::from("uploads"), |p| p.join("uploads"));

    tokio::fs::create_dir_all(&upload_dir)
        .await
        .expect("Failed to create upload directory");
    info!("Upload directory at {}", upload_dir.display());

    let image_store = images::ImageStore::new(upload_dir);
    image_store.cleanup_orphans(&pool).await;
    image_store.generate_missing_thumbnails(&pool).await;

    let ai_provider: Option<Arc<dyn AiProvider>> = config.ai_api_key.as_ref().map(|key| {
        info!(
            "AI provider enabled (model: {}, base: {})",
            config.ai_model, config.ai_base_url
        );
        Arc::new(OpenAiProvider::new(
            key.clone(),
            config.ai_base_url.clone(),
            config.ai_model.clone(),
        )) as Arc<dyn AiProvider>
    });
    if ai_provider.is_none() {
        info!("AI provider disabled (no FLOWL_AI_API_KEY set)");
    }

    let state = AppState {
        pool: pool.clone(),
        image_store,
        mqtt_client: mqtt_client.clone(),
        mqtt_prefix: mqtt_prefix.clone(),
        mqtt_connected: mqtt_connected.clone(),
        mqtt_host: config.mqtt_host.clone(),
        mqtt_port: config.mqtt_port,
        mqtt_disabled: config.mqtt_disabled,
        ai_provider,
        ai_base_url: config.ai_base_url,
        ai_model: config.ai_model,
    };
    let router = server::router(state);

    let checker_handle =
        mqtt::spawn_state_checker(pool, mqtt_client.clone(), mqtt_prefix, mqtt_needs_republish);

    if let Err(e) = server::serve(router, config.port).await {
        error!("Server error: {e}");
    }

    info!("Shutting down");
    if let Some(handle) = checker_handle {
        handle.abort();
    }
    if let Some(handle) = mqtt_handle {
        handle.disconnect().await;
    }
}
