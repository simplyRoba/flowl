mod common;

use std::io;
use std::time::Duration;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{body_json, json_request};
use rumqttc::{AsyncClient, MqttOptions};
use sqlx::SqlitePool;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;
use tower::ServiceExt;

async fn app() -> (axum::Router, tempfile::TempDir) {
    common::test_app().await
}

async fn app_with_pool() -> (axum::Router, SqlitePool, tempfile::TempDir) {
    let pool = common::test_pool().await;
    let tmp = tempfile::TempDir::new().expect("temporary upload directory");
    let state = flowl::state::AppState {
        pool: pool.clone(),
        image_store: flowl::images::ImageStore::new(tmp.path().to_path_buf()),
        mqtt_client: None,
        mqtt_prefix: "flowl".to_string(),
        mqtt_connected: None,
        mqtt_host: "localhost".to_string(),
        mqtt_port: 1883,
        mqtt_disabled: true,
        ai_provider: None,
        ai_base_url: String::new(),
        ai_model: String::new(),
        ai_rate_limiter: None,
        auth: None,
    };

    (flowl::server::router(state), pool, tmp)
}

async fn insert_care_event(
    pool: &SqlitePool,
    plant_id: i64,
    event_type: &str,
    occurred_at: &str,
) -> i64 {
    sqlx::query_scalar(
        "INSERT INTO care_events (plant_id, event_type, occurred_at, created_at) \
         VALUES (?, ?, ?, ?) RETURNING id",
    )
    .bind(plant_id)
    .bind(event_type)
    .bind(occurred_at)
    .bind("2026-01-01T00:00:00Z")
    .fetch_one(pool)
    .await
    .expect("care event inserted")
}

async fn create_plant(app: &axum::Router) -> i64 {
    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/plants",
            Some(r#"{"name":"TestPlant"}"#),
        ))
        .await
        .unwrap();
    let json = body_json(resp).await;
    json["id"].as_i64().unwrap()
}

fn multipart_request(uri: &str, content_type: &str, data: &[u8]) -> Request<Body> {
    let boundary = "----testboundary";
    let mut body_bytes = Vec::new();
    body_bytes.extend_from_slice(b"------testboundary\r\n");
    body_bytes.extend_from_slice(
        format!(
            "Content-Disposition: form-data; name=\"file\"; filename=\"test.jpg\"\r\n\
             Content-Type: {content_type}\r\n\r\n"
        )
        .as_bytes(),
    );
    body_bytes.extend_from_slice(data);
    body_bytes.extend_from_slice(b"\r\n------testboundary--\r\n");

    Request::builder()
        .method("POST")
        .uri(uri)
        .header(
            "content-type",
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(body_bytes))
        .unwrap()
}

async fn plant_last_watered(app: &axum::Router, plant_id: i64) -> serde_json::Value {
    let response = app
        .clone()
        .oneshot(json_request(
            "GET",
            &format!("/api/plants/{plant_id}"),
            None,
        ))
        .await
        .unwrap();
    body_json(response).await["last_watered"].clone()
}

struct MqttPublication {
    topic: String,
    payload: Vec<u8>,
}

struct FakeMqttBroker {
    port: u16,
    publications: mpsc::Receiver<MqttPublication>,
    connected: oneshot::Receiver<()>,
    task: JoinHandle<()>,
}

async fn read_mqtt_packet(stream: &mut TcpStream) -> io::Result<Option<(u8, Vec<u8>)>> {
    let mut header = [0];
    match stream.read_exact(&mut header).await {
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(error) => return Err(error),
    }

    let mut remaining_len = 0_usize;
    let mut multiplier = 1_usize;
    loop {
        let encoded = stream.read_u8().await?;
        remaining_len += usize::from(encoded & 0x7f) * multiplier;
        if encoded & 0x80 == 0 {
            break;
        }
        multiplier *= 128;
    }

    let mut payload = vec![0; remaining_len];
    stream.read_exact(&mut payload).await?;
    Ok(Some((header[0], payload)))
}

fn parse_mqtt_publish(header: u8, packet: Vec<u8>) -> MqttPublication {
    let topic_len = usize::from(u16::from_be_bytes([packet[0], packet[1]]));
    let topic = String::from_utf8(packet[2..2 + topic_len].to_vec()).expect("valid MQTT topic");
    let packet_id_len = if (header >> 1) & 0x03 == 0 { 0 } else { 2 };
    let payload = packet[2 + topic_len + packet_id_len..].to_vec();
    MqttPublication { topic, payload }
}

async fn start_fake_mqtt_broker() -> FakeMqttBroker {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("fake broker binds");
    let port = listener.local_addr().expect("fake broker address").port();
    let (publication_tx, publications) = mpsc::channel(8);
    let (connected_tx, connected) = oneshot::channel();

    let task = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.expect("MQTT client connects");
        let (header, _) = read_mqtt_packet(&mut stream)
            .await
            .expect("MQTT CONNECT packet")
            .expect("MQTT client remains connected");
        assert_eq!(header >> 4, 1, "first MQTT packet is CONNECT");
        stream
            .write_all(&[0x20, 0x02, 0x00, 0x00])
            .await
            .expect("MQTT CONNACK write");
        connected_tx
            .send(())
            .expect("connection readiness received");

        while let Some((header, packet)) = read_mqtt_packet(&mut stream)
            .await
            .expect("valid MQTT packet")
        {
            if header >> 4 != 3 {
                continue;
            }

            let qos = (header >> 1) & 0x03;
            let packet_id_offset = usize::from(u16::from_be_bytes([packet[0], packet[1]])) + 2;
            let packet_id = if qos == 0 {
                None
            } else {
                Some([packet[packet_id_offset], packet[packet_id_offset + 1]])
            };
            publication_tx
                .send(parse_mqtt_publish(header, packet))
                .await
                .expect("publication receiver remains available");
            if let Some(packet_id) = packet_id {
                stream
                    .write_all(&[0x40, 0x02, packet_id[0], packet_id[1]])
                    .await
                    .expect("MQTT PUBACK write");
            }
        }
    });

    FakeMqttBroker {
        port,
        publications,
        connected,
        task,
    }
}

async fn next_mqtt_publication(
    publications: &mut mpsc::Receiver<MqttPublication>,
) -> MqttPublication {
    tokio::time::timeout(Duration::from_secs(1), publications.recv())
        .await
        .expect("timed out waiting for MQTT publication")
        .expect("MQTT publication channel closed")
}

async fn assert_watering_mqtt_publications(
    publications: &mut mpsc::Receiver<MqttPublication>,
    plant_id: i64,
    last_watered: Option<&str>,
) {
    let first = next_mqtt_publication(publications).await;
    let second = next_mqtt_publication(publications).await;
    let expected_state_topic = format!("flowl/plant/{plant_id}/state");
    let expected_attributes_topic = format!("flowl/plant/{plant_id}/attributes");
    let messages = [first, second];

    let state = messages
        .iter()
        .find(|message| message.topic == expected_state_topic)
        .expect("watering state publication");
    assert!(matches!(
        std::str::from_utf8(&state.payload).expect("UTF-8 state"),
        "ok" | "due" | "overdue"
    ));

    let attributes = messages
        .iter()
        .find(|message| message.topic == expected_attributes_topic)
        .expect("watering attributes publication");
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&attributes.payload).expect("attributes JSON")
            ["last_watered"],
        serde_json::json!(last_watered)
    );
}

#[tokio::test]
async fn list_empty() {
    let (app, _dir) = app().await;
    let id = create_plant(&app).await;

    let resp = app
        .oneshot(json_request("GET", &format!("/api/plants/{id}/care"), None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    assert_eq!(json, serde_json::json!([]));
}

#[tokio::test]
async fn create_valid_event() {
    let (app, _dir) = app().await;
    let id = create_plant(&app).await;

    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/plants/{id}/care"),
            Some(r#"{"event_type":"fertilized","notes":"Half strength"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let json = body_json(resp).await;
    assert_eq!(json["event_type"], "fertilized");
    assert_eq!(json["notes"], "Half strength");
    assert_eq!(json["plant_id"], id);
    assert_eq!(json["plant_name"], "TestPlant");
    assert!(json["id"].is_number());
    assert!(json["occurred_at"].is_string());
    assert!(json["created_at"].is_string());
}

#[tokio::test]
async fn create_with_explicit_occurred_at() {
    let (app, _dir) = app().await;
    let id = create_plant(&app).await;

    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/plants/{id}/care"),
            Some(r#"{"event_type":"repotted","occurred_at":"2026-02-14T10:00:00"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let json = body_json(resp).await;
    assert_eq!(json["occurred_at"], "2026-02-14T10:00:00");
}

#[tokio::test]
async fn create_rejects_invalid_occurred_at() {
    let (app, _dir) = app().await;
    let id = create_plant(&app).await;

    let response = app
        .clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/plants/{id}/care"),
            Some(r#"{"event_type":"repotted","occurred_at":"not-a-datetime"}"#),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let json = body_json(response).await;
    assert_eq!(json["code"], "CARE_EVENT_INVALID_OCCURRED_AT");
    assert_eq!(json["message"], "Invalid occurrence time");

    let response = app
        .oneshot(json_request("GET", &format!("/api/plants/{id}/care"), None))
        .await
        .unwrap();
    assert_eq!(body_json(response).await, serde_json::json!([]));
}

// --- Care event updates ---

#[tokio::test]
async fn update_event_preserves_immutable_fields_photo_and_clears_notes() {
    let (app, _dir) = common::test_app_with_uploads().await;
    let plant_id = create_plant(&app).await;

    let response = app
        .clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/plants/{plant_id}/care"),
            Some(
                r#"{"event_type":"fertilized","notes":"Original note","occurred_at":"2026-02-10T10:00:00Z"}"#,
            ),
        ))
        .await
        .unwrap();
    let created = body_json(response).await;
    let event_id = created["id"].as_i64().unwrap();

    let response = app
        .clone()
        .oneshot(multipart_request(
            &format!("/api/plants/{plant_id}/care/{event_id}/photo"),
            "image/jpeg",
            &[0xFF, 0xD8, 0xFF, 0xE0],
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let with_photo = body_json(response).await;

    let response = app
        .clone()
        .oneshot(json_request(
            "PUT",
            &format!("/api/plants/{plant_id}/care/{event_id}"),
            Some(
                r#"{"event_type":"pruned","notes":"Updated note","occurred_at":"2026-02-11T11:00:00Z"}"#,
            ),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let updated = body_json(response).await;
    assert_eq!(updated["event_type"], "pruned");
    assert_eq!(updated["notes"], "Updated note");
    assert_eq!(updated["occurred_at"], "2026-02-11T11:00:00Z");
    assert_eq!(updated["id"], created["id"]);
    assert_eq!(updated["plant_id"], created["plant_id"]);
    assert_eq!(updated["photo_url"], with_photo["photo_url"]);
    assert_eq!(updated["created_at"], created["created_at"]);

    let response = app
        .oneshot(json_request(
            "PUT",
            &format!("/api/plants/{plant_id}/care/{event_id}"),
            Some(r#"{"event_type":"pruned","notes":null,"occurred_at":"2026-02-11T11:00:00Z"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let cleared = body_json(response).await;
    assert!(cleared["notes"].is_null());
}

#[tokio::test]
async fn update_event_rejects_invalid_input_without_mutation() {
    let (app, _dir) = app().await;
    let plant_id = create_plant(&app).await;

    let response = app
        .clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/plants/{plant_id}/care"),
            Some(r#"{"event_type":"pruned","notes":"Original","occurred_at":"2026-02-10T10:00:00Z"}"#),
        ))
        .await
        .unwrap();
    let created = body_json(response).await;
    let event_id = created["id"].as_i64().unwrap();
    let update_uri = format!("/api/plants/{plant_id}/care/{event_id}");

    let response = app
        .clone()
        .oneshot(json_request("PUT", &update_uri, Some("{")))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let future = (chrono::Utc::now() + chrono::Days::new(1)).to_rfc3339();
    let invalid_bodies = [
        r#"{"notes":null,"occurred_at":"2026-02-11T10:00:00Z"}"#.to_string(),
        r#"{"event_type":"pruned","occurred_at":"2026-02-11T10:00:00Z"}"#.to_string(),
        r#"{"event_type":"pruned","notes":null}"#.to_string(),
        r#"{"event_type":"unknown","notes":null,"occurred_at":"2026-02-11T10:00:00Z"}"#.to_string(),
        r#"{"event_type":"pruned","notes":null,"occurred_at":"not-a-datetime"}"#.to_string(),
        format!(r#"{{"event_type":"pruned","notes":null,"occurred_at":"{future}"}}"#),
    ];

    for body in invalid_bodies {
        let response = app
            .clone()
            .oneshot(json_request("PUT", &update_uri, Some(&body)))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    let response = app
        .oneshot(json_request(
            "GET",
            &format!("/api/plants/{plant_id}/care"),
            None,
        ))
        .await
        .unwrap();
    let events = body_json(response).await;
    assert_eq!(events, serde_json::json!([created]));
}

#[tokio::test]
async fn update_event_accepts_legacy_sqlite_datetime() {
    let (app, _dir) = app().await;
    let plant_id = create_plant(&app).await;

    let response = app
        .clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/plants/{plant_id}/care"),
            Some(r#"{"event_type":"pruned","occurred_at":"2026-02-10 10:00:00"}"#),
        ))
        .await
        .unwrap();
    let event_id = body_json(response).await["id"].as_i64().unwrap();

    let response = app
        .oneshot(json_request(
            "PUT",
            &format!("/api/plants/{plant_id}/care/{event_id}"),
            Some(r#"{"event_type":"fertilized","notes":null,"occurred_at":"2026-02-11 10:00:00"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        body_json(response).await["occurred_at"],
        "2026-02-11T10:00:00Z"
    );
}

#[tokio::test]
async fn update_event_returns_not_found_for_missing_plant_or_wrong_owner() {
    let (app, _dir) = app().await;
    let first_plant_id = create_plant(&app).await;
    let second_plant_id = create_plant(&app).await;

    let response = app
        .clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/plants/{second_plant_id}/care"),
            Some(r#"{"event_type":"fertilized","notes":"Original","occurred_at":"2026-02-10T10:00:00Z"}"#),
        ))
        .await
        .unwrap();
    let created = body_json(response).await;
    let event_id = created["id"].as_i64().unwrap();
    let update_body =
        r#"{"event_type":"pruned","notes":null,"occurred_at":"2026-02-11T10:00:00Z"}"#;

    let response = app
        .clone()
        .oneshot(json_request(
            "PUT",
            &format!("/api/plants/999/care/{event_id}"),
            Some(update_body),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let response = app
        .clone()
        .oneshot(json_request(
            "PUT",
            &format!("/api/plants/{first_plant_id}/care/{event_id}"),
            Some(update_body),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let response = app
        .oneshot(json_request(
            "GET",
            &format!("/api/plants/{second_plant_id}/care"),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(body_json(response).await, serde_json::json!([created]));
}

#[tokio::test]
async fn update_watering_mqtt_publications_follow_old_and_new_event_types() {
    let FakeMqttBroker {
        port,
        mut publications,
        connected,
        task: broker_task,
    } = start_fake_mqtt_broker().await;
    let options = MqttOptions::new("care-events-test", "127.0.0.1", port);
    let (client, mut event_loop) = AsyncClient::new(options, 10);
    let event_loop_task = tokio::spawn(async move { while event_loop.poll().await.is_ok() {} });
    connected.await.expect("MQTT client receives CONNACK");

    let (app, _dir) = common::test_app_with_mqtt(client).await;
    let plant_id = create_plant(&app).await;
    // Plant creation publishes discovery plus its initial state and attributes.
    for _ in 0..3 {
        next_mqtt_publication(&mut publications).await;
    }
    let now = chrono::Utc::now();
    let added_at =
        (now - chrono::Duration::minutes(3)).to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let moved_from =
        (now - chrono::Duration::minutes(4)).to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let moved_to =
        (now - chrono::Duration::minutes(2)).to_rfc3339_opts(chrono::SecondsFormat::Secs, true);

    let response = app
        .clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/plants/{plant_id}/care"),
            Some(r#"{"event_type":"fertilized"}"#),
        ))
        .await
        .unwrap();
    let added_event_id = body_json(response).await["id"].as_i64().unwrap();
    let response = app
        .clone()
        .oneshot(json_request(
            "PUT",
            &format!("/api/plants/{plant_id}/care/{added_event_id}"),
            Some(
                &serde_json::json!({
                    "event_type": "watered",
                    "notes": null,
                    "occurred_at": &added_at,
                })
                .to_string(),
            ),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_watering_mqtt_publications(&mut publications, plant_id, Some(&added_at)).await;

    let response = app
        .clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/plants/{plant_id}/care"),
            Some(&format!(
                r#"{{"event_type":"watered","occurred_at":"{moved_from}"}}"#
            )),
        ))
        .await
        .unwrap();
    let removed_event_id = body_json(response).await["id"].as_i64().unwrap();
    assert_watering_mqtt_publications(&mut publications, plant_id, Some(&added_at)).await;
    let response = app
        .clone()
        .oneshot(json_request(
            "PUT",
            &format!("/api/plants/{plant_id}/care/{removed_event_id}"),
            Some(
                &serde_json::json!({
                    "event_type": "fertilized",
                    "notes": null,
                    "occurred_at": &moved_from,
                })
                .to_string(),
            ),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_watering_mqtt_publications(&mut publications, plant_id, Some(&added_at)).await;

    let response = app
        .clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/plants/{plant_id}/care"),
            Some(&format!(
                r#"{{"event_type":"watered","occurred_at":"{moved_from}"}}"#
            )),
        ))
        .await
        .unwrap();
    let moved_event_id = body_json(response).await["id"].as_i64().unwrap();
    assert_watering_mqtt_publications(&mut publications, plant_id, Some(&added_at)).await;
    let response = app
        .clone()
        .oneshot(json_request(
            "PUT",
            &format!("/api/plants/{plant_id}/care/{moved_event_id}"),
            Some(
                &serde_json::json!({
                    "event_type": "watered",
                    "notes": null,
                    "occurred_at": &moved_to,
                })
                .to_string(),
            ),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_watering_mqtt_publications(&mut publications, plant_id, Some(&moved_to)).await;

    let response = app
        .clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/plants/{plant_id}/care"),
            Some(r#"{"event_type":"pruned"}"#),
        ))
        .await
        .unwrap();
    let non_watering_event_id = body_json(response).await["id"].as_i64().unwrap();
    let response = app
        .oneshot(json_request(
            "PUT",
            &format!("/api/plants/{plant_id}/care/{non_watering_event_id}"),
            Some(
                &serde_json::json!({
                    "event_type": "custom",
                    "notes": null,
                    "occurred_at": &moved_to,
                })
                .to_string(),
            ),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(
        tokio::time::timeout(Duration::from_millis(100), publications.recv())
            .await
            .is_err(),
        "non-watered updates must not publish watering MQTT state"
    );

    event_loop_task.abort();
    broker_task.abort();
}

#[tokio::test]
async fn update_to_watered_updates_plant_last_watered() {
    let (app, _dir) = app().await;
    let plant_id = create_plant(&app).await;

    let response = app
        .clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/plants/{plant_id}/care"),
            Some(r#"{"event_type":"fertilized","occurred_at":"2026-02-10T10:00:00Z"}"#),
        ))
        .await
        .unwrap();
    let event_id = body_json(response).await["id"].as_i64().unwrap();

    let response = app
        .clone()
        .oneshot(json_request(
            "PUT",
            &format!("/api/plants/{plant_id}/care/{event_id}"),
            Some(r#"{"event_type":"watered","notes":null,"occurred_at":"2026-02-15T10:00:00Z"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        plant_last_watered(&app, plant_id).await,
        "2026-02-15T10:00:00Z"
    );
}

#[tokio::test]
async fn update_from_watered_recomputes_plant_last_watered() {
    let (app, _dir) = app().await;
    let plant_id = create_plant(&app).await;

    app.clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/plants/{plant_id}/care"),
            Some(r#"{"event_type":"watered","occurred_at":"2026-02-10T10:00:00Z"}"#),
        ))
        .await
        .unwrap();
    let response = app
        .clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/plants/{plant_id}/care"),
            Some(r#"{"event_type":"watered","occurred_at":"2026-02-15T10:00:00Z"}"#),
        ))
        .await
        .unwrap();
    let event_id = body_json(response).await["id"].as_i64().unwrap();

    let response = app
        .clone()
        .oneshot(json_request(
            "PUT",
            &format!("/api/plants/{plant_id}/care/{event_id}"),
            Some(
                r#"{"event_type":"fertilized","notes":null,"occurred_at":"2026-02-15T10:00:00Z"}"#,
            ),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        plant_last_watered(&app, plant_id).await,
        "2026-02-10T10:00:00Z"
    );
}

#[tokio::test]
async fn update_offset_datetime_normalizes_and_preserves_last_watered_chronology() {
    let (app, _dir) = app().await;
    let plant_id = create_plant(&app).await;

    let response = app
        .clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/plants/{plant_id}/care"),
            Some(r#"{"event_type":"watered","occurred_at":"2026-02-10T10:00:00Z"}"#),
        ))
        .await
        .unwrap();
    let event_id = body_json(response).await["id"].as_i64().unwrap();

    app.clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/plants/{plant_id}/care"),
            Some(r#"{"event_type":"watered","occurred_at":"2026-02-15T10:00:00Z"}"#),
        ))
        .await
        .unwrap();

    let response = app
        .clone()
        .oneshot(json_request(
            "PUT",
            &format!("/api/plants/{plant_id}/care/{event_id}"),
            Some(r#"{"event_type":"watered","notes":null,"occurred_at":"2026-02-15T09:30:00-01:00"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        body_json(response).await["occurred_at"],
        "2026-02-15T10:30:00Z"
    );

    let response = app
        .clone()
        .oneshot(json_request(
            "GET",
            &format!("/api/plants/{plant_id}/care"),
            None,
        ))
        .await
        .unwrap();
    let events = body_json(response).await;
    assert!(
        events
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event["id"] == event_id && event["occurred_at"] == "2026-02-15T10:30:00Z")
    );
    assert_eq!(
        plant_last_watered(&app, plant_id).await,
        "2026-02-15T10:30:00Z"
    );
}

#[tokio::test]
async fn update_watered_occurrence_recomputes_plant_last_watered() {
    let (app, _dir) = app().await;
    let plant_id = create_plant(&app).await;

    let response = app
        .clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/plants/{plant_id}/care"),
            Some(r#"{"event_type":"watered","occurred_at":"2026-02-10T10:00:00Z"}"#),
        ))
        .await
        .unwrap();
    let event_id = body_json(response).await["id"].as_i64().unwrap();

    let response = app
        .clone()
        .oneshot(json_request(
            "PUT",
            &format!("/api/plants/{plant_id}/care/{event_id}"),
            Some(r#"{"event_type":"watered","notes":null,"occurred_at":"2026-02-15T10:00:00Z"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        plant_last_watered(&app, plant_id).await,
        "2026-02-15T10:00:00Z"
    );
}

#[tokio::test]
async fn update_between_non_watered_events_does_not_change_last_watered() {
    let (app, _dir) = app().await;
    let plant_id = create_plant(&app).await;

    app.clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/plants/{plant_id}/care"),
            Some(r#"{"event_type":"watered","occurred_at":"2026-02-10T10:00:00Z"}"#),
        ))
        .await
        .unwrap();
    let response = app
        .clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/plants/{plant_id}/care"),
            Some(r#"{"event_type":"fertilized","occurred_at":"2026-02-11T10:00:00Z"}"#),
        ))
        .await
        .unwrap();
    let event_id = body_json(response).await["id"].as_i64().unwrap();

    let response = app
        .clone()
        .oneshot(json_request(
            "PUT",
            &format!("/api/plants/{plant_id}/care/{event_id}"),
            Some(r#"{"event_type":"pruned","notes":null,"occurred_at":"2026-02-12T10:00:00Z"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        plant_last_watered(&app, plant_id).await,
        "2026-02-10T10:00:00Z"
    );
}

#[tokio::test]
async fn create_invalid_type() {
    let (app, _dir) = app().await;
    let id = create_plant(&app).await;

    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/plants/{id}/care"),
            Some(r#"{"event_type":"unknown"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn create_missing_type() {
    let (app, _dir) = app().await;
    let id = create_plant(&app).await;

    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/plants/{id}/care"),
            Some(r"{}"),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn create_nonexistent_plant() {
    let (app, _dir) = app().await;
    let resp = app
        .oneshot(json_request(
            "POST",
            "/api/plants/999/care",
            Some(r#"{"event_type":"watered"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn list_nonexistent_plant() {
    let (app, _dir) = app().await;
    let resp = app
        .oneshot(json_request("GET", "/api/plants/999/care", None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_event() {
    let (app, _dir) = app().await;
    let plant_id = create_plant(&app).await;

    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/plants/{plant_id}/care"),
            Some(r#"{"event_type":"pruned"}"#),
        ))
        .await
        .unwrap();
    let json = body_json(resp).await;
    let event_id = json["id"].as_i64().unwrap();

    let resp = app
        .clone()
        .oneshot(json_request(
            "DELETE",
            &format!("/api/plants/{plant_id}/care/{event_id}"),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    // Verify gone
    let resp = app
        .oneshot(json_request(
            "GET",
            &format!("/api/plants/{plant_id}/care"),
            None,
        ))
        .await
        .unwrap();
    let json = body_json(resp).await;
    assert_eq!(json, serde_json::json!([]));
}

#[tokio::test]
async fn delete_nonexistent_event() {
    let (app, _dir) = app().await;
    let plant_id = create_plant(&app).await;

    let resp = app
        .oneshot(json_request(
            "DELETE",
            &format!("/api/plants/{plant_id}/care/999"),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn list_ordered_by_occurred_at_desc() {
    let (app, _dir) = app().await;
    let id = create_plant(&app).await;

    // Create events with different occurred_at
    app.clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/plants/{id}/care"),
            Some(r#"{"event_type":"watered","occurred_at":"2026-02-10T08:00:00"}"#),
        ))
        .await
        .unwrap();
    app.clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/plants/{id}/care"),
            Some(r#"{"event_type":"fertilized","occurred_at":"2026-02-12T08:00:00"}"#),
        ))
        .await
        .unwrap();
    app.clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/plants/{id}/care"),
            Some(r#"{"event_type":"pruned","occurred_at":"2026-02-11T08:00:00"}"#),
        ))
        .await
        .unwrap();

    let resp = app
        .oneshot(json_request("GET", &format!("/api/plants/{id}/care"), None))
        .await
        .unwrap();
    let json = body_json(resp).await;
    let events = json.as_array().unwrap();
    assert_eq!(events.len(), 3);
    assert_eq!(events[0]["event_type"], "fertilized");
    assert_eq!(events[1]["event_type"], "pruned");
    assert_eq!(events[2]["event_type"], "watered");
}

// --- Global endpoint tests ---

#[tokio::test]
async fn global_empty() {
    let (app, _dir) = app().await;
    let resp = app
        .oneshot(json_request("GET", "/api/care", None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    assert_eq!(json["events"], serde_json::json!([]));
    assert_eq!(json["has_more"], false);
}

#[tokio::test]
async fn global_returns_events_across_plants() {
    let (app, _dir) = app().await;

    // Create two plants
    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/plants",
            Some(r#"{"name":"Plant A"}"#),
        ))
        .await
        .unwrap();
    let id_a = body_json(resp).await["id"].as_i64().unwrap();

    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/plants",
            Some(r#"{"name":"Plant B"}"#),
        ))
        .await
        .unwrap();
    let id_b = body_json(resp).await["id"].as_i64().unwrap();

    // Add events to each
    app.clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/plants/{id_a}/care"),
            Some(r#"{"event_type":"watered","occurred_at":"2026-02-10T08:00:00"}"#),
        ))
        .await
        .unwrap();
    app.clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/plants/{id_b}/care"),
            Some(r#"{"event_type":"fertilized","occurred_at":"2026-02-11T08:00:00"}"#),
        ))
        .await
        .unwrap();

    let resp = app
        .oneshot(json_request("GET", "/api/care", None))
        .await
        .unwrap();
    let json = body_json(resp).await;
    let events = json["events"].as_array().unwrap();
    assert_eq!(events.len(), 2);
    // Newest first
    assert_eq!(events[0]["plant_name"], "Plant B");
    assert_eq!(events[1]["plant_name"], "Plant A");
}

#[tokio::test]
async fn global_respects_limit() {
    let (app, _dir) = app().await;
    let id = create_plant(&app).await;

    for i in 0..5 {
        app.clone()
            .oneshot(json_request(
                "POST",
                &format!("/api/plants/{id}/care"),
                Some(&format!(
                    r#"{{"event_type":"watered","occurred_at":"2026-02-{:02}T08:00:00"}}"#,
                    10 + i
                )),
            ))
            .await
            .unwrap();
    }

    let resp = app
        .oneshot(json_request("GET", "/api/care?limit=2", None))
        .await
        .unwrap();
    let json = body_json(resp).await;
    assert_eq!(json["events"].as_array().unwrap().len(), 2);
    assert_eq!(json["has_more"], true);
}

#[tokio::test]
async fn global_cursor_pagination() {
    let (app, _dir) = app().await;
    let id = create_plant(&app).await;

    for i in 0..4 {
        app.clone()
            .oneshot(json_request(
                "POST",
                &format!("/api/plants/{id}/care"),
                Some(&format!(
                    r#"{{"event_type":"watered","occurred_at":"2026-02-{:02}T08:00:00"}}"#,
                    10 + i
                )),
            ))
            .await
            .unwrap();
    }

    // Get first page
    let resp = app
        .clone()
        .oneshot(json_request("GET", "/api/care?limit=2", None))
        .await
        .unwrap();
    let json = body_json(resp).await;
    let events = json["events"].as_array().unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(json["has_more"], true);
    let last_id = events[1]["id"].as_i64().unwrap();

    // Get second page
    let resp = app
        .clone()
        .oneshot(json_request(
            "GET",
            &format!("/api/care?limit=2&before={last_id}"),
            None,
        ))
        .await
        .unwrap();
    let json = body_json(resp).await;
    let events = json["events"].as_array().unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(json["has_more"], false);
}

#[tokio::test]
async fn global_defaults_to_twenty_and_preserves_page_shape() {
    let (app, pool, _dir) = app_with_pool().await;
    let plant_id = create_plant(&app).await;

    for second in 0..21 {
        insert_care_event(
            &pool,
            plant_id,
            "watered",
            &format!("2026-01-01T00:00:{second:02}Z"),
        )
        .await;
    }

    let response = app
        .oneshot(json_request("GET", "/api/care", None))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    assert_eq!(json.as_object().unwrap().len(), 2);
    assert_eq!(json["events"].as_array().unwrap().len(), 20);
    assert_eq!(json["has_more"], true);
}

#[tokio::test]
async fn global_supports_and_caps_pages_at_five_hundred() {
    let (app, pool, _dir) = app_with_pool().await;
    let plant_id = create_plant(&app).await;

    for _ in 0..501 {
        insert_care_event(&pool, plant_id, "watered", "2026-01-01T00:00:00Z").await;
    }

    for limit in [500, 999] {
        let response = app
            .clone()
            .oneshot(json_request(
                "GET",
                &format!("/api/care?limit={limit}"),
                None,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let json = body_json(response).await;
        let events = json["events"].as_array().unwrap();
        assert_eq!(events.len(), 500);
        assert_eq!(json["has_more"], true);
        assert_eq!(events[0]["id"], 501);
        assert_eq!(events[499]["id"], 2);
    }
}

#[tokio::test]
async fn global_cursor_follows_chronology_for_backdated_ids() {
    let (app, pool, _dir) = app_with_pool().await;
    let plant_id = create_plant(&app).await;
    let newest_id = insert_care_event(&pool, plant_id, "watered", "2026-01-03T00:00:00Z").await;
    let oldest_id = insert_care_event(&pool, plant_id, "watered", "2026-01-01T00:00:00Z").await;
    let middle_id = insert_care_event(&pool, plant_id, "watered", "2026-01-02T00:00:00Z").await;

    let mut before = None;
    let mut actual_ids = Vec::new();
    loop {
        let uri = before.map_or_else(
            || "/api/care?limit=1".to_string(),
            |event_id| format!("/api/care?limit=1&before={event_id}"),
        );
        let response = app
            .clone()
            .oneshot(json_request("GET", &uri, None))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let json = body_json(response).await;
        let events = json["events"].as_array().unwrap();
        actual_ids.extend(events.iter().map(|event| event["id"].as_i64().unwrap()));

        if !json["has_more"].as_bool().unwrap() {
            break;
        }
        before = events.last().and_then(|event| event["id"].as_i64());
    }

    assert_eq!(actual_ids, [newest_id, middle_id, oldest_id]);
}

#[tokio::test]
async fn global_cursor_orders_supported_timestamp_formats_chronologically() {
    let (app, _dir) = app().await;
    let plant_id = create_plant(&app).await;
    let mut ids = Vec::new();

    for occurred_at in [
        "2026-02-15T10:00:00Z",
        "2026-02-15T09:30:00-01:00",
        "2026-02-15 09:00:00",
    ] {
        let response = app
            .clone()
            .oneshot(json_request(
                "POST",
                &format!("/api/plants/{plant_id}/care"),
                Some(&format!(
                    r#"{{"event_type":"watered","occurred_at":"{occurred_at}"}}"#
                )),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
        ids.push(body_json(response).await["id"].as_i64().unwrap());
    }

    let expected_ids = [ids[1], ids[0], ids[2]];
    let mut actual_ids = Vec::new();
    let mut before = None;

    loop {
        let uri = before.map_or_else(
            || "/api/care?limit=1".to_string(),
            |event_id| format!("/api/care?limit=1&before={event_id}"),
        );
        let response = app
            .clone()
            .oneshot(json_request("GET", &uri, None))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let json = body_json(response).await;
        let events = json["events"].as_array().unwrap();
        actual_ids.extend(events.iter().map(|event| event["id"].as_i64().unwrap()));

        if !json["has_more"].as_bool().unwrap() {
            break;
        }
        before = events.last().and_then(|event| event["id"].as_i64());
    }

    assert_eq!(actual_ids, expected_ids);
}

#[tokio::test]
async fn global_cursor_keeps_malformed_historical_timestamps_reachable() {
    let (app, pool, _dir) = app_with_pool().await;
    let plant_id = create_plant(&app).await;
    let valid_id = insert_care_event(&pool, plant_id, "watered", "2026-01-01T00:00:00Z").await;
    let first_malformed_id = insert_care_event(&pool, plant_id, "watered", "not-a-datetime").await;
    let second_malformed_id = insert_care_event(&pool, plant_id, "watered", "also-invalid").await;

    let mut actual_ids = Vec::new();
    let mut before = None;
    loop {
        let uri = before.map_or_else(
            || "/api/care?limit=1".to_string(),
            |event_id| format!("/api/care?limit=1&before={event_id}"),
        );
        let response = app
            .clone()
            .oneshot(json_request("GET", &uri, None))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let json = body_json(response).await;
        let events = json["events"].as_array().unwrap();
        actual_ids.extend(events.iter().map(|event| event["id"].as_i64().unwrap()));

        if !json["has_more"].as_bool().unwrap() {
            break;
        }
        before = events.last().and_then(|event| event["id"].as_i64());
    }

    assert_eq!(
        actual_ids,
        [valid_id, second_malformed_id, first_malformed_id]
    );
}

#[tokio::test]
async fn global_cursor_breaks_equal_timestamps_by_descending_id() {
    let (app, pool, _dir) = app_with_pool().await;
    let plant_id = create_plant(&app).await;
    let first_id = insert_care_event(&pool, plant_id, "watered", "2026-01-01T00:00:00Z").await;
    let second_id = insert_care_event(&pool, plant_id, "watered", "2026-01-01T00:00:00Z").await;
    let third_id = insert_care_event(&pool, plant_id, "watered", "2026-01-01T00:00:00Z").await;

    let first_response = app
        .clone()
        .oneshot(json_request("GET", "/api/care?limit=2", None))
        .await
        .unwrap();
    let first_page = body_json(first_response).await;
    assert_eq!(first_page["events"][0]["id"], third_id);
    assert_eq!(first_page["events"][1]["id"], second_id);
    assert_eq!(first_page["has_more"], true);

    let second_response = app
        .oneshot(json_request(
            "GET",
            &format!("/api/care?limit=2&before={second_id}"),
            None,
        ))
        .await
        .unwrap();
    let second_page = body_json(second_response).await;
    assert_eq!(
        second_page["events"],
        serde_json::json!([{"id": first_id, "plant_id": plant_id, "plant_name": "TestPlant", "event_type": "watered", "notes": null, "photo_url": null, "occurred_at": "2026-01-01T00:00:00Z", "created_at": "2026-01-01T00:00:00Z"}])
    );
    assert_eq!(second_page["has_more"], false);
}

#[tokio::test]
async fn global_filtered_continuation_paginates_matching_events() {
    let (app, pool, _dir) = app_with_pool().await;
    let plant_id = create_plant(&app).await;
    let newest_id = insert_care_event(&pool, plant_id, "watered", "2026-01-04T00:00:00Z").await;
    insert_care_event(&pool, plant_id, "fertilized", "2026-01-03T00:00:00Z").await;
    let middle_id = insert_care_event(&pool, plant_id, "watered", "2026-01-02T00:00:00Z").await;
    let oldest_id = insert_care_event(&pool, plant_id, "watered", "2026-01-01T00:00:00Z").await;

    let first_response = app
        .clone()
        .oneshot(json_request("GET", "/api/care?limit=1&type=watered", None))
        .await
        .unwrap();
    assert_eq!(first_response.status(), StatusCode::OK);
    let first_page = body_json(first_response).await;
    assert_eq!(first_page["events"][0]["id"], newest_id);
    assert_eq!(first_page["has_more"], true);

    let second_response = app
        .clone()
        .oneshot(json_request(
            "GET",
            &format!("/api/care?limit=1&type=watered&before={newest_id}"),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(second_response.status(), StatusCode::OK);
    let second_page = body_json(second_response).await;
    assert_eq!(second_page["events"][0]["id"], middle_id);
    assert_eq!(second_page["has_more"], true);

    let third_response = app
        .oneshot(json_request(
            "GET",
            &format!("/api/care?limit=1&type=watered&before={middle_id}"),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(third_response.status(), StatusCode::OK);
    let third_page = body_json(third_response).await;
    assert_eq!(third_page["events"][0]["id"], oldest_id);
    assert_eq!(third_page["has_more"], false);
}

#[tokio::test]
async fn global_cursor_is_resolved_independent_of_event_type_filters() {
    let (app, pool, _dir) = app_with_pool().await;
    let plant_id = create_plant(&app).await;
    let newest_id = insert_care_event(&pool, plant_id, "watered", "2026-01-03T00:00:00Z").await;
    let cursor_id = insert_care_event(&pool, plant_id, "fertilized", "2026-01-02T00:00:00Z").await;
    let oldest_id = insert_care_event(&pool, plant_id, "watered", "2026-01-01T00:00:00Z").await;

    let response = app
        .oneshot(json_request(
            "GET",
            &format!("/api/care?limit=1&type=watered&before={cursor_id}"),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    assert_eq!(json["events"][0]["id"], oldest_id);
    assert_ne!(json["events"][0]["id"], newest_id);
    assert_eq!(json["has_more"], false);
}

#[tokio::test]
async fn global_unknown_cursor_returns_validation_error() {
    let (app, _dir) = app().await;

    let response = app
        .oneshot(json_request("GET", "/api/care?before=999", None))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let json = body_json(response).await;
    assert_eq!(json["code"], "CARE_EVENT_INVALID_CURSOR");
    assert_eq!(json["message"], "Invalid care event cursor");
}

#[tokio::test]
async fn global_type_filter() {
    let (app, _dir) = app().await;
    let id = create_plant(&app).await;

    app.clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/plants/{id}/care"),
            Some(r#"{"event_type":"watered"}"#),
        ))
        .await
        .unwrap();
    app.clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/plants/{id}/care"),
            Some(r#"{"event_type":"fertilized"}"#),
        ))
        .await
        .unwrap();

    let resp = app
        .clone()
        .oneshot(json_request("GET", "/api/care?type=watered", None))
        .await
        .unwrap();
    let json = body_json(resp).await;
    let events = json["events"].as_array().unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["event_type"], "watered");
}

#[tokio::test]
async fn global_multi_type_filter() {
    let (app, _dir) = app().await;
    let id = create_plant(&app).await;

    for t in &["watered", "fertilized", "pruned"] {
        app.clone()
            .oneshot(json_request(
                "POST",
                &format!("/api/plants/{id}/care"),
                Some(&format!(r#"{{"event_type":"{t}"}}"#)),
            ))
            .await
            .unwrap();
    }

    let resp = app
        .clone()
        .oneshot(json_request(
            "GET",
            "/api/care?type=watered&type=fertilized",
            None,
        ))
        .await
        .unwrap();
    let json = body_json(resp).await;
    let events = json["events"].as_array().unwrap();
    assert_eq!(events.len(), 2);
    let types: Vec<&str> = events
        .iter()
        .map(|e| e["event_type"].as_str().unwrap())
        .collect();
    assert!(types.contains(&"watered"));
    assert!(types.contains(&"fertilized"));
    assert!(!types.contains(&"pruned"));
}

#[tokio::test]
async fn global_multi_type_filter_paginates_matching_events() {
    let (app, pool, _dir) = app_with_pool().await;
    let plant_id = create_plant(&app).await;
    let newest_id = insert_care_event(&pool, plant_id, "watered", "2026-01-04T00:00:00Z").await;
    insert_care_event(&pool, plant_id, "pruned", "2026-01-03T00:00:00Z").await;
    let middle_id = insert_care_event(&pool, plant_id, "fertilized", "2026-01-02T00:00:00Z").await;
    let oldest_id = insert_care_event(&pool, plant_id, "watered", "2026-01-01T00:00:00Z").await;

    let mut actual_ids = Vec::new();
    let mut before = None;
    loop {
        let uri = before.map_or_else(
            || "/api/care?limit=1&type=watered&type=fertilized".to_string(),
            |event_id| format!("/api/care?limit=1&type=watered&type=fertilized&before={event_id}"),
        );
        let response = app
            .clone()
            .oneshot(json_request("GET", &uri, None))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let json = body_json(response).await;
        let events = json["events"].as_array().unwrap();
        actual_ids.extend(events.iter().map(|event| event["id"].as_i64().unwrap()));

        if !json["has_more"].as_bool().unwrap() {
            break;
        }
        before = events.last().and_then(|event| event["id"].as_i64());
    }

    assert_eq!(actual_ids, [newest_id, middle_id, oldest_id]);
}

#[tokio::test]
async fn global_invalid_type_filter() {
    let (app, _dir) = app().await;
    let resp = app
        .oneshot(json_request("GET", "/api/care?type=invalid", None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn global_invalid_type_in_multi_filter() {
    let (app, _dir) = app().await;
    let resp = app
        .oneshot(json_request(
            "GET",
            "/api/care?type=watered&type=invalid",
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn global_no_type_filter_returns_all() {
    let (app, _dir) = app().await;
    let id = create_plant(&app).await;

    for t in &["watered", "fertilized", "pruned"] {
        app.clone()
            .oneshot(json_request(
                "POST",
                &format!("/api/plants/{id}/care"),
                Some(&format!(r#"{{"event_type":"{t}"}}"#)),
            ))
            .await
            .unwrap();
    }

    let resp = app
        .oneshot(json_request("GET", "/api/care", None))
        .await
        .unwrap();
    let json = body_json(resp).await;
    assert_eq!(json["events"].as_array().unwrap().len(), 3);
}

// --- Water auto-logs care event ---

#[tokio::test]
async fn water_auto_logs_care_event() {
    let (app, _dir) = app().await;
    let id = create_plant(&app).await;

    // Water the plant
    app.clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/plants/{id}/water"),
            None,
        ))
        .await
        .unwrap();

    // Check care events
    let resp = app
        .oneshot(json_request("GET", &format!("/api/plants/{id}/care"), None))
        .await
        .unwrap();
    let json = body_json(resp).await;
    let events = json.as_array().unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["event_type"], "watered");
    assert_eq!(events[0]["plant_id"], id);
}

// --- Watered care event updates computed last_watered ---

#[tokio::test]
async fn create_watered_event_updates_plant_last_watered() {
    let (app, _dir) = app().await;
    let plant_id = create_plant(&app).await;

    // Plant starts with no last_watered
    let resp = app
        .clone()
        .oneshot(json_request(
            "GET",
            &format!("/api/plants/{plant_id}"),
            None,
        ))
        .await
        .unwrap();
    let json = body_json(resp).await;
    assert!(json["last_watered"].is_null());

    // Create a watered care event
    app.clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/plants/{plant_id}/care"),
            Some(r#"{"event_type":"watered","occurred_at":"2026-02-15T10:00:00"}"#),
        ))
        .await
        .unwrap();

    // Plant's last_watered should now reflect the care event
    let resp = app
        .oneshot(json_request(
            "GET",
            &format!("/api/plants/{plant_id}"),
            None,
        ))
        .await
        .unwrap();
    let json = body_json(resp).await;
    assert_eq!(json["last_watered"], "2026-02-15T10:00:00");
}

#[tokio::test]
async fn delete_watered_event_updates_plant_last_watered() {
    let (app, _dir) = app().await;
    let plant_id = create_plant(&app).await;

    // Create two watered events
    app.clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/plants/{plant_id}/care"),
            Some(r#"{"event_type":"watered","occurred_at":"2026-02-10T10:00:00"}"#),
        ))
        .await
        .unwrap();

    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/plants/{plant_id}/care"),
            Some(r#"{"event_type":"watered","occurred_at":"2026-02-15T10:00:00"}"#),
        ))
        .await
        .unwrap();
    let later_event = body_json(resp).await;
    let later_event_id = later_event["id"].as_i64().unwrap();

    // last_watered should be the latest event
    let resp = app
        .clone()
        .oneshot(json_request(
            "GET",
            &format!("/api/plants/{plant_id}"),
            None,
        ))
        .await
        .unwrap();
    let json = body_json(resp).await;
    assert_eq!(json["last_watered"], "2026-02-15T10:00:00");

    // Delete the later event
    let resp = app
        .clone()
        .oneshot(json_request(
            "DELETE",
            &format!("/api/plants/{plant_id}/care/{later_event_id}"),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    // last_watered should revert to the earlier event
    let resp = app
        .oneshot(json_request(
            "GET",
            &format!("/api/plants/{plant_id}"),
            None,
        ))
        .await
        .unwrap();
    let json = body_json(resp).await;
    assert_eq!(json["last_watered"], "2026-02-10T10:00:00");
}

#[tokio::test]
async fn non_watered_event_does_not_affect_last_watered() {
    let (app, _dir) = app().await;
    let plant_id = create_plant(&app).await;

    // Create a non-watered care event
    app.clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/plants/{plant_id}/care"),
            Some(r#"{"event_type":"fertilized"}"#),
        ))
        .await
        .unwrap();

    // Plant's last_watered should still be null
    let resp = app
        .oneshot(json_request(
            "GET",
            &format!("/api/plants/{plant_id}"),
            None,
        ))
        .await
        .unwrap();
    let json = body_json(resp).await;
    assert!(json["last_watered"].is_null());
}

// --- AI consultation event type ---

#[tokio::test]
async fn create_ai_consultation_event() {
    let (app, _dir) = app().await;
    let id = create_plant(&app).await;

    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/plants/{id}/care"),
            Some(r#"{"event_type":"ai-consultation","notes":"Diagnosed overwatering"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let json = body_json(resp).await;
    assert_eq!(json["event_type"], "ai-consultation");
    assert_eq!(json["notes"], "Diagnosed overwatering");
}

#[tokio::test]
async fn ai_consultation_does_not_affect_last_watered() {
    let (app, _dir) = app().await;
    let plant_id = create_plant(&app).await;

    app.clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/plants/{plant_id}/care"),
            Some(r#"{"event_type":"ai-consultation","notes":"Summary"}"#),
        ))
        .await
        .unwrap();

    let resp = app
        .oneshot(json_request(
            "GET",
            &format!("/api/plants/{plant_id}"),
            None,
        ))
        .await
        .unwrap();
    let json = body_json(resp).await;
    assert!(json["last_watered"].is_null());
}

// --- Cascade delete ---

#[tokio::test]
async fn delete_plant_cascades_care_events() {
    let (app, _dir) = app().await;
    let id = create_plant(&app).await;

    // Create care event
    app.clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/plants/{id}/care"),
            Some(r#"{"event_type":"watered"}"#),
        ))
        .await
        .unwrap();

    // Delete plant
    let resp = app
        .clone()
        .oneshot(json_request("DELETE", &format!("/api/plants/{id}"), None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    // Global endpoint should be empty
    let resp = app
        .oneshot(json_request("GET", "/api/care", None))
        .await
        .unwrap();
    let json = body_json(resp).await;
    assert_eq!(json["events"], serde_json::json!([]));
}
