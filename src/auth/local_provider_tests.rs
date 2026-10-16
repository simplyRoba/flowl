//! Loopback-only protocol coverage. The fixture is intentionally local: production configuration
//! cannot opt into HTTP, and no test contacts an external identity provider.

use std::collections::HashMap;
use std::io::{self, Write};
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use axum::extract::{Form, Query, State};
use axum::http::{HeaderMap, HeaderValue, Request, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use base64::Engine;
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use rsa::RsaPrivateKey;
use rsa::pkcs8::{EncodePrivateKey, LineEnding};
use rsa::rand_core::OsRng;
use rsa::traits::PublicKeyParts;
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

use tower::ServiceExt;
use tracing::Level;
use tracing_subscriber::fmt::MakeWriter;

use super::{
    AuthState, CallbackError, Clock, MonotonicClock, PendingLogin, ReqwestAuthHttpClient,
    TokenAuthMethod,
};
use crate::auth::return_to::SafeReturnTo;
use crate::config::EnabledAuthConfig;
use crate::images::ImageStore;
use crate::server;
use crate::state::AppState;

#[path = "lifecycle_tests.rs"]
mod lifecycle;

const CLIENT_ID: &str = "flowl-test-client";
const CLIENT_SECRET: &str = "flowl-test-secret";

#[derive(Clone)]
struct SigningKey {
    pem: String,
    kid: String,
    jwks: serde_json::Value,
}

struct Provider {
    base: String,
    method: TokenAuthMethod,
    signing_key: Mutex<SigningKey>,
    discovery_requests: AtomicUsize,
    authorization_requests: AtomicUsize,
    jwks_requests: AtomicUsize,
    jwks_failing: AtomicBool,
    malformed_jwks: AtomicBool,
    jwks_override: Mutex<Option<serde_json::Value>>,
    discovery_override: Mutex<Option<serde_json::Value>>,
    omit_token_auth_methods: AtomicBool,
    token_protocol_error: AtomicBool,
    token_protocol_error_description: Mutex<Option<String>>,
    token_unparseable_response: AtomicBool,
    token_requests: AtomicUsize,
    error_uri_requests: AtomicUsize,
    observed_basic: Mutex<Option<bool>>,
    discovery_issuer: Mutex<Option<String>>,
    expected_nonce: Mutex<Option<String>>,
    expected_challenge: Mutex<Option<String>>,
    expected_authorizations: Mutex<HashMap<String, (String, String)>>,
    token_claim_overrides: Mutex<serde_json::Map<String, serde_json::Value>>,
    omit_id_token: AtomicBool,
    token_signing_key_override: Mutex<Option<SigningKey>>,
    token_algorithm: Mutex<Algorithm>,
    discovery_redirect: AtomicBool,
    jwks_redirect: AtomicBool,
    token_response_override: Mutex<Option<(StatusCode, String)>>,
    unsigned_token: AtomicBool,
    token_gate: Mutex<Option<Arc<lifecycle::TokenGate>>>,
}

struct RunningProvider {
    provider: Arc<Provider>,
    task: tokio::task::JoinHandle<()>,
}

impl Drop for RunningProvider {
    fn drop(&mut self) {
        self.task.abort();
    }
}

impl RunningProvider {
    async fn start(method: TokenAuthMethod) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind loopback provider");
        let base = format!(
            "http://{}",
            listener.local_addr().expect("provider address")
        );
        let signing_key = generate_signing_key();
        let provider = Arc::new(Provider {
            base,
            method,
            signing_key: Mutex::new(signing_key),
            discovery_requests: AtomicUsize::new(0),
            authorization_requests: AtomicUsize::new(0),
            jwks_requests: AtomicUsize::new(0),
            jwks_failing: AtomicBool::new(false),
            malformed_jwks: AtomicBool::new(false),
            jwks_override: Mutex::new(None),
            discovery_override: Mutex::new(None),
            omit_token_auth_methods: AtomicBool::new(false),
            token_protocol_error: AtomicBool::new(false),
            token_protocol_error_description: Mutex::new(None),
            token_unparseable_response: AtomicBool::new(false),
            token_requests: AtomicUsize::new(0),
            error_uri_requests: AtomicUsize::new(0),
            observed_basic: Mutex::new(None),
            discovery_issuer: Mutex::new(None),
            expected_nonce: Mutex::new(None),
            expected_challenge: Mutex::new(None),
            expected_authorizations: Mutex::new(HashMap::new()),
            token_claim_overrides: Mutex::new(serde_json::Map::new()),
            omit_id_token: AtomicBool::new(false),
            token_signing_key_override: Mutex::new(None),
            token_algorithm: Mutex::new(Algorithm::RS256),
            discovery_redirect: AtomicBool::new(false),
            jwks_redirect: AtomicBool::new(false),
            token_response_override: Mutex::new(None),
            unsigned_token: AtomicBool::new(false),
            token_gate: Mutex::new(None),
        });
        let app = Router::new()
            .route("/.well-known/openid-configuration", get(discovery))
            .route("/authorize", get(authorize))
            .route("/jwks", get(jwks))
            .route("/token", post(token))
            .route("/error-uri-sentinel", get(error_uri_sentinel))
            .route(
                "/redirected",
                get(error_uri_sentinel).post(error_uri_sentinel),
            )
            .with_state(provider.clone());
        let task = tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .expect("serve loopback provider");
        });
        Self { provider, task }
    }

    async fn remember_authorization(&self, nonce: String, challenge: String) {
        self.remember_authorization_for_code("accepted-code", nonce, challenge)
            .await;
    }

    async fn remember_authorization_for_code(&self, code: &str, nonce: String, challenge: String) {
        self.provider
            .expected_authorizations
            .lock()
            .await
            .insert(code.to_string(), (nonce, challenge));
    }

    async fn rotate_signing_key(&self) {
        *self.provider.signing_key.lock().await = generate_signing_key();
    }

    async fn rotate_signing_key_with_same_kid(&self) {
        let kid = self.provider.signing_key.lock().await.kid.clone();
        *self.provider.signing_key.lock().await = generate_signing_key_with_kid(&kid);
    }

    fn fail_jwks(&self, failing: bool) {
        self.provider.jwks_failing.store(failing, Ordering::SeqCst);
    }

    async fn set_discovery_issuer(&self, issuer: String) {
        *self.provider.discovery_issuer.lock().await = Some(issuer);
    }

    async fn set_discovery_override(&self, document: serde_json::Value) {
        *self.provider.discovery_override.lock().await = Some(document);
    }

    async fn set_jwks_override(&self, jwks: serde_json::Value) {
        *self.provider.jwks_override.lock().await = Some(jwks);
    }

    fn fail_jwks_parsing(&self, malformed: bool) {
        self.provider
            .malformed_jwks
            .store(malformed, Ordering::SeqCst);
    }

    fn omit_token_auth_methods(&self, omitted: bool) {
        self.provider
            .omit_token_auth_methods
            .store(omitted, Ordering::SeqCst);
    }

    fn return_protocol_token_error(&self, enabled: bool) {
        self.provider
            .token_protocol_error
            .store(enabled, Ordering::SeqCst);
    }

    async fn return_protocol_token_error_with_description(&self, description: &str) {
        self.return_protocol_token_error(true);
        *self.provider.token_protocol_error_description.lock().await =
            Some(description.to_string());
    }

    fn return_unparseable_token_response(&self, enabled: bool) {
        self.provider
            .token_unparseable_response
            .store(enabled, Ordering::SeqCst);
    }

    async fn set_token_claim(&self, name: &str, value: serde_json::Value) {
        self.provider
            .token_claim_overrides
            .lock()
            .await
            .insert(name.to_string(), value);
    }

    async fn remove_token_claim(&self, name: &str) {
        self.provider
            .token_claim_overrides
            .lock()
            .await
            .insert(name.to_string(), serde_json::Value::Null);
    }

    fn omit_id_token(&self, omitted: bool) {
        self.provider.omit_id_token.store(omitted, Ordering::SeqCst);
    }

    async fn sign_tokens_with_unknown_key(&self) {
        *self.provider.token_signing_key_override.lock().await = Some(generate_signing_key());
    }

    async fn sign_tokens_with_invalid_same_kid_key(&self) {
        let kid = self.provider.signing_key.lock().await.kid.clone();
        *self.provider.token_signing_key_override.lock().await =
            Some(generate_signing_key_with_kid(&kid));
    }

    async fn set_token_algorithm(&self, algorithm: Algorithm) {
        *self.provider.token_algorithm.lock().await = algorithm;
    }

    fn redirect_discovery(&self, redirect: bool) {
        self.provider
            .discovery_redirect
            .store(redirect, Ordering::SeqCst);
    }
}

#[derive(Clone)]
struct CapturedWriter(Arc<StdMutex<Vec<u8>>>);

impl Write for CapturedWriter {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        self.0
            .lock()
            .expect("captured log lock")
            .extend_from_slice(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'writer> MakeWriter<'writer> for CapturedWriter {
    type Writer = Self;

    fn make_writer(&'writer self) -> Self::Writer {
        self.clone()
    }
}

struct TestMonotonicClock {
    start: Instant,
    elapsed: StdMutex<Duration>,
}

impl TestMonotonicClock {
    fn advance(&self, duration: Duration) {
        *self.elapsed.lock().expect("monotonic test lock") += duration;
    }
}

impl MonotonicClock for TestMonotonicClock {
    fn now(&self) -> Instant {
        self.start + *self.elapsed.lock().expect("monotonic test lock")
    }
}

struct TestClock {
    time: StdMutex<SystemTime>,
}

impl TestClock {
    fn advance(&self, duration: Duration) {
        *self.time.lock().expect("wall-clock test lock") += duration;
    }
}

impl Clock for TestClock {
    fn now(&self) -> time::OffsetDateTime {
        (*self.time.lock().expect("wall-clock test lock")).into()
    }
}

fn discovery_document(base: &str, methods: &serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "issuer": base,
        "authorization_endpoint": format!("{base}/authorize"),
        "token_endpoint": format!("{base}/token"),
        "jwks_uri": format!("{base}/jwks"),
        "response_types_supported": ["code"],
        "subject_types_supported": ["public"],
        "id_token_signing_alg_values_supported": ["RS256"],
        "token_endpoint_auth_methods_supported": methods,
    })
}

fn generate_signing_key() -> SigningKey {
    generate_signing_key_with_kid(&format!("local-test-key-{}", uuid::Uuid::new_v4()))
}

fn generate_signing_key_with_kid(key_id: &str) -> SigningKey {
    let private = RsaPrivateKey::new(&mut OsRng, 2_048).expect("generate test key");
    let public = private.to_public_key();
    let encode = base64::engine::general_purpose::URL_SAFE_NO_PAD;
    SigningKey {
        kid: key_id.to_owned(),
        pem: private
            .to_pkcs8_pem(LineEnding::LF)
            .expect("serialize test key")
            .to_string(),
        jwks: serde_json::json!({
            "keys": [{
                "kty": "RSA",
                "kid": key_id,
                "use": "sig",
                "alg": "RS256",
                "n": encode.encode(public.n().to_bytes_be()),
                "e": encode.encode(public.e().to_bytes_be()),
            }]
        }),
    }
}

async fn discovery(State(provider): State<Arc<Provider>>) -> Response {
    provider.discovery_requests.fetch_add(1, Ordering::SeqCst);
    if provider.discovery_redirect.load(Ordering::SeqCst) {
        return (StatusCode::FOUND, [(header::LOCATION, "/redirected")]).into_response();
    }
    if let Some(document) = provider.discovery_override.lock().await.clone() {
        return Json(document).into_response();
    }
    let methods = match provider.method {
        TokenAuthMethod::Basic => serde_json::json!(["client_secret_basic", "client_secret_post"]),
        TokenAuthMethod::Post => serde_json::json!(["client_secret_post"]),
    };
    let issuer = provider
        .discovery_issuer
        .lock()
        .await
        .clone()
        .unwrap_or_else(|| provider.base.clone());
    let mut document = discovery_document(&issuer, &methods);
    if provider.omit_token_auth_methods.load(Ordering::SeqCst) {
        document
            .as_object_mut()
            .expect("provider document is an object")
            .remove("token_endpoint_auth_methods_supported");
    }
    Json(document).into_response()
}

async fn authorize(
    State(provider): State<Arc<Provider>>,
    Query(query): Query<HashMap<String, String>>,
) -> Response {
    provider
        .authorization_requests
        .fetch_add(1, Ordering::SeqCst);
    if query.get("response_type") != Some(&"code".to_string())
        || !query.contains_key("client_id")
        || !query.contains_key("redirect_uri")
        || !query.contains_key("state")
        || !query.contains_key("nonce")
        || !query.contains_key("code_challenge")
        || query.get("code_challenge_method") != Some(&"S256".to_string())
    {
        return StatusCode::BAD_REQUEST.into_response();
    }
    StatusCode::OK.into_response()
}

async fn jwks(State(provider): State<Arc<Provider>>) -> Response {
    provider.jwks_requests.fetch_add(1, Ordering::SeqCst);
    if provider.jwks_redirect.load(Ordering::SeqCst) {
        return (StatusCode::FOUND, [(header::LOCATION, "/redirected")]).into_response();
    }
    if provider.jwks_failing.load(Ordering::SeqCst) {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    }
    if provider.malformed_jwks.load(Ordering::SeqCst) {
        return ([(header::CONTENT_TYPE, "application/json")], "not-json").into_response();
    }
    let jwks = match provider.jwks_override.lock().await.clone() {
        Some(jwks) => jwks,
        None => provider.signing_key.lock().await.jwks.clone(),
    };
    Json(jwks).into_response()
}

async fn error_uri_sentinel(State(provider): State<Arc<Provider>>) -> StatusCode {
    provider.error_uri_requests.fetch_add(1, Ordering::SeqCst);
    StatusCode::OK
}

async fn token_response_override(provider: &Provider) -> Option<Response> {
    if let Some((status, body)) = provider.token_response_override.lock().await.clone() {
        return Some(
            (
                status,
                [
                    (header::CONTENT_TYPE, "application/json"),
                    (header::LOCATION, "/redirected"),
                ],
                body,
            )
                .into_response(),
        );
    }
    if provider.token_protocol_error.load(Ordering::SeqCst) {
        let mut error = serde_json::json!({ "error": "invalid_grant" });
        if let Some(description) = provider
            .token_protocol_error_description
            .lock()
            .await
            .as_ref()
        {
            error["error_description"] = serde_json::json!(description);
        }
        return Some((StatusCode::BAD_REQUEST, Json(error)).into_response());
    }
    provider
        .token_unparseable_response
        .load(Ordering::SeqCst)
        .then(|| StatusCode::SERVICE_UNAVAILABLE.into_response())
}

async fn token(
    State(provider): State<Arc<Provider>>,
    headers: HeaderMap,
    Form(form): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    provider.token_requests.fetch_add(1, Ordering::SeqCst);
    lifecycle::wait_token_gate(&provider).await;
    if let Some(response) = token_response_override(&provider).await {
        return response;
    }
    let has_basic = headers.get(header::AUTHORIZATION).is_some_and(|value| {
        value
            .to_str()
            .is_ok_and(|value| value == "Basic Zmxvd2wtdGVzdC1jbGllbnQ6Zmxvd2wtdGVzdC1zZWNyZXQ=")
    });
    *provider.observed_basic.lock().await = Some(has_basic);
    let expected_credentials_in_body = provider.method == TokenAuthMethod::Post;
    if has_basic != (provider.method == TokenAuthMethod::Basic)
        || (form.get("client_id") == Some(&CLIENT_ID.to_string())) != expected_credentials_in_body
        || (form.get("client_secret") == Some(&CLIENT_SECRET.to_string()))
            != expected_credentials_in_body
    {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let verifier = form.get("code_verifier").cloned().unwrap_or_default();
    let code = form.get("code").cloned().unwrap_or_default();
    let expected = provider
        .expected_authorizations
        .lock()
        .await
        .get(&code)
        .cloned();
    let (nonce, expected_challenge) = match expected {
        Some((nonce, challenge)) => (nonce, Some(challenge)),
        None => (
            provider
                .expected_nonce
                .lock()
                .await
                .clone()
                .unwrap_or_default(),
            provider.expected_challenge.lock().await.clone(),
        ),
    };
    let challenge =
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(Sha256::digest(verifier));
    if expected_challenge.as_deref() != Some(challenge.as_str()) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "invalid_grant" })),
        )
            .into_response();
    }
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("current time")
        .as_secs();
    let mut claims = serde_json::json!({
        "iss": provider.base.clone(),
        "aud": CLIENT_ID,
        "exp": now + 3_600,
        "iat": now,
        "nonce": nonce,
        "sub": "local-provider-user",
    });
    let overrides = provider.token_claim_overrides.lock().await;
    let claims = claims.as_object_mut().expect("test claims are an object");
    for (name, value) in overrides.iter() {
        if value.is_null() {
            claims.remove(name);
        } else {
            claims.insert(name.clone(), value.clone());
        }
    }
    drop(overrides);
    let signing_key = match provider.token_signing_key_override.lock().await.clone() {
        Some(key) => key,
        None => provider.signing_key.lock().await.clone(),
    };
    let mut header = Header::new(*provider.token_algorithm.lock().await);
    header.kid = Some(signing_key.kid);
    let mut id_token = jsonwebtoken::encode(
        &header,
        &claims,
        &EncodingKey::from_rsa_pem(signing_key.pem.as_bytes()).expect("read test signing key"),
    )
    .expect("sign ID token");
    if provider.unsigned_token.load(Ordering::SeqCst) {
        id_token = without_signature(&id_token);
    }
    let mut response = serde_json::json!({
        "access_token": "local-access-token",
        "token_type": "Bearer",
        "expires_in": 3600,
        "id_token": id_token,
    });
    if provider.omit_id_token.load(Ordering::SeqCst) {
        response
            .as_object_mut()
            .expect("token response is an object")
            .remove("id_token");
    }
    Json(response).into_response()
}

fn without_signature(id_token: &str) -> String {
    let payload = id_token.split('.').nth(1).expect("JWT payload");
    let header = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(br#"{"alg":"none"}"#);
    format!("{header}.{payload}.")
}

async fn discover_provider(
    provider: &RunningProvider,
) -> Result<AuthState, super::AuthStartupError> {
    AuthState::with_dependencies(
        EnabledAuthConfig::loopback_test(&provider.provider.base, "http://127.0.0.1"),
        Arc::new(super::SystemClock),
        Arc::new(ReqwestAuthHttpClient::new().expect("HTTP client")),
    )
    .await
}

#[tokio::test]
async fn unsafe_discovered_destinations_are_never_contacted() {
    let provider = RunningProvider::start(TokenAuthMethod::Basic).await;
    for field in ["jwks_uri", "token_endpoint", "authorization_endpoint"] {
        for destination in [
            format!(
                "{}/error-uri-sentinel#private-sentinel",
                provider.provider.base
            ),
            provider
                .provider
                .base
                .replace("http://", "http://user:private-sentinel@")
                + "/error-uri-sentinel",
            provider.provider.base.replace("http://", "ftp://") + "/error-uri-sentinel",
            String::from("http://example.invalid/error-uri-sentinel"),
        ] {
            let mut document = discovery_document(
                &provider.provider.base,
                &serde_json::json!(["client_secret_basic"]),
            );
            document[field] = serde_json::json!(destination);
            provider.set_discovery_override(document).await;
            let previous_jwks = provider.provider.jwks_requests.load(Ordering::SeqCst);
            let error = discover_provider(&provider).await.unwrap_err();
            match field {
                "jwks_uri" => assert!(matches!(
                    error,
                    super::AuthStartupError::Oidc(super::OidcStartupError::Discovery)
                )),
                "token_endpoint" => assert!(matches!(
                    error,
                    super::AuthStartupError::Oidc(super::OidcStartupError::UnsafeEndpoint("token"))
                )),
                "authorization_endpoint" => assert!(matches!(
                    error,
                    super::AuthStartupError::Oidc(super::OidcStartupError::UnsafeEndpoint(
                        "authorization"
                    ))
                )),
                _ => unreachable!(),
            }
            assert_eq!(
                provider.provider.error_uri_requests.load(Ordering::SeqCst),
                0
            );
            assert_eq!(provider.provider.token_requests.load(Ordering::SeqCst), 0);
            if field == "jwks_uri" {
                assert_eq!(
                    provider.provider.jwks_requests.load(Ordering::SeqCst),
                    previous_jwks
                );
            }
        }
    }
}

#[tokio::test]
async fn discovery_jwks_extension_is_not_revalidated_as_metadata() {
    let provider = RunningProvider::start(TokenAuthMethod::Basic).await;
    let mut keys = provider.provider.signing_key.lock().await.jwks.clone();
    keys["jwks_uri"] = serde_json::json!("http://unsafe.invalid/#private-sentinel");
    provider.set_jwks_override(keys).await;
    assert!(discover_provider(&provider).await.is_ok());
    assert_eq!(provider.provider.jwks_requests.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn standalone_code_and_advertised_grants_are_required() {
    let provider = RunningProvider::start(TokenAuthMethod::Basic).await;
    for (responses, grants, accepted) in [
        (serde_json::json!(["code id_token"]), None, false),
        (
            serde_json::json!(["code"]),
            Some(serde_json::json!([])),
            false,
        ),
        (
            serde_json::json!(["code"]),
            Some(serde_json::json!(["refresh_token"])),
            false,
        ),
        (
            serde_json::json!(["code"]),
            Some(serde_json::json!(["authorization_code"])),
            true,
        ),
        (serde_json::json!(["code", "code id_token"]), None, true),
    ] {
        let mut document = discovery_document(
            &provider.provider.base,
            &serde_json::json!(["client_secret_basic"]),
        );
        document["response_types_supported"] = responses;
        if let Some(grants) = grants {
            document["grant_types_supported"] = grants;
        }
        provider.set_discovery_override(document).await;
        assert_eq!(discover_provider(&provider).await.is_ok(), accepted);
    }
    for (field, value) in [("use", "enc"), ("alg", "RS512")] {
        let mut keys = provider.provider.signing_key.lock().await.jwks.clone();
        keys["keys"][0][field] = serde_json::json!(value);
        provider.set_jwks_override(keys).await;
        let document = discovery_document(
            &provider.provider.base,
            &serde_json::json!(["client_secret_basic"]),
        );
        provider.set_discovery_override(document).await;
        assert!(matches!(
            discover_provider(&provider).await,
            Err(super::AuthStartupError::Oidc(
                super::OidcStartupError::UnusableSigningKeys
            ))
        ));
    }
}

#[tokio::test]
async fn provider_redirects_never_contact_targets_and_failed_refresh_retains_keys() {
    let provider = RunningProvider::start(TokenAuthMethod::Basic).await;
    provider.redirect_discovery(true);
    assert!(matches!(
        discover_provider(&provider).await,
        Err(super::AuthStartupError::Oidc(
            super::OidcStartupError::Discovery
        ))
    ));
    provider.redirect_discovery(false);
    provider
        .provider
        .jwks_redirect
        .store(true, Ordering::SeqCst);
    assert!(matches!(
        discover_provider(&provider).await,
        Err(super::AuthStartupError::Oidc(
            super::OidcStartupError::Discovery
        ))
    ));
    provider
        .provider
        .jwks_redirect
        .store(false, Ordering::SeqCst);
    let auth = discover_provider(&provider).await.unwrap();
    *provider.provider.token_response_override.lock().await =
        Some((StatusCode::FOUND, "private-sentinel".into()));
    assert_eq!(
        auth.exchange_and_verify(
            "accepted-code".into(),
            pending_login(&auth, &provider).await
        )
        .await,
        Err(CallbackError::ProviderUnavailable)
    );
    *provider.provider.token_response_override.lock().await = None;
    let original = provider.provider.signing_key.lock().await.clone();
    provider.rotate_signing_key().await;
    provider
        .provider
        .jwks_redirect
        .store(true, Ordering::SeqCst);
    assert_eq!(
        auth.exchange_and_verify(
            "accepted-code".into(),
            pending_login(&auth, &provider).await
        )
        .await,
        Err(CallbackError::ProviderUnavailable)
    );
    *provider.provider.signing_key.lock().await = original;
    assert_eq!(
        auth.exchange_and_verify(
            "accepted-code".into(),
            pending_login(&auth, &provider).await
        )
        .await,
        Ok(())
    );
    assert_eq!(
        provider.provider.error_uri_requests.load(Ordering::SeqCst),
        0
    );
}

#[tokio::test]
async fn token_response_categories_and_logs_are_safe() {
    use tracing::instrument::WithSubscriber;
    let provider = RunningProvider::start(TokenAuthMethod::Basic).await;
    let auth = discover_provider(&provider).await.unwrap();
    let captured = Arc::new(StdMutex::new(Vec::new()));
    let subscriber = tracing_subscriber::fmt()
        .with_writer(CapturedWriter(captured.clone()))
        .with_ansi(false)
        .without_time()
        .with_max_level(Level::DEBUG)
        .finish();
    async {
        for (status, body, expected) in [
            (StatusCode::SERVICE_UNAVAILABLE, r#"{"error":"invalid_grant","error_description":"private-sentinel"}"#, CallbackError::ProviderUnavailable),
            (StatusCode::BAD_REQUEST, r#"{"error":"invalid_grant","error_description":"private-sentinel"}"#, CallbackError::Authentication),
            (StatusCode::OK, "{private-sentinel", CallbackError::ProviderUnavailable),
            (StatusCode::CREATED, "private-sentinel", CallbackError::ProviderUnavailable),
            (StatusCode::OK, r#"{"id_token":"private-sentinel"}"#, CallbackError::ProviderUnavailable),
            (StatusCode::OK, r#"{"access_token":"private-sentinel","token_type":"Bearer","id_token":"private-sentinel"}"#, CallbackError::Authentication),
            (StatusCode::OK, r#"{"access_token":"private-sentinel","token_type":"Bearer","id_token":42}"#, CallbackError::Authentication),
            (StatusCode::FOUND, "private-sentinel", CallbackError::ProviderUnavailable),
        ] {
            *provider.provider.token_response_override.lock().await = Some((status, body.to_owned()));
            assert_eq!(auth.exchange_and_verify("private-sentinel".into(), pending_login(&auth, &provider).await).await, Err(expected));
        }
        provider.set_discovery_issuer("https://private-sentinel.invalid/?private-sentinel".into()).await;
        let error = discover_provider(&provider).await.err().unwrap();
        assert!(!format!("{error:?} {error}").contains("private-sentinel"));
    }.with_subscriber(subscriber).await;
    let logs = String::from_utf8(captured.lock().unwrap().clone()).unwrap();
    assert!(logs.contains("OIDC"));
    assert!(logs.contains("503"));
    assert!(logs.contains("Authentication"));
    assert!(logs.contains("Unavailable"));
    for sensitive in [
        "private-sentinel",
        CLIENT_SECRET,
        "http://",
        "https://",
        "id_token",
        "code=",
        "state=",
    ] {
        assert!(!logs.contains(sensitive), "logs leaked {sensitive}");
    }
    assert_eq!(
        provider.provider.error_uri_requests.load(Ordering::SeqCst),
        0
    );
    assert_eq!(provider.provider.jwks_requests.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn positive_old_and_future_iat_and_matching_azp_remain_valid() {
    let provider = RunningProvider::start(TokenAuthMethod::Basic).await;
    let auth = discover_provider(&provider).await.unwrap();
    provider
        .set_token_claim("azp", serde_json::json!(CLIENT_ID))
        .await;
    for iat in [1, 4_000_000_000_i64] {
        provider
            .set_token_claim("iat", serde_json::json!(iat))
            .await;
        assert_eq!(
            auth.exchange_and_verify(
                "accepted-code".into(),
                pending_login(&auth, &provider).await
            )
            .await,
            Ok(())
        );
    }
    assert_eq!(provider.provider.jwks_requests.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn basic_and_post_authentication_survive_same_kid_refresh() {
    for method in [TokenAuthMethod::Basic, TokenAuthMethod::Post] {
        let provider = RunningProvider::start(method).await;
        let auth = discover_provider(&provider).await.unwrap();
        provider.rotate_signing_key_with_same_kid().await;
        for _ in 0..2 {
            assert_eq!(
                auth.exchange_and_verify(
                    "accepted-code".into(),
                    pending_login(&auth, &provider).await
                )
                .await,
                Ok(())
            );
            assert_eq!(
                *provider.provider.observed_basic.lock().await,
                Some(method == TokenAuthMethod::Basic)
            );
        }
        assert_eq!(
            provider.provider.discovery_requests.load(Ordering::SeqCst),
            1
        );
        assert_eq!(provider.provider.jwks_requests.load(Ordering::SeqCst), 2);
    }
}

fn test_state(auth: Arc<AuthState>) -> (AppState, tempfile::TempDir) {
    let directory = tempfile::TempDir::new().expect("temporary uploads");
    let state = AppState {
        pool: SqlitePool::connect_lazy("sqlite::memory:").expect("in-memory pool"),
        image_store: ImageStore::new(directory.path().to_path_buf()),
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
        auth: Some(auth),
    };
    (state, directory)
}

fn cookie(response: &axum::response::Response) -> String {
    response
        .headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .next_back()
        .expect("session cookie")
        .to_str()
        .expect("cookie text")
        .split(';')
        .next()
        .expect("cookie pair")
        .to_string()
}

fn request(uri: &str, cookie: Option<&str>) -> Request<axum::body::Body> {
    request_with_method("GET", uri, cookie)
}

fn request_with_method(method: &str, uri: &str, cookie: Option<&str>) -> Request<axum::body::Body> {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(cookie) = cookie {
        builder = builder.header(
            header::COOKIE,
            HeaderValue::from_str(cookie).expect("cookie header"),
        );
    }
    builder.body(axum::body::Body::empty()).expect("request")
}

#[tokio::test]
async fn loopback_provider_rejects_an_exact_issuer_slash_mismatch() {
    let provider = RunningProvider::start(TokenAuthMethod::Basic).await;
    provider
        .set_discovery_issuer(format!("{}/", provider.provider.base))
        .await;
    let auth = AuthState::with_dependencies(
        EnabledAuthConfig::loopback_test(&provider.provider.base, "http://127.0.0.1"),
        Arc::new(super::SystemClock),
        Arc::new(ReqwestAuthHttpClient::new().expect("HTTP client")),
    )
    .await;
    assert!(matches!(
        auth,
        Err(super::AuthStartupError::Oidc(
            super::OidcStartupError::Discovery
        ))
    ));
    assert_eq!(
        provider.provider.discovery_requests.load(Ordering::SeqCst),
        1
    );
}

#[tokio::test]
async fn startup_rejects_unusable_initial_jwks_and_disallowed_signing_metadata() {
    let provider = RunningProvider::start(TokenAuthMethod::Basic).await;
    provider
        .set_jwks_override(serde_json::json!({ "keys": [] }))
        .await;
    let auth = AuthState::with_dependencies(
        EnabledAuthConfig::loopback_test(&provider.provider.base, "http://127.0.0.1"),
        Arc::new(super::SystemClock),
        Arc::new(ReqwestAuthHttpClient::new().expect("HTTP client")),
    )
    .await;
    assert!(matches!(
        auth,
        Err(super::AuthStartupError::Oidc(
            super::OidcStartupError::UnusableSigningKeys
        ))
    ));

    let provider = RunningProvider::start(TokenAuthMethod::Basic).await;
    provider
        .set_discovery_override(serde_json::json!({
            "issuer": provider.provider.base,
            "authorization_endpoint": format!("{}/authorize", provider.provider.base),
            "token_endpoint": format!("{}/token", provider.provider.base),
            "jwks_uri": format!("{}/jwks", provider.provider.base),
            "response_types_supported": ["code"],
            "subject_types_supported": ["public"],
            "id_token_signing_alg_values_supported": ["none", "RS256"],
            "token_endpoint_auth_methods_supported": ["client_secret_basic"],
        }))
        .await;
    let auth = AuthState::with_dependencies(
        EnabledAuthConfig::loopback_test(&provider.provider.base, "http://127.0.0.1"),
        Arc::new(super::SystemClock),
        Arc::new(ReqwestAuthHttpClient::new().expect("HTTP client")),
    )
    .await;
    assert!(matches!(
        auth,
        Err(super::AuthStartupError::Oidc(
            super::OidcStartupError::UnusableSigningKeys
        ))
    ));
}

#[tokio::test]
async fn startup_rejects_unavailable_and_malformed_initial_jwks() {
    let provider = RunningProvider::start(TokenAuthMethod::Basic).await;
    provider.fail_jwks(true);
    let auth = AuthState::with_dependencies(
        EnabledAuthConfig::loopback_test(&provider.provider.base, "http://127.0.0.1"),
        Arc::new(super::SystemClock),
        Arc::new(ReqwestAuthHttpClient::new().expect("HTTP client")),
    )
    .await;
    assert!(matches!(
        auth,
        Err(super::AuthStartupError::Oidc(
            super::OidcStartupError::Discovery
        ))
    ));

    let provider = RunningProvider::start(TokenAuthMethod::Basic).await;
    provider.fail_jwks_parsing(true);
    let auth = AuthState::with_dependencies(
        EnabledAuthConfig::loopback_test(&provider.provider.base, "http://127.0.0.1"),
        Arc::new(super::SystemClock),
        Arc::new(ReqwestAuthHttpClient::new().expect("HTTP client")),
    )
    .await;
    assert!(matches!(
        auth,
        Err(super::AuthStartupError::Oidc(
            super::OidcStartupError::Discovery
        ))
    ));
}

#[tokio::test]
async fn startup_rejects_unsafe_metadata_and_unsupported_token_authentication() {
    for (field, value) in [
        (
            "authorization_endpoint",
            serde_json::json!("http://insecure.example/authorize"),
        ),
        (
            "authorization_endpoint",
            serde_json::json!("https://user:password@issuer.example/authorize"),
        ),
        (
            "token_endpoint",
            serde_json::json!("http://insecure.example/token"),
        ),
        (
            "jwks_uri",
            serde_json::json!("http://insecure.example/jwks"),
        ),
        ("response_types_supported", serde_json::json!(["token"])),
        (
            "id_token_signing_alg_values_supported",
            serde_json::json!([]),
        ),
    ] {
        let provider = RunningProvider::start(TokenAuthMethod::Basic).await;
        let methods = serde_json::json!(["client_secret_basic"]);
        let mut document = discovery_document(&provider.provider.base, &methods);
        document
            .as_object_mut()
            .expect("provider document is an object")
            .insert(field.to_string(), value);
        provider.set_discovery_override(document).await;
        let auth = AuthState::with_dependencies(
            EnabledAuthConfig::loopback_test(&provider.provider.base, "http://127.0.0.1"),
            Arc::new(super::SystemClock),
            Arc::new(ReqwestAuthHttpClient::new().expect("HTTP client")),
        )
        .await;
        assert!(auth.is_err(), "startup must reject unsafe {field}");
    }

    let provider = RunningProvider::start(TokenAuthMethod::Basic).await;
    provider
        .set_discovery_override(discovery_document(
            &provider.provider.base,
            &serde_json::json!(["private_key_jwt"]),
        ))
        .await;
    let auth = AuthState::with_dependencies(
        EnabledAuthConfig::loopback_test(&provider.provider.base, "http://127.0.0.1"),
        Arc::new(super::SystemClock),
        Arc::new(ReqwestAuthHttpClient::new().expect("HTTP client")),
    )
    .await;
    assert!(matches!(
        auth,
        Err(super::AuthStartupError::Oidc(
            super::OidcStartupError::UnsupportedClientAuthentication
        ))
    ));
}

#[tokio::test]
async fn malformed_discovery_fails_before_authentication_state_is_available() {
    let provider = RunningProvider::start(TokenAuthMethod::Basic).await;
    provider
        .set_discovery_override(serde_json::json!({ "unsafe": "incomplete" }))
        .await;
    let auth = AuthState::with_dependencies(
        EnabledAuthConfig::loopback_test(&provider.provider.base, "http://127.0.0.1"),
        Arc::new(super::SystemClock),
        Arc::new(ReqwestAuthHttpClient::new().expect("HTTP client")),
    )
    .await;
    assert!(matches!(
        auth,
        Err(super::AuthStartupError::Oidc(
            super::OidcStartupError::Discovery
        ))
    ));
}

#[tokio::test]
async fn omitted_token_auth_metadata_defaults_to_basic() {
    let provider = RunningProvider::start(TokenAuthMethod::Basic).await;
    provider.omit_token_auth_methods(true);
    let auth = AuthState::with_dependencies(
        EnabledAuthConfig::loopback_test(&provider.provider.base, "http://127.0.0.1"),
        Arc::new(super::SystemClock),
        Arc::new(ReqwestAuthHttpClient::new().expect("HTTP client")),
    )
    .await
    .expect("omitted metadata selects the OIDC Basic default");
    assert_eq!(auth.runtime.auth_method, TokenAuthMethod::Basic);
}

#[tokio::test]
async fn parsed_token_protocol_errors_are_invalid_not_provider_unavailable() {
    let provider = RunningProvider::start(TokenAuthMethod::Basic).await;
    let auth = AuthState::with_dependencies(
        EnabledAuthConfig::loopback_test(&provider.provider.base, "http://127.0.0.1"),
        Arc::new(super::SystemClock),
        Arc::new(ReqwestAuthHttpClient::new().expect("HTTP client")),
    )
    .await
    .expect("local discovery succeeds");
    provider.return_protocol_token_error(true);
    assert_eq!(
        auth.exchange_and_verify(
            "rejected-code".to_string(),
            pending_login(&auth, &provider).await
        )
        .await,
        Err(CallbackError::Authentication)
    );
    assert_eq!(provider.provider.token_requests.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn wrong_pkce_verifier_is_rejected_without_authentication() {
    let provider = RunningProvider::start(TokenAuthMethod::Basic).await;
    let auth = AuthState::with_dependencies(
        EnabledAuthConfig::loopback_test(&provider.provider.base, "http://127.0.0.1"),
        Arc::new(super::SystemClock),
        Arc::new(ReqwestAuthHttpClient::new().expect("HTTP client")),
    )
    .await
    .expect("local discovery succeeds");
    let pending = pending_login(&auth, &provider).await;
    provider
        .remember_authorization(
            pending.nonce.secret().clone(),
            "wrong-challenge".to_string(),
        )
        .await;
    assert_eq!(
        auth.exchange_and_verify("accepted-code".to_string(), pending)
            .await,
        Err(CallbackError::Authentication)
    );
    assert_eq!(provider.provider.jwks_requests.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn unparseable_token_endpoint_responses_remain_provider_unavailable() {
    let provider = RunningProvider::start(TokenAuthMethod::Basic).await;
    let auth = AuthState::with_dependencies(
        EnabledAuthConfig::loopback_test(&provider.provider.base, "http://127.0.0.1"),
        Arc::new(super::SystemClock),
        Arc::new(ReqwestAuthHttpClient::new().expect("HTTP client")),
    )
    .await
    .expect("local discovery succeeds");
    provider.return_unparseable_token_response(true);
    assert_eq!(
        auth.exchange_and_verify(
            "unavailable-code".to_string(),
            pending_login(&auth, &provider).await
        )
        .await,
        Err(CallbackError::ProviderUnavailable)
    );
}

#[tokio::test]
async fn unavailable_token_endpoint_redirects_callback_to_generic_provider_unavailable() {
    let provider = RunningProvider::start(TokenAuthMethod::Basic).await;
    let auth = Arc::new(
        AuthState::with_dependencies(
            EnabledAuthConfig::loopback_test(&provider.provider.base, "http://127.0.0.1"),
            Arc::new(super::SystemClock),
            Arc::new(ReqwestAuthHttpClient::new().expect("HTTP client")),
        )
        .await
        .expect("local discovery succeeds"),
    );
    let (state, _directory) = test_state(auth);
    let app = server::router(state);
    let (browser_cookie, state) = begin_login(&app, &provider).await;
    provider.return_unparseable_token_response(true);

    let response = app
        .oneshot(request(
            &format!("/auth/callback?state={state}&code=accepted-code"),
            Some(&browser_cookie),
        ))
        .await
        .expect("callback response");
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        response.headers().get(header::LOCATION),
        Some(&HeaderValue::from_static(
            "/login?error=provider_unavailable&return_to=%2Fplants%3Ftab%3Dcare"
        ))
    );
    assert_eq!(provider.provider.token_requests.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn authorization_url_contains_openid_scope_exactly_once() {
    let provider = RunningProvider::start(TokenAuthMethod::Basic).await;
    let auth = AuthState::with_dependencies(
        EnabledAuthConfig::loopback_test(&provider.provider.base, "http://127.0.0.1"),
        Arc::new(super::SystemClock),
        Arc::new(ReqwestAuthHttpClient::new().expect("HTTP client")),
    )
    .await
    .expect("local discovery succeeds");

    let (authorization_url, _state, _nonce, _verifier) =
        auth.authorization_url().await.expect("authorization URL");
    let scope_values: Vec<_> = authorization_url
        .query_pairs()
        .filter_map(|(name, value)| (name == "scope").then(|| value.into_owned()))
        .collect();
    let scopes: Vec<_> = scope_values
        .iter()
        .flat_map(|value| value.split(' '))
        .filter(|scope| !scope.is_empty())
        .collect();

    assert!(scopes.contains(&"openid"));
    assert_eq!(scopes.iter().filter(|scope| **scope == "openid").count(), 1);
}

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn loopback_provider_covers_discovery_pkce_callback_rotation_and_route_policy() {
    let provider = RunningProvider::start(TokenAuthMethod::Basic).await;
    let config = EnabledAuthConfig::loopback_test(&provider.provider.base, "http://127.0.0.1");
    let auth = Arc::new(
        AuthState::with_dependencies(
            config,
            Arc::new(super::SystemClock),
            Arc::new(ReqwestAuthHttpClient::new().expect("HTTP client")),
        )
        .await
        .expect("local discovery succeeds"),
    );
    assert_eq!(
        provider.provider.discovery_requests.load(Ordering::SeqCst),
        1
    );
    assert_eq!(provider.provider.jwks_requests.load(Ordering::SeqCst), 1);
    provider.rotate_signing_key().await;

    let (state, _directory) = test_state(auth);
    let app = server::router(state);
    let login = app
        .clone()
        .oneshot(request(
            "/auth/login?return_to=%2Fplants%2F7%3Ftab%3Dcare",
            None,
        ))
        .await
        .expect("login response");
    assert_eq!(login.status(), StatusCode::SEE_OTHER);
    let preauth_cookie = cookie(&login);
    assert!(
        login
            .headers()
            .get(header::SET_COOKIE)
            .expect("cookie attributes")
            .to_str()
            .expect("cookie text")
            .contains("HttpOnly")
    );
    let cookie_attributes = login
        .headers()
        .get(header::SET_COOKIE)
        .expect("cookie attributes")
        .to_str()
        .expect("cookie text");
    assert!(cookie_attributes.contains("Secure"));
    assert!(cookie_attributes.contains("SameSite=Lax"));
    assert!(cookie_attributes.contains("Path=/"));
    assert!(cookie_attributes.contains("Max-Age="));
    let location = login
        .headers()
        .get(header::LOCATION)
        .expect("authorization location");
    let authorization_url =
        url::Url::parse(location.to_str().expect("location text")).expect("authorization URL");
    let query: HashMap<_, _> = url::form_urlencoded::parse(
        authorization_url
            .query()
            .expect("authorization query")
            .as_bytes(),
    )
    .into_owned()
    .collect();
    assert_eq!(query.get("response_type"), Some(&"code".to_string()));
    assert_eq!(
        query.get("redirect_uri"),
        Some(&"http://127.0.0.1/auth/callback".to_string())
    );
    assert_eq!(
        query.get("code_challenge_method"),
        Some(&"S256".to_string())
    );
    assert_eq!(
        reqwest::Client::new()
            .get(authorization_url)
            .send()
            .await
            .expect("authorization endpoint response")
            .status(),
        reqwest::StatusCode::OK
    );
    assert_eq!(
        provider
            .provider
            .authorization_requests
            .load(Ordering::SeqCst),
        1
    );
    provider
        .remember_authorization(
            query.get("nonce").expect("nonce").clone(),
            query.get("code_challenge").expect("challenge").clone(),
        )
        .await;

    let state = query.get("state").expect("state");
    let callback = app
        .clone()
        .oneshot(request(
            &format!("/auth/callback?state={state}&code=accepted-code"),
            Some(&preauth_cookie),
        ))
        .await
        .expect("callback response");
    assert_eq!(callback.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        callback.headers().get(header::LOCATION).unwrap(),
        "/plants/7?tab=care"
    );
    let authenticated_cookie = cookie(&callback);
    assert_ne!(preauth_cookie, authenticated_cookie);
    assert_eq!(provider.provider.token_requests.load(Ordering::SeqCst), 1);
    assert_eq!(*provider.provider.observed_basic.lock().await, Some(true));
    assert_eq!(provider.provider.jwks_requests.load(Ordering::SeqCst), 2);

    let preauth_api = app
        .clone()
        .oneshot(request("/api/info", Some(&preauth_cookie)))
        .await
        .expect("preauth API response");
    assert_eq!(preauth_api.status(), StatusCode::UNAUTHORIZED);
    let authenticated_api = app
        .clone()
        .oneshot(request("/api/info", Some(&authenticated_cookie)))
        .await
        .expect("authenticated API response");
    assert_eq!(authenticated_api.status(), StatusCode::OK);
    let logout = app
        .clone()
        .oneshot(request_with_method(
            "POST",
            "/auth/logout",
            Some(&authenticated_cookie),
        ))
        .await
        .expect("logout response");
    assert_eq!(logout.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        logout.headers().get(header::LOCATION).unwrap(),
        "/login?logged_out=1"
    );
    assert!(
        logout
            .headers()
            .get(header::SET_COOKIE)
            .expect("cookie removal")
            .to_str()
            .expect("cookie text")
            .contains("Max-Age=0")
    );
    assert_eq!(
        app.clone()
            .oneshot(request("/api/info", Some(&authenticated_cookie)))
            .await
            .expect("flushed session response")
            .status(),
        StatusCode::UNAUTHORIZED
    );

    let replay = app
        .clone()
        .oneshot(request(
            &format!("/auth/callback?state={state}&code=accepted-code"),
            Some(&preauth_cookie),
        ))
        .await
        .expect("replay response");
    assert_eq!(replay.status(), StatusCode::SEE_OTHER);
    assert_eq!(provider.provider.token_requests.load(Ordering::SeqCst), 1);
    assert_eq!(
        app.clone()
            .oneshot(request("/auth/unknown", None))
            .await
            .expect("unknown auth response")
            .status(),
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        app.clone()
            .oneshot(request("/auth/logout", None))
            .await
            .expect("logout method response")
            .status(),
        StatusCode::METHOD_NOT_ALLOWED
    );
    assert_eq!(
        app.clone()
            .oneshot(request("/uploads/private.jpg", None))
            .await
            .expect("protected upload response")
            .status(),
        StatusCode::UNAUTHORIZED
    );
}

async fn begin_login(app: &Router, provider: &RunningProvider) -> (String, String) {
    begin_login_with_target(app, provider, "/plants?tab=care").await
}

async fn begin_login_with_target(
    app: &Router,
    provider: &RunningProvider,
    target: &str,
) -> (String, String) {
    let query = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("return_to", target)
        .finish();
    let login = app
        .clone()
        .oneshot(request(&format!("/auth/login?{query}"), None))
        .await
        .expect("login response");
    assert_eq!(
        login.status(),
        StatusCode::SEE_OTHER,
        "{:?}",
        login.headers()
    );
    let location = login
        .headers()
        .get(header::LOCATION)
        .expect("authorization location")
        .to_str()
        .expect("location text");
    let authorization_url = url::Url::parse(location)
        .unwrap_or_else(|error| panic!("authorization URL {location:?}: {error}"));
    let query: HashMap<_, _> = url::form_urlencoded::parse(
        authorization_url
            .query()
            .expect("authorization query")
            .as_bytes(),
    )
    .into_owned()
    .collect();
    provider
        .remember_authorization(
            query.get("nonce").expect("nonce").clone(),
            query.get("code_challenge").expect("challenge").clone(),
        )
        .await;
    (cookie(&login), query.get("state").expect("state").clone())
}

#[tokio::test]
async fn login_and_callback_preserve_only_safe_serialized_return_targets() {
    let provider = RunningProvider::start(TokenAuthMethod::Basic).await;
    let auth = Arc::new(
        AuthState::with_dependencies(
            EnabledAuthConfig::loopback_test(&provider.provider.base, "http://127.0.0.1"),
            Arc::new(super::SystemClock),
            Arc::new(ReqwestAuthHttpClient::new().expect("HTTP client")),
        )
        .await
        .expect("local discovery succeeds"),
    );
    let (state, _directory) = test_state(auth);
    let app = server::router(state);
    let boundary = format!("/{}a", "%61".repeat(682));
    let oversized = format!("{boundary}a");
    assert_eq!(boundary.len(), 2048);
    assert_eq!(oversized.len(), 2049);
    for (target, accepted) in [
        ("/plants/42?label=a%20b&ratio=100%25#entry%20one", true),
        ("/apiary?ratio=100%25", true),
        (boundary.as_str(), true),
        (oversized.as_str(), false),
        ("/plants%2f42", false),
        ("/plants%3fquery", false),
        ("/plants%23entry", false),
        ("/api", false),
        ("/%61pi/plants", false),
        ("/health", false),
        ("/health/status", false),
        ("/uploads", false),
        ("/uploads/photo.jpg", false),
    ] {
        let expected = if accepted { target } else { "/" };
        let (preauth_cookie, state) = begin_login_with_target(&app, &provider, target).await;
        let callback_query = url::form_urlencoded::Serializer::new(String::new())
            .append_pair("state", &state)
            .append_pair("code", "accepted-code")
            .finish();
        let callback = app
            .clone()
            .oneshot(request(
                &format!("/auth/callback?{callback_query}"),
                Some(&preauth_cookie),
            ))
            .await
            .expect("callback response");
        assert_eq!(callback.status(), StatusCode::SEE_OTHER, "{target}");
        assert_eq!(
            callback.headers()[header::LOCATION].to_str().unwrap(),
            expected,
            "{target}"
        );
    }
}

#[tokio::test]
async fn private_cookies_are_opaque_and_successful_login_discards_oidc_values() {
    let provider = RunningProvider::start(TokenAuthMethod::Basic).await;
    provider
        .set_token_claim("sub", serde_json::json!("seeded-identity-claim"))
        .await;
    let auth = Arc::new(
        AuthState::with_dependencies(
            EnabledAuthConfig::loopback_test(&provider.provider.base, "http://127.0.0.1"),
            Arc::new(super::SystemClock),
            Arc::new(ReqwestAuthHttpClient::new().expect("HTTP client")),
        )
        .await
        .expect("local discovery succeeds"),
    );
    let (state, _directory) = test_state(auth);
    let app = server::router(state);
    let login = app
        .clone()
        .oneshot(request("/auth/login", None))
        .await
        .expect("login response");
    let preauth_cookie = cookie(&login);
    let authorization_url = url::Url::parse(
        login
            .headers()
            .get(header::LOCATION)
            .expect("authorization location")
            .to_str()
            .expect("location text"),
    )
    .expect("authorization URL");
    let query: HashMap<_, _> = url::form_urlencoded::parse(
        authorization_url
            .query()
            .expect("authorization query")
            .as_bytes(),
    )
    .into_owned()
    .collect();
    let state = query.get("state").expect("state").clone();
    let nonce = query.get("nonce").expect("nonce").clone();
    let challenge = query.get("code_challenge").expect("challenge").clone();
    for secret in [
        state.as_str(),
        nonce.as_str(),
        challenge.as_str(),
        "accepted-code",
        "local-access-token",
        CLIENT_SECRET,
        "seeded-identity-claim",
    ] {
        assert!(
            !preauth_cookie.contains(secret),
            "pre-auth cookie leaked {secret}"
        );
    }
    assert!(!preauth_cookie.contains("flowl.auth."));
    provider.remember_authorization(nonce, challenge).await;

    let callback = app
        .oneshot(request(
            &format!("/auth/callback?state={state}&code=accepted-code"),
            Some(&preauth_cookie),
        ))
        .await
        .expect("callback response");
    let authenticated_cookie = cookie(&callback);
    for secret in [
        state.as_str(),
        "accepted-code",
        "local-access-token",
        CLIENT_SECRET,
        "seeded-identity-claim",
    ] {
        assert!(
            !authenticated_cookie.contains(secret),
            "authenticated cookie leaked {secret}"
        );
    }
    assert!(!authenticated_cookie.contains("flowl.auth."));
}

async fn pending_login(auth: &AuthState, provider: &RunningProvider) -> PendingLogin {
    pending_login_for_code(auth, provider, "accepted-code").await
}

async fn pending_login_for_code(
    auth: &AuthState,
    provider: &RunningProvider,
    code: &str,
) -> PendingLogin {
    let (authorization_url, _state, nonce, verifier) =
        auth.authorization_url().await.expect("authorization URL");
    let query: HashMap<_, _> = url::form_urlencoded::parse(
        authorization_url
            .query()
            .expect("authorization query")
            .as_bytes(),
    )
    .into_owned()
    .collect();
    provider
        .remember_authorization_for_code(
            code,
            nonce.secret().clone(),
            query.get("code_challenge").expect("challenge").clone(),
        )
        .await;
    PendingLogin {
        nonce,
        pkce_verifier: verifier,
        return_to: SafeReturnTo::fallback(),
        expires_at: (SystemTime::now() + super::LOGIN_TRANSACTION_TTL).into(),
    }
}

#[tokio::test]
async fn loopback_provider_uses_client_secret_post_when_basic_is_not_advertised() {
    let provider = RunningProvider::start(TokenAuthMethod::Post).await;
    let config = EnabledAuthConfig::loopback_test(&provider.provider.base, "http://127.0.0.1");
    let auth = Arc::new(
        AuthState::with_dependencies(
            config,
            Arc::new(super::SystemClock),
            Arc::new(ReqwestAuthHttpClient::new().expect("HTTP client")),
        )
        .await
        .expect("local discovery succeeds"),
    );
    assert_eq!(auth.runtime.auth_method, TokenAuthMethod::Post);
    let (state, _directory) = test_state(auth);
    let app = server::router(state);
    let login = app
        .clone()
        .oneshot(request("/auth/login", None))
        .await
        .expect("login response");
    let preauth_cookie = cookie(&login);
    let authorization_url = url::Url::parse(
        login
            .headers()
            .get(header::LOCATION)
            .expect("authorization location")
            .to_str()
            .expect("location text"),
    )
    .expect("authorization URL");
    let query: HashMap<_, _> = url::form_urlencoded::parse(
        authorization_url
            .query()
            .expect("authorization query")
            .as_bytes(),
    )
    .into_owned()
    .collect();
    provider
        .remember_authorization(
            query.get("nonce").expect("nonce").clone(),
            query.get("code_challenge").expect("challenge").clone(),
        )
        .await;
    let callback = app
        .oneshot(request(
            &format!(
                "/auth/callback?state={}&code=accepted-code",
                query.get("state").expect("state")
            ),
            Some(&preauth_cookie),
        ))
        .await
        .expect("callback response");
    assert_eq!(callback.status(), StatusCode::SEE_OTHER);
    assert_eq!(*provider.provider.observed_basic.lock().await, Some(false));
    assert_eq!(provider.provider.token_requests.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn failed_rotated_jwks_is_cooled_down_then_recovers_with_an_injected_monotonic_clock() {
    let provider = RunningProvider::start(TokenAuthMethod::Basic).await;
    let monotonic = Arc::new(TestMonotonicClock {
        start: Instant::now(),
        elapsed: StdMutex::new(Duration::ZERO),
    });
    let auth = AuthState::with_all_dependencies(
        EnabledAuthConfig::loopback_test(&provider.provider.base, "http://127.0.0.1"),
        Arc::new(super::SystemClock),
        monotonic.clone(),
        Arc::new(ReqwestAuthHttpClient::new().expect("HTTP client")),
    )
    .await
    .expect("local discovery succeeds");
    assert_eq!(provider.provider.jwks_requests.load(Ordering::SeqCst), 1);

    provider.rotate_signing_key().await;
    provider.fail_jwks(true);
    assert_eq!(
        auth.exchange_and_verify(
            "accepted-code".to_string(),
            pending_login(&auth, &provider).await
        )
        .await,
        Err(CallbackError::ProviderUnavailable)
    );
    assert_eq!(provider.provider.jwks_requests.load(Ordering::SeqCst), 2);

    assert_eq!(
        auth.exchange_and_verify(
            "accepted-code".to_string(),
            pending_login(&auth, &provider).await
        )
        .await,
        Err(CallbackError::ProviderUnavailable)
    );
    assert_eq!(provider.provider.jwks_requests.load(Ordering::SeqCst), 2);

    provider.fail_jwks(false);
    monotonic.advance(super::JWKS_REFRESH_COOLDOWN);
    assert_eq!(
        auth.exchange_and_verify(
            "accepted-code".to_string(),
            pending_login(&auth, &provider).await
        )
        .await,
        Ok(())
    );
    assert_eq!(provider.provider.jwks_requests.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn pending_registry_accepts_1024_rejects_full_and_prunes_at_the_expiry_boundary() {
    let provider = RunningProvider::start(TokenAuthMethod::Basic).await;
    let clock = Arc::new(TestClock {
        time: StdMutex::new(SystemTime::now()),
    });
    let auth = Arc::new(
        AuthState::with_dependencies(
            EnabledAuthConfig::loopback_test(&provider.provider.base, "http://127.0.0.1"),
            clock.clone(),
            Arc::new(ReqwestAuthHttpClient::new().expect("HTTP client")),
        )
        .await
        .expect("local discovery succeeds"),
    );
    for index in 0..super::MAX_PENDING_LOGIN_TRANSACTIONS - 1 {
        auth.transactions.entries.lock().await.insert(
            format!("active-{index}"),
            pending_login(&auth, &provider).await,
        );
    }
    assert_eq!(
        auth.transactions.entries.lock().await.len(),
        super::MAX_PENDING_LOGIN_TRANSACTIONS - 1
    );
    let (state, _directory) = test_state(auth.clone());
    let app = server::router(state);
    let (browser_cookie, browser_state) = begin_login(&app, &provider).await;
    assert_eq!(
        auth.transactions.entries.lock().await.len(),
        super::MAX_PENDING_LOGIN_TRANSACTIONS
    );

    let full = app
        .clone()
        .oneshot(request("/auth/login", Some(&browser_cookie)))
        .await
        .expect("full registry response");
    assert_eq!(full.status(), StatusCode::SEE_OTHER);
    assert!(
        full.headers()
            .get(header::LOCATION)
            .expect("full location")
            .as_bytes()
            .starts_with(b"/login?error=provider_unavailable")
    );
    let transactions = auth.transactions.entries.lock().await;
    assert_eq!(transactions.len(), super::MAX_PENDING_LOGIN_TRANSACTIONS);
    assert!(transactions.contains_key(&browser_state));
    assert!(transactions.contains_key("active-0"));
    drop(transactions);

    assert!(auth.consume_transaction(&browser_state).await.is_some());
    let (_, replacement_state) = begin_login(&app, &provider).await;
    assert_eq!(
        auth.transactions.entries.lock().await.len(),
        super::MAX_PENDING_LOGIN_TRANSACTIONS
    );
    assert!(
        auth.transactions
            .entries
            .lock()
            .await
            .contains_key(&replacement_state)
    );

    assert!(auth.consume_transaction(&replacement_state).await.is_some());
    let mut expired = pending_login(&auth, &provider).await;
    expired.expires_at = clock.now();
    auth.transactions
        .entries
        .lock()
        .await
        .insert("expired-at-boundary".to_string(), expired);
    let (_, after_expiry) = begin_login(&app, &provider).await;
    let transactions = auth.transactions.entries.lock().await;
    assert!(!transactions.contains_key("expired-at-boundary"));
    assert!(transactions.contains_key(&after_expiry));
}

// Only complete success/provider-error shapes consume a browser-bound attempt.
// Retrying the original callback distinguishes rejection from consumption.
#[tokio::test]
async fn shared_callback_vectors_validate_bound_attempts() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../../tests/fixtures/auth/callbacks.json"))
            .expect("callback fixture should parse");
    let provider = RunningProvider::start(TokenAuthMethod::Basic).await;
    let auth = Arc::new(
        AuthState::with_dependencies(
            EnabledAuthConfig::loopback_test(&provider.provider.base, "http://127.0.0.1"),
            Arc::new(super::SystemClock),
            Arc::new(ReqwestAuthHttpClient::new().expect("HTTP client")),
        )
        .await
        .expect("local discovery succeeds"),
    );
    let (state, _directory) = test_state(auth.clone());
    let app = server::router(state);
    let error_uri = url::form_urlencoded::byte_serialize(
        format!("{}/error-uri-sentinel", provider.provider.base).as_bytes(),
    )
    .collect::<String>();
    for vector in fixture["vectors"].as_array().unwrap() {
        let id = vector["id"].as_str().unwrap();
        let target = vector["target"]["flowl"].as_str().unwrap();
        let consumed = target != "invalid";
        let (browser_cookie, state) = begin_login(&app, &provider).await;
        let query = vector["query"].as_str().map(|query| {
            query
                .replace("{state}", &state)
                .replace("{code}", "accepted-code")
                .replace("{error_uri}", &error_uri)
        });
        let uri = query.map_or_else(
            || "/auth/callback".to_string(),
            |query| format!("/auth/callback?{query}"),
        );
        let before = provider.provider.token_requests.load(Ordering::SeqCst);
        let response = app
            .clone()
            .oneshot(request(&uri, Some(&browser_cookie)))
            .await
            .expect("callback response");
        assert_eq!(response.status(), StatusCode::SEE_OTHER, "{id}");
        let expected = match target {
            "success" => "/plants?tab=care",
            "error" => "/login?error=authentication_failed&return_to=%2Fplants%3Ftab%3Dcare",
            "invalid" => "/login?error=authentication_failed&return_to=%2F",
            other => panic!("unknown target shape {other}"),
        };
        assert_eq!(
            response.headers().get(header::LOCATION).unwrap(),
            expected,
            "{id}"
        );
        assert_eq!(
            response.headers().get(header::CACHE_CONTROL).unwrap(),
            "no-store",
            "{id}"
        );
        let after = provider.provider.token_requests.load(Ordering::SeqCst);
        assert_eq!(after - before, usize::from(target == "success"), "{id}");
        let location = response
            .headers()
            .get(header::LOCATION)
            .unwrap()
            .to_str()
            .unwrap();
        assert!(!location.contains("private-sentinel"), "{id}");
        assert!(!location.contains(&provider.provider.base), "{id}");
        assert_eq!(
            auth.transactions.entries.lock().await.contains_key(&state),
            !consumed,
            "{id}"
        );
        assert_original_callback_retry(&app, &provider, &browser_cookie, &state, consumed, id)
            .await;
        assert_eq!(
            provider.provider.error_uri_requests.load(Ordering::SeqCst),
            0,
            "{id}"
        );
    }
}

async fn assert_original_callback_retry(
    app: &Router,
    provider: &RunningProvider,
    cookie: &str,
    state: &str,
    consumed: bool,
    id: &str,
) {
    let before = provider.provider.token_requests.load(Ordering::SeqCst);
    let retry = app
        .clone()
        .oneshot(request(
            &format!("/auth/callback?state={state}&code=accepted-code"),
            Some(cookie),
        ))
        .await
        .expect("retry original binding");
    assert_eq!(retry.status(), StatusCode::SEE_OTHER, "{id}");
    assert_eq!(
        retry.headers()[header::LOCATION],
        if consumed {
            "/login?error=authentication_failed&return_to=%2F"
        } else {
            "/plants?tab=care"
        },
        "{id} (retry original binding)"
    );
    assert_eq!(
        provider.provider.token_requests.load(Ordering::SeqCst) - before,
        usize::from(!consumed),
        "{id}"
    );
}

#[tokio::test]
async fn provider_error_uri_consumes_once_without_following_or_reflecting_it() {
    let provider = RunningProvider::start(TokenAuthMethod::Basic).await;
    let auth = Arc::new(
        AuthState::with_dependencies(
            EnabledAuthConfig::loopback_test(&provider.provider.base, "http://127.0.0.1"),
            Arc::new(super::SystemClock),
            Arc::new(ReqwestAuthHttpClient::new().expect("HTTP client")),
        )
        .await
        .expect("local discovery succeeds"),
    );
    let (state, _directory) = test_state(auth.clone());
    let app = server::router(state);
    let (browser_cookie, state) = begin_login(&app, &provider).await;
    let error_uri = url::form_urlencoded::byte_serialize(
        format!(
            "{}/error-uri-sentinel?private-sentinel",
            provider.provider.base
        )
        .as_bytes(),
    )
    .collect::<String>();
    let uri = format!("/auth/callback?state={state}&error=access_denied&error_uri={error_uri}");
    // A wrong browser must not consume even a provider-error attempt.
    let wrong_browser = app
        .clone()
        .oneshot(request(&uri, None))
        .await
        .expect("wrong browser");
    assert_eq!(wrong_browser.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        wrong_browser.headers().get(header::LOCATION).unwrap(),
        "/login?error=authentication_failed&return_to=%2F"
    );
    assert!(auth.transactions.entries.lock().await.contains_key(&state));
    let response = app
        .clone()
        .oneshot(request(&uri, Some(&browser_cookie)))
        .await
        .expect("provider error");
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        response.headers().get(header::LOCATION).unwrap(),
        "/login?error=authentication_failed&return_to=%2Fplants%3Ftab%3Dcare"
    );
    assert!(!auth.transactions.entries.lock().await.contains_key(&state));
    let retry = app
        .oneshot(request(
            &format!("/auth/callback?state={state}&code=accepted-code"),
            Some(&browser_cookie),
        ))
        .await
        .expect("replay");
    assert_eq!(
        retry.headers().get(header::LOCATION).unwrap(),
        "/login?error=authentication_failed&return_to=%2F"
    );
    assert_eq!(provider.provider.token_requests.load(Ordering::SeqCst), 0);
    assert_eq!(
        provider.provider.error_uri_requests.load(Ordering::SeqCst),
        0
    );
}

#[tokio::test]
async fn callback_failure_matrix_stays_generic_and_never_exchanges_an_untrusted_code() {
    for case in [
        "missing code",
        "mismatched state",
        "expired state",
        "provider error",
        "duplicate state",
        "mixed success and error",
    ] {
        let provider = RunningProvider::start(TokenAuthMethod::Basic).await;
        let clock = Arc::new(TestClock {
            time: StdMutex::new(SystemTime::now()),
        });
        let auth = Arc::new(
            AuthState::with_dependencies(
                EnabledAuthConfig::loopback_test(&provider.provider.base, "http://127.0.0.1"),
                clock.clone(),
                Arc::new(ReqwestAuthHttpClient::new().expect("HTTP client")),
            )
            .await
            .expect("local discovery succeeds"),
        );
        let (state, _directory) = test_state(auth);
        let app = server::router(state);
        let (browser_cookie, state) = begin_login(&app, &provider).await;
        let (uri, expected_location) = match case {
            "missing code" => (
                format!("/auth/callback?state={state}"),
                "/login?error=authentication_failed&return_to=%2F",
            ),
            "mismatched state" => (
                "/auth/callback?state=wrong-state&code=untrusted-code".to_string(),
                "/login?error=authentication_failed&return_to=%2F",
            ),
            "expired state" => {
                clock.advance(super::LOGIN_TRANSACTION_TTL);
                (
                    format!("/auth/callback?state={state}&code=untrusted-code"),
                    "/login?error=authentication_failed&return_to=%2F",
                )
            }
            "provider error" => (
                format!("/auth/callback?state={state}&error=access_denied"),
                "/login?error=authentication_failed&return_to=%2Fplants%3Ftab%3Dcare",
            ),
            "duplicate state" => (
                format!("/auth/callback?state={state}&state=other&code=untrusted-code"),
                "/login?error=authentication_failed&return_to=%2F",
            ),
            "mixed success and error" => (
                format!("/auth/callback?state={state}&code=untrusted-code&error=access_denied"),
                "/login?error=authentication_failed&return_to=%2F",
            ),
            _ => unreachable!("covered callback failure case"),
        };
        let response = app
            .oneshot(request(&uri, Some(&browser_cookie)))
            .await
            .expect("callback response");
        assert_eq!(response.status(), StatusCode::SEE_OTHER, "{case}");
        assert_eq!(
            response.headers().get(header::LOCATION),
            Some(&HeaderValue::from_static(expected_location)),
            "{case}"
        );
        assert_eq!(
            provider.provider.token_requests.load(Ordering::SeqCst),
            0,
            "{case}"
        );
    }
}

#[tokio::test]
async fn callback_binding_is_wrong_browser_safe_and_concurrent_consumption_is_single_use() {
    let provider = RunningProvider::start(TokenAuthMethod::Basic).await;
    let auth = Arc::new(
        AuthState::with_dependencies(
            EnabledAuthConfig::loopback_test(&provider.provider.base, "http://127.0.0.1"),
            Arc::new(super::SystemClock),
            Arc::new(ReqwestAuthHttpClient::new().expect("HTTP client")),
        )
        .await
        .expect("local discovery succeeds"),
    );
    let (state, _directory) = test_state(auth);
    let app = server::router(state);
    let (cookie, state) = begin_login(&app, &provider).await;
    let wrong_browser = app
        .clone()
        .oneshot(request(
            &format!("/auth/callback?state={state}&code=accepted-code"),
            None,
        ))
        .await
        .expect("wrong-browser callback");
    assert_eq!(wrong_browser.status(), StatusCode::SEE_OTHER);
    assert_eq!(provider.provider.token_requests.load(Ordering::SeqCst), 0);

    let first = app.clone().oneshot(request(
        &format!("/auth/callback?state={state}&code=accepted-code"),
        Some(&cookie),
    ));
    let second = app.clone().oneshot(request(
        &format!("/auth/callback?state={state}&code=accepted-code"),
        Some(&cookie),
    ));
    let (first, second) = tokio::join!(first, second);
    let successful = [
        first.expect("first callback"),
        second.expect("second callback"),
    ]
    .iter()
    .filter(|response| {
        response.headers().get(header::LOCATION)
            == Some(&HeaderValue::from_static("/plants?tab=care"))
    })
    .count();
    assert_eq!(successful, 1);
    assert_eq!(provider.provider.token_requests.load(Ordering::SeqCst), 1);
    let replay = app
        .oneshot(request(
            &format!("/auth/callback?state={state}&code=accepted-code"),
            Some(&cookie),
        ))
        .await
        .expect("replay callback");
    assert_eq!(replay.status(), StatusCode::SEE_OTHER);
    assert_eq!(provider.provider.token_requests.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn callback_consumption_cannot_erase_a_concurrent_replacement_login() {
    let provider = RunningProvider::start(TokenAuthMethod::Basic).await;
    let auth = Arc::new(
        AuthState::with_dependencies(
            EnabledAuthConfig::loopback_test(&provider.provider.base, "http://127.0.0.1"),
            Arc::new(super::SystemClock),
            Arc::new(ReqwestAuthHttpClient::new().expect("HTTP client")),
        )
        .await
        .expect("local discovery succeeds"),
    );
    let (state, _directory) = test_state(auth.clone());
    let app = server::router(state);
    let (original_cookie, old_state) = begin_login(&app, &provider).await;

    let reload = auth
        .sessions
        .store
        .faults
        .pause_next(super::StoreOperation::Load);

    let old_callback = tokio::spawn({
        let app = app.clone();
        let original_cookie = original_cookie.clone();
        async move {
            app.oneshot(request(
                &format!("/auth/callback?state={old_state}&error=access_denied"),
                Some(&original_cookie),
            ))
            .await
            .expect("old provider-error callback")
        }
    });
    reload.wait_entered().await;
    let replacement_load = auth
        .sessions
        .store
        .faults
        .pause_next(super::StoreOperation::Load);
    let replacement_login = tokio::spawn({
        let app = app.clone();
        let original_cookie = original_cookie.clone();
        async move {
            app.oneshot(request(
                "/auth/login?return_to=%2Freplacement",
                Some(&original_cookie),
            ))
            .await
            .expect("replacement login")
        }
    });
    replacement_load.wait_entered().await;
    replacement_load.release();
    assert!(!replacement_login.is_finished());
    reload.release();

    let old_callback = old_callback.await.expect("old callback task");
    assert_eq!(old_callback.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        old_callback.headers().get(header::LOCATION),
        Some(&HeaderValue::from_static(
            "/login?error=authentication_failed&return_to=%2Fplants%3Ftab%3Dcare"
        ))
    );

    let replacement_login = replacement_login.await.expect("replacement login task");
    assert_eq!(replacement_login.status(), StatusCode::SEE_OTHER);
    let replacement_cookie = cookie(&replacement_login);
    let authorization_url = url::Url::parse(
        replacement_login
            .headers()
            .get(header::LOCATION)
            .expect("replacement authorization location")
            .to_str()
            .expect("replacement location text"),
    )
    .expect("replacement authorization URL");
    let replacement_state = authorization_url
        .query_pairs()
        .find_map(|(key, value)| (key == "state").then(|| value.into_owned()))
        .expect("replacement state");

    let replacement_callback = app
        .oneshot(request(
            &format!("/auth/callback?state={replacement_state}&error=access_denied"),
            Some(&replacement_cookie),
        ))
        .await
        .expect("replacement provider-error callback");
    assert_eq!(replacement_callback.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        replacement_callback.headers().get(header::LOCATION),
        Some(&HeaderValue::from_static(
            "/login?error=authentication_failed&return_to=%2Freplacement"
        ))
    );
    assert_eq!(provider.provider.token_requests.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn token_claim_failures_are_invalid_without_jwks_refresh() {
    let cases = [
        ("missing ID token", "id_token"),
        ("wrong issuer spelling", "iss"),
        ("untrusted audience", "aud"),
        (
            "multiple audiences without an authorized party",
            "aud_multiple",
        ),
        ("invalid authorized party", "azp"),
        ("missing issued at", "iat"),
        ("zero issued at", "iat_zero"),
        ("negative issued at", "iat_negative"),
        ("malformed issued at", "iat_malformed"),
        ("extra audience with matching azp", "aud_azp"),
        ("actually unsigned ID token", "unsigned"),
        ("expired", "exp"),
        ("missing nonce", "nonce"),
        ("mismatched nonce", "nonce_mismatch"),
        ("bad access token hash", "at_hash"),
        ("disallowed signing algorithm", "alg"),
    ];
    for (description, claim) in cases {
        let provider = RunningProvider::start(TokenAuthMethod::Basic).await;
        let auth = AuthState::with_dependencies(
            EnabledAuthConfig::loopback_test(&provider.provider.base, "http://127.0.0.1"),
            Arc::new(super::SystemClock),
            Arc::new(ReqwestAuthHttpClient::new().expect("HTTP client")),
        )
        .await
        .expect("local discovery succeeds");
        override_invalid_claim(&provider, claim).await;
        assert_eq!(
            auth.exchange_and_verify(
                "accepted-code".to_string(),
                pending_login(&auth, &provider).await
            )
            .await,
            Err(CallbackError::Authentication),
            "{description}"
        );
        assert_eq!(
            provider.provider.jwks_requests.load(Ordering::SeqCst),
            1,
            "{description}"
        );
    }
}

async fn override_invalid_claim(provider: &RunningProvider, claim: &str) {
    let override_value = match claim {
        "id_token" => {
            provider.omit_id_token(true);
            return;
        }
        "iat" | "nonce" => {
            provider.remove_token_claim(claim).await;
            return;
        }
        "unsigned" => {
            provider
                .provider
                .unsigned_token
                .store(true, Ordering::SeqCst);
            return;
        }
        "alg" => {
            provider.set_token_algorithm(Algorithm::RS512).await;
            return;
        }
        "iss" => (
            "iss",
            serde_json::json!(format!("{}/", provider.provider.base)),
        ),
        "aud" => ("aud", serde_json::json!("other-client")),
        "aud_multiple" => ("aud", serde_json::json!([CLIENT_ID, "other-client"])),
        "azp" => ("azp", serde_json::json!("other-client")),
        "iat_zero" => ("iat", serde_json::json!(0)),
        "iat_negative" => ("iat", serde_json::json!(-1)),
        "iat_malformed" => ("iat", serde_json::json!("private-sentinel")),
        "aud_azp" => {
            provider
                .set_token_claim("azp", serde_json::json!(CLIENT_ID))
                .await;
            ("aud", serde_json::json!([CLIENT_ID, "other-client"]))
        }
        "exp" => ("exp", serde_json::json!(1)),
        "nonce_mismatch" => ("nonce", serde_json::json!("wrong-nonce")),
        "at_hash" => ("at_hash", serde_json::json!("mismatch")),
        _ => unreachable!("covered claim case"),
    };
    provider
        .set_token_claim(override_value.0, override_value.1)
        .await;
}

#[tokio::test]
async fn valid_applicable_access_token_hash_is_accepted() {
    let provider = RunningProvider::start(TokenAuthMethod::Basic).await;
    let auth = AuthState::with_dependencies(
        EnabledAuthConfig::loopback_test(&provider.provider.base, "http://127.0.0.1"),
        Arc::new(super::SystemClock),
        Arc::new(ReqwestAuthHttpClient::new().expect("HTTP client")),
    )
    .await
    .expect("local discovery succeeds");
    let digest = Sha256::digest("local-access-token");
    provider
        .set_token_claim(
            "at_hash",
            serde_json::json!(
                base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&digest[..16])
            ),
        )
        .await;
    assert_eq!(
        auth.exchange_and_verify(
            "accepted-code".to_string(),
            pending_login(&auth, &provider).await
        )
        .await,
        Ok(())
    );
}

#[tokio::test]
async fn crypto_error_from_a_rotated_same_kid_key_refreshes_jwks_once() {
    let provider = RunningProvider::start(TokenAuthMethod::Basic).await;
    let auth = AuthState::with_dependencies(
        EnabledAuthConfig::loopback_test(&provider.provider.base, "http://127.0.0.1"),
        Arc::new(super::SystemClock),
        Arc::new(ReqwestAuthHttpClient::new().expect("HTTP client")),
    )
    .await
    .expect("local discovery succeeds");

    provider.rotate_signing_key_with_same_kid().await;
    assert_eq!(
        auth.exchange_and_verify(
            "accepted-code".to_string(),
            pending_login(&auth, &provider).await
        )
        .await,
        Ok(())
    );
    assert_eq!(provider.provider.jwks_requests.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn invalid_signature_after_refresh_fails_callback_without_a_second_refresh() {
    let provider = RunningProvider::start(TokenAuthMethod::Basic).await;
    let auth = Arc::new(
        AuthState::with_dependencies(
            EnabledAuthConfig::loopback_test(&provider.provider.base, "http://127.0.0.1"),
            Arc::new(super::SystemClock),
            Arc::new(ReqwestAuthHttpClient::new().expect("HTTP client")),
        )
        .await
        .expect("local discovery succeeds"),
    );
    let (state, _directory) = test_state(auth);
    let app = server::router(state);
    let (preauth_cookie, state) = begin_login(&app, &provider).await;
    provider.sign_tokens_with_invalid_same_kid_key().await;

    let callback = app
        .clone()
        .oneshot(request(
            &format!("/auth/callback?state={state}&code=accepted-code"),
            Some(&preauth_cookie),
        ))
        .await
        .expect("callback response");
    assert_eq!(callback.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        callback.headers().get(header::LOCATION),
        Some(&HeaderValue::from_static(
            "/login?error=authentication_failed&return_to=%2Fplants%3Ftab%3Dcare"
        ))
    );
    assert_eq!(provider.provider.token_requests.load(Ordering::SeqCst), 1);
    assert_eq!(
        provider.provider.jwks_requests.load(Ordering::SeqCst),
        2,
        "initial discovery plus exactly one callback refresh"
    );
    assert_eq!(
        app.oneshot(request("/api/info", Some(&preauth_cookie)))
            .await
            .expect("pre-auth session response")
            .status(),
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn missing_key_refresh_is_deduplicated_for_concurrent_callbacks() {
    let provider = RunningProvider::start(TokenAuthMethod::Basic).await;
    let auth = Arc::new(
        AuthState::with_dependencies(
            EnabledAuthConfig::loopback_test(&provider.provider.base, "http://127.0.0.1"),
            Arc::new(super::SystemClock),
            Arc::new(ReqwestAuthHttpClient::new().expect("HTTP client")),
        )
        .await
        .expect("local discovery succeeds"),
    );
    let first_pending = pending_login_for_code(&auth, &provider, "accepted-code-one").await;
    let second_pending = pending_login_for_code(&auth, &provider, "accepted-code-two").await;
    provider.rotate_signing_key().await;
    let first = auth.exchange_and_verify("accepted-code-one".to_string(), first_pending);
    let second = auth.exchange_and_verify("accepted-code-two".to_string(), second_pending);
    let (first, second) = tokio::join!(first, second);
    assert_eq!(first, Ok(()));
    assert_eq!(second, Ok(()));
    assert_eq!(provider.provider.jwks_requests.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn access_logs_redact_login_token_callback_and_logout_values() {
    let provider = RunningProvider::start(TokenAuthMethod::Basic).await;
    let auth = Arc::new(
        AuthState::with_dependencies(
            EnabledAuthConfig::loopback_test(&provider.provider.base, "http://127.0.0.1"),
            Arc::new(super::SystemClock),
            Arc::new(ReqwestAuthHttpClient::new().expect("HTTP client")),
        )
        .await
        .expect("local discovery succeeds"),
    );
    let (state, _directory) = test_state(auth);
    let app = server::router(state);
    let captured = Arc::new(StdMutex::new(Vec::new()));
    let subscriber = tracing_subscriber::fmt()
        .with_writer(CapturedWriter(captured.clone()))
        .with_ansi(false)
        .without_time()
        .with_max_level(Level::DEBUG)
        .finish();
    let guard = tracing::subscriber::set_default(subscriber);
    let login = app
        .clone()
        .oneshot(request(
            "/auth/login?return_to=%2Fplants%3Flogin-query-secret%3D1",
            None,
        ))
        .await
        .expect("login response");
    let preauth_cookie = cookie(&login);
    let authorization_url = url::Url::parse(
        login
            .headers()
            .get(header::LOCATION)
            .expect("authorization location")
            .to_str()
            .expect("location text"),
    )
    .expect("authorization URL");
    let query: HashMap<_, _> = url::form_urlencoded::parse(
        authorization_url
            .query()
            .expect("authorization query")
            .as_bytes(),
    )
    .into_owned()
    .collect();
    let state = query.get("state").expect("state").clone();
    let nonce = query.get("nonce").expect("nonce").clone();
    let challenge = query.get("code_challenge").expect("challenge").clone();
    provider
        .remember_authorization(nonce.clone(), challenge.clone())
        .await;
    provider
        .return_protocol_token_error_with_description("provider-response-body-secret")
        .await;
    let callback = app
        .clone()
        .oneshot(request(
            &format!("/auth/callback?state={state}&code=token-code-secret"),
            Some(&preauth_cookie),
        ))
        .await
        .expect("callback response");
    assert_eq!(callback.status(), StatusCode::SEE_OTHER);
    let logout = app
        .oneshot(request_with_method(
            "POST",
            "/auth/logout",
            Some(&preauth_cookie),
        ))
        .await
        .expect("logout response");
    drop(guard);
    assert_eq!(logout.status(), StatusCode::SEE_OTHER);
    let logs = String::from_utf8(captured.lock().expect("captured log lock").clone())
        .expect("UTF-8 log output");
    assert!(logs.contains("access"));
    for secret in [
        "login-query-secret",
        state.as_str(),
        nonce.as_str(),
        challenge.as_str(),
        "token-code-secret",
        "provider-response-body-secret",
        CLIENT_SECRET,
        preauth_cookie.as_str(),
    ] {
        assert!(!logs.contains(secret), "log leaked {secret}");
    }
}

#[tokio::test]
async fn auth_config_is_minimal_data_and_all_auth_responses_are_no_store() {
    let provider = RunningProvider::start(TokenAuthMethod::Basic).await;
    let auth = Arc::new(
        AuthState::with_dependencies(
            EnabledAuthConfig::loopback_test(&provider.provider.base, "http://127.0.0.1"),
            Arc::new(super::SystemClock),
            Arc::new(ReqwestAuthHttpClient::new().expect("HTTP client")),
        )
        .await
        .expect("local discovery succeeds"),
    );
    let (state, _directory) = test_state(auth.clone());
    let app = server::router(state);
    let config = app
        .clone()
        .oneshot(request("/auth/config", None))
        .await
        .expect("auth config");
    let body = axum::body::to_bytes(config.into_body(), usize::MAX)
        .await
        .expect("config body");
    let payload = serde_json::from_slice::<serde_json::Value>(&body).expect("config JSON");
    assert_eq!(
        payload,
        serde_json::json!({ "enabled": true, "provider_name": "OpenID Connect" })
    );
    let object = payload.as_object().expect("config object");
    assert_eq!(object.len(), 2);
    let serialized = String::from_utf8(body.to_vec()).expect("config body text");
    for sensitive in [CLIENT_ID, CLIENT_SECRET, provider.provider.base.as_str()] {
        assert!(!serialized.contains(sensitive), "config leaked {sensitive}");
    }
    assert_auth_response_headers(&app, true, None).await;
    assert_auth_response_headers(&app, true, Some("flowl.sid=invalid")).await;
    let (mut disabled_state, _disabled_directory) = test_state(auth);
    disabled_state.auth = None;
    let disabled = server::router(disabled_state);
    assert_auth_response_headers(&disabled, false, None).await;
    let namespace_neighbor = disabled.oneshot(request("/authentic", None)).await.unwrap();
    assert_eq!(namespace_neighbor.status(), StatusCode::OK);
    assert!(
        !namespace_neighbor
            .headers()
            .contains_key(header::CACHE_CONTROL)
    );
}

async fn assert_auth_response_headers(app: &Router, enabled: bool, cookie: Option<&str>) {
    for (method, uri, enabled_status, disabled_status) in [
        ("GET", "/auth", 404, 200),
        ("GET", "/auth/", 404, 200),
        ("DELETE", "/auth/", 404, 200),
        ("GET", "/auth/login/", 404, 200),
        ("GET", "/auth/config/", 404, 200),
        ("GET", "/auth/config", 200, 200),
        ("POST", "/auth/config", 405, 405),
        ("GET", "/auth/login", 303, 200),
        ("GET", "/auth/login?return_to=a&return_to=b", 303, 200),
        (
            "GET",
            "/auth/callback?state=untrusted&code=untrusted",
            303,
            200,
        ),
        ("POST", "/auth/callback", 405, 200),
        ("POST", "/auth/logout", 303, 200),
        ("GET", "/auth/logout", 405, 200),
        ("GET", "/auth/not-found", 404, 200),
        ("DELETE", "/auth/not-found", 404, 200),
    ] {
        let response = app
            .clone()
            .oneshot(request_with_method(method, uri, cookie))
            .await
            .expect("auth route response");
        assert_eq!(
            response.status().as_u16(),
            if enabled {
                enabled_status
            } else {
                disabled_status
            },
            "{method} {uri}"
        );
        assert_eq!(
            response.headers()[header::CACHE_CONTROL],
            "no-store",
            "{method} {uri}"
        );
        if !enabled && disabled_status == 200 && uri != "/auth/config" {
            let expected = crate::embedded::static_handler("/".parse().unwrap()).await;
            assert_eq!(
                axum::body::to_bytes(response.into_body(), usize::MAX)
                    .await
                    .unwrap(),
                axum::body::to_bytes(expected.into_body(), usize::MAX)
                    .await
                    .unwrap(),
            );
        }
    }
}

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn enabled_and_disabled_route_policy_keeps_public_resources_and_uploads_correctly_scoped() {
    let provider = RunningProvider::start(TokenAuthMethod::Basic).await;
    let auth = Arc::new(
        AuthState::with_dependencies(
            EnabledAuthConfig::loopback_test(&provider.provider.base, "http://127.0.0.1"),
            Arc::new(super::SystemClock),
            Arc::new(ReqwestAuthHttpClient::new().expect("HTTP client")),
        )
        .await
        .expect("local discovery succeeds"),
    );
    let (state, directory) = test_state(auth.clone());
    std::fs::write(
        directory.path().join("public-test.txt"),
        b"authenticated upload",
    )
    .expect("write test upload");
    let app = server::router(state);
    let immutable_asset = crate::embedded::immutable_asset_path();
    for (method, path, expected_status) in [
        ("GET", "/health", StatusCode::OK),
        ("GET", "/login", StatusCode::OK),
        ("GET", "/auth/config", StatusCode::OK),
        ("GET", "/auth/login", StatusCode::SEE_OTHER),
        ("GET", "/auth/callback", StatusCode::SEE_OTHER),
        ("POST", "/auth/logout", StatusCode::SEE_OTHER),
        ("GET", "/service-worker.js", StatusCode::OK),
        ("GET", "/manifest.json", StatusCode::OK),
        ("GET", "/favicon.svg", StatusCode::OK),
        ("GET", "/icon-192.png", StatusCode::OK),
        ("GET", "/offline.html", StatusCode::OK),
        ("GET", immutable_asset.as_str(), StatusCode::OK),
    ] {
        assert_eq!(
            app.clone()
                .oneshot(request_with_method(method, path, None))
                .await
                .expect("public route response")
                .status(),
            expected_status,
            "{method} {path} must remain a public or protocol route"
        );
    }
    for (path, expected_location) in [
        ("/", "/login?return_to=%2F"),
        ("/index.html", "/login?return_to=%2Findex.html"),
        (
            "/apiary?ratio=100%25",
            "/login?return_to=%2Fapiary%3Fratio%3D100%2525",
        ),
        ("/plants%2f42", "/login?return_to=%2F"),
        ("/health/status", "/login?return_to=%2F"),
        (
            "/plants?ratio=100%25",
            "/login?return_to=%2Fplants%3Fratio%3D100%2525",
        ),
        (
            "/plants/42?tab=care",
            "/login?return_to=%2Fplants%2F42%3Ftab%3Dcare",
        ),
        (
            "/unknown/route?source=test",
            "/login?return_to=%2Funknown%2Froute%3Fsource%3Dtest",
        ),
    ] {
        let document = app
            .clone()
            .oneshot(request(path, None))
            .await
            .expect("protected document");
        assert_eq!(document.status(), StatusCode::SEE_OTHER, "{path}");
        assert_eq!(document.headers()[header::CACHE_CONTROL], "no-store");
        assert_eq!(
            document.headers().get(header::LOCATION),
            Some(&HeaderValue::from_str(expected_location).expect("expected redirect")),
            "{path}"
        );
    }
    for path in ["/api", "/api/", "/api/info", "/api/ai/status"] {
        let api = app
            .clone()
            .oneshot(request(path, None))
            .await
            .expect("unauthenticated API");
        assert_eq!(api.status(), StatusCode::UNAUTHORIZED, "{path}");
        assert_eq!(
            api.headers().get(header::CACHE_CONTROL),
            Some(&HeaderValue::from_static("no-store")),
            "{path}"
        );
        let api_body = axum::body::to_bytes(api.into_body(), usize::MAX)
            .await
            .expect("API JSON body");
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&api_body).expect("API JSON"),
            serde_json::json!({
                "code": "AUTHENTICATION_REQUIRED",
                "message": "Authentication is required"
            }),
            "{path}"
        );
    }
    for path in ["/uploads", "/uploads/", "/uploads/public-test.txt"] {
        let upload = app
            .clone()
            .oneshot(request(path, None))
            .await
            .expect("unauthenticated upload");
        assert_eq!(upload.status(), StatusCode::UNAUTHORIZED, "{path}");
        assert_eq!(
            upload.headers()[header::CACHE_CONTROL],
            "no-store",
            "{path}"
        );
        assert!(
            axum::body::to_bytes(upload.into_body(), usize::MAX)
                .await
                .unwrap()
                .is_empty()
        );
    }
    for path in [
        "/service-worker.js",
        "/favicon.svg",
        immutable_asset.as_str(),
    ] {
        let asset = app.clone().oneshot(request(path, None)).await.unwrap();
        assert_eq!(asset.status(), StatusCode::OK);
        assert!(!asset.headers().contains_key(header::CACHE_CONTROL));
    }
    let config = app
        .clone()
        .oneshot(request("/auth/config", None))
        .await
        .expect("auth config");
    assert_eq!(
        config.headers().get(header::CACHE_CONTROL),
        Some(&HeaderValue::from_static("no-store"))
    );
    let config_body = axum::body::to_bytes(config.into_body(), usize::MAX)
        .await
        .expect("config body");
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&config_body).expect("config JSON"),
        serde_json::json!({ "enabled": true, "provider_name": "OpenID Connect" })
    );
    for _ in 0..2 {
        let logout = app
            .clone()
            .oneshot(request_with_method("POST", "/auth/logout", None))
            .await
            .expect("idempotent logout");
        assert_eq!(logout.status(), StatusCode::SEE_OTHER);
        assert_eq!(
            logout.headers().get(header::LOCATION),
            Some(&HeaderValue::from_static("/login?logged_out=1"))
        );
        assert_eq!(
            logout.headers().get(header::CACHE_CONTROL),
            Some(&HeaderValue::from_static("no-store"))
        );
    }

    let (preauth_cookie, state) = begin_login(&app, &provider).await;
    let callback = app
        .clone()
        .oneshot(request(
            &format!("/auth/callback?state={state}&code=accepted-code"),
            Some(&preauth_cookie),
        ))
        .await
        .expect("successful callback");
    let authenticated_cookie = cookie(&callback);
    for path in ["/api/", "/uploads/"] {
        let response = app
            .clone()
            .oneshot(request(path, Some(&authenticated_cookie)))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND, "{path}");
        assert!(response.headers().get(header::LOCATION).is_none());
        assert!(
            axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap()
                .is_empty()
        );
    }
    let upload = app
        .clone()
        .oneshot(request(
            "/uploads/public-test.txt",
            Some(&authenticated_cookie),
        ))
        .await
        .expect("authenticated upload");
    assert_eq!(upload.status(), StatusCode::OK);
    assert_eq!(
        axum::body::to_bytes(upload.into_body(), usize::MAX)
            .await
            .expect("upload bytes"),
        "authenticated upload"
    );

    let (mut disabled_state, disabled_directory) = test_state(auth);
    disabled_state.auth = None;
    std::fs::write(
        disabled_directory.path().join("public-test.txt"),
        b"disabled upload",
    )
    .expect("write disabled upload");
    let disabled = server::router(disabled_state);
    let disabled_config = disabled
        .clone()
        .oneshot(request("/auth/config", None))
        .await
        .expect("disabled config");
    assert_eq!(disabled_config.status(), StatusCode::OK);
    assert_eq!(
        disabled_config.headers().get(header::CACHE_CONTROL),
        Some(&HeaderValue::from_static("no-store"))
    );
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(
            &axum::body::to_bytes(disabled_config.into_body(), usize::MAX)
                .await
                .expect("disabled config body")
        )
        .expect("disabled config JSON"),
        serde_json::json!({ "enabled": false })
    );
    for path in [
        "/health",
        "/login",
        "/service-worker.js",
        "/manifest.json",
        "/favicon.svg",
        "/icon-192.png",
        "/offline.html",
        immutable_asset.as_str(),
        "/",
        "/index.html",
        "/plants/42",
        "/unknown/route",
        "/api/info",
        "/api/ai/status",
    ] {
        assert_eq!(
            disabled
                .clone()
                .oneshot(request(path, None))
                .await
                .expect("disabled public response")
                .status(),
            StatusCode::OK,
            "{path} must retain auth-disabled public behavior"
        );
    }
    let disabled_upload = disabled
        .oneshot(request("/uploads/public-test.txt", None))
        .await
        .expect("disabled upload");
    assert_eq!(disabled_upload.status(), StatusCode::OK);
    assert_eq!(
        axum::body::to_bytes(disabled_upload.into_body(), usize::MAX)
            .await
            .expect("disabled upload bytes"),
        "disabled upload"
    );
}

#[tokio::test]
async fn authenticated_session_has_an_absolute_twelve_hour_deadline_and_does_not_survive_restart() {
    let provider = RunningProvider::start(TokenAuthMethod::Basic).await;
    let clock = Arc::new(TestClock {
        time: StdMutex::new(SystemTime::now()),
    });
    let auth = Arc::new(
        AuthState::with_dependencies(
            EnabledAuthConfig::loopback_test(&provider.provider.base, "http://127.0.0.1"),
            clock.clone(),
            Arc::new(ReqwestAuthHttpClient::new().expect("HTTP client")),
        )
        .await
        .expect("local discovery succeeds"),
    );
    let (state, _directory) = test_state(auth.clone());
    let app = server::router(state);
    let (preauth_cookie, state) = begin_login(&app, &provider).await;
    let callback = app
        .clone()
        .oneshot(request(
            &format!("/auth/callback?state={state}&code=accepted-code"),
            Some(&preauth_cookie),
        ))
        .await
        .expect("successful callback");
    let authenticated_cookie = cookie(&callback);
    assert_eq!(
        app.clone()
            .oneshot(request("/api/info", Some(&authenticated_cookie)))
            .await
            .expect("immediate API response")
            .status(),
        StatusCode::OK
    );
    assert_bypass_survives_five_minutes(&app, &clock, &authenticated_cookie).await;
    clock.advance(
        super::SESSION_TTL
            .checked_sub(super::LOGIN_TRANSACTION_TTL)
            .and_then(|remaining| remaining.checked_sub(Duration::from_secs(2)))
            .expect("session lifetime exceeds the login regression interval"),
    );
    assert_eq!(
        app.clone()
            .oneshot(request("/api/info", Some(&authenticated_cookie)))
            .await
            .expect("active session response")
            .status(),
        StatusCode::OK
    );
    clock.advance(Duration::from_secs(1));
    let expired = app
        .clone()
        .oneshot(request("/api/info", Some(&authenticated_cookie)))
        .await
        .expect("expired session response");
    assert_eq!(expired.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        expired.headers().get(header::CACHE_CONTROL),
        Some(&HeaderValue::from_static("no-store"))
    );
    assert!(
        expired
            .headers()
            .get_all(header::SET_COOKIE)
            .iter()
            .any(|value| value
                .as_bytes()
                .windows(b"Max-Age=0".len())
                .any(|window| window == b"Max-Age=0"))
    );
    for presented_cookie in [Some(authenticated_cookie.as_str()), None] {
        for _ in 0..2 {
            let logout = app
                .clone()
                .oneshot(request_with_method(
                    "POST",
                    "/auth/logout",
                    presented_cookie,
                ))
                .await
                .expect("idempotent logout");
            assert_eq!(logout.status(), StatusCode::SEE_OTHER);
            assert_eq!(
                logout.headers().get(header::LOCATION),
                Some(&HeaderValue::from_static("/login?logged_out=1"))
            );
            assert_eq!(
                logout.headers().get(header::CACHE_CONTROL),
                Some(&HeaderValue::from_static("no-store"))
            );
        }
    }

    let (restarted_state, _directory) = test_state(auth);
    let restarted = server::router(restarted_state)
        .oneshot(request("/api/info", Some(&authenticated_cookie)))
        .await
        .expect("restart response");
    assert_eq!(restarted.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn discovery_transport_redirect_and_signed_unknown_key_are_safely_rejected() {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("reserve port");
    let unavailable = format!("http://{}", listener.local_addr().expect("port"));
    drop(listener);
    assert!(matches!(
        AuthState::with_dependencies(
            EnabledAuthConfig::loopback_test(&unavailable, "http://127.0.0.1"),
            Arc::new(super::SystemClock),
            Arc::new(ReqwestAuthHttpClient::new().expect("HTTP client")),
        )
        .await,
        Err(super::AuthStartupError::Oidc(
            super::OidcStartupError::Discovery
        ))
    ));

    let provider = RunningProvider::start(TokenAuthMethod::Basic).await;
    provider.redirect_discovery(true);
    assert!(matches!(
        AuthState::with_dependencies(
            EnabledAuthConfig::loopback_test(&provider.provider.base, "http://127.0.0.1"),
            Arc::new(super::SystemClock),
            Arc::new(ReqwestAuthHttpClient::new().expect("HTTP client")),
        )
        .await,
        Err(super::AuthStartupError::Oidc(
            super::OidcStartupError::Discovery
        ))
    ));

    let provider = RunningProvider::start(TokenAuthMethod::Basic).await;
    let auth = AuthState::with_dependencies(
        EnabledAuthConfig::loopback_test(&provider.provider.base, "http://127.0.0.1"),
        Arc::new(super::SystemClock),
        Arc::new(ReqwestAuthHttpClient::new().expect("HTTP client")),
    )
    .await
    .expect("local discovery succeeds");
    provider.sign_tokens_with_unknown_key().await;
    assert_eq!(
        auth.exchange_and_verify(
            "accepted-code".to_string(),
            pending_login(&auth, &provider).await
        )
        .await,
        Err(CallbackError::Authentication)
    );
}

async fn assert_bypass_survives_five_minutes(app: &Router, clock: &TestClock, cookie: &str) {
    for path in ["/login?error=failed", "/auth/login?return_to=%2Fplants"] {
        let bypass = app
            .clone()
            .oneshot(request(path, Some(cookie)))
            .await
            .unwrap();
        assert_eq!(bypass.status(), StatusCode::SEE_OTHER);
        assert!(!bypass.headers().contains_key(header::SET_COOKIE));
    }
    clock.advance(super::LOGIN_TRANSACTION_TTL + Duration::from_secs(1));
    assert_eq!(
        app.clone()
            .oneshot(request("/api/info", Some(cookie)))
            .await
            .unwrap()
            .status(),
        StatusCode::OK,
        "authenticated login must not shorten the session to five minutes",
    );
}

#[tokio::test]
async fn authenticated_login_preserves_the_entire_session_record() {
    use crate::auth::session::{OWNER_KEY, Owner};
    use tower_sessions::{Session, SessionStore};

    let provider = RunningProvider::start(TokenAuthMethod::Basic).await;
    let auth = Arc::new(
        AuthState::with_dependencies(
            EnabledAuthConfig::loopback_test(&provider.provider.base, "http://127.0.0.1"),
            Arc::new(super::SystemClock),
            Arc::new(ReqwestAuthHttpClient::new().expect("HTTP client")),
        )
        .await
        .expect("local discovery"),
    );
    let store = Arc::new(auth.sessions.store.clone());
    let expiry = (SystemTime::now() + super::SESSION_TTL).into();
    let (key, lineage) = store.allocate(expiry);
    let owner = Owner::fresh(key);
    {
        let mut authority = lineage.authority.lock().await;
        authority.permit = Some(owner.clone());
        authority.creating = true;
    }
    let session = Session::new(None, store.clone(), None);
    session.insert(OWNER_KEY, owner).await.unwrap();
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    session
        .insert(
            crate::auth::session::AUTHENTICATED_SESSION_KEY,
            crate::auth::session::AuthenticatedSession {
                authenticated_at: i64::try_from(now).unwrap(),
                expires_at: i64::try_from(now + super::SESSION_TTL.as_secs()).unwrap(),
            },
        )
        .await
        .unwrap();
    session
        .insert("unrelated", serde_json::json!({"keep": [1, 2]}))
        .await
        .unwrap();
    session
        .insert(super::PREAUTH_STATE_KEY, "preserved-binding")
        .await
        .unwrap();
    session.set_expiry(Some(tower_sessions::Expiry::AtDateTime(
        (SystemTime::now() + super::SESSION_TTL).into(),
    )));
    session.save().await.unwrap();
    let id = session.id().unwrap();
    let before = store.load(&id).await.unwrap().unwrap();
    let session = Session::new(
        Some(id),
        store.clone(),
        Some(tower_sessions::Expiry::AtDateTime(before.expiry_date)),
    );
    let app = auth
        .protocol_routes()
        .layer(axum::Extension(crate::auth::session::OriginalSessionId(
            Some(id),
        )))
        .layer(axum::Extension(session.clone()));
    let response = app
        .oneshot(request("/auth/login?return_to=%2Fplants", None))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(session.id(), Some(id));
    assert!(!session.is_modified());
    assert_eq!(session.expiry_date(), before.expiry_date);
    session.save().await.unwrap();
    let after = store.load(&id).await.unwrap().unwrap();
    assert_eq!(after.data, before.data);
    assert_eq!(after.expiry_date, before.expiry_date);
}

async fn registry_snapshot(auth: &AuthState) -> serde_json::Value {
    let transactions = auth.transactions.entries.lock().await;
    serde_json::Value::Object(
        transactions
            .iter()
            .map(|(state, pending)| {
                (
                    state.clone(),
                    serde_json::json!({
                        "nonce": pending.nonce.secret(),
                        "verifier": pending.pkce_verifier.secret(),
                        "return_to": pending.return_to.as_str(),
                        "expires_at": pending.expires_at,
                    }),
                )
            })
            .collect(),
    )
}

fn provider_counts(provider: &RunningProvider) -> [usize; 3] {
    [
        provider.provider.discovery_requests.load(Ordering::SeqCst),
        provider.provider.jwks_requests.load(Ordering::SeqCst),
        provider.provider.token_requests.load(Ordering::SeqCst),
    ]
}

#[tokio::test]
async fn authenticated_login_bypasses_full_registry_without_cookie_or_provider_changes() {
    let provider = RunningProvider::start(TokenAuthMethod::Basic).await;
    let auth = Arc::new(
        AuthState::with_dependencies(
            EnabledAuthConfig::loopback_test(&provider.provider.base, "http://127.0.0.1"),
            Arc::new(super::SystemClock),
            Arc::new(ReqwestAuthHttpClient::new().expect("HTTP client")),
        )
        .await
        .expect("local discovery"),
    );
    let (state, _directory) = test_state(auth.clone());
    let app = server::router(state);
    let (preauth_cookie, state) = begin_login(&app, &provider).await;
    let callback = app
        .clone()
        .oneshot(request(
            &format!("/auth/callback?state={state}&code=accepted-code"),
            Some(&preauth_cookie),
        ))
        .await
        .expect("callback");
    let authenticated_cookie = cookie(&callback);
    for index in 0..super::MAX_PENDING_LOGIN_TRANSACTIONS {
        auth.transactions.entries.lock().await.insert(
            format!("active-{index}"),
            pending_login(&auth, &provider).await,
        );
    }
    let registry = registry_snapshot(&auth).await;
    let counts = provider_counts(&provider);
    for (path, target) in [
        ("/login", "/"),
        (
            "/login?error=authentication_failed&return_to=%2Fplants",
            "/",
        ),
        (
            "/login?logged_out=1&return_to=https%3A%2F%2Fevil.example",
            "/",
        ),
        ("/auth/login", "/"),
        (
            "/auth/login?error=failed&logged_out=1&return_to=%2Fplants%3Ftab%3Dcare",
            "/plants?tab=care",
        ),
        ("/auth/login?return_to=%2F%2Fevil.example", "/"),
        ("/auth/login?return_to=%2Fauth%2Fconfig", "/"),
        ("/auth/login?return_to=%2Fuploads", "/"),
    ] {
        let response = app
            .clone()
            .oneshot(request(path, Some(&authenticated_cookie)))
            .await
            .expect("bypass response");
        assert_eq!(response.status(), StatusCode::SEE_OTHER, "{path}");
        assert_eq!(response.headers()[header::LOCATION], target);
        assert!(
            !response.headers().contains_key(header::SET_COOKIE),
            "{path}"
        );
        assert!(
            axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap()
                .is_empty()
        );
        assert_eq!(registry_snapshot(&auth).await, registry);
        assert_eq!(provider_counts(&provider), counts);
    }
}
