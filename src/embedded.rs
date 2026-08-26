use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "ui/build/"]
struct Assets;

pub async fn static_handler(uri: axum::http::Uri) -> Response {
    let path = uri.path().trim_start_matches('/');

    // Try the exact path first
    if let Some(file) = Assets::get(path) {
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        return (
            StatusCode::OK,
            [(header::CONTENT_TYPE, mime.as_ref())],
            file.data,
        )
            .into_response();
    }

    // Fall back to index.html for SPA routing
    if let Some(file) = Assets::get("index.html") {
        let mime = mime_guess::from_path("index.html").first_or_octet_stream();
        return (
            StatusCode::OK,
            [(header::CONTENT_TYPE, mime.as_ref())],
            file.data,
        )
            .into_response();
    }

    StatusCode::NOT_FOUND.into_response()
}

/// Serves the embedded SPA document for the one public login document entry.
pub async fn index_handler() -> Response {
    match Assets::get("index.html") {
        Some(file) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
            file.data,
        )
            .into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

/// Returns whether this is an exact, non-document embedded resource. `index.html` and unknown
/// paths intentionally remain documents behind the authentication boundary.
pub fn is_public_asset(path: &str) -> bool {
    let path = path.trim_start_matches('/');
    if path.is_empty() || path == "index.html" || Assets::get(path).is_none() {
        return false;
    }
    matches!(
        std::path::Path::new(path)
            .extension()
            .and_then(|extension| extension.to_str()),
        Some(
            "js" | "css"
                | "map"
                | "png"
                | "jpg"
                | "jpeg"
                | "webp"
                | "svg"
                | "ico"
                | "woff"
                | "woff2"
                | "webmanifest"
                | "json"
        )
    ) || matches!(
        path,
        "service-worker.js" | "manifest.json" | "offline.html" | "favicon.ico"
    )
}

/// Serves a previously classified exact public embedded resource without SPA fallback.
pub async fn exact_static_handler(uri: axum::http::Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    match Assets::get(path) {
        Some(file) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, mime.as_ref())],
                file.data,
            )
                .into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}
