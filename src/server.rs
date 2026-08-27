use std::sync::Arc;
use std::time::Instant;

use axum::Extension;
use axum::Router;
use axum::extract::Request;
use axum::http::{StatusCode, Uri};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Json, Redirect, Response};
use axum::routing::{get, post};
use serde_json::{Value, json};
use sqlx::SqlitePool;
use tokio::net::TcpListener;
use tokio::signal;
use tower_http::services::ServeDir;
use tower_sessions::cookie::SameSite;
use tower_sessions::{Expiry, MemoryStore, Session, SessionManagerLayer};
use tracing::{debug, info};

use crate::api;
use crate::api::error::ApiError;
use crate::auth;
use crate::auth::return_to::SafeReturnTo;
use crate::embedded::{exact_static_handler, index_handler, is_public_asset, static_handler};
use crate::state::AppState;

pub fn router(state: AppState) -> Router {
    if state.auth.is_some() {
        enabled_router(&state)
    } else {
        disabled_router(state)
    }
}

fn disabled_router(state: AppState) -> Router {
    let uploads = ServeDir::new(state.image_store.upload_dir());
    let pool = state.pool.clone();
    Router::new()
        .route("/health", get(move || health(pool)))
        .route("/auth/config", get(auth::routes::config))
        .route("/api/info", get(info))
        .nest("/api", api::router(state))
        .nest_service("/uploads", uploads)
        .fallback(static_handler)
        .layer(middleware::from_fn(access_log))
}

fn enabled_router(state: &AppState) -> Router {
    let pool = state.pool.clone();
    let api_state = state.clone();
    let auth = state
        .auth
        .clone()
        .expect("enabled router requires auth state");
    let uploads = Router::new()
        .fallback_service(ServeDir::new(state.image_store.upload_dir()))
        .layer(middleware::from_fn_with_state(
            auth.clone(),
            require_upload_auth,
        ));

    Router::new()
        .route("/health", get(move || health(pool)))
        .route("/login", get(index_handler))
        .nest(
            "/auth",
            Router::new()
                .route("/config", get(auth::routes::config))
                .route("/login", get(auth::routes::login))
                .route("/callback", get(auth::routes::callback))
                .route("/logout", post(auth::routes::logout))
                .fallback(StatusCode::NOT_FOUND),
        )
        .route(
            "/api/info",
            get(info).route_layer(middleware::from_fn_with_state(
                auth.clone(),
                require_api_auth,
            )),
        )
        .nest(
            "/api",
            api::router(api_state).layer(middleware::from_fn_with_state(
                auth.clone(),
                require_api_auth,
            )),
        )
        .nest("/uploads", uploads)
        .fallback({
            let auth = auth.clone();
            move |session: Session, uri: Uri| protected_document(auth.clone(), session, uri)
        })
        .layer(Extension(auth))
        .layer(
            SessionManagerLayer::new(MemoryStore::default())
                .with_name("flowl.sid")
                .with_http_only(true)
                .with_secure(true)
                .with_same_site(SameSite::Lax)
                .with_path("/")
                // Each login sets an absolute deadline: five minutes for pre-auth and twelve
                // hours for the authenticated session. A global inactivity expiry would slide.
                .with_expiry(Expiry::OnSessionEnd)
                .with_private(tower_sessions::cookie::Key::generate()),
        )
        .layer(middleware::from_fn(no_store_auth_responses))
        .layer(middleware::from_fn(access_log))
}

async fn no_store_auth_responses(request: Request, next: Next) -> Response {
    let is_auth_route =
        matches!(request.uri().path(), "/auth") || request.uri().path().starts_with("/auth/");
    let mut response = next.run(request).await;
    if is_auth_route {
        response.headers_mut().insert(
            axum::http::header::CACHE_CONTROL,
            axum::http::HeaderValue::from_static("no-store"),
        );
    }
    response
}

async fn require_api_auth(
    Extension(auth): Extension<Arc<auth::AuthState>>,
    request: Request,
    next: Next,
) -> Response {
    let session = request.extensions().get::<Session>().cloned();
    if has_session(auth.as_ref(), session).await {
        next.run(request).await
    } else {
        ApiError::Unauthorized("AUTHENTICATION_REQUIRED").into_response()
    }
}

async fn require_upload_auth(
    Extension(auth): Extension<Arc<auth::AuthState>>,
    request: Request,
    next: Next,
) -> Response {
    let session = request.extensions().get::<Session>().cloned();
    if has_session(auth.as_ref(), session).await {
        next.run(request).await
    } else {
        let mut response = StatusCode::UNAUTHORIZED.into_response();
        response.headers_mut().insert(
            axum::http::header::CACHE_CONTROL,
            axum::http::HeaderValue::from_static("no-store"),
        );
        response
    }
}

async fn has_session(auth: &auth::AuthState, session: Option<Session>) -> bool {
    let Some(session) = session else {
        return false;
    };
    auth::routes::has_authenticated_session(&session, auth).await
}

async fn protected_document(auth: Arc<auth::AuthState>, session: Session, uri: Uri) -> Response {
    if is_public_asset(uri.path()) {
        return exact_static_handler(uri).await;
    }
    if auth::routes::has_authenticated_session(&session, auth.as_ref()).await {
        return static_handler(uri).await;
    }
    let target = SafeReturnTo::fallback(
        &uri.path_and_query()
            .map_or_else(|| "/".to_string(), |value| value.as_str().to_string()),
    );
    let location = format!(
        "/login?return_to={}",
        url::form_urlencoded::byte_serialize(target.as_str().as_bytes()).collect::<String>()
    );
    let mut response = Redirect::to(&location).into_response();
    response.headers_mut().insert(
        axum::http::header::CACHE_CONTROL,
        axum::http::HeaderValue::from_static("no-store"),
    );
    response
}

async fn access_log(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let start = Instant::now();
    let response = next.run(request).await;
    let status = response.status().as_u16();
    let latency = start.elapsed();
    debug!(method = %method, path, status, latency_ms = latency.as_millis(), "access");
    response
}

async fn health(pool: SqlitePool) -> impl IntoResponse {
    match sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&pool)
        .await
    {
        Ok(_) => (StatusCode::OK, Json(json!({"status": "ok"}))),
        Err(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"status": "unhealthy"})),
        ),
    }
}

async fn info() -> Json<Value> {
    Json(json!({
        "version": env!("CARGO_PKG_VERSION"),
        "repository": env!("CARGO_PKG_REPOSITORY"),
        "license": env!("CARGO_PKG_LICENSE"),
    }))
}

/// # Errors
///
/// Returns an error if the TCP listener cannot bind to the given port.
pub async fn serve(router: Router, port: u16) -> std::io::Result<()> {
    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&addr).await?;
    info!("Listening on {addr}");

    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }

    info!("Shutdown signal received");
}
