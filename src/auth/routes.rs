//! OIDC endpoint handlers and authoritative browser lifecycle coordination.

use std::sync::Arc;

use axum::Extension;
use axum::extract::Query;
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Redirect, Response};
use openidconnect::CsrfToken;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tower_sessions::{Expiry, Session};

use super::{AuthState, CallbackError, LOGIN_TRANSACTION_TTL, PendingLogin, SESSION_TTL};
use crate::auth::return_to::SafeReturnTo;

pub(crate) const PREAUTH_STATE_KEY: &str = "flowl.auth.pending_state";
pub(crate) const AUTHENTICATED_KEY: &str = "flowl.auth.authenticated";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct AuthenticatedSession {
    pub(crate) authenticated_at: i64,
    pub(crate) expires_at: i64,
}

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
    Extension(authentication): Extension<Arc<AuthState>>,
    session: Session,
    Extension(super::session::OriginalSessionId(original_id)): Extension<
        super::session::OriginalSessionId,
    >,
    Query(query): Query<LoginQuery>,
) -> Response {
    let target = SafeReturnTo::fallback(query.return_to.as_deref().unwrap_or("/"));
    let return_to = target.as_str().to_owned();
    let store = &authentication.sessions.store;
    let now = authentication.clock.now();
    // Loading can clear an unknown ID. Remember known aliases first, and retain
    // the ORIGINAL request revision: reloading must not promote a stale request.
    let known = original_id.and_then(|id| store.for_id(id));
    let Ok(snapshot) = session
        .get::<super::session_store::Owner>(super::session_store::OWNER_KEY)
        .await
    else {
        return failure_redirect(FailureCode::ProviderUnavailable, &return_to);
    };
    let (key, lineage) = if let Some(marker) = &snapshot {
        let Some(lineage) = store.lineage(&marker.lineage) else {
            return failure_redirect(FailureCode::ProviderUnavailable, &return_to);
        };
        (marker.lineage.clone(), lineage)
    } else if known.is_some() {
        return failure_redirect(FailureCode::ProviderUnavailable, &return_to);
    } else {
        store.allocate((now + LOGIN_TRANSACTION_TTL).into())
    };
    let snapshot_id = session.id();
    let _gate = lineage.gate.lock().await;
    if let Some(snapshot) = snapshot {
        let owned = match snapshot_id {
            Some(id) => store.owns(id, &snapshot).await,
            None => false,
        };
        if !owned || store.refresh(&session).await.is_err() || !store.current(&session).await {
            return failure_redirect(FailureCode::ProviderUnavailable, &return_to);
        }
    }
    if authentication.is_authenticated(&session).await {
        return redirect(&return_to);
    }

    let old_state = session
        .get::<String>(PREAUTH_STATE_KEY)
        .await
        .ok()
        .flatten();
    let Ok((authorization_url, state, nonce, pkce_verifier)) =
        authentication.authorization_url().await
    else {
        if lineage.authority.lock().await.committed.is_none() {
            store.retire_empty(&key);
        }
        return failure_redirect(FailureCode::ProviderUnavailable, &return_to);
    };
    let state_secret = state.secret().clone();
    let now = authentication.clock.now();
    let expires_at = now + LOGIN_TRANSACTION_TTL;
    if authentication
        .insert_transaction(
            state_secret.clone(),
            PendingLogin {
                nonce,
                verifier: pkce_verifier,
                return_to: target,
                expires_at,
            },
        )
        .await
        .is_err()
    {
        if lineage.authority.lock().await.committed.is_none() {
            store.retire_empty(&key);
        }
        return failure_redirect(FailureCode::ProviderUnavailable, &return_to);
    }

    let marker = super::session_store::Owner::fresh(key.clone());
    {
        let mut authority = lineage.authority.lock().await;
        authority.permit = Some(marker.clone());
        authority.creating = session.id().is_none();
    }
    session.set_expiry(Some(Expiry::AtDateTime(expires_at.into())));
    let saved = async {
        session
            .insert(super::session_store::OWNER_KEY, marker)
            .await?;
        session.insert(PREAUTH_STATE_KEY, &state_secret).await?;
        session.save().await
    }
    .await;
    if saved.is_err() || !store.current(&session).await {
        authentication.consume_transaction(&state_secret).await;
        store.rollback_binding(&session, &lineage, &key).await;
        return failure_redirect(FailureCode::ProviderUnavailable, &return_to);
    }
    if let Some(old) = old_state {
        authentication.consume_transaction(&old).await;
    }
    redirect(authorization_url.as_str())
}

pub async fn callback(
    Extension(authentication): Extension<Arc<AuthState>>,
    session: Session,
    Extension(capture): Extension<super::session::ResponseCapture>,
    raw_query: axum::extract::RawQuery,
) -> Response {
    let response = complete_callback(&authentication, &session, raw_query.0.as_deref()).await;
    if let Some(intent) = response
        .extensions()
        .get::<super::session::CallbackIntent>()
    {
        capture.remember_callback(intent.clone());
    }
    response
}

async fn complete_callback(
    authentication: &AuthState,
    session: &Session,
    raw_query: Option<&str>,
) -> Response {
    let Ok(query) = parse_callback_parameters(raw_query.unwrap_or_default()) else {
        return callback_failure(FailureCode::AuthenticationFailed, "/");
    };
    let (returned_state, code) = match query {
        CallbackParameters::Success { state, code } => (state, Some(code)),
        CallbackParameters::ProviderError { state } => (state, None),
    };
    let store = &authentication.sessions.store;
    let Some(lineage) = session.id().and_then(|id| store.for_id(id)) else {
        return callback_failure(FailureCode::AuthenticationFailed, "/");
    };
    let gate = lineage.gate.lock().await;
    if store.refresh(session).await.is_err() {
        return callback_failure(FailureCode::ProviderUnavailable, "/");
    }
    if !store.current(session).await {
        return callback_failure(FailureCode::AuthenticationFailed, "/");
    }
    let bound_state = session
        .get::<String>(PREAUTH_STATE_KEY)
        .await
        .ok()
        .flatten();
    let state_matches = bound_state.is_some_and(|bound_state| {
        CsrfToken::new(bound_state) == CsrfToken::new(returned_state.clone())
    });
    if !state_matches {
        return callback_failure(FailureCode::AuthenticationFailed, "/");
    }

    let Some(transaction) = authentication.consume_transaction(&returned_state).await else {
        return callback_failure(FailureCode::AuthenticationFailed, "/");
    };
    // Consumption only grants exchange permission. The session's committed
    // revision remains the completion guard until replacement/logout/expiry.
    drop(gate);

    let Some(code) = code else {
        return callback_failure(
            FailureCode::AuthenticationFailed,
            transaction.return_to.as_str(),
        );
    };

    let return_to = transaction.return_to.as_str().to_owned();
    match authentication.exchange_and_verify(code, transaction).await {
        Ok(()) => {}
        Err(CallbackError::Invalid) => {
            return callback_failure(FailureCode::AuthenticationFailed, &return_to);
        }
        Err(CallbackError::Unavailable) => {
            return callback_failure(FailureCode::ProviderUnavailable, &return_to);
        }
    }

    persist_callback(authentication, session, &lineage, &return_to).await
}

async fn persist_callback(
    authentication: &AuthState,
    session: &Session,
    lineage: &super::session_store::Lineage,
    return_to: &str,
) -> Response {
    let store = &authentication.sessions.store;
    let _gate = lineage.gate.lock().await;
    if !store.current(session).await {
        return callback_failure(FailureCode::AuthenticationFailed, return_to);
    }
    if store.refresh(session).await.is_err() {
        let _ = store.revoke(lineage).await;
        session.clear().await;
        let _ = session.flush().await;
        return callback_failure(FailureCode::ProviderUnavailable, return_to);
    }
    let now = authentication.clock.now();
    let expires_at = now + SESSION_TTL;
    let previous = match session
        .get::<super::session_store::Owner>(super::session_store::OWNER_KEY)
        .await
    {
        Ok(Some(previous)) => previous,
        Ok(None) => return callback_failure(FailureCode::AuthenticationFailed, return_to),
        Err(_) => return callback_failure(FailureCode::ProviderUnavailable, return_to),
    };
    if !store.current(session).await {
        return callback_failure(FailureCode::AuthenticationFailed, return_to);
    }
    let marker = super::session_store::Owner::fresh(previous.lineage);
    {
        let mut authority = lineage.authority.lock().await;
        authority.permit = Some(marker.clone());
        authority.creating = true;
    }
    let saved = async {
        session.cycle_id().await?;
        session.clear().await;
        session.set_expiry(Some(Expiry::AtDateTime(expires_at.into())));
        session
            .insert(super::session_store::OWNER_KEY, marker)
            .await?;
        session
            .insert(
                AUTHENTICATED_KEY,
                AuthenticatedSession {
                    authenticated_at: OffsetDateTime::from(now).unix_timestamp(),
                    expires_at: OffsetDateTime::from(expires_at).unix_timestamp(),
                },
            )
            .await?;
        session.save().await
    }
    .await;
    if saved.is_err() || !store.current(session).await {
        let _ = store.revoke(lineage).await;
        // cycle_id has already deleted the old ID. Do not let middleware retry
        // a provisional create, including after a partial backing-store write.
        session.clear().await;
        let _ = session.flush().await;
        return callback_failure(FailureCode::ProviderUnavailable, return_to);
    }
    let mut response = redirect(return_to);
    response
        .extensions_mut()
        .insert(super::session::CallbackIntent::verified(return_to));
    response
}

impl AuthState {
    pub(crate) async fn is_authenticated(&self, session: &Session) -> bool {
        let record = session
            .get::<AuthenticatedSession>(AUTHENTICATED_KEY)
            .await
            .ok()
            .flatten();
        match record {
            Some(record)
                if record.expires_at > OffsetDateTime::from(self.clock.now()).unix_timestamp() =>
            {
                self.sessions.store.current(session).await
            }
            Some(_) => {
                let _ = session
                    .remove::<AuthenticatedSession>(AUTHENTICATED_KEY)
                    .await;
                false
            }
            None => false,
        }
    }
}

pub async fn logout(
    Extension(authentication): Extension<Arc<AuthState>>,
    session: Session,
    Extension(super::session::OriginalSessionId(original_id)): Extension<
        super::session::OriginalSessionId,
    >,
    Extension(capture): Extension<super::session::ResponseCapture>,
) -> Response {
    capture.remember_logout();
    let store = &authentication.sessions.store;
    let lineage = original_id.and_then(|id| store.for_id(id));
    let mut failed;
    if let Some(lineage) = lineage {
        let _gate = lineage.gate.lock().await;
        let state = lineage
            .authority
            .lock()
            .await
            .committed
            .as_ref()
            .and_then(|record| record.data.get(PREAUTH_STATE_KEY))
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned);
        if let Some(state) = state {
            authentication.consume_transaction(&state).await;
        }
        failed = store.revoke(&lineage).await.is_err();
        // Flush the independently cached request too. Aliases cannot restore the
        // current record, and a failure still returns 500 with cookie expiry.
        failed |= session.flush().await.is_err();
    } else {
        failed = session.flush().await.is_err();
    }
    let mut response = if failed {
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    } else {
        redirect("/login?logged_out=1")
    };
    response
        .extensions_mut()
        .insert(super::session::LogoutExpiry);
    response
        .headers_mut()
        .append(header::SET_COOKIE, super::session::removal_cookie(true));
    response
}

/// Checks one cohesive local timing record and its authoritative ownership without sliding expiry.
pub async fn has_authenticated_session(session: &Session, auth: &AuthState) -> bool {
    auth.is_authenticated(session).await
}

#[derive(Clone, Copy)]
pub(super) enum FailureCode {
    AuthenticationFailed,
    ProviderUnavailable,
}

impl FailureCode {
    fn as_str(self) -> &'static str {
        match self {
            Self::AuthenticationFailed => "authentication_failed",
            Self::ProviderUnavailable => "provider_unavailable",
        }
    }
}

fn callback_failure(error: FailureCode, return_to: &str) -> Response {
    let mut response = failure_redirect(error, return_to);
    response
        .extensions_mut()
        .insert(super::session::CallbackIntent::failed(return_to));
    response
}

pub(super) fn failure_redirect(error: FailureCode, return_to: &str) -> Response {
    generic_redirect(error.as_str(), &SafeReturnTo::fallback(return_to))
}

pub(crate) fn redirect(location: &str) -> Response {
    no_store(Redirect::to(location).into_response())
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
    use super::*;

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
