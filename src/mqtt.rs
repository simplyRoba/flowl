use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use serde::Serialize;
use serde_json::json;
use sqlx::SqlitePool;
use tokio::sync::Notify;
use tokio::task::JoinHandle;
use tracing::{debug, info, warn};
use url::Url;

use crate::api::plants::compute_watering_status;
use crate::config::Config;

pub struct MqttHandle {
    pub client: AsyncClient,
    task: JoinHandle<()>,
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use rumqttc::{EventLoop, Publish, Request};
    use sqlx::sqlite::SqlitePoolOptions;

    use super::*;

    #[tokio::test]
    async fn spawn_state_checker_skips_when_mqtt_disabled() {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .expect("Failed to create in-memory pool");

        let handle = spawn_state_checker(pool, None, "flowl".to_string(), None, None);
        assert!(handle.is_none());
    }

    #[tokio::test]
    async fn connection_notification_wakes_checker_immediately() {
        let notify = Notify::new();
        let mut interval = tokio::time::interval(std::time::Duration::from_hours(1));
        interval.tick().await;
        notify.notify_one();

        let wake = tokio::time::timeout(
            std::time::Duration::from_millis(50),
            next_checker_wake(&mut interval, &notify),
        )
        .await
        .expect("connection notification should wake checker");

        assert_eq!(wake, CheckerWake::FullRepublish);
    }

    async fn checker_test_pool(plant_ids: &[i64]) -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        crate::db::run_migrations(&pool).await.unwrap();
        for id in plant_ids {
            sqlx::query(
                "INSERT INTO plants (id, name, created_at, updated_at) VALUES \
                 (?, 'Test plant', '2026-03-01T10:00:00Z', '2026-03-01T10:00:00Z')",
            )
            .bind(id)
            .execute(&pool)
            .await
            .unwrap();
        }
        pool
    }

    async fn next_queued_publish(event_loop: &mut EventLoop) -> Publish {
        tokio::time::timeout(Duration::from_secs(2), async {
            loop {
                // Drain queued requests into the retransmission buffer without opening a socket.
                event_loop.clean();
                match event_loop.pending.pop_front() {
                    Some(Request::Publish(publish)) => return publish,
                    Some(other) => panic!("expected an MQTT publish, got {other:?}"),
                    None => tokio::time::sleep(Duration::from_millis(1)).await,
                }
            }
        })
        .await
        .expect("checker queued an MQTT publish")
    }

    async fn assert_checker_state_and_attributes(event_loop: &mut EventLoop, id: i64) {
        let state = next_queued_publish(event_loop).await;
        assert_eq!(state.topic, format!("myplants/plant/{id}/state"));
        assert_eq!(&state.payload[..], b"due");

        let attributes = next_queued_publish(event_loop).await;
        assert_eq!(attributes.topic, format!("myplants/plant/{id}/attributes"));
        assert!(attributes.retain);
        let payload: serde_json::Value = serde_json::from_slice(&attributes.payload).unwrap();
        assert_eq!(
            payload,
            json!({
                "last_watered": null,
                "next_due": null,
                "watering_interval_days": 7,
                "plant_url": format!("https://flowl.example.org/plants/{id}"),
            })
        );
    }

    #[tokio::test]
    async fn checker_full_sync_publishes_links_on_initial_connection_and_reconnect() {
        let pool = checker_test_pool(&[7, 42]).await;
        let (client, mut event_loop) =
            AsyncClient::new(MqttOptions::new("test", "localhost", 1883), 10);
        let notify = Arc::new(Notify::new());
        let origin = Url::parse("https://flowl.example.org").unwrap();
        // The connection loop uses this same notification for each CONNACK.
        notify.notify_one();
        let checker = spawn_state_checker(
            pool,
            Some(client),
            "myplants".to_string(),
            Some(notify.clone()),
            Some(origin),
        )
        .unwrap();

        for reconnect in [false, true] {
            if reconnect {
                notify.notify_one();
            }
            let mut seen = HashSet::new();
            for _ in 0..2 {
                let discovery = next_queued_publish(&mut event_loop).await;
                let id = extract_plant_id(&discovery.topic, "myplants").unwrap();
                assert!(seen.insert(id));
                assert_eq!(
                    discovery.topic,
                    format!("homeassistant/sensor/myplants_plant_{id}/config")
                );
                let payload: serde_json::Value =
                    serde_json::from_slice(&discovery.payload).unwrap();
                assert_eq!(payload["unique_id"], format!("myplants_plant_{id}"));
                assert!(payload["device"].get("configuration_url").is_none());
                assert_checker_state_and_attributes(&mut event_loop, id).await;
            }
            assert_eq!(seen, HashSet::from([7, 42]));
        }
        checker.abort();
    }

    #[tokio::test]
    async fn checker_periodic_reconciliation_publishes_links() {
        let pool = checker_test_pool(&[7]).await;
        let (client, mut event_loop) =
            AsyncClient::new(MqttOptions::new("test", "localhost", 1883), 10);
        let checker = spawn_checker_with_period(
            pool,
            client,
            "myplants".to_string(),
            Arc::new(Notify::new()),
            Some(Url::parse("https://flowl.example.org").unwrap()),
            Duration::from_millis(10),
        );
        assert_checker_state_and_attributes(&mut event_loop, 7).await;
        checker.abort();
    }

    #[test]
    fn checker_caches_only_fully_published_state() {
        let mut cache = HashMap::new();

        for (state_published, attributes_published) in
            [(false, false), (true, false), (false, true)]
        {
            cache_successful_publish(
                &mut cache,
                1,
                "due".to_string(),
                state_published,
                attributes_published,
            );
            assert!(!cache.contains_key(&1));
        }

        cache_successful_publish(&mut cache, 1, "due".to_string(), true, true);
        assert_eq!(cache.get(&1).map(String::as_str), Some("due"));
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
            None,
        );
        assert_eq!(topic, "flowl/plant/7/attributes");

        let json: serde_json::Value = serde_json::from_str(&payload).unwrap();
        assert_eq!(json["last_watered"], "2026-03-01T10:00:00Z");
        assert_eq!(json["next_due"], "2026-03-08T10:00:00Z");
        assert_eq!(json["watering_interval_days"], 7);
        assert!(json.get("plant_url").is_none());
    }

    #[test]
    fn attributes_payload_links_each_never_watered_plant_with_custom_prefix() {
        let origin = Url::parse("http://flowl.lan:4100/").unwrap();
        for id in [7, 42] {
            let (topic, payload) =
                attributes_topic_and_payload("myplants", id, None, None, 14, Some(&origin));
            assert_eq!(topic, format!("myplants/plant/{id}/attributes"));

            let json: serde_json::Value = serde_json::from_str(&payload).unwrap();
            assert!(json["last_watered"].is_null());
            assert!(json["next_due"].is_null());
            assert_eq!(json["watering_interval_days"], 14);
            assert_eq!(
                json["plant_url"],
                format!("http://flowl.lan:4100/plants/{id}")
            );
        }
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
    republish_notify: Arc<Notify>,
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
        let max_delay = std::time::Duration::from_mins(2);

        loop {
            match event_loop.poll().await {
                Ok(Event::Incoming(Packet::ConnAck(_))) => {
                    connected.store(true, Ordering::Relaxed);
                    republish_notify.notify_one();
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
    external_url: Option<&Url>,
) -> (String, String) {
    let topic = format!("{prefix}/plant/{plant_id}/attributes");
    let mut payload = json!({
        "last_watered": last_watered,
        "next_due": next_due,
        "watering_interval_days": interval_days,
    });
    if let Some(origin) = external_url {
        let mut plant_url = origin.clone();
        plant_url.set_path(&format!("/plants/{plant_id}"));
        payload["plant_url"] = json!(plant_url.as_str());
    }
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
const MAX_RETRIES: u32 = 3;

async fn publish_with_retry(
    client: &AsyncClient,
    topic: &str,
    payload: &[u8],
    label: &str,
) -> bool {
    for attempt in 1..=MAX_RETRIES {
        match client
            .publish(topic, QoS::AtLeastOnce, true, payload.to_vec())
            .await
        {
            Ok(()) => {
                debug!(topic, label, "MQTT published");
                return true;
            }
            Err(e) => {
                warn!(
                    topic, label, attempt, max = MAX_RETRIES,
                    error = %e, "MQTT publish failed, retrying"
                );
                tokio::time::sleep(std::time::Duration::from_secs(u64::from(attempt))).await;
            }
        }
    }
    warn!(
        topic,
        label, "MQTT publish failed after {MAX_RETRIES} retries, falling back to checker"
    );
    false
}

pub async fn publish_discovery(
    client: Option<&AsyncClient>,
    prefix: &str,
    plant_id: i64,
    plant_name: &str,
) {
    let Some(client) = client else { return };
    let (topic, payload) = discovery_topic_and_payload(prefix, plant_id, plant_name);
    publish_with_retry(client, &topic, payload.as_bytes(), "discovery").await;
}

/// Publish watering state (`ok`, `due`, `overdue`) to the plant's state topic.
pub async fn publish_state(
    client: Option<&AsyncClient>,
    prefix: &str,
    plant_id: i64,
    status: &str,
) -> bool {
    let Some(client) = client else { return false };
    let topic = state_topic(prefix, plant_id);
    publish_with_retry(client, &topic, status.as_bytes(), "state").await
}

/// Publish watering attributes and the optional link to the plant's attributes topic.
pub async fn publish_attributes(
    client: Option<&AsyncClient>,
    prefix: &str,
    plant_id: i64,
    last_watered: Option<&str>,
    next_due: Option<&str>,
    interval_days: i64,
    external_url: Option<&Url>,
) -> bool {
    let Some(client) = client else { return false };
    let (topic, payload) = attributes_topic_and_payload(
        prefix,
        plant_id,
        last_watered,
        next_due,
        interval_days,
        external_url,
    );
    publish_with_retry(client, &topic, payload.as_bytes(), "attributes").await
}

/// Remove a plant from HA by publishing empty retained payloads to its topics.
pub async fn remove_plant(client: Option<&AsyncClient>, prefix: &str, plant_id: i64) {
    let Some(client) = client else { return };

    for topic in &removal_topics(prefix, plant_id) {
        publish_with_retry(client, topic, &[], "remove").await;
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
    external_url: Option<&Url>,
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
    republish_all(pool, client, prefix, external_url).await;
    let published = db_ids.len();

    info!("MQTT repair complete: cleared {cleared} orphans, published {published} plants");

    RepairResult { cleared, published }
}

/// Republish discovery, state, and attributes for all current plants.
pub async fn republish_all(
    pool: &SqlitePool,
    client: &AsyncClient,
    prefix: &str,
    external_url: Option<&Url>,
) {
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
            external_url,
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

fn cache_successful_publish(
    cache: &mut HashMap<i64, String>,
    plant_id: i64,
    status: String,
    state_published: bool,
    attributes_published: bool,
) {
    if state_published && attributes_published {
        cache.insert(plant_id, status);
    }
}

#[derive(Debug, PartialEq, Eq)]
enum CheckerWake {
    FullRepublish,
    ScheduledCheck,
}

async fn next_checker_wake(
    interval: &mut tokio::time::Interval,
    republish_notify: &Notify,
) -> CheckerWake {
    tokio::select! {
        () = republish_notify.notified() => CheckerWake::FullRepublish,
        _ = interval.tick() => CheckerWake::ScheduledCheck,
    }
}

/// Spawn a background task that checks all plants hourly and publishes state transitions to MQTT.
/// On initial connection or reconnection, immediately republishes all plant topics.
pub fn spawn_state_checker(
    pool: SqlitePool,
    client: Option<AsyncClient>,
    prefix: String,
    republish_notify: Option<Arc<Notify>>,
    external_url: Option<Url>,
) -> Option<JoinHandle<()>> {
    let client = client?;
    let republish_notify = republish_notify?;

    info!("Starting MQTT background state checker");

    Some(spawn_checker_with_period(
        pool,
        client,
        prefix,
        republish_notify,
        external_url,
        std::time::Duration::from_hours(1),
    ))
}

fn spawn_checker_with_period(
    pool: SqlitePool,
    client: AsyncClient,
    prefix: String,
    republish_notify: Arc<Notify>,
    external_url: Option<Url>,
    period: std::time::Duration,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut cache: HashMap<i64, String> = HashMap::new();
        let mut interval = tokio::time::interval(period);
        interval.tick().await;

        loop {
            if next_checker_wake(&mut interval, &republish_notify).await
                == CheckerWake::FullRepublish
            {
                info!("MQTT (re)connected, triggering full republish");
                republish_all(&pool, &client, &prefix, external_url.as_ref()).await;
                cache.clear();
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
                            let state_published =
                                publish_state(Some(&client), &prefix, row.id, &status).await;
                            let attributes_published = publish_attributes(
                                Some(&client),
                                &prefix,
                                row.id,
                                row.last_watered.as_deref(),
                                next_due.as_deref(),
                                row.watering_interval_days,
                                external_url.as_ref(),
                            )
                            .await;
                            cache_successful_publish(
                                &mut cache,
                                row.id,
                                status,
                                state_published,
                                attributes_published,
                            );
                        }
                    }

                    // Remove cached entries for deleted plants
                    cache.retain(|id, _| rows.iter().any(|r| r.id == *id));
                }
                Err(e) => {
                    warn!("MQTT state checker query error: {e}");
                }
            }
        }
    })
}
