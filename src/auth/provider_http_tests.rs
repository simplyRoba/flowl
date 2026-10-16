use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use openidconnect::{AsyncHttpClient, HttpRequest, http};

use crate::auth::runtime::{DiscoveryHttpClient, GuardedHttpClient};
use crate::auth::{GuardedHttpError, ProviderTransport, TransportFuture};
use crate::config::EnabledAuthConfig;

struct CountingTransport(AtomicUsize);

impl ProviderTransport for CountingTransport {
    fn send(&self, _: HttpRequest) -> TransportFuture<'_> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Box::pin(async { Ok(http::Response::new(Vec::new())) })
    }
}

fn production_policy_config() -> EnabledAuthConfig {
    // HTTPS issuer means even test builds do not enable the HTTP fixture policy.
    EnabledAuthConfig::loopback_test("https://issuer.example", "https://app.example")
}

#[tokio::test]
async fn injected_transport_cannot_bypass_production_policy() {
    let transport = Arc::new(CountingTransport(AtomicUsize::new(0)));
    let config = production_policy_config();
    let guard = GuardedHttpClient::new(super::provider_policy(&config), transport.clone());
    for destination in [
        "http://example.invalid/jwks",
        "http://127.0.0.1/jwks",
        "http://localhost/jwks",
        "http://[::1]/jwks",
        "ftp://localhost/jwks",
        "https://user:private-sentinel@example.com/jwks",
    ] {
        let request = http::Request::builder()
            .uri(destination)
            .body(Vec::new())
            .unwrap();
        assert!(matches!(
            guard.call(request).await,
            Err(GuardedHttpError::UnsafeUrl)
        ));
    }
    for destination in [
        "https://example.com/jwks#private-sentinel",
        "https://example.com/jwks#",
    ] {
        assert!(guard.validate_raw_url(destination).is_err());
    }
    assert_eq!(transport.0.load(Ordering::SeqCst), 0);
    let request = http::Request::builder()
        .uri("https://example.com/token?allowed=query")
        .body(Vec::new())
        .unwrap();
    assert!(guard.call(request).await.is_ok());
    assert_eq!(transport.0.load(Ordering::SeqCst), 1);
    assert!(config.secure_cookie());
    assert!(
        EnabledAuthConfig::loopback_test("http://127.0.0.1", "http://127.0.0.1").secure_cookie()
    );
}

#[tokio::test]
async fn initial_metadata_fragment_is_checked_before_library_jwks_fetch() {
    struct UnsafeMetadata(AtomicUsize);
    impl ProviderTransport for UnsafeMetadata {
        fn send(&self, _: HttpRequest) -> TransportFuture<'_> {
            self.0.fetch_add(1, Ordering::SeqCst);
            Box::pin(async {
                Ok(http::Response::new(
                    br#"{"jwks_uri":"https://keys.example/jwks#private-sentinel"}"#.to_vec(),
                ))
            })
        }
    }
    let transport = Arc::new(UnsafeMetadata(AtomicUsize::new(0)));
    let guard = GuardedHttpClient::new(
        super::provider_policy(&production_policy_config()),
        transport.clone(),
    );
    let discovery = DiscoveryHttpClient::new(&guard);
    let request = http::Request::builder()
        .uri("https://issuer.example/metadata")
        .body(Vec::new())
        .unwrap();
    assert!(matches!(
        discovery.call(request).await,
        Err(GuardedHttpError::UnsafeUrl)
    ));
    assert_eq!(transport.0.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn library_token_and_refresh_requests_use_the_guard() {
    let transport = Arc::new(CountingTransport(AtomicUsize::new(0)));
    let config = production_policy_config();
    let guard = GuardedHttpClient::new(super::provider_policy(&config), transport.clone());
    for destination in [
        "http://127.0.0.1/provider",
        "https://user:private-sentinel@example.com/provider",
    ] {
        let metadata = serde_json::from_value::<openidconnect::core::CoreProviderMetadata>(serde_json::json!({
            "issuer": "https://issuer.example", "authorization_endpoint": "https://issuer.example/authorize",
            "token_endpoint": destination, "jwks_uri": destination,
            "response_types_supported": ["code"], "subject_types_supported": ["public"],
            "id_token_signing_alg_values_supported": ["RS256"]
        })).unwrap();
        let client = openidconnect::core::CoreClient::from_provider_metadata(
            metadata.clone(),
            openidconnect::ClientId::new(config.client_id.clone()),
            Some(openidconnect::ClientSecret::new(
                config.client_secret.clone(),
            )),
        )
        .set_redirect_uri(
            openidconnect::RedirectUrl::new(config.callback_url.to_string()).unwrap(),
        );
        let error = client
            .exchange_code(openidconnect::AuthorizationCode::new(
                "private-sentinel".into(),
            ))
            .unwrap()
            .request_async(&guard)
            .await
            .unwrap_err();
        assert!(matches!(
            error,
            openidconnect::RequestTokenError::Request(GuardedHttpError::UnsafeUrl)
        ));
        assert!(
            openidconnect::core::CoreJsonWebKeySet::fetch_async(metadata.jwks_uri(), &guard)
                .await
                .is_err()
        );
    }
    assert_eq!(transport.0.load(Ordering::SeqCst), 0);
}
