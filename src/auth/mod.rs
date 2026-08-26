//! Optional OIDC authentication primitives and routes.

pub mod return_to;
pub mod routes;

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

use tower_sessions::{Expiry, Session};

use openidconnect::core::{
    CoreClientAuthMethod, CoreJsonWebKeySet, CoreJwsSigningAlgorithm, CoreProviderMetadata,
    CoreRequestTokenError, CoreResponseType,
};
use openidconnect::{
    AuthType, ClientId, ClientSecret, HttpClientError, JsonWebKey, JsonWebKeyAlgorithm,
    JsonWebKeyUse, JwsSigningAlgorithm, OAuth2TokenResponse, RedirectUrl, TokenResponse,
};
use tokio::sync::{Mutex, RwLock};

use crate::auth::return_to::SafeReturnTo;
use crate::config::EnabledAuthConfig;

pub const LOGIN_TRANSACTION_TTL: Duration = Duration::from_mins(5);
pub const SESSION_TTL: Duration = Duration::from_hours(12);
pub const JWKS_REFRESH_COOLDOWN: Duration = Duration::from_secs(30);
pub const MAX_PENDING_LOGIN_TRANSACTIONS: usize = 1_024;

/// Wall-clock seam for session and login-transaction expiry checks.
pub trait Clock: Send + Sync {
    fn now(&self) -> SystemTime;
}

#[derive(Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> SystemTime {
        SystemTime::now()
    }
}

/// Process-monotonic seam used only for the JWKS refresh cooldown.
pub trait MonotonicClock: Send + Sync {
    fn now(&self) -> Instant;
}

#[derive(Default)]
pub struct SystemMonotonicClock;

impl MonotonicClock for SystemMonotonicClock {
    fn now(&self) -> Instant {
        Instant::now()
    }
}

/// HTTP-client seam used by startup discovery, token exchange, and JWKS refresh.
pub trait AuthHttpClient: Send + Sync {
    fn client(&self) -> &reqwest::Client;
}

pub struct ReqwestAuthHttpClient {
    client: reqwest::Client,
}

impl ReqwestAuthHttpClient {
    /// # Errors
    ///
    /// Returns an error if the redirect-disabled Rustls HTTP client cannot be built.
    pub fn new() -> Result<Self, reqwest::Error> {
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .use_rustls_tls()
            .build()?;
        Ok(Self { client })
    }
}

impl AuthHttpClient for ReqwestAuthHttpClient {
    fn client(&self) -> &reqwest::Client {
        &self.client
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenAuthMethod {
    Basic,
    RequestBody,
}

impl TokenAuthMethod {
    const fn oidc(self) -> AuthType {
        match self {
            Self::Basic => AuthType::BasicAuth,
            Self::RequestBody => AuthType::RequestBody,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthStartupError {
    Transport,
    HttpStatus,
    Parsing,
    IssuerMismatch,
    UnsafeMetadata,
    UnsupportedTokenAuthentication,
}

impl fmt::Display for AuthStartupError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let category = match self {
            Self::Transport => "transport failure",
            Self::HttpStatus => "HTTP status failure",
            Self::Parsing => "metadata parsing failure",
            Self::IssuerMismatch => "issuer mismatch",
            Self::UnsafeMetadata => "unsafe or incompatible metadata",
            Self::UnsupportedTokenAuthentication => "unsupported token endpoint authentication",
        };
        write!(formatter, "OIDC discovery failed: {category}")
    }
}

impl std::error::Error for AuthStartupError {}

pub struct PendingLogin {
    pub nonce: openidconnect::Nonce,
    pub verifier: openidconnect::PkceCodeVerifier,
    pub return_to: SafeReturnTo,
    pub expires_at: SystemTime,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TransactionBindError {
    Failed,
}

type CachedCoreClient = openidconnect::core::CoreClient<
    openidconnect::EndpointSet,
    openidconnect::EndpointNotSet,
    openidconnect::EndpointNotSet,
    openidconnect::EndpointNotSet,
    openidconnect::EndpointMaybeSet,
    openidconnect::EndpointMaybeSet,
>;

struct ClientCache {
    metadata: CoreProviderMetadata,
    client: CachedCoreClient,
    generation: u64,
    refresh_retry_at: Option<Instant>,
}

/// Shared enabled-authentication boundary.
///
/// Discovery is completed before this state is returned, so enabled startup never accepts a
/// request with unknown endpoints or key material. Provider responses are deliberately reduced
/// to typed, body-free diagnostics.
pub struct AuthState {
    config: EnabledAuthConfig,
    clock: Arc<dyn Clock>,
    http_client: Arc<dyn AuthHttpClient>,
    monotonic_clock: Arc<dyn MonotonicClock>,
    token_auth_method: TokenAuthMethod,
    clients: RwLock<ClientCache>,
    refresh_lock: Mutex<()>,
    /// Serializes replacement of the browser binding with its registry entry.
    login_lock: Mutex<()>,
    transactions: Mutex<HashMap<String, PendingLogin>>,
}

impl AuthState {
    /// # Errors
    ///
    /// Returns a safe startup diagnostic when discovery or provider metadata validation fails.
    pub async fn new(config: EnabledAuthConfig) -> Result<Self, AuthStartupError> {
        Self::with_all_dependencies(
            config,
            Arc::new(SystemClock),
            Arc::new(SystemMonotonicClock),
            Arc::new(ReqwestAuthHttpClient::new().map_err(|_| AuthStartupError::Transport)?),
        )
        .await
    }

    /// Builds initialized authentication state using injectable clock and HTTP seams.
    ///
    /// # Errors
    ///
    /// Returns a safe startup diagnostic when discovery or provider metadata validation fails.
    pub async fn with_dependencies(
        config: EnabledAuthConfig,
        clock: Arc<dyn Clock>,
        http_client: Arc<dyn AuthHttpClient>,
    ) -> Result<Self, AuthStartupError> {
        Self::with_all_dependencies(config, clock, Arc::new(SystemMonotonicClock), http_client)
            .await
    }

    /// Builds initialized authentication state with all testable time and HTTP seams.
    ///
    /// # Errors
    ///
    /// Returns a safe startup diagnostic when discovery or provider metadata validation fails.
    pub async fn with_all_dependencies(
        config: EnabledAuthConfig,
        clock: Arc<dyn Clock>,
        monotonic_clock: Arc<dyn MonotonicClock>,
        http_client: Arc<dyn AuthHttpClient>,
    ) -> Result<Self, AuthStartupError> {
        let metadata = CoreProviderMetadata::discover_async(
            config.issuer().issuer_url().clone(),
            http_client.client(),
        )
        .await
        .map_err(|error| discovery_error_category(&error))?;

        validate_metadata(&config, &metadata)?;
        let token_auth_method = select_token_auth_method(&metadata)?;

        let client = build_client(&config, token_auth_method, metadata.clone())?;
        Ok(Self {
            config,
            clock,
            http_client,
            monotonic_clock,
            token_auth_method,
            clients: RwLock::new(ClientCache {
                metadata,
                client,
                generation: 0,
                refresh_retry_at: None,
            }),
            refresh_lock: Mutex::new(()),
            login_lock: Mutex::new(()),
            transactions: Mutex::new(HashMap::new()),
        })
    }

    pub const fn config(&self) -> &EnabledAuthConfig {
        &self.config
    }

    pub const fn clock(&self) -> &Arc<dyn Clock> {
        &self.clock
    }

    pub const fn http_client(&self) -> &Arc<dyn AuthHttpClient> {
        &self.http_client
    }

    pub const fn token_auth_method(&self) -> TokenAuthMethod {
        self.token_auth_method
    }

    /// Creates an authorization URL using the currently cached, validated provider metadata.
    ///
    /// # Errors
    ///
    /// Returns a safe error if the cached client cannot construct an authorization request.
    pub async fn authorization_url(
        &self,
    ) -> Result<
        (
            url::Url,
            openidconnect::CsrfToken,
            openidconnect::Nonce,
            openidconnect::PkceCodeVerifier,
        ),
        AuthStartupError,
    > {
        let client = self.clients.read().await.client.clone();
        let (challenge, verifier) = openidconnect::PkceCodeChallenge::new_random_sha256();
        let (url, state, nonce) = client
            .authorize_url(
                openidconnect::core::CoreAuthenticationFlow::AuthorizationCode,
                openidconnect::CsrfToken::new_random,
                openidconnect::Nonce::new_random,
            )
            .add_scope(openidconnect::Scope::new("openid".to_string()))
            .set_pkce_challenge(challenge)
            .url();
        Ok((url, state, nonce, verifier))
    }

    /// Atomically replaces the current browser's pending transaction and its private-session
    /// binding. A capacity rejection intentionally happens before removing the prior transaction.
    ///
    /// Explicit saves make the binding durable before the provider redirect. If either insertion
    /// or save fails, the registry and session are rolled back while serialized against callbacks.
    pub(crate) async fn bind_transaction(
        &self,
        session: &Session,
        state: String,
        transaction: PendingLogin,
    ) -> Result<(), TransactionBindError> {
        let _login = self.login_lock.lock().await;
        let previous_state = session
            .get::<String>(crate::auth::routes::PREAUTH_STATE_KEY)
            .await
            .map_err(|_| TransactionBindError::Failed)?;
        let now = self.clock.now();
        let mut transactions = self.transactions.lock().await;
        transactions.retain(|_, pending| pending.expires_at > now);
        if transactions.len() >= MAX_PENDING_LOGIN_TRANSACTIONS {
            return Err(TransactionBindError::Failed);
        }
        let previous = previous_state
            .as_deref()
            .and_then(|previous_state| transactions.remove(previous_state));
        let expires_at = transaction.expires_at;
        transactions.insert(state.clone(), transaction);

        let saved = if session
            .insert(crate::auth::routes::PREAUTH_STATE_KEY, state.clone())
            .await
            .is_ok()
        {
            session.set_expiry(Some(Expiry::AtDateTime(expires_at.into())));
            session.save().await.is_ok()
        } else {
            false
        };
        if saved {
            return Ok(());
        }

        transactions.remove(&state);
        if let (Some(previous_state), Some(previous)) = (previous_state.as_deref(), previous) {
            transactions.insert(previous_state.to_string(), previous);
            let _ = session
                .insert(crate::auth::routes::PREAUTH_STATE_KEY, previous_state)
                .await;
            let _ = session.save().await;
        } else {
            let _ = session
                .remove::<String>(crate::auth::routes::PREAUTH_STATE_KEY)
                .await;
            let _ = session.flush().await;
        }
        Err(TransactionBindError::Failed)
    }

    /// Consumes a transaction exactly once before any provider I/O.
    pub async fn consume_transaction(&self, state: &str) -> Option<PendingLogin> {
        let now = self.clock.now();
        let mut transactions = self.transactions.lock().await;
        let pending = transactions.remove(state)?;
        (pending.expires_at > now).then_some(pending)
    }

    /// Exchanges a code and verifies the ID token using the cached metadata. The returned claims
    /// are never retained by this method or its caller.
    ///
    /// # Errors
    ///
    /// Returns a generic callback category without retaining or exposing provider values.
    pub async fn exchange_and_verify(
        &self,
        code: String,
        pending: PendingLogin,
    ) -> Result<(), CallbackError> {
        let cache = self.clients.read().await;
        let generation = cache.generation;
        let client = cache.client.clone();
        drop(cache);
        let token_response = client
            .exchange_code(openidconnect::AuthorizationCode::new(code))
            .map_err(|_| CallbackError::Invalid)?
            .set_pkce_verifier(pending.verifier)
            .request_async(self.http_client.client())
            .await
            .map_err(|error| callback_token_error(&error))?;
        let id_token = token_response.id_token().ok_or(CallbackError::Invalid)?;
        let verifier = client.id_token_verifier();
        match id_token.claims(&verifier, &pending.nonce) {
            Ok(claims) => self.validate_verified_token(
                claims,
                id_token,
                &verifier,
                token_response.access_token(),
            ),
            Err(error) if rotation_may_resolve(&error) => {
                self.refresh_jwks_for_generation(generation).await?;
                let refreshed_client = self.clients.read().await.client.clone();
                let refreshed_verifier = refreshed_client.id_token_verifier();
                let refreshed_claims = id_token
                    .claims(&refreshed_verifier, &pending.nonce)
                    .map_err(|_| CallbackError::Invalid)?;
                self.validate_verified_token(
                    refreshed_claims,
                    id_token,
                    &refreshed_verifier,
                    token_response.access_token(),
                )
            }
            Err(_) => Err(CallbackError::Invalid),
        }
    }

    fn validate_verified_token(
        &self,
        claims: &openidconnect::core::CoreIdTokenClaims,
        id_token: &openidconnect::core::CoreIdToken,
        verifier: &openidconnect::core::CoreIdTokenVerifier,
        access_token: &openidconnect::AccessToken,
    ) -> Result<(), CallbackError> {
        if !self.config.issuer().matches_raw(claims.issuer().as_str())
            || claims.issue_time().timestamp() <= 0
        {
            return Err(CallbackError::Invalid);
        }
        if claims.audiences().len() > 1 && claims.authorized_party().is_none() {
            return Err(CallbackError::Invalid);
        }
        if let Some(authorized_party) = claims.authorized_party()
            && authorized_party.as_str() != self.config.client_id()
        {
            return Err(CallbackError::Invalid);
        }
        if let Some(expected_hash) = claims.access_token_hash() {
            let actual_hash = openidconnect::AccessTokenHash::from_token(
                access_token,
                id_token.signing_alg().map_err(|_| CallbackError::Invalid)?,
                id_token
                    .signing_key(verifier)
                    .map_err(|_| CallbackError::Invalid)?,
            )
            .map_err(|_| CallbackError::Invalid)?;
            if actual_hash != *expected_hash {
                return Err(CallbackError::Invalid);
            }
        }
        Ok(())
    }

    /// Refreshes only the already discovered JWKS endpoint. The separate refresh mutex deduplicates
    /// callbacks, and the cache lock is never held during provider I/O.
    async fn refresh_jwks_for_generation(
        &self,
        failed_generation: u64,
    ) -> Result<(), CallbackError> {
        let _refresh = self.refresh_lock.lock().await;
        let cache = self.clients.read().await;
        if cache.generation != failed_generation {
            return Ok(());
        }
        if cache
            .refresh_retry_at
            .is_some_and(|retry_at| retry_at > self.monotonic_clock.now())
        {
            return Err(CallbackError::Unavailable);
        }
        let metadata = cache.metadata.clone();
        let jwks_url = metadata.jwks_uri().url().clone();
        drop(cache);

        let response = match self.http_client.client().get(jwks_url).send().await {
            Ok(response) if response.status().is_success() => response,
            _ => return self.set_jwks_cooldown().await,
        };
        let jwks = match response.json::<CoreJsonWebKeySet>().await {
            Ok(jwks) if has_usable_signing_key(&jwks, &metadata) => jwks,
            _ => return self.set_jwks_cooldown().await,
        };
        let mut cache = self.clients.write().await;
        if cache.generation == failed_generation {
            let metadata = cache.metadata.clone().set_jwks(jwks);
            let client = build_client(&self.config, self.token_auth_method, metadata.clone())
                .map_err(|_| CallbackError::Unavailable)?;
            cache.metadata = metadata;
            cache.client = client;
            cache.generation += 1;
            cache.refresh_retry_at = None;
        }
        Ok(())
    }

    async fn set_jwks_cooldown(&self) -> Result<(), CallbackError> {
        self.clients.write().await.refresh_retry_at =
            Some(self.monotonic_clock.now() + JWKS_REFRESH_COOLDOWN);
        Err(CallbackError::Unavailable)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallbackError {
    Invalid,
    Unavailable,
}

fn discovery_error_category<E>(error: &openidconnect::DiscoveryError<E>) -> AuthStartupError
where
    E: std::error::Error + 'static,
{
    // Do not format this error: `Response` contains an untrusted response body. Only retain the
    // library's safe category in the startup diagnostic.
    match error {
        openidconnect::DiscoveryError::Request(_) => AuthStartupError::Transport,
        openidconnect::DiscoveryError::Response(_, _, _) => AuthStartupError::HttpStatus,
        openidconnect::DiscoveryError::Parse(_) | openidconnect::DiscoveryError::UrlParse(_) => {
            AuthStartupError::Parsing
        }
        openidconnect::DiscoveryError::Validation(_) => AuthStartupError::UnsafeMetadata,
        openidconnect::DiscoveryError::Other(_) | _ => AuthStartupError::Parsing,
    }
}

fn validate_metadata(
    config: &EnabledAuthConfig,
    metadata: &CoreProviderMetadata,
) -> Result<(), AuthStartupError> {
    if !config.issuer().matches_raw(metadata.issuer().as_str()) {
        return Err(AuthStartupError::IssuerMismatch);
    }
    let has_safe_endpoint = |value: &str| is_safe_endpoint(config, value);
    if !has_safe_endpoint(metadata.authorization_endpoint().as_str())
        || !has_safe_endpoint(metadata.jwks_uri().as_str())
        || !metadata
            .token_endpoint()
            .is_some_and(|endpoint| has_safe_endpoint(endpoint.as_str()))
        || !has_permitted_signing_algorithms(metadata)
        || !has_usable_signing_key(metadata.jwks(), metadata)
        || !metadata
            .response_types_supported()
            .iter()
            .any(|response_types| {
                response_types
                    .iter()
                    .any(|response_type| response_type == &CoreResponseType::Code)
            })
    {
        return Err(AuthStartupError::UnsafeMetadata);
    }
    Ok(())
}

#[cfg(not(test))]
fn is_safe_endpoint(_: &EnabledAuthConfig, value: &str) -> bool {
    is_https_endpoint(value)
}

#[cfg(test)]
fn is_safe_endpoint(config: &EnabledAuthConfig, value: &str) -> bool {
    is_https_endpoint(value)
        || config.permits_loopback_http()
            && value.parse::<url::Url>().is_ok_and(|url| {
                url.scheme() == "http" && url.host().is_some_and(|host| is_loopback_host(&host))
            })
}

fn is_https_endpoint(value: &str) -> bool {
    value.parse::<url::Url>().is_ok_and(|url| {
        url.scheme() == "https"
            && url.host().is_some()
            && url.username().is_empty()
            && url.password().is_none()
    })
}

#[cfg(test)]
fn is_loopback_host(host: &url::Host<&str>) -> bool {
    matches!(host, url::Host::Domain("localhost"))
        || host
            .to_string()
            .parse::<std::net::IpAddr>()
            .is_ok_and(|address| address.is_loopback())
}

fn has_permitted_signing_algorithms(metadata: &CoreProviderMetadata) -> bool {
    let algorithms = metadata.id_token_signing_alg_values_supported();
    !algorithms.is_empty()
        && algorithms
            .iter()
            .all(|algorithm| !matches!(algorithm, CoreJwsSigningAlgorithm::None))
}

fn has_usable_signing_key(jwks: &CoreJsonWebKeySet, metadata: &CoreProviderMetadata) -> bool {
    let permitted_algorithms = metadata.id_token_signing_alg_values_supported();
    jwks.keys().iter().any(|key| {
        key.key_use().is_none_or(JsonWebKeyUse::allows_signature)
            && permitted_algorithms.iter().any(|algorithm| {
                !matches!(algorithm, CoreJwsSigningAlgorithm::None)
                    && algorithm.key_type().as_ref() == Some(key.key_type())
                    && match key.signing_alg() {
                        JsonWebKeyAlgorithm::Algorithm(key_algorithm) => key_algorithm == algorithm,
                        JsonWebKeyAlgorithm::Unspecified => true,
                        JsonWebKeyAlgorithm::Unsupported => false,
                    }
            })
    })
}

fn callback_token_error(
    error: &CoreRequestTokenError<HttpClientError<reqwest::Error>>,
) -> CallbackError {
    // A parsed OAuth error is a completed provider protocol exchange (for example invalid_grant
    // after a PKCE failure), not an availability outage. Do not format it: response details can
    // contain provider-controlled data. Transport, response parsing, and unexpected HTTP/status
    // responses cannot establish a protocol outcome and remain recoverable availability failures.
    match error {
        CoreRequestTokenError::ServerResponse(_) => CallbackError::Invalid,
        CoreRequestTokenError::Request(_)
        | CoreRequestTokenError::Parse(_, _)
        | CoreRequestTokenError::Other(_) => CallbackError::Unavailable,
    }
}

fn build_client(
    config: &EnabledAuthConfig,
    token_auth_method: TokenAuthMethod,
    metadata: CoreProviderMetadata,
) -> Result<CachedCoreClient, AuthStartupError> {
    let redirect_url =
        RedirectUrl::new(config.callback_url()).map_err(|_| AuthStartupError::Parsing)?;
    Ok(openidconnect::core::CoreClient::from_provider_metadata(
        metadata,
        ClientId::new(config.client_id().to_string()),
        Some(ClientSecret::new(config.client_secret().to_string())),
    )
    .set_auth_type(token_auth_method.oidc())
    .set_redirect_uri(redirect_url))
}

fn rotation_may_resolve(error: &openidconnect::ClaimsVerificationError) -> bool {
    matches!(
        error,
        openidconnect::ClaimsVerificationError::SignatureVerification(
            openidconnect::SignatureVerificationError::NoMatchingKey
                | openidconnect::SignatureVerificationError::CryptoError(_)
        )
    )
}

/// Selects the only supported confidential token-endpoint authentication method.
///
/// # Errors
///
/// Returns an error when discovery advertises neither Basic nor form-body client authentication.
pub fn select_token_auth_method(
    metadata: &CoreProviderMetadata,
) -> Result<TokenAuthMethod, AuthStartupError> {
    let Some(methods) = metadata.token_endpoint_auth_methods_supported() else {
        return Ok(TokenAuthMethod::Basic);
    };
    if methods.contains(&CoreClientAuthMethod::ClientSecretBasic) {
        Ok(TokenAuthMethod::Basic)
    } else if methods.contains(&CoreClientAuthMethod::ClientSecretPost) {
        Ok(TokenAuthMethod::RequestBody)
    } else {
        Err(AuthStartupError::UnsupportedTokenAuthentication)
    }
}

#[cfg(test)]
mod local_provider_tests;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_and_transaction_expiry_constants_are_bounded() {
        assert_eq!(LOGIN_TRANSACTION_TTL, Duration::from_secs(300));
        assert_eq!(SESSION_TTL, Duration::from_hours(12));
        assert_eq!(MAX_PENDING_LOGIN_TRANSACTIONS, 1_024);
    }

    #[test]
    fn provider_diagnostics_are_fixed_categories_without_sensitive_values() {
        let sensitive = "secret-marker callback-code nonce verifier access-token response-body";
        for error in [
            AuthStartupError::Transport,
            AuthStartupError::HttpStatus,
            AuthStartupError::Parsing,
            AuthStartupError::IssuerMismatch,
            AuthStartupError::UnsafeMetadata,
            AuthStartupError::UnsupportedTokenAuthentication,
        ] {
            let diagnostic = error.to_string();
            assert!(diagnostic.starts_with("OIDC discovery failed:"));
            assert!(!diagnostic.contains(sensitive));
            assert!(!diagnostic.contains("secret-marker"));
        }
        assert_eq!(format!("{:?}", CallbackError::Invalid), "Invalid");
        assert_eq!(format!("{:?}", CallbackError::Unavailable), "Unavailable");
    }
}
