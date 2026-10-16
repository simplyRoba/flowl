//! Guarded provider I/O. Injection is always behind destination validation.

use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use openidconnect::{AsyncHttpClient, HttpRequest, HttpResponse, http};

use super::is_safe_endpoint;
use crate::config::EnabledAuthConfig;

pub type TransportFuture<'a> =
    Pin<Box<dyn Future<Output = Result<HttpResponse, GuardedHttpError>> + Send + Sync + 'a>>;

/// Injectable transport; all runtime callers wrap it in the destination guard.
pub trait AuthHttpClient: Send + Sync {
    fn send(&self, request: HttpRequest) -> TransportFuture<'_>;
}

pub struct ReqwestAuthHttpClient {
    client: reqwest::Client,
}

impl ReqwestAuthHttpClient {
    /// # Errors
    /// Returns an error if the redirect-disabled Rustls client cannot be built.
    pub fn new() -> Result<Self, reqwest::Error> {
        Ok(Self {
            client: reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .use_rustls_tls()
                .build()?,
        })
    }
}

impl AuthHttpClient for ReqwestAuthHttpClient {
    fn send(&self, request: HttpRequest) -> TransportFuture<'_> {
        Box::pin(async move {
            let response = self
                .client
                .execute(request.try_into().map_err(|_| GuardedHttpError::Request)?)
                .await
                .map_err(|_| GuardedHttpError::Request)?;
            // Reject these statuses before buffering any untrusted body.
            if response.status().is_server_error() || response.status().is_redirection() {
                return Err(GuardedHttpError::ServerStatus(response.status()));
            }
            let mut builder = http::Response::builder()
                .status(response.status())
                .version(response.version());
            for (name, value) in response.headers() {
                builder = builder.header(name, value);
            }
            builder
                .body(
                    response
                        .bytes()
                        .await
                        .map_err(|_| GuardedHttpError::Request)?
                        .to_vec(),
                )
                .map_err(|_| GuardedHttpError::Http)
        })
    }
}

#[derive(Debug)]
pub enum GuardedHttpError {
    UnsafeUrl,
    ServerStatus(http::StatusCode),
    Request,
    Http,
}

impl fmt::Display for GuardedHttpError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsafeUrl => formatter.write_str("refused unsafe OIDC transport URL"),
            Self::ServerStatus(status) => write!(formatter, "OIDC HTTP status {}", status.as_u16()),
            Self::Request => formatter.write_str("OIDC HTTP request failed"),
            Self::Http => formatter.write_str("OIDC HTTP response conversion failed"),
        }
    }
}
impl std::error::Error for GuardedHttpError {}

pub(super) struct GuardedHttpClient {
    transport: Arc<dyn AuthHttpClient>,
    config: EnabledAuthConfig,
}

impl GuardedHttpClient {
    pub(super) fn new(config: EnabledAuthConfig, transport: Arc<dyn AuthHttpClient>) -> Self {
        Self { transport, config }
    }

    // Call before converting raw URLs to http::Uri, which drops fragments.
    pub(super) fn validate_raw_url(&self, value: &str) -> Result<(), GuardedHttpError> {
        if is_safe_endpoint(&self.config, value) {
            Ok(())
        } else {
            Err(GuardedHttpError::UnsafeUrl)
        }
    }
}

impl<'client> AsyncHttpClient<'client> for GuardedHttpClient {
    type Error = GuardedHttpError;
    type Future = TransportFuture<'client>;

    fn call(&'client self, request: HttpRequest) -> Self::Future {
        Box::pin(async move {
            self.validate_raw_url(&request.uri().to_string())?;
            let response = self.transport.send(request).await?;
            if response.status().is_server_error() || response.status().is_redirection() {
                return Err(GuardedHttpError::ServerStatus(response.status()));
            }
            Ok(response)
        })
    }
}

// Discovery fetches metadata first, then JWKS. Inspect only original metadata JSON:
// http::Uri drops fragments, while JWKS extension fields do not select destinations.
pub(super) struct DiscoveryHttpClient<'a> {
    http: &'a GuardedHttpClient,
    metadata_pending: AtomicBool,
}

impl<'a> DiscoveryHttpClient<'a> {
    pub(super) fn new(http: &'a GuardedHttpClient) -> Self {
        Self {
            http,
            metadata_pending: AtomicBool::new(true),
        }
    }
}

impl<'client> AsyncHttpClient<'client> for DiscoveryHttpClient<'client> {
    type Error = GuardedHttpError;
    type Future = TransportFuture<'client>;

    fn call(&'client self, request: HttpRequest) -> Self::Future {
        let is_metadata = self.metadata_pending.swap(false, Ordering::AcqRel);
        Box::pin(async move {
            let response = self.http.call(request).await?;
            if is_metadata
                && let Ok(value) = serde_json::from_slice::<serde_json::Value>(response.body())
                && let Some(jwks_uri) = value.get("jwks_uri").and_then(serde_json::Value::as_str)
            {
                self.http.validate_raw_url(jwks_uri)?;
            }
            Ok(response)
        })
    }
}
