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
use rand::rngs::OsRng;
use rsa::RsaPrivateKey;
use rsa::pkcs8::{EncodePrivateKey, LineEnding};
use rsa::traits::PublicKeyParts;
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use tokio::net::TcpListener;
use tokio::sync::{Barrier, Mutex, Notify};
use tower::ServiceExt;
use tracing::Level;
use tracing_subscriber::fmt::MakeWriter;

use super::{
    AuthHttpClient, AuthState, CallbackError, Clock, MonotonicClock, PendingLogin,
    ReqwestAuthHttpClient, TokenAuthMethod,
};
use crate::auth::return_to::SafeReturnTo;
use crate::config::EnabledAuthConfig;
use crate::images::ImageStore;
use crate::server;
use crate::state::AppState;

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
        });
        let app = Router::new()
            .route("/.well-known/openid-configuration", get(discovery))
            .route("/authorize", get(authorize))
            .route("/jwks", get(jwks))
            .route("/token", post(token))
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
        *self.provider.signing_key.lock().await = generate_signing_key_with_kid(kid);
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
            Some(generate_signing_key_with_kid(kid));
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
    fn now(&self) -> SystemTime {
        *self.time.lock().expect("wall-clock test lock")
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
    generate_signing_key_with_kid(format!("local-test-key-{}", uuid::Uuid::new_v4()))
}

fn generate_signing_key_with_kid(key_id: String) -> SigningKey {
    let private = RsaPrivateKey::new(&mut OsRng, 2_048).expect("generate test key");
    let public = private.to_public_key();
    let encode = base64::engine::general_purpose::URL_SAFE_NO_PAD;
    SigningKey {
        kid: key_id.clone(),
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
        TokenAuthMethod::RequestBody => serde_json::json!(["client_secret_post"]),
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

async fn token(
    State(provider): State<Arc<Provider>>,
    headers: HeaderMap,
    Form(form): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    provider.token_requests.fetch_add(1, Ordering::SeqCst);
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
        return (StatusCode::BAD_REQUEST, Json(error)).into_response();
    }
    if provider.token_unparseable_response.load(Ordering::SeqCst) {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    }
    let has_basic = headers.get(header::AUTHORIZATION).is_some_and(|value| {
        value
            .to_str()
            .is_ok_and(|value| value == "Basic Zmxvd2wtdGVzdC1jbGllbnQ6Zmxvd2wtdGVzdC1zZWNyZXQ=")
    });
    *provider.observed_basic.lock().await = Some(has_basic);
    let expected_credentials_in_body = provider.method == TokenAuthMethod::RequestBody;
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
    let id_token = jsonwebtoken::encode(
        &header,
        &claims,
        &EncodingKey::from_rsa_pem(signing_key.pem.as_bytes()).expect("read test signing key"),
    )
    .expect("sign ID token");
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
    assert!(matches!(auth, Err(super::AuthStartupError::IssuerMismatch)));
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
    assert!(matches!(auth, Err(super::AuthStartupError::UnsafeMetadata)));

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
    assert!(matches!(auth, Err(super::AuthStartupError::UnsafeMetadata)));
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
    assert!(matches!(auth, Err(super::AuthStartupError::HttpStatus)));

    let provider = RunningProvider::start(TokenAuthMethod::Basic).await;
    provider.fail_jwks_parsing(true);
    let auth = AuthState::with_dependencies(
        EnabledAuthConfig::loopback_test(&provider.provider.base, "http://127.0.0.1"),
        Arc::new(super::SystemClock),
        Arc::new(ReqwestAuthHttpClient::new().expect("HTTP client")),
    )
    .await;
    assert!(matches!(auth, Err(super::AuthStartupError::Parsing)));
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
        Err(super::AuthStartupError::UnsupportedTokenAuthentication)
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
    assert!(matches!(auth, Err(super::AuthStartupError::Parsing)));
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
    assert_eq!(auth.token_auth_method(), TokenAuthMethod::Basic);
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
        Err(CallbackError::Invalid)
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
        Err(CallbackError::Invalid)
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
        Err(CallbackError::Unavailable)
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
        ReqwestAuthHttpClient::new()
            .expect("HTTP client")
            .client()
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
    let login = app
        .clone()
        .oneshot(request(
            "/auth/login?return_to=%2Fplants%3Ftab%3Dcare",
            None,
        ))
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
        verifier,
        return_to: SafeReturnTo::fallback("/"),
        expires_at: SystemTime::now() + super::LOGIN_TRANSACTION_TTL,
    }
}

#[tokio::test]
async fn loopback_provider_uses_client_secret_post_when_basic_is_not_advertised() {
    let provider = RunningProvider::start(TokenAuthMethod::RequestBody).await;
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
    assert_eq!(auth.token_auth_method(), TokenAuthMethod::RequestBody);
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
        Err(CallbackError::Unavailable)
    );
    assert_eq!(provider.provider.jwks_requests.load(Ordering::SeqCst), 2);

    assert_eq!(
        auth.exchange_and_verify(
            "accepted-code".to_string(),
            pending_login(&auth, &provider).await
        )
        .await,
        Err(CallbackError::Unavailable)
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
        auth.transactions.lock().await.insert(
            format!("active-{index}"),
            pending_login(&auth, &provider).await,
        );
    }
    assert_eq!(
        auth.transactions.lock().await.len(),
        super::MAX_PENDING_LOGIN_TRANSACTIONS - 1
    );
    let (state, _directory) = test_state(auth.clone());
    let app = server::router(state);
    let (browser_cookie, browser_state) = begin_login(&app, &provider).await;
    assert_eq!(
        auth.transactions.lock().await.len(),
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
    let transactions = auth.transactions.lock().await;
    assert_eq!(transactions.len(), super::MAX_PENDING_LOGIN_TRANSACTIONS);
    assert!(transactions.contains_key(&browser_state));
    assert!(transactions.contains_key("active-0"));
    drop(transactions);

    assert!(auth.consume_transaction(&browser_state).await.is_some());
    let (_, replacement_state) = begin_login(&app, &provider).await;
    assert_eq!(
        auth.transactions.lock().await.len(),
        super::MAX_PENDING_LOGIN_TRANSACTIONS
    );
    assert!(
        auth.transactions
            .lock()
            .await
            .contains_key(&replacement_state)
    );

    assert!(auth.consume_transaction(&replacement_state).await.is_some());
    let mut expired = pending_login(&auth, &provider).await;
    expired.expires_at = clock.now();
    auth.transactions
        .lock()
        .await
        .insert("expired-at-boundary".to_string(), expired);
    let (_, after_expiry) = begin_login(&app, &provider).await;
    let transactions = auth.transactions.lock().await;
    assert!(!transactions.contains_key("expired-at-boundary"));
    assert!(transactions.contains_key(&after_expiry));
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

    let consumed = Arc::new(Barrier::new(2));
    let resume = Arc::new(Barrier::new(2));
    *auth.callback_consume_gate.lock().await = Some((Arc::clone(&consumed), Arc::clone(&resume)));

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
    consumed.wait().await;

    let replacement_reached = Arc::new(Notify::new());
    *auth.bind_transaction_reached.lock().await = Some(Arc::clone(&replacement_reached));
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
    replacement_reached.notified().await;
    resume.wait().await;

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

    *auth.callback_consume_gate.lock().await = None;
    *auth.bind_transaction_reached.lock().await = None;
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
        match claim {
            "id_token" => provider.omit_id_token(true),
            "iss" => {
                provider
                    .set_token_claim(
                        "iss",
                        serde_json::json!(format!("{}/", provider.provider.base)),
                    )
                    .await
            }
            "aud" => {
                provider
                    .set_token_claim("aud", serde_json::json!("other-client"))
                    .await
            }
            "aud_multiple" => {
                provider
                    .set_token_claim("aud", serde_json::json!([CLIENT_ID, "other-client"]))
                    .await
            }
            "azp" => {
                provider
                    .set_token_claim("azp", serde_json::json!("other-client"))
                    .await
            }
            "iat" | "nonce" => provider.remove_token_claim(claim).await,
            "exp" => provider.set_token_claim("exp", serde_json::json!(1)).await,
            "nonce_mismatch" => {
                provider
                    .set_token_claim("nonce", serde_json::json!("wrong-nonce"))
                    .await
            }
            "at_hash" => {
                provider
                    .set_token_claim("at_hash", serde_json::json!("mismatch"))
                    .await
            }
            "alg" => provider.set_token_algorithm(Algorithm::RS512).await,
            _ => unreachable!("covered claim case"),
        }
        assert!(
            auth.exchange_and_verify(
                "accepted-code".to_string(),
                pending_login(&auth, &provider).await
            )
            .await
            .is_err(),
            "{description}"
        );
        assert_eq!(
            provider.provider.jwks_requests.load(Ordering::SeqCst),
            1,
            "{description}"
        );
    }
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
    let (state, _directory) = test_state(auth);
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
    for (method, uri) in [
        ("GET", "/auth"),
        ("GET", "/auth/config"),
        ("GET", "/auth/login"),
        ("GET", "/auth/callback?state=untrusted&code=untrusted"),
        ("POST", "/auth/logout"),
        ("GET", "/auth/logout"),
        ("GET", "/auth/not-found"),
    ] {
        let response = app
            .clone()
            .oneshot(request_with_method(method, uri, None))
            .await
            .expect("auth route response");
        assert_eq!(
            response.headers().get(header::CACHE_CONTROL),
            Some(&HeaderValue::from_static("no-store")),
            "{method} {uri}"
        );
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
        assert_eq!(
            document.headers().get(header::LOCATION),
            Some(&HeaderValue::from_str(expected_location).expect("expected redirect")),
            "{path}"
        );
    }
    for path in ["/api/info", "/api/ai/status"] {
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
    assert_eq!(
        app.clone()
            .oneshot(request("/uploads/public-test.txt", None))
            .await
            .expect("unauthenticated upload")
            .status(),
        StatusCode::UNAUTHORIZED
    );
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
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(
            &axum::body::to_bytes(disabled_config.into_body(), usize::MAX)
                .await
                .expect("disabled config body")
        )
        .expect("disabled config JSON"),
        serde_json::json!({ "enabled": false, "provider_name": null })
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
    clock.advance(super::SESSION_TTL - Duration::from_secs(1));
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
        Err(super::AuthStartupError::Transport)
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
        Err(super::AuthStartupError::HttpStatus)
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
        Err(CallbackError::Invalid)
    );
}
