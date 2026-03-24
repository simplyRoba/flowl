mod common;

use flowl::ai::prompts::build_plant_context;

async fn insert_plant(pool: &sqlx::SqlitePool) -> i64 {
    sqlx::query_scalar::<_, i64>(
        "INSERT INTO plants (name, light_needs, watering_interval_days, created_at, updated_at) \
         VALUES ('TestPlant', 'indirect', 7, '2025-01-01T00:00:00Z', '2025-01-01T00:00:00Z') \
         RETURNING id",
    )
    .fetch_one(pool)
    .await
    .expect("Failed to insert plant")
}

async fn insert_care_event(
    pool: &sqlx::SqlitePool,
    plant_id: i64,
    event_type: &str,
    notes: Option<&str>,
    occurred_at: &str,
) {
    sqlx::query(
        "INSERT INTO care_events (plant_id, event_type, notes, occurred_at, created_at) \
         VALUES (?, ?, ?, ?, '2025-01-01T00:00:00Z')",
    )
    .bind(plant_id)
    .bind(event_type)
    .bind(notes)
    .bind(occurred_at)
    .execute(pool)
    .await
    .expect("Failed to insert care event");
}

#[tokio::test]
async fn watering_older_than_one_year_excluded_from_watering_dates() {
    let pool = common::test_pool().await;
    let plant_id = insert_plant(&pool).await;

    // Recent watering (within 1 year) — should appear in watering_dates
    insert_care_event(&pool, plant_id, "watered", None, "2026-03-01T10:00:00Z").await;

    // Old watering (>1 year ago) — should NOT appear in watering_dates
    insert_care_event(&pool, plant_id, "watered", None, "2024-06-01T10:00:00Z").await;

    let ctx = build_plant_context(&pool, plant_id)
        .await
        .map_err(|_| "build_plant_context failed")
        .unwrap();
    let json = serde_json::to_value(&ctx).unwrap();

    let dates = json["watering_dates"].as_array().unwrap();
    assert_eq!(dates.len(), 1);
    assert_eq!(dates[0], "2026-03-01");
}

#[tokio::test]
async fn old_watering_with_notes_in_care_events_but_not_watering_dates() {
    let pool = common::test_pool().await;
    let plant_id = insert_plant(&pool).await;

    // Old watering with notes (>1yr but <5yr) — should be in care_events only
    insert_care_event(
        &pool,
        plant_id,
        "watered",
        Some("Looked thirsty"),
        "2024-06-01T10:00:00Z",
    )
    .await;

    let ctx = build_plant_context(&pool, plant_id)
        .await
        .map_err(|_| "build_plant_context failed")
        .unwrap();
    let json = serde_json::to_value(&ctx).unwrap();

    // Not in watering_dates (older than 1 year)
    assert!(
        json.get("watering_dates").is_none()
            || json["watering_dates"].as_array().unwrap().is_empty()
    );

    // In care_events (within 5 years, has notes)
    let events = json["care_events"].as_array().unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["event_type"], "watered");
    assert_eq!(events[0]["notes"], "Looked thirsty");
}

#[tokio::test]
async fn non_watering_events_included_up_to_five_years() {
    let pool = common::test_pool().await;
    let plant_id = insert_plant(&pool).await;

    // Repotting 4 years ago — within 5yr window, should appear
    insert_care_event(&pool, plant_id, "repotted", None, "2022-06-01T10:00:00Z").await;

    // Fertilizing 2 years ago — within 5yr window, should appear
    insert_care_event(
        &pool,
        plant_id,
        "fertilized",
        Some("Half strength"),
        "2024-06-01T10:00:00Z",
    )
    .await;

    let ctx = build_plant_context(&pool, plant_id)
        .await
        .map_err(|_| "build_plant_context failed")
        .unwrap();
    let json = serde_json::to_value(&ctx).unwrap();

    let events = json["care_events"].as_array().unwrap();
    assert_eq!(events.len(), 2);
    // Ordered DESC — fertilized (2024) first, repotted (2022) second
    assert_eq!(events[0]["event_type"], "fertilized");
    assert_eq!(events[1]["event_type"], "repotted");
}

#[tokio::test]
async fn events_older_than_five_years_excluded() {
    let pool = common::test_pool().await;
    let plant_id = insert_plant(&pool).await;

    // Event 6 years ago — should be excluded from everything
    insert_care_event(&pool, plant_id, "repotted", None, "2020-01-01T10:00:00Z").await;

    // Old watering 6 years ago — excluded from both lists
    insert_care_event(
        &pool,
        plant_id,
        "watered",
        Some("Ancient note"),
        "2020-01-01T10:00:00Z",
    )
    .await;

    let ctx = build_plant_context(&pool, plant_id)
        .await
        .map_err(|_| "build_plant_context failed")
        .unwrap();
    let json = serde_json::to_value(&ctx).unwrap();

    // Both fields should be absent (empty vecs are skipped)
    assert!(json.get("watering_dates").is_none());
    assert!(json.get("care_events").is_none());
}

#[tokio::test]
async fn mixed_events_watering_with_notes_in_both_lists() {
    let pool = common::test_pool().await;
    let plant_id = insert_plant(&pool).await;

    // Recent watering without notes
    insert_care_event(&pool, plant_id, "watered", None, "2026-03-20T10:00:00Z").await;

    // Recent watering WITH notes — should appear in both lists
    insert_care_event(
        &pool,
        plant_id,
        "watered",
        Some("Soil was bone dry"),
        "2026-03-15T10:00:00Z",
    )
    .await;

    // Non-watering event
    insert_care_event(
        &pool,
        plant_id,
        "fertilized",
        Some("Liquid feed"),
        "2026-03-10T10:00:00Z",
    )
    .await;

    let ctx = build_plant_context(&pool, plant_id)
        .await
        .map_err(|_| "build_plant_context failed")
        .unwrap();
    let json = serde_json::to_value(&ctx).unwrap();

    // watering_dates: both watering events (with and without notes)
    let dates = json["watering_dates"].as_array().unwrap();
    assert_eq!(dates.len(), 2);
    assert_eq!(dates[0], "2026-03-20");
    assert_eq!(dates[1], "2026-03-15");

    // care_events: watering-with-notes + fertilized (no plain watering)
    let events = json["care_events"].as_array().unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0]["event_type"], "watered");
    assert_eq!(events[0]["notes"], "Soil was bone dry");
    assert_eq!(events[1]["event_type"], "fertilized");
    assert_eq!(events[1]["notes"], "Liquid feed");
}

#[tokio::test]
async fn no_care_events_omits_both_fields() {
    let pool = common::test_pool().await;
    let plant_id = insert_plant(&pool).await;

    let ctx = build_plant_context(&pool, plant_id)
        .await
        .map_err(|_| "build_plant_context failed")
        .unwrap();
    let json = serde_json::to_value(&ctx).unwrap();

    assert!(json.get("watering_dates").is_none());
    assert!(json.get("care_events").is_none());
}
