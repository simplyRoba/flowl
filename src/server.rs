use std::sync::Arc;
use std::time::Instant;

use axum::Extension;
use axum::Router;
use axum::extract::Request;
use axum::http::{StatusCode, Uri};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Json, Response};
use axum::routing::{any, get};
use serde_json::{Value, json};
use sqlx::SqlitePool;
use tokio::net::TcpListener;
use tokio::signal;
use tower_http::services::ServeDir;
use tower_sessions::Session;
use tracing::{debug, info};

use crate::api;
use crate::api::error::ApiError;
use crate::auth;
use crate::embedded::{exact_static_handler, index_handler, is_public_asset, static_handler};
use crate::state::AppState;

pub fn router(state: AppState) -> Router {
    let app = if state.auth.is_some() {
        enabled_router(&state)
    } else {
        disabled_router(state)
    };
    app.layer(middleware::from_fn(no_store_auth_responses))
}

fn disabled_router(state: AppState) -> Router {
    let uploads = ServeDir::new(state.image_store.upload_dir());
    let pool = state.pool.clone();
    Router::new()
        .route("/health", get(move || health(pool)))
        .route("/auth/config", get(auth::app::public_config))
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
        .route("/login", get(login_document))
        .route("/auth/config", get(auth::app::public_config))
        .merge(auth.protocol_routes())
        .route("/auth", any(|| async { StatusCode::NOT_FOUND }))
        .route("/auth/", any(|| async { StatusCode::NOT_FOUND }))
        .route("/auth/{*path}", any(|| async { StatusCode::NOT_FOUND }))
        .route(
            "/api/info",
            get(info).route_layer(middleware::from_fn_with_state(
                auth.clone(),
                require_api_auth,
            )),
        )
        .route("/api", any(authenticated_api_root))
        .route("/api/", any(authenticated_api_root))
        .nest(
            "/api",
            api::router(api_state).layer(middleware::from_fn_with_state(
                auth.clone(),
                require_api_auth,
            )),
        )
        .route(
            "/uploads/",
            any(|| async { StatusCode::NOT_FOUND }).layer(middleware::from_fn_with_state(
                auth.clone(),
                require_upload_auth,
            )),
        )
        .nest("/uploads", uploads)
        .fallback({
            let auth = auth.clone();
            move |session: Session, uri: Uri| protected_document(auth.clone(), session, uri)
        })
        .layer(Extension(auth.clone()))
        .layer(auth.session_layer())
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

async fn login_document(
    Extension(auth): Extension<Arc<auth::Authentication>>,
    session: Session,
) -> Response {
    if auth.is_authenticated(&session).await {
        auth::redirect("/")
    } else {
        index_handler().await
    }
}

async fn authenticated_api_root(
    Extension(auth): Extension<Arc<auth::Authentication>>,
    session: Session,
) -> Response {
    if has_session(auth.as_ref(), Some(session)).await {
        StatusCode::NOT_FOUND.into_response()
    } else {
        ApiError::Unauthorized("AUTHENTICATION_REQUIRED").into_response()
    }
}

async fn require_api_auth(
    Extension(auth): Extension<Arc<auth::Authentication>>,
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
    Extension(auth): Extension<Arc<auth::Authentication>>,
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

async fn has_session(auth: &auth::Authentication, session: Option<Session>) -> bool {
    let Some(session) = session else {
        return false;
    };
    auth.is_authenticated(&session).await
}

async fn protected_document(
    auth: Arc<auth::Authentication>,
    session: Session,
    uri: Uri,
) -> Response {
    if is_public_asset(uri.path()) {
        return exact_static_handler(uri).await;
    }
    if auth.is_authenticated(&session).await {
        return static_handler(uri).await;
    }
    let target = auth::validate_return_to(
        uri.path_and_query()
            .map(axum::http::uri::PathAndQuery::as_str),
    );
    auth::login_redirect(&target)
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

#[cfg(test)]
mod tests {
    use super::*;
    use tower::ServiceExt;

    #[tokio::test]
    async fn outer_auth_headers_cover_inner_middleware_errors_only_in_auth_namespace() {
        // Synthetic middleware failure: this checks layer ordering, not session-store faults.
        let app = Router::new()
            .fallback(|| async { StatusCode::OK })
            .layer(middleware::from_fn(|_: Request, _: Next| async {
                StatusCode::INTERNAL_SERVER_ERROR
            }))
            .layer(middleware::from_fn(no_store_auth_responses));
        for path in ["/auth", "/auth/config", "/auth/unknown", "/authentic", "/"] {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri(path)
                        .body(axum::body::Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
            assert_eq!(
                response
                    .headers()
                    .get(axum::http::header::CACHE_CONTROL)
                    .map(|value| value.to_str().unwrap()),
                if path == "/auth" || path.starts_with("/auth/") {
                    Some("no-store")
                } else {
                    None
                },
            );
        }
    }
}
