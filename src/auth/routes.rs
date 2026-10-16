//! OIDC endpoint handlers and session helpers.

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::Extension;
use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use serde::Deserialize;
use tower_sessions::Session;

use super::{AuthState, CallbackError, Clock, LOGIN_TRANSACTION_TTL, PendingLogin, SESSION_TTL};
use crate::auth::return_to::SafeReturnTo;

pub(crate) const PREAUTH_STATE_KEY: &str = "flowl.auth.pending_state";
const AUTHENTICATED_KEY: &str = "flowl.auth.authenticated";
const AUTHENTICATED_AT_KEY: &str = "flowl.auth.authenticated_at";
const EXPIRES_AT_KEY: &str = "flowl.auth.expires_at";

#[derive(Deserialize)]
pub struct LoginQuery {
    return_to: Option<String>,
}

/// Public, non-sensitive configuration consumed by the login page.
pub async fn config(auth: Option<Extension<Arc<AuthState>>>) -> Response {
    let payload = match auth {
        Some(Extension(auth)) => config_payload(Some(auth.config().provider_name())),
        None => config_payload(None),
    };
    no_store((StatusCode::OK, axum::Json(payload)).into_response())
}

fn config_payload(provider_name: Option<&str>) -> serde_json::Value {
    match provider_name {
        Some(name) => serde_json::json!({ "enabled": true, "provider_name": name }),
        None => serde_json::json!({ "enabled": false }),
    }
}

pub async fn login(
    Extension(auth): Extension<Arc<AuthState>>,
    Query(query): Query<LoginQuery>,
    session: Session,
) -> Response {
    let return_to = SafeReturnTo::fallback(query.return_to.as_deref().unwrap_or("/"));
    let Ok((url, state_token, nonce, verifier)) = auth.authorization_url().await else {
        return generic_redirect("provider_unavailable", &return_to);
    };
    let pending = PendingLogin {
        nonce,
        verifier,
        return_to: return_to.clone(),
        expires_at: auth.clock().now() + LOGIN_TRANSACTION_TTL,
    };
    let state_value = state_token.secret().clone();
    if auth
        .bind_transaction(&session, state_value, pending)
        .await
        .is_err()
    {
        return generic_redirect("provider_unavailable", &return_to);
    }
    no_store(Redirect::to(url.as_str()).into_response())
}

/// Parses callback parameters manually so duplicate and mixed success/error values are rejected
/// before a state can cause any provider request.
pub async fn callback(
    Extension(auth): Extension<Arc<AuthState>>,
    session: Session,
    raw_query: axum::extract::RawQuery,
) -> Response {
    let parameters = raw_query.0.unwrap_or_default();
    let parsed = parse_callback_parameters(&parameters);
    let (state_value, code) = match parsed {
        Ok(CallbackParameters::Success { state, code }) => (state, code),
        Ok(CallbackParameters::ProviderError {
            state: provider_state,
        }) => {
            let target = auth
                .consume_bound_transaction(&session, &provider_state)
                .await
                .map_or_else(|| SafeReturnTo::fallback("/"), |pending| pending.return_to);
            return generic_redirect("authentication_failed", &target);
        }
        Err(()) => return generic_redirect("authentication_failed", &SafeReturnTo::fallback("/")),
    };

    let Some(pending) = auth.consume_bound_transaction(&session, &state_value).await else {
        return generic_redirect("authentication_failed", &SafeReturnTo::fallback("/"));
    };

    let return_to = pending.return_to.clone();
    match auth.exchange_and_verify(code, pending).await {
        Ok(()) => {
            if establish_authenticated_session(&session, auth.clock().as_ref())
                .await
                .is_err()
            {
                return generic_redirect("provider_unavailable", &return_to);
            }
            no_store(Redirect::to(return_to.as_str()).into_response())
        }
        Err(CallbackError::Unavailable) => generic_redirect("provider_unavailable", &return_to),
        Err(CallbackError::Invalid) => generic_redirect("authentication_failed", &return_to),
    }
}

pub async fn logout(session: Session) -> Response {
    let _ = session.flush().await;
    no_store(Redirect::to("/login?logged_out=1").into_response())
}

/// Checks the local marker and absolute deadline. This deliberately does not touch timestamps,
/// making the server-side twelve-hour expiration non-sliding.
pub async fn has_authenticated_session(session: &Session, auth: &AuthState) -> bool {
    let authenticated = session
        .get::<bool>(AUTHENTICATED_KEY)
        .await
        .ok()
        .flatten()
        .unwrap_or(false);
    let expires_at = session.get::<u64>(EXPIRES_AT_KEY).await.ok().flatten();
    let now = seconds_since_epoch(auth.clock().now());
    if authenticated && expires_at.is_some_and(|deadline| now < deadline) {
        return true;
    }
    if authenticated || expires_at.is_some() {
        let _ = session.flush().await;
    }
    false
}

async fn establish_authenticated_session(session: &Session, clock: &dyn Clock) -> Result<(), ()> {
    // Rotate only after all OIDC checks succeeded. Remove the pre-auth binding before persisting
    // the authenticated marker, so the only retained data is local authentication timing.
    session.cycle_id().await.map_err(|_| ())?;
    session.clear().await;
    let authenticated_time = clock.now();
    let authenticated_at = seconds_since_epoch(authenticated_time);
    let expires_at = authenticated_at.saturating_add(SESSION_TTL.as_secs());
    session
        .insert(AUTHENTICATED_KEY, true)
        .await
        .map_err(|_| ())?;
    session
        .insert(AUTHENTICATED_AT_KEY, authenticated_at)
        .await
        .map_err(|_| ())?;
    session
        .insert(EXPIRES_AT_KEY, expires_at)
        .await
        .map_err(|_| ())?;
    session.set_expiry(Some(tower_sessions::Expiry::AtDateTime(
        (authenticated_time + SESSION_TTL).into(),
    )));
    session.save().await.map_err(|_| ())?;
    Ok(())
}

fn seconds_since_epoch(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn generic_redirect(error: &str, return_to: &SafeReturnTo) -> Response {
    let target = format!(
        "/login?error={error}&return_to={}",
        url::form_urlencoded::byte_serialize(return_to.as_str().as_bytes()).collect::<String>()
    );
    no_store(Redirect::to(&target).into_response())
}

fn no_store(mut response: Response) -> Response {
    response.headers_mut().insert(
        axum::http::header::CACHE_CONTROL,
        axum::http::HeaderValue::from_static("no-store"),
    );
    response
}

enum CallbackParameters {
    Success { state: String, code: String },
    ProviderError { state: String },
}

fn parse_callback_parameters(query: &str) -> Result<CallbackParameters, ()> {
    let mut state = None;
    let mut code = None;
    let mut error = None;
    let mut error_description = None;
    let mut error_uri = None;
    for pair in query.split('&') {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        let key = decode_query_component(key).ok_or(())?;
        let value = decode_query_component(value).ok_or(())?;
        let target = match key.as_str() {
            "code" => &mut code,
            "state" => &mut state,
            "error" => &mut error,
            "error_description" => &mut error_description,
            "error_uri" => &mut error_uri,
            _ => continue,
        };
        if target.replace(value).is_some() {
            return Err(());
        }
    }
    let state = state.filter(|state| !state.is_empty()).ok_or(())?;
    match (code, error, error_description, error_uri) {
        (Some(code), None, None, None) if !code.is_empty() => {
            Ok(CallbackParameters::Success { state, code })
        }
        (None, Some(error), _, _) if !error.is_empty() => {
            Ok(CallbackParameters::ProviderError { state })
        }
        _ => Err(()),
    }
}

fn decode_query_component(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'+' => decoded.push(b' '),
            b'%' => {
                let high = *bytes.get(index + 1)?;
                let low = *bytes.get(index + 2)?;
                decoded.push(hex(high)? << 4 | hex(low)?);
                index += 2;
            }
            byte => decoded.push(byte),
        }
        index += 1;
    }
    String::from_utf8(decoded).ok()
}

fn hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use tower_sessions::session::Record;
    use tower_sessions::{SessionStore, session_store};

    use super::*;

    #[derive(Debug, Default)]
    struct CapturingStore {
        record: Mutex<Option<Record>>,
    }

    #[async_trait]
    impl SessionStore for CapturingStore {
        async fn create(&self, record: &mut Record) -> session_store::Result<()> {
            *self.record.lock().expect("record lock") = Some(record.clone());
            Ok(())
        }

        async fn save(&self, record: &Record) -> session_store::Result<()> {
            *self.record.lock().expect("record lock") = Some(record.clone());
            Ok(())
        }

        async fn load(
            &self,
            _: &tower_sessions::session::Id,
        ) -> session_store::Result<Option<Record>> {
            Ok(None)
        }

        async fn delete(&self, _: &tower_sessions::session::Id) -> session_store::Result<()> {
            Ok(())
        }
    }

    struct FixedClock(SystemTime);

    impl Clock for FixedClock {
        fn now(&self) -> SystemTime {
            self.0
        }
    }

    #[test]
    fn shared_callback_vectors_follow_the_common_contract() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("../../tests/fixtures/auth/callbacks.json"))
                .expect("callback fixture should parse");
        for vector in fixture["vectors"].as_array().unwrap() {
            let id = vector["id"].as_str().unwrap();
            let shape = match parse_callback_parameters(vector["query"].as_str().unwrap_or("")) {
                Ok(CallbackParameters::Success { .. }) => "success",
                Ok(CallbackParameters::ProviderError { .. }) => "error",
                Err(()) => "invalid",
            };
            assert_eq!(shape, vector["target"]["flowl"], "{id}");
        }
    }

    #[test]
    fn callback_shapes_decode_names_and_values_before_validation() {
        for query in [
            "error=denied",
            "error=denied&state=",
            "state=s&code",
            "error=&state=s&error_uri=detail",
            "error=denied&state=s&error_description=&%65rror_description=detail",
            "error=denied&state=s&error_uri=&%65rror_uri=detail",
            "code=c&state=s&%63ode=",
            "error=denied&state=s&%65rror=",
            "code=c&state=s&%GG=ignored",
            "code=c&state=s&extra=ok&extra=%FF",
        ] {
            assert!(parse_callback_parameters(query).is_err(), "{query}");
        }
        assert!(matches!(
            parse_callback_parameters("error=denied&state=s&error_description=&error_uri="),
            Ok(CallbackParameters::ProviderError { state }) if state == "s"
        ));
        assert!(matches!(
            parse_callback_parameters("%63ode=a%2Bb+c&%73tate=s%2Bt+u&extra=1&extra=2"),
            Ok(CallbackParameters::Success { state, code }) if state == "s+t u" && code == "a+b c"
        ));
    }

    #[test]
    fn config_payload_keeps_provider_name_as_json_data() {
        let provider_name = "<provider>&\\\"name";
        let payload = config_payload(Some(provider_name));
        assert_eq!(
            payload
                .get("provider_name")
                .and_then(serde_json::Value::as_str),
            Some(provider_name)
        );
        assert_eq!(payload.get("enabled"), Some(&serde_json::json!(true)));
    }

    #[tokio::test]
    async fn successful_session_retains_only_local_authentication_timestamps() {
        let store = Arc::new(CapturingStore::default());
        let session = Session::new(None, store.clone(), None);
        for (key, value) in [
            (PREAUTH_STATE_KEY, "seeded-state"),
            ("oidc.nonce", "seeded-nonce"),
            ("oidc.code", "seeded-code"),
            ("oidc.token", "seeded-token"),
            ("oidc.claims", "seeded-claims"),
            ("oidc.secret", "seeded-secret"),
        ] {
            session
                .insert(key, value)
                .await
                .expect("seed session value");
        }
        let clock = FixedClock(UNIX_EPOCH + std::time::Duration::from_secs(100));

        establish_authenticated_session(&session, &clock)
            .await
            .expect("establish session");

        let record = store
            .record
            .lock()
            .expect("record lock")
            .clone()
            .expect("saved session record");
        assert_eq!(record.data.len(), 3);
        assert_eq!(
            record.data.get(AUTHENTICATED_KEY),
            Some(&serde_json::json!(true))
        );
        assert_eq!(
            record.data.get(AUTHENTICATED_AT_KEY),
            Some(&serde_json::json!(100))
        );
        assert_eq!(
            record.data.get(EXPIRES_AT_KEY),
            Some(&serde_json::json!(100 + SESSION_TTL.as_secs()))
        );
        for secret in [
            "seeded-state",
            "seeded-nonce",
            "seeded-code",
            "seeded-token",
            "seeded-claims",
            "seeded-secret",
        ] {
            assert!(
                !record.data.values().any(|value| value == secret),
                "session retained {secret}"
            );
        }
    }

    #[test]
    fn callback_rejects_duplicate_and_mixed_parameters() {
        for query in [
            "state=one&state=two&code=code",
            "state=one&code=one&code=two",
            "state=one&code=one&error=access_denied",
            "state=one&error=access_denied&error=server_error",
            "state=one&code=one&error_description=pollution",
            "code=one",
            "state=one",
        ] {
            assert!(parse_callback_parameters(query).is_err(), "{query}");
        }
    }

    #[test]
    fn callback_accepts_only_unambiguous_success_or_error() {
        assert!(matches!(
            parse_callback_parameters("state=one&code=two"),
            Ok(CallbackParameters::Success { .. })
        ));
        assert!(matches!(
            parse_callback_parameters("error=access_denied&state=one"),
            Ok(CallbackParameters::ProviderError { .. })
        ));
    }
}
