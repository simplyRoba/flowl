mod common;

use axum::http::StatusCode;
use common::{body_json, json_request};
use serde_json::json;
use tower::ServiceExt;

async fn app() -> (axum::Router, tempfile::TempDir) {
    common::test_app().await
}

#[tokio::test]
async fn get_returns_defaults() {
    let (app, _dir) = app().await;
    let resp = app
        .oneshot(json_request("GET", "/api/settings", None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp).await;
    assert_eq!(body, json!({"theme": "system", "locale": "en"}));
}

#[tokio::test]
async fn put_theme_only() {
    let (app, _dir) = app().await;
    let resp = app
        .oneshot(json_request(
            "PUT",
            "/api/settings",
            Some(r#"{"theme":"dark"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp).await;
    assert_eq!(body, json!({"theme": "dark", "locale": "en"}));
}

#[tokio::test]
async fn put_locale_only() {
    let (app, _dir) = app().await;
    let resp = app
        .oneshot(json_request(
            "PUT",
            "/api/settings",
            Some(r#"{"locale":"de"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp).await;
    assert_eq!(body, json!({"theme": "system", "locale": "de"}));
}

#[tokio::test]
async fn put_both_fields() {
    let (app, _dir) = app().await;
    let resp = app
        .oneshot(json_request(
            "PUT",
            "/api/settings",
            Some(r#"{"theme":"light","locale":"es"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp).await;
    assert_eq!(body, json!({"theme": "light", "locale": "es"}));
}

#[tokio::test]
async fn put_empty_body() {
    let (app, _dir) = app().await;
    let resp = app
        .oneshot(json_request("PUT", "/api/settings", Some("{}")))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp).await;
    assert_eq!(body, json!({"theme": "system", "locale": "en"}));
}

#[tokio::test]
async fn put_invalid_theme() {
    let (app, _dir) = app().await;
    let resp = app
        .oneshot(json_request(
            "PUT",
            "/api/settings",
            Some(r#"{"theme":"blue"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn put_invalid_locale() {
    let (app, _dir) = app().await;
    let resp = app
        .oneshot(json_request(
            "PUT",
            "/api/settings",
            Some(r#"{"locale":"fr"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}
