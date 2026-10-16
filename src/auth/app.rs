//! Flowl-owned authentication policy and configuration-source adapter.

use super::config::{AuthConfigSource, OidcConfig};
use super::runtime::UrlPolicy;

pub(super) const ENV_PREFIX: &str = "FLOWL_";
pub const SESSION_COOKIE_NAME: &str = "flowl.sid";
pub(super) const NON_DOCUMENT_ROOTS: &[&str] = &["/uploads"];

pub(super) fn config_policy() -> UrlPolicy {
    UrlPolicy::HttpsOnly
}

#[cfg(not(test))]
pub(super) fn provider_policy(_: &OidcConfig) -> UrlPolicy {
    UrlPolicy::HttpsOnly
}

#[cfg(test)]
pub(super) fn provider_policy(config: &OidcConfig) -> UrlPolicy {
    if config.issuer.starts_with("http://")
        && UrlPolicy::LoopbackHttp.validate(&config.issuer).is_ok()
    {
        UrlPolicy::LoopbackHttp
    } else {
        UrlPolicy::HttpsOnly
    }
}

pub(super) fn cookie_secure(_: &OidcConfig) -> bool {
    true
}

#[cfg(test)]
pub(super) const FIXTURE_APP: &str = "flowl";

pub(crate) struct AuthSource<'a, S>(pub &'a S);

impl<S: crate::config::ConfigSource> AuthConfigSource for AuthSource<'_, S> {
    fn get(&self, key: &str) -> Result<Option<String>, std::env::VarError> {
        self.0.get_auth(&format!("{ENV_PREFIX}{key}"))
    }
}

/// Public app DTO; no provider protocol or navigation implementation lives here.
pub(crate) async fn public_config(
    auth: Option<axum::Extension<std::sync::Arc<super::Authentication>>>,
) -> axum::Json<serde_json::Value> {
    axum::Json(config_payload(
        auth.as_ref()
            .map(|axum::Extension(auth)| auth.config().provider_name.as_str()),
    ))
}

fn config_payload(provider_name: Option<&str>) -> serde_json::Value {
    match provider_name {
        Some(name) => serde_json::json!({ "enabled": true, "provider_name": name }),
        None => serde_json::json!({ "enabled": false }),
    }
}

#[cfg(test)]
#[path = "app_tests.rs"]
mod tests;

#[cfg(test)]
impl OidcConfig {
    pub(crate) fn loopback_test(issuer: &str, external_url: &str) -> Self {
        let external_url = url::Url::parse(external_url).expect("valid fixture origin");
        Self {
            callback_url: external_url.join("/auth/callback").unwrap(),
            external_url,
            issuer: issuer.to_owned(),
            client_id: "flowl-test-client".to_owned(),
            client_secret: "flowl-test-secret".to_owned(),
            provider_name: "OpenID Connect".to_owned(),
        }
    }
}

#[cfg(test)]
#[path = "local_provider_tests.rs"]
mod local_provider_tests;
#[cfg(test)]
#[path = "provider_http_tests.rs"]
mod provider_http_tests;

#[cfg(test)]
use super::runtime::{
    ClientAuthMethod as TokenAuthMethod, OidcStartupError, ProtocolError as CallbackError,
};
#[cfg(test)]
use super::transaction::{LoginTransaction as PendingLogin, SystemClock};
#[cfg(test)]
use super::{Authentication as AuthState, AuthenticationStartupError as AuthStartupError, Clock};
#[cfg(test)]
use super::{ProviderTransport as AuthHttpClient, ReqwestTransport as ReqwestAuthHttpClient};
#[cfg(test)]
use super::{SessionFaults, StoreOperation};
#[cfg(test)]
use std::sync::Arc;

#[cfg(test)]
const PREAUTH_STATE_KEY: &str = "oidc_transaction";
#[cfg(test)]
const LOGIN_TRANSACTION_TTL: std::time::Duration = std::time::Duration::from_secs(300);
#[cfg(test)]
const SESSION_TTL: std::time::Duration = std::time::Duration::from_hours(12);
#[cfg(test)]
const JWKS_REFRESH_COOLDOWN: std::time::Duration = std::time::Duration::from_secs(30);
#[cfg(test)]
const MAX_PENDING_LOGIN_TRANSACTIONS: usize = 1_024;

// Local compatibility seams adapt the older Flowl harness, not production behavior.
#[cfg(test)]
trait MonotonicClock: Send + Sync {
    fn now(&self) -> std::time::Instant;
}
#[cfg(test)]
struct SystemMonotonicClock;
#[cfg(test)]
impl MonotonicClock for SystemMonotonicClock {
    fn now(&self) -> std::time::Instant {
        std::time::Instant::now()
    }
}
#[cfg(test)]
struct JoinedClock {
    wall: Arc<dyn Clock>,
    monotonic: Arc<dyn MonotonicClock>,
}
#[cfg(test)]
impl Clock for JoinedClock {
    fn now(&self) -> time::OffsetDateTime {
        self.wall.now()
    }
    fn monotonic_now(&self) -> std::time::Instant {
        self.monotonic.now()
    }
}

#[cfg(test)]
impl super::Authentication {
    async fn with_dependencies(
        config: OidcConfig,
        clock: Arc<dyn Clock>,
        transport: Arc<dyn AuthHttpClient>,
    ) -> Result<Self, AuthStartupError> {
        Self::with_all_dependencies(config, clock, Arc::new(SystemMonotonicClock), transport).await
    }

    async fn with_all_dependencies(
        config: OidcConfig,
        clock: Arc<dyn Clock>,
        monotonic: Arc<dyn MonotonicClock>,
        transport: Arc<dyn AuthHttpClient>,
    ) -> Result<Self, AuthStartupError> {
        Self::with_session_faults(
            config,
            clock,
            monotonic,
            transport,
            SessionFaults::default(),
        )
        .await
    }

    async fn with_session_faults(
        config: OidcConfig,
        wall: Arc<dyn Clock>,
        monotonic: Arc<dyn MonotonicClock>,
        transport: Arc<dyn AuthHttpClient>,
        faults: SessionFaults,
    ) -> Result<Self, AuthStartupError> {
        Self::bootstrap_with_dependencies(
            config,
            Arc::new(JoinedClock { wall, monotonic }),
            faults,
            transport,
        )
        .await
    }

    async fn authorization_url(
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
        Ok(self.runtime.authorization_url().await)
    }

    async fn exchange_and_verify(
        &self,
        code: String,
        pending: PendingLogin,
    ) -> Result<(), CallbackError> {
        self.runtime
            .exchange_and_verify(code, &pending.nonce, pending.pkce_verifier)
            .await
    }

    async fn consume_transaction(&self, state: &str) -> Option<PendingLogin> {
        self.transactions.consume(state, self.clock.now()).await
    }
}
