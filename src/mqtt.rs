use std::collections::HashMap;

use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use serde_json::json;
use sqlx::SqlitePool;
use tokio::task::JoinHandle;
use tracing::{info, warn};

use crate::api::plants::compute_watering_status;
use crate::config::Config;

pub struct MqttHandle {
    pub client: AsyncClient,
    task: JoinHandle<()>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    #[tokio::test]
    async fn spawn_state_checker_skips_when_mqtt_disabled() {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .expect("Failed to create in-memory pool");

        let handle = spawn_state_checker(pool, None, "flowl".to_string());
        assert!(handle.is_none());
    }
}

impl MqttHandle {
    pub async fn disconnect(self) {
        if let Err(e) = self.client.disconnect().await {
            warn!("MQTT disconnect error: {e}");
        }
        self.task.abort();
    }
}

pub fn connect(config: &Config) -> Option<MqttHandle> {
    if config.mqtt_disabled {
        info!("FLOWL_MQTT_DISABLED=true, skipping MQTT client setup");
        return None;
    }
    let client_id = format!("{}-{}", config.mqtt_topic_prefix, std::process::id());
    let mut options = MqttOptions::new(&client_id, &config.mqtt_host, config.mqtt_port);
    options.set_keep_alive(std::time::Duration::from_secs(30));

    let (client, mut event_loop) = AsyncClient::new(options, 10);

    let task = tokio::spawn(async move {
        loop {
            match event_loop.poll().await {
                Ok(Event::Incoming(Packet::ConnAck(_))) => {
                    info!("MQTT connected");
                }
                Ok(_) => {}
                Err(e) => {
                    warn!("MQTT connection error: {e}");
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
            }
        }
    });

    Some(MqttHandle { client, task })
}

/// Publish HA auto-discovery config for a plant sensor entity.
pub async fn publish_discovery(
    client: Option<&AsyncClient>,
    prefix: &str,
    plant_id: i64,
    plant_name: &str,
) {
    let Some(client) = client else { return };

    let topic = format!("homeassistant/sensor/{prefix}_plant_{plant_id}/config");
    let payload = json!({
        "name": plant_name,
        "unique_id": format!("{prefix}_plant_{plant_id}"),
        "state_topic": format!("{prefix}/plant/{plant_id}/state"),
        "json_attributes_topic": format!("{prefix}/plant/{plant_id}/attributes"),
        "icon": "mdi:flower",
        "device": {
            "identifiers": [prefix],
            "name": prefix,
            "manufacturer": "flowl"
        }
    });

    if let Err(e) = client
        .publish(&topic, QoS::AtLeastOnce, true, payload.to_string())
        .await
    {
        warn!("MQTT publish discovery error for plant {plant_id}: {e}");
    }
}

/// Publish watering state (`ok`, `due`, `overdue`) to the plant's state topic.
pub async fn publish_state(
    client: Option<&AsyncClient>,
    prefix: &str,
    plant_id: i64,
    status: &str,
) {
    let Some(client) = client else { return };

    let topic = format!("{prefix}/plant/{plant_id}/state");

    if let Err(e) = client.publish(&topic, QoS::AtLeastOnce, true, status).await {
        warn!("MQTT publish state error for plant {plant_id}: {e}");
    }
}

/// Publish watering attributes (`next_due`, `last_watered`, interval) to the plant's attributes topic.
pub async fn publish_attributes(
    client: Option<&AsyncClient>,
    prefix: &str,
    plant_id: i64,
    last_watered: Option<&str>,
    next_due: Option<&str>,
    interval_days: i64,
) {
    let Some(client) = client else { return };

    let topic = format!("{prefix}/plant/{plant_id}/attributes");
    let payload = json!({
        "last_watered": last_watered,
        "next_due": next_due,
        "watering_interval_days": interval_days,
    });

    if let Err(e) = client
        .publish(&topic, QoS::AtLeastOnce, true, payload.to_string())
        .await
    {
        warn!("MQTT publish attributes error for plant {plant_id}: {e}");
    }
}

/// Remove a plant from HA by publishing empty retained payloads to its topics.
pub async fn remove_plant(client: Option<&AsyncClient>, prefix: &str, plant_id: i64) {
    let Some(client) = client else { return };

    let topics = [
        format!("homeassistant/sensor/{prefix}_plant_{plant_id}/config"),
        format!("{prefix}/plant/{plant_id}/state"),
        format!("{prefix}/plant/{plant_id}/attributes"),
    ];

    for topic in &topics {
        if let Err(e) = client
            .publish(topic, QoS::AtLeastOnce, true, Vec::<u8>::new())
            .await
        {
            warn!("MQTT remove error for plant {plant_id} on {topic}: {e}");
        }
    }
}

#[derive(sqlx::FromRow)]
struct CheckerRow {
    id: i64,
    name: String,
    watering_interval_days: i64,
    last_watered: Option<String>,
}

/// Spawn a background task that checks all plants every 60 seconds and publishes
/// state transitions to MQTT. On first run, publishes discovery configs for all plants
/// when MQTT is enabled.
pub fn spawn_state_checker(
    pool: SqlitePool,
    client: Option<AsyncClient>,
    prefix: String,
) -> Option<JoinHandle<()>> {
    if client.is_none() {
        info!("MQTT disabled, skipping background state checker");
        return None;
    }

    Some(tokio::spawn(async move {
        let mut cache: HashMap<i64, String> = HashMap::new();
        let mut first_run = true;

        loop {
            match sqlx::query_as::<_, CheckerRow>(
                "SELECT id, name, watering_interval_days, last_watered FROM plants",
            )
            .fetch_all(&pool)
            .await
            {
                Ok(rows) => {
                    for row in &rows {
                        let (status, next_due) = compute_watering_status(
                            row.last_watered.as_deref(),
                            row.watering_interval_days,
                        );

                        if first_run {
                            publish_discovery(client.as_ref(), &prefix, row.id, &row.name).await;
                        }

                        let changed = cache.get(&row.id).is_none_or(|prev| *prev != status);
                        if changed || first_run {
                            publish_state(client.as_ref(), &prefix, row.id, &status).await;
                            publish_attributes(
                                client.as_ref(),
                                &prefix,
                                row.id,
                                row.last_watered.as_deref(),
                                next_due.as_deref(),
                                row.watering_interval_days,
                            )
                            .await;
                            cache.insert(row.id, status);
                        }
                    }

                    // Remove cached entries for deleted plants
                    cache.retain(|id, _| rows.iter().any(|r| r.id == *id));
                }
                Err(e) => {
                    warn!("MQTT state checker query error: {e}");
                }
            }

            first_run = false;
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
        }
    }))
}
