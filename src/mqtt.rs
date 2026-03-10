use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use serde::Serialize;
use serde_json::json;
use sqlx::SqlitePool;
use tokio::task::JoinHandle;
use tracing::{debug, info, warn};

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

        let handle = spawn_state_checker(pool, None, "flowl".to_string(), None);
        assert!(handle.is_none());
    }

    #[test]
    fn extract_plant_id_from_discovery_topic() {
        assert_eq!(
            extract_plant_id("homeassistant/sensor/flowl_plant_1/config", "flowl"),
            Some(1)
        );
        assert_eq!(
            extract_plant_id("homeassistant/sensor/flowl_plant_42/config", "flowl"),
            Some(42)
        );
        assert_eq!(
            extract_plant_id("homeassistant/sensor/myplants_plant_7/config", "myplants"),
            Some(7)
        );
    }

    #[test]
    fn extract_plant_id_from_state_topic() {
        assert_eq!(extract_plant_id("flowl/plant/1/state", "flowl"), Some(1));
        assert_eq!(extract_plant_id("flowl/plant/99/state", "flowl"), Some(99));
        assert_eq!(
            extract_plant_id("myplants/plant/3/state", "myplants"),
            Some(3)
        );
    }

    #[test]
    fn extract_plant_id_from_attributes_topic() {
        assert_eq!(
            extract_plant_id("flowl/plant/1/attributes", "flowl"),
            Some(1)
        );
        assert_eq!(
            extract_plant_id("flowl/plant/55/attributes", "flowl"),
            Some(55)
        );
    }

    #[test]
    fn extract_plant_id_returns_none_for_unrelated_topics() {
        assert_eq!(extract_plant_id("some/other/topic", "flowl"), None);
        assert_eq!(
            extract_plant_id("homeassistant/sensor/other_sensor/config", "flowl"),
            None
        );
        assert_eq!(extract_plant_id("flowl/plant/abc/state", "flowl"), None);
        assert_eq!(extract_plant_id("flowl/plant/1/unknown", "flowl"), None);
    }

    #[test]
    fn extract_plant_id_wrong_prefix() {
        assert_eq!(extract_plant_id("flowl/plant/1/state", "otherprefix"), None);
        assert_eq!(
            extract_plant_id("homeassistant/sensor/flowl_plant_1/config", "otherprefix"),
            None
        );
    }

    #[test]
    fn discovery_payload_structure() {
        let (topic, payload) = discovery_topic_and_payload("flowl", 42, "Monstera");
        assert_eq!(topic, "homeassistant/sensor/flowl_plant_42/config");

        let json: serde_json::Value = serde_json::from_str(&payload).unwrap();
        assert_eq!(json["name"], "Monstera");
        assert_eq!(json["unique_id"], "flowl_plant_42");
        assert_eq!(json["state_topic"], "flowl/plant/42/state");
        assert_eq!(json["json_attributes_topic"], "flowl/plant/42/attributes");
        assert_eq!(json["icon"], "mdi:flower");
        assert_eq!(json["device"]["identifiers"][0], "flowl");
        assert_eq!(json["device"]["manufacturer"], "flowl");
    }

    #[test]
    fn discovery_payload_custom_prefix() {
        let (topic, payload) = discovery_topic_and_payload("myplants", 1, "Cactus");
        assert_eq!(topic, "homeassistant/sensor/myplants_plant_1/config");

        let json: serde_json::Value = serde_json::from_str(&payload).unwrap();
        assert_eq!(json["unique_id"], "myplants_plant_1");
        assert_eq!(json["state_topic"], "myplants/plant/1/state");
        assert_eq!(json["device"]["identifiers"][0], "myplants");
        assert_eq!(json["device"]["name"], "myplants");
    }

    #[test]
    fn state_topic_format() {
        assert_eq!(state_topic("flowl", 1), "flowl/plant/1/state");
        assert_eq!(state_topic("myplants", 99), "myplants/plant/99/state");
    }

    #[test]
    fn attributes_payload_with_all_fields() {
        let (topic, payload) = attributes_topic_and_payload(
            "flowl",
            7,
            Some("2026-03-01T10:00:00Z"),
            Some("2026-03-08T10:00:00Z"),
            7,
        );
        assert_eq!(topic, "flowl/plant/7/attributes");

        let json: serde_json::Value = serde_json::from_str(&payload).unwrap();
        assert_eq!(json["last_watered"], "2026-03-01T10:00:00Z");
        assert_eq!(json["next_due"], "2026-03-08T10:00:00Z");
        assert_eq!(json["watering_interval_days"], 7);
    }

    #[test]
    fn attributes_payload_with_null_fields() {
        let (_, payload) = attributes_topic_and_payload("flowl", 1, None, None, 14);

        let json: serde_json::Value = serde_json::from_str(&payload).unwrap();
        assert!(json["last_watered"].is_null());
        assert!(json["next_due"].is_null());
        assert_eq!(json["watering_interval_days"], 14);
    }

    #[test]
    fn removal_topics_format() {
        let topics = removal_topics("flowl", 5);
        assert_eq!(topics[0], "homeassistant/sensor/flowl_plant_5/config");
        assert_eq!(topics[1], "flowl/plant/5/state");
        assert_eq!(topics[2], "flowl/plant/5/attributes");
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

pub fn connect(
    config: &Config,
    connected: Arc<AtomicBool>,
    needs_republish: Arc<AtomicBool>,
) -> Option<MqttHandle> {
    if config.mqtt_disabled {
        info!("FLOWL_MQTT_DISABLED=true, skipping MQTT client setup");
        return None;
    }
    let client_id = format!("{}-{}", config.mqtt_topic_prefix, std::process::id());
    let mut options = MqttOptions::new(&client_id, &config.mqtt_host, config.mqtt_port);
    options.set_keep_alive(std::time::Duration::from_secs(30));

    let (client, mut event_loop) = AsyncClient::new(options, 10);

    let task = tokio::spawn(async move {
        let mut delay = std::time::Duration::from_secs(5);
        let max_delay = std::time::Duration::from_secs(120);

        loop {
            match event_loop.poll().await {
                Ok(Event::Incoming(Packet::ConnAck(_))) => {
                    connected.store(true, Ordering::Relaxed);
                    needs_republish.store(true, Ordering::Relaxed);
                    delay = std::time::Duration::from_secs(5);
                    info!("MQTT connected");
                }
                Ok(_) => {}
                Err(e) => {
                    connected.store(false, Ordering::Relaxed);
                    warn!("MQTT connection error: {e}");
                    tokio::time::sleep(delay).await;
                    delay = (delay * 2).min(max_delay);
                }
            }
        }
    });

    Some(MqttHandle { client, task })
}

fn discovery_topic_and_payload(prefix: &str, plant_id: i64, plant_name: &str) -> (String, String) {
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
    (topic, payload.to_string())
}

fn state_topic(prefix: &str, plant_id: i64) -> String {
    format!("{prefix}/plant/{plant_id}/state")
}

fn attributes_topic_and_payload(
    prefix: &str,
    plant_id: i64,
    last_watered: Option<&str>,
    next_due: Option<&str>,
    interval_days: i64,
) -> (String, String) {
    let topic = format!("{prefix}/plant/{plant_id}/attributes");
    let payload = json!({
        "last_watered": last_watered,
        "next_due": next_due,
        "watering_interval_days": interval_days,
    });
    (topic, payload.to_string())
}

fn removal_topics(prefix: &str, plant_id: i64) -> [String; 3] {
    [
        format!("homeassistant/sensor/{prefix}_plant_{plant_id}/config"),
        format!("{prefix}/plant/{plant_id}/state"),
        format!("{prefix}/plant/{plant_id}/attributes"),
    ]
}

/// Publish HA auto-discovery config for a plant sensor entity.
pub async fn publish_discovery(
    client: Option<&AsyncClient>,
    prefix: &str,
    plant_id: i64,
    plant_name: &str,
) {
    let Some(client) = client else { return };
    let (topic, payload) = discovery_topic_and_payload(prefix, plant_id, plant_name);

    match client
        .publish(&topic, QoS::AtLeastOnce, true, payload)
        .await
    {
        Ok(()) => debug!(plant_id, "MQTT published discovery"),
        Err(e) => warn!(plant_id, error = %e, "MQTT publish discovery failed"),
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
    let topic = state_topic(prefix, plant_id);

    match client.publish(&topic, QoS::AtLeastOnce, true, status).await {
        Ok(()) => debug!(plant_id, status, "MQTT published state"),
        Err(e) => warn!(plant_id, error = %e, "MQTT publish state failed"),
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
    let (topic, payload) =
        attributes_topic_and_payload(prefix, plant_id, last_watered, next_due, interval_days);

    match client
        .publish(&topic, QoS::AtLeastOnce, true, payload)
        .await
    {
        Ok(()) => debug!(plant_id, "MQTT published attributes"),
        Err(e) => warn!(plant_id, error = %e, "MQTT publish attributes failed"),
    }
}

/// Remove a plant from HA by publishing empty retained payloads to its topics.
pub async fn remove_plant(client: Option<&AsyncClient>, prefix: &str, plant_id: i64) {
    let Some(client) = client else { return };

    for topic in &removal_topics(prefix, plant_id) {
        if let Err(e) = client
            .publish(topic, QoS::AtLeastOnce, true, Vec::<u8>::new())
            .await
        {
            warn!(plant_id, topic, error = %e, "MQTT remove plant failed");
        }
    }
    debug!(plant_id, "MQTT removed plant topics");
}

/// Extract a plant ID from an MQTT topic name matching known patterns.
fn extract_plant_id(topic: &str, prefix: &str) -> Option<i64> {
    // homeassistant/sensor/{prefix}_plant_{id}/config
    if let Some(rest) = topic.strip_prefix("homeassistant/sensor/") {
        let expected_prefix = format!("{prefix}_plant_");
        if let Some(rest) = rest.strip_prefix(&expected_prefix)
            && let Some(id_str) = rest.strip_suffix("/config")
        {
            return id_str.parse().ok();
        }
    }

    // {prefix}/plant/{id}/state or {prefix}/plant/{id}/attributes
    let plant_prefix = format!("{prefix}/plant/");
    if let Some(rest) = topic.strip_prefix(&plant_prefix)
        && let Some(id_str) = rest
            .strip_suffix("/state")
            .or_else(|| rest.strip_suffix("/attributes"))
    {
        return id_str.parse().ok();
    }

    None
}

/// Create a temporary MQTT client, subscribe to wildcard topic patterns, collect
/// retained messages, and return the set of plant IDs found on the broker.
async fn discover_broker_plant_ids(host: &str, port: u16, prefix: &str) -> HashSet<i64> {
    let client_id = format!("{prefix}-repair-{}", std::process::id());
    let mut options = MqttOptions::new(&client_id, host, port);
    options.set_keep_alive(std::time::Duration::from_secs(10));

    let (client, mut event_loop) = AsyncClient::new(options, 50);
    let mut ids: HashSet<i64> = HashSet::new();
    let mut connected = false;
    let mut subscribed = false;

    let topics = [
        "homeassistant/sensor/+/config".to_string(),
        format!("{prefix}/plant/+/state"),
        format!("{prefix}/plant/+/attributes"),
    ];

    let timeout_duration = std::time::Duration::from_secs(2);
    let mut last_message = tokio::time::Instant::now();

    loop {
        let poll_result = tokio::time::timeout(timeout_duration, event_loop.poll()).await;

        match poll_result {
            Ok(Ok(Event::Incoming(Packet::ConnAck(_)))) => {
                connected = true;
                for topic in &topics {
                    if let Err(e) = client.subscribe(topic, QoS::AtMostOnce).await {
                        warn!("MQTT repair subscribe error for {topic}: {e}");
                    }
                }
                subscribed = true;
                last_message = tokio::time::Instant::now();
            }
            Ok(Ok(Event::Incoming(Packet::Publish(publish)))) => {
                if let Some(id) = extract_plant_id(&publish.topic, prefix) {
                    ids.insert(id);
                }
                last_message = tokio::time::Instant::now();
            }
            Ok(Ok(_)) => {}
            Ok(Err(e)) => {
                warn!("MQTT repair connection error: {e}");
                break;
            }
            Err(_) => {
                if connected && subscribed {
                    break;
                }
                if !connected {
                    warn!("MQTT repair: timed out waiting for broker connection");
                    break;
                }
            }
        }

        if subscribed && last_message.elapsed() >= timeout_duration {
            break;
        }
    }

    let _ = client.disconnect().await;

    ids
}

#[derive(Serialize)]
pub struct RepairResult {
    pub cleared: usize,
    pub published: usize,
}

/// Repair MQTT broker state: discover orphaned topics, clear them, then republish
/// fresh state for all current plants.
pub async fn repair(
    pool: &SqlitePool,
    client: &AsyncClient,
    host: &str,
    port: u16,
    prefix: &str,
) -> RepairResult {
    // Discover what's on the broker
    let broker_ids = discover_broker_plant_ids(host, port, prefix).await;

    // Get current plant IDs from DB
    let db_ids: HashSet<i64> = match sqlx::query_scalar::<_, i64>("SELECT id FROM plants")
        .fetch_all(pool)
        .await
    {
        Ok(ids) => ids.into_iter().collect(),
        Err(e) => {
            warn!("MQTT repair query error: {e}");
            return RepairResult {
                cleared: 0,
                published: 0,
            };
        }
    };

    // Diff: orphans are IDs on the broker but not in the DB
    let orphans: Vec<i64> = broker_ids.difference(&db_ids).copied().collect();
    let cleared = orphans.len();

    for id in &orphans {
        remove_plant(Some(client), prefix, *id).await;
    }

    // Republish fresh state for all current plants
    republish_all(pool, client, prefix).await;
    let published = db_ids.len();

    info!("MQTT repair complete: cleared {cleared} orphans, published {published} plants");

    RepairResult { cleared, published }
}

/// Republish discovery, state, and attributes for all current plants.
pub async fn republish_all(pool: &SqlitePool, client: &AsyncClient, prefix: &str) {
    let rows = match sqlx::query_as::<_, CheckerRow>(
        "SELECT p.id, p.name, p.watering_interval_days, lw.last_watered \
         FROM plants p LEFT JOIN plant_last_watered lw ON lw.plant_id = p.id",
    )
    .fetch_all(pool)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            warn!("MQTT republish_all query error: {e}");
            return;
        }
    };

    for row in &rows {
        let (status, next_due) =
            compute_watering_status(row.last_watered.as_deref(), row.watering_interval_days);

        publish_discovery(Some(client), prefix, row.id, &row.name).await;
        publish_state(Some(client), prefix, row.id, &status).await;
        publish_attributes(
            Some(client),
            prefix,
            row.id,
            row.last_watered.as_deref(),
            next_due.as_deref(),
            row.watering_interval_days,
        )
        .await;
    }

    info!("MQTT republish_all complete: {} plants", rows.len());
}

#[derive(sqlx::FromRow)]
struct CheckerRow {
    id: i64,
    name: String,
    watering_interval_days: i64,
    last_watered: Option<String>,
}

/// Spawn a background task that checks all plants every 60 seconds and publishes
/// state transitions to MQTT. On first run or after reconnect, publishes discovery
/// configs for all plants when MQTT is enabled.
pub fn spawn_state_checker(
    pool: SqlitePool,
    client: Option<AsyncClient>,
    prefix: String,
    needs_republish: Option<Arc<AtomicBool>>,
) -> Option<JoinHandle<()>> {
    let client = client?;
    let needs_republish = needs_republish?;

    info!("Starting MQTT background state checker");

    Some(tokio::spawn(async move {
        let mut cache: HashMap<i64, String> = HashMap::new();

        loop {
            if needs_republish.swap(false, Ordering::Relaxed) {
                info!("MQTT (re)connected, triggering full republish");
                republish_all(&pool, &client, &prefix).await;
                cache.clear();
                tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
                continue;
            }

            match sqlx::query_as::<_, CheckerRow>(
                "SELECT p.id, p.name, p.watering_interval_days, lw.last_watered \
                 FROM plants p LEFT JOIN plant_last_watered lw ON lw.plant_id = p.id",
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

                        let changed = cache.get(&row.id).is_none_or(|prev| *prev != status);
                        if changed {
                            publish_state(Some(&client), &prefix, row.id, &status).await;
                            publish_attributes(
                                Some(&client),
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

            tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
        }
    }))
}
