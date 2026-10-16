use std::future::Future;

use crate::auth::{SessionFaults, StoreOperation};
use axum::body::Body;
use axum::http::HeaderValue;
use tokio::sync::Semaphore;

use super::*;

#[derive(Debug)]
pub(super) struct TokenGate {
    entered: Semaphore,
    released: Semaphore,
}

impl TokenGate {
    async fn install(provider: &RunningProvider) -> Arc<Self> {
        let gate = Arc::new(Self {
            entered: Semaphore::new(0),
            released: Semaphore::new(0),
        });
        *provider.provider.token_gate.lock().await = Some(Arc::clone(&gate));
        gate
    }

    async fn entered(&self) {
        self.entered.acquire().await.unwrap().forget();
    }
    fn release(&self) {
        self.released.add_permits(1);
    }
}

pub(super) async fn wait_token_gate(state: &Provider) {
    let gate = state.token_gate.lock().await.take();
    if let Some(gate) = gate {
        gate.entered.add_permits(1);
        gate.released.acquire().await.unwrap().forget();
    }
}

async fn bounded<T>(future: impl Future<Output = T>) -> T {
    tokio::time::timeout(std::time::Duration::from_secs(5), future)
        .await
        .expect("unrelated lifecycle work must not stall")
}

async fn request(app: &Router, method: &str, path: &str, cookie: &str) -> Response {
    app.clone()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(path)
                .header(header::COOKIE, cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap()
}

async fn replacement(app: &Router, provider: &RunningProvider, cookie: &str) -> LoginAttempt {
    let response = request(app, "GET", "/auth/login?return_to=%2Fnew", cookie).await;
    let url = url::Url::parse(&response_location(&response)).unwrap();
    let query: HashMap<_, _> = url.query_pairs().into_owned().collect();
    let attempt = LoginAttempt {
        cookie: response_cookie(&response),
        state: query["state"].clone(),
        nonce: query["nonce"].clone(),
        challenge: query["code_challenge"].clone(),
    };
    provider
        .remember_authorization_for_code(
            &attempt.state,
            attempt.nonce.clone(),
            attempt.challenge.clone(),
        )
        .await;
    attempt
}

async fn with_faults() -> (RunningProvider, Router, SessionFaults) {
    let provider = RunningProvider::start(TokenAuthMethod::Basic).await;
    let faults = SessionFaults::default();
    let auth = AuthState::with_session_faults(
        EnabledAuthConfig::loopback_test(&provider.provider.base, "http://127.0.0.1"),
        Arc::new(TestClock {
            time: StdMutex::new(SystemTime::now()),
        }),
        Arc::new(crate::auth::app::SystemMonotonicClock),
        Arc::new(ReqwestAuthHttpClient::new().unwrap()),
        faults.clone(),
    )
    .await
    .unwrap();
    (provider, server_app(Arc::new(auth)), faults)
}

async fn assert_unauthenticated(app: &Router, cookie: &str) {
    assert_eq!(
        request(app, "GET", "/api/info", cookie).await.status(),
        StatusCode::UNAUTHORIZED
    );
}

async fn preloaded_request(
    State(gate): State<Arc<TokenGate>>,
    session: tower_sessions::Session,
    request: Request<Body>,
    next: axum::middleware::Next,
) -> Response {
    let phase = request
        .headers()
        .get("x-pause")
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    if phase.is_some() {
        // Every HTTP request has its own Session::new cache, not a cloned Session.
        session
            .get::<String>(crate::auth::app::PREAUTH_STATE_KEY)
            .await
            .unwrap();
    }
    if phase.as_deref() == Some("before") {
        gate.entered.add_permits(1);
        gate.released.acquire().await.unwrap().forget();
    }
    let mut response = next.run(request).await;
    if phase.as_deref() == Some("after") || phase.as_deref() == Some("empty") {
        gate.entered.add_permits(1);
        gate.released.acquire().await.unwrap().forget();
        if phase.as_deref() == Some("empty") {
            session.clear().await;
        }
        response.headers_mut().append(
            header::SET_COOKIE,
            HeaderValue::from_static("unrelated=kept; Path=/"),
        );
    }
    response
}

async fn preloaded_app(
    provider: &RunningProvider,
    faults: SessionFaults,
) -> (Router, Arc<TokenGate>) {
    preloaded_app_with_clock(
        provider,
        faults,
        Arc::new(TestClock {
            time: StdMutex::new(SystemTime::now()),
        }),
    )
    .await
}

async fn preloaded_app_with_clock(
    provider: &RunningProvider,
    faults: SessionFaults,
    clock: Arc<TestClock>,
) -> (Router, Arc<TokenGate>) {
    let auth = Arc::new(
        AuthState::with_session_faults(
            EnabledAuthConfig::loopback_test(&provider.provider.base, "http://127.0.0.1"),
            clock,
            Arc::new(crate::auth::app::SystemMonotonicClock),
            Arc::new(ReqwestAuthHttpClient::new().unwrap()),
            faults,
        )
        .await
        .unwrap(),
    );
    let gate = Arc::new(TokenGate {
        entered: Semaphore::new(0),
        released: Semaphore::new(0),
    });
    let app = protocol_routes(&auth)
        .layer(axum::middleware::from_fn_with_state(
            Arc::clone(&gate),
            preloaded_request,
        ))
        .layer(auth.session_layer());
    (app, gate)
}

fn paused_login(app: Router, cookie: String, phase: &str) -> tokio::task::JoinHandle<Response> {
    let phase = phase.to_owned();
    tokio::spawn(async move {
        app.oneshot(
            Request::builder()
                .uri("/auth/login?return_to=%2Fdelayed")
                .header(header::COOKIE, cookie)
                .header("x-pause", phase)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap()
    })
}

async fn attempt_from_response(
    provider: &RunningProvider,
    response: &Response,
    cookie: String,
) -> LoginAttempt {
    let url = url::Url::parse(&response_location(response)).unwrap();
    let query: HashMap<_, _> = url.query_pairs().into_owned().collect();
    let attempt = LoginAttempt {
        cookie,
        state: query["state"].clone(),
        nonce: query["nonce"].clone(),
        challenge: query["code_challenge"].clone(),
    };
    provider
        .remember_authorization_for_code(
            &attempt.state,
            attempt.nonce.clone(),
            attempt.challenge.clone(),
        )
        .await;
    attempt
}

#[tokio::test]
async fn independently_preloaded_login_cannot_replace_the_newer_winning_binding() {
    let provider = RunningProvider::start(TokenAuthMethod::Basic).await;
    let (app, gate) = preloaded_app(&provider, SessionFaults::default()).await;
    let original = login_attempt(&app, &provider, "%2Foriginal").await;
    let task = paused_login(app.clone(), original.cookie.clone(), "before");
    bounded(gate.entered()).await;
    let middle = bounded(replacement(&app, &provider, &original.cookie)).await;
    gate.release();
    let stale = bounded(task).await.unwrap();
    assert_eq!(stale.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        response_location(&stale),
        "/login?error=provider_unavailable&return_to=%2Fdelayed"
    );
    assert!(stale.headers().get(header::SET_COOKIE).is_none());
    assert_eq!(
        response_location(&callback_request(&app, &original, "").await),
        "/login?error=authentication_failed&return_to=%2F"
    );
    assert_eq!(
        response_location(&callback_request(&app, &middle, "").await),
        "/new"
    );
    assert_eq!(
        response_location(&callback_request(&app, &middle, "").await),
        "/login?error=authentication_failed&return_to=%2F"
    );
    assert_eq!(provider.provider.token_requests.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn logout_rejects_preloaded_login_but_allows_a_new_cookieless_lineage() {
    let provider = RunningProvider::start(TokenAuthMethod::Basic).await;
    let (app, gate) = preloaded_app(&provider, SessionFaults::default()).await;
    let original = login_attempt(&app, &provider, "%2Foriginal").await;
    let task = paused_login(app.clone(), original.cookie.clone(), "before");
    bounded(gate.entered()).await;
    let logout = bounded(request(&app, "POST", "/auth/logout", &original.cookie)).await;
    assert_eq!(response_location(&logout), "/login?logged_out=1");
    let fresh = bounded(login_attempt(&app, &provider, "%2Ffresh")).await;
    gate.release();
    let stale = bounded(task).await.unwrap();
    assert_eq!(
        response_location(&stale),
        "/login?error=provider_unavailable&return_to=%2Fdelayed"
    );
    assert!(stale.headers().get(header::SET_COOKIE).is_none());
    assert_eq!(
        response_location(&callback_request(&app, &original, "").await),
        "/login?error=authentication_failed&return_to=%2F"
    );
    assert_eq!(provider.provider.token_requests.load(Ordering::SeqCst), 0);
    assert_eq!(
        response_location(&callback_request(&app, &fresh, "").await),
        "/fresh"
    );
    assert_eq!(provider.provider.token_requests.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn pruned_preloaded_owner_cannot_allocate_a_fresh_lineage() {
    let provider = RunningProvider::start(TokenAuthMethod::Basic).await;
    let clock = Arc::new(TestClock {
        time: StdMutex::new(SystemTime::now()),
    });
    let (app, gate) =
        preloaded_app_with_clock(&provider, SessionFaults::default(), clock.clone()).await;
    let original = login_attempt(&app, &provider, "%2Foriginal").await;
    let task = paused_login(app.clone(), original.cookie, "before");
    bounded(gate.entered()).await;
    clock.advance(Duration::from_mins(5));
    gate.release();
    let stale = bounded(task).await.unwrap();
    assert_eq!(
        response_location(&stale),
        "/login?error=provider_unavailable&return_to=%2Fdelayed"
    );
    assert!(stale.headers().get(header::SET_COOKIE).is_none());
    assert_eq!(provider.provider.token_requests.load(Ordering::SeqCst), 0);
    let unknown = request(&app, "GET", "/auth/login", "flowl.sid=tampered").await;
    assert!(
        response_location(&unknown)
            .starts_with(provider.provider.base.as_str().trim_end_matches('/'))
    );
    assert!(response_set_cookie(&unknown).starts_with("flowl.sid="));
}

#[tokio::test]
async fn rotation_alias_cannot_rebind_the_authenticated_or_revoked_lineage() {
    let (provider, app, _) = with_faults().await;
    let original = login_attempt(&app, &provider, "%2Foriginal").await;
    let authenticated = callback_request(&app, &original, "").await;
    let cookie = response_cookie(&authenticated);
    for revoked in [false, true] {
        if revoked {
            request(&app, "POST", "/auth/logout", &cookie).await;
        }
        let stale = request(
            &app,
            "GET",
            "/auth/login?return_to=%2Frequested",
            &original.cookie,
        )
        .await;
        assert_eq!(
            response_location(&stale),
            "/login?error=provider_unavailable&return_to=%2Frequested"
        );
        assert!(
            stale
                .headers()
                .get_all(header::SET_COOKIE)
                .iter()
                .all(|c| c.to_str().unwrap().starts_with("flowl.sid=;"))
        );
        if !revoked {
            assert_eq!(
                request(&app, "GET", "/api/info", &cookie).await.status(),
                StatusCode::OK
            );
        }
    }
    assert_eq!(provider.provider.token_requests.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn preloading_a_missing_rotation_alias_does_not_erase_its_request_identity() {
    let provider = RunningProvider::start(TokenAuthMethod::Basic).await;
    let (app, _) = preloaded_app(&provider, SessionFaults::default()).await;
    let original = login_attempt(&app, &provider, "%2Foriginal").await;
    let authenticated = callback_request(&app, &original, "").await;
    let cookie = response_cookie(&authenticated);
    for (method, path) in [
        ("GET", "/auth/login?return_to=%2Fstale"),
        ("POST", "/auth/logout"),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(path)
                    .header(header::COOKIE, &original.cookie)
                    .header("x-pause", "load")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        if method == "GET" {
            assert_eq!(
                response_location(&response),
                "/login?error=provider_unavailable&return_to=%2Fstale"
            );
            assert!(
                response.headers().get(header::SET_COOKIE).is_none(),
                "an alias response cannot expire the winner's cookie"
            );
            assert_eq!(
                response_location(
                    &request(&app, "GET", "/auth/login?return_to=%2Fstill", &cookie).await
                ),
                "/still"
            );
        } else {
            assert_eq!(response_location(&response), "/login?logged_out=1");
            assert!(response_set_cookie(&response).contains("Max-Age=0"));
            assert_eq!(
                response_location(
                    &request(&app, "GET", "/auth/login?return_to=%2Fstill", &cookie).await
                ),
                "/login?error=provider_unavailable&return_to=%2Fstill"
            );
        }
    }
    assert_eq!(provider.provider.token_requests.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn callback_expiry_during_completion_reload_is_fail_closed_not_a_panic() {
    let provider = RunningProvider::start(TokenAuthMethod::Basic).await;
    let clock = Arc::new(TestClock {
        time: StdMutex::new(SystemTime::now()),
    });
    let faults = SessionFaults::default();
    let auth = AuthState::with_session_faults(
        EnabledAuthConfig::loopback_test(&provider.provider.base, "http://127.0.0.1"),
        clock.clone(),
        Arc::new(crate::auth::app::SystemMonotonicClock),
        Arc::new(ReqwestAuthHttpClient::new().unwrap()),
        faults.clone(),
    )
    .await
    .unwrap();
    let app = server_app(Arc::new(auth));
    let attempt = login_attempt(&app, &provider, "%2Fstored").await;
    let gate = TokenGate::install(&provider).await;
    let task = {
        let app = app.clone();
        let attempt = attempt.clone();
        tokio::spawn(async move { callback_request(&app, &attempt, "").await })
    };
    bounded(gate.entered()).await;
    let reload = faults.pause_next(StoreOperation::Load);
    gate.release();
    bounded(reload.wait_entered()).await;
    clock.advance(Duration::from_mins(5));
    reload.release();
    let failed = bounded(task).await.unwrap();
    assert_eq!(failed.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        response_location(&failed),
        "/login?error=authentication_failed&return_to=%2Fstored"
    );
    assert_unauthenticated(&app, &attempt.cookie).await;
    assert_eq!(provider.provider.token_requests.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn callback_completion_reload_error_uses_provider_unavailable_and_stored_target() {
    let (provider, app, faults) = with_faults().await;
    let attempt = login_attempt(&app, &provider, "%2Fstored").await;
    let gate = TokenGate::install(&provider).await;
    let task = {
        let app = app.clone();
        let attempt = attempt.clone();
        tokio::spawn(async move { callback_request(&app, &attempt, "").await })
    };
    bounded(gate.entered()).await;
    faults.fail_next(StoreOperation::Load, false);
    gate.release();
    let failed = bounded(task).await.unwrap();
    assert_eq!(failed.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        response_location(&failed),
        "/login?error=provider_unavailable&return_to=%2Fstored"
    );
    assert_unauthenticated(&app, &attempt.cookie).await;
    assert_eq!(provider.provider.token_requests.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn late_failed_auto_save_cannot_restore_old_binding_or_overwrite_cookie() {
    for phase in ["after", "empty"] {
        let provider = RunningProvider::start(TokenAuthMethod::Basic).await;
        let faults = SessionFaults::default();
        let (app, gate) = preloaded_app(&provider, faults.clone()).await;
        let original = login_attempt(&app, &provider, "%2Foriginal").await;
        let task = paused_login(app.clone(), original.cookie.clone(), phase);
        bounded(gate.entered()).await;
        let winner = bounded(replacement(&app, &provider, &original.cookie)).await;
        // The older handler already saved explicitly. Its later middleware save
        // must not retry/roll back or turn this intentional 303 into a 500.
        if phase == "after" {
            faults.fail_next(StoreOperation::Save, true);
        }
        gate.release();
        let late = bounded(task).await.unwrap();
        assert_eq!(late.status(), StatusCode::SEE_OTHER);
        let cookies: Vec<_> = late.headers().get_all(header::SET_COOKIE).iter().collect();
        assert_eq!(
            cookies,
            vec![&HeaderValue::from_static("unrelated=kept; Path=/")]
        );
        let obsolete = attempt_from_response(&provider, &late, original.cookie).await;
        assert_eq!(
            response_location(&callback_request(&app, &obsolete, "").await),
            "/login?error=authentication_failed&return_to=%2F"
        );
        assert_eq!(
            response_location(&callback_request(&app, &winner, "").await),
            "/new"
        );
        assert_eq!(provider.provider.token_requests.load(Ordering::SeqCst), 1);
    }
}

#[tokio::test]
async fn failed_replacement_preserves_an_already_exchanging_completion_owner() {
    let (provider, app, faults) = with_faults().await;
    let original = login_attempt(&app, &provider, "%2Foriginal").await;
    let gate = TokenGate::install(&provider).await;
    let task = {
        let app = app.clone();
        let original = original.clone();
        tokio::spawn(async move { callback_request(&app, &original, "").await })
    };
    bounded(gate.entered()).await;
    faults.fail_next(StoreOperation::Save, true);
    let failed = bounded(request(
        &app,
        "GET",
        "/auth/login?return_to=%2Fnew",
        &original.cookie,
    ))
    .await;
    assert_eq!(
        response_location(&failed),
        "/login?error=provider_unavailable&return_to=%2Fnew"
    );
    gate.release();
    let current = bounded(task).await.unwrap();
    assert_eq!(response_location(&current), "/original");
    assert_eq!(
        request(&app, "GET", "/api/info", &response_cookie(&current))
            .await
            .status(),
        StatusCode::OK
    );
    assert_eq!(provider.provider.token_requests.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn replacement_revokes_blocked_completion_without_stalling_other_browsers() {
    let (provider, app, _) = with_faults().await;
    let old = login_attempt(&app, &provider, "%2Fold").await;
    let gate = TokenGate::install(&provider).await;
    let task = {
        let app = app.clone();
        let old = old.clone();
        tokio::spawn(async move { callback_request(&app, &old, "").await })
    };
    bounded(gate.entered()).await;
    let new = bounded(replacement(&app, &provider, &old.cookie)).await;
    let other = bounded(login_attempt(&app, &provider, "%2Fother")).await;
    let authenticated = bounded(callback_request(&app, &other, "")).await;
    let other_cookie = response_cookie(&authenticated);
    assert_eq!(
        bounded(request(&app, "GET", "/api/info", &other_cookie))
            .await
            .status(),
        StatusCode::OK
    );
    assert_eq!(
        bounded(request(&app, "POST", "/auth/logout", &other_cookie))
            .await
            .status(),
        StatusCode::SEE_OTHER
    );
    gate.release();
    let stale = bounded(task).await.unwrap();
    assert_eq!(
        response_location(&stale),
        "/login?error=authentication_failed&return_to=%2Fold"
    );
    assert!(stale.headers().get(header::SET_COOKIE).is_none());
    let current = callback_request(&app, &new, "").await;
    assert_eq!(response_location(&current), "/new");
    assert_eq!(provider.provider.token_requests.load(Ordering::SeqCst), 3);
    assert_unauthenticated(&app, &old.cookie).await;
}

#[tokio::test]
async fn logout_revokes_blocked_exchange_and_preserves_anonymous_cookie_expiry() {
    let (provider, app, _) = with_faults().await;
    let attempt = login_attempt(&app, &provider, "%2Fsettings").await;
    let gate = TokenGate::install(&provider).await;
    let task = {
        let app = app.clone();
        let attempt = attempt.clone();
        tokio::spawn(async move { callback_request(&app, &attempt, "").await })
    };
    bounded(gate.entered()).await;
    for cookie in [&attempt.cookie, ""] {
        let logout = bounded(request(&app, "POST", "/auth/logout", cookie)).await;
        assert_eq!(response_location(&logout), "/login?logged_out=1");
        assert!(response_set_cookie(&logout).contains("Max-Age=0"));
    }
    gate.release();
    let stale = bounded(task).await.unwrap();
    assert_eq!(
        response_location(&stale),
        "/login?error=authentication_failed&return_to=%2Fsettings"
    );
    assert!(stale.headers().get(header::SET_COOKIE).is_none());
    assert_unauthenticated(&app, &attempt.cookie).await;
    assert_eq!(provider.provider.token_requests.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn logout_with_rotation_alias_suppresses_late_middleware_save_and_cookie() {
    let (provider, app, faults) = with_faults().await;
    let attempt = login_attempt(&app, &provider, "%2Fsettings").await;
    // Callback explicitly creates the rotated record; its next save is the
    // outer SessionManager's automatic duplicate, after the lifecycle gate exits.
    let pause = faults.pause_next(StoreOperation::Save);
    faults.fail_next(StoreOperation::Save, true);
    let task = {
        let app = app.clone();
        let attempt = attempt.clone();
        tokio::spawn(async move { callback_request(&app, &attempt, "").await })
    };
    bounded(pause.wait_entered()).await;
    let logout = bounded(request(&app, "POST", "/auth/logout", &attempt.cookie)).await;
    assert_eq!(response_location(&logout), "/login?logged_out=1");
    pause.release();
    let late = bounded(task).await.unwrap();
    assert_eq!(
        late.status(),
        StatusCode::SEE_OTHER,
        "obsolete save is suppression, not middleware 500"
    );
    assert!(late.headers().get(header::SET_COOKIE).is_none());
    assert_unauthenticated(&app, &attempt.cookie).await;
    assert_eq!(
        response_location(&callback_request(&app, &attempt, "").await),
        "/login?error=authentication_failed&return_to=%2F"
    );
}

async fn capture_session(
    State(capture): State<Arc<StdMutex<Option<tower_sessions::Session>>>>,
    session: tower_sessions::Session,
    request: Request<Body>,
    next: axum::middleware::Next,
) -> Response {
    let response = next.run(request).await;
    *capture.lock().unwrap() = Some(session);
    response
}

#[tokio::test]
async fn replacement_exchange_waits_for_the_binding_to_be_persisted() {
    use tower_sessions::Session;
    let provider = RunningProvider::start(TokenAuthMethod::Basic).await;
    let faults = SessionFaults::default();
    let auth = Arc::new(
        AuthState::with_session_faults(
            EnabledAuthConfig::loopback_test(&provider.provider.base, "http://127.0.0.1"),
            Arc::new(crate::auth::SystemClock),
            Arc::new(crate::auth::app::SystemMonotonicClock),
            Arc::new(ReqwestAuthHttpClient::new().unwrap()),
            faults.clone(),
        )
        .await
        .unwrap(),
    );
    let capture = Arc::new(StdMutex::new(None::<Session>));
    let app = protocol_routes(&auth)
        .layer(axum::middleware::from_fn_with_state(
            Arc::clone(&capture),
            capture_session,
        ))
        .layer(auth.session_layer());
    let original = login_attempt(&app, &provider, "%2Foriginal").await;
    let original_id = capture.lock().unwrap().as_ref().unwrap().id().unwrap();
    let pause = faults.pause_next(StoreOperation::Save);
    let replacing = {
        let app = app.clone();
        let cookie = original.cookie.clone();
        tokio::spawn(
            async move { request(&app, "GET", "/auth/login?return_to=%2Fnew", &cookie).await },
        )
    };
    bounded(pause.wait_entered()).await;
    let next = {
        let transactions = auth.transactions.entries.lock().await;
        assert_eq!(transactions.len(), 2);
        let (state, pending) = transactions
            .iter()
            .find(|(state, _)| **state != original.state)
            .unwrap();
        LoginAttempt {
            cookie: original.cookie.clone(),
            state: state.clone(),
            nonce: pending.nonce.secret().clone(),
            challenge: openidconnect::PkceCodeChallenge::from_code_verifier_sha256(
                &pending.pkce_verifier,
            )
            .as_str()
            .to_owned(),
        }
    };
    provider
        .remember_authorization_for_code(&next.state, next.nonce.clone(), next.challenge.clone())
        .await;
    let snapshot = Session::new(
        Some(original_id),
        Arc::new(auth.sessions.store.clone()),
        None,
    );
    assert_eq!(
        snapshot
            .get::<String>(crate::auth::app::PREAUTH_STATE_KEY)
            .await
            .unwrap(),
        Some(original.state.clone())
    );
    let (entered, entry) = tokio::sync::oneshot::channel();
    let completing = {
        let app = app.clone();
        tokio::spawn(async move {
            entered.send(()).unwrap();
            callback_request(&app, &next, "").await
        })
    };
    entry.await.unwrap();
    tokio::task::yield_now().await;
    assert!(
        !completing.is_finished(),
        "exchange waits for durable binding"
    );
    assert_eq!(provider.provider.token_requests.load(Ordering::SeqCst), 0);
    let other = bounded(login_attempt(&app, &provider, "%2Fother")).await;
    assert_ne!(other.state, original.state);
    pause.release();
    let replaced = bounded(replacing).await.unwrap();
    assert!(response_location(&replaced).starts_with(&provider.provider.base));
    assert_eq!(
        response_location(&bounded(completing).await.unwrap()),
        "/new"
    );
    assert_eq!(provider.provider.token_requests.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn failed_replacement_restores_prior_binding_and_exchange_permission() {
    for after_write in [false, true] {
        let (provider, app, faults) = with_faults().await;
        let prior = login_attempt(&app, &provider, "%2Fprior").await;
        faults.fail_next(StoreOperation::Save, after_write);
        let failed = request(
            &app,
            "GET",
            "/auth/login?return_to=%2Frequested",
            &prior.cookie,
        )
        .await;
        assert_eq!(
            response_location(&failed),
            "/login?error=provider_unavailable&return_to=%2Frequested"
        );
        assert_eq!(
            response_location(&callback_request(&app, &prior, "").await),
            "/prior"
        );
        assert_eq!(provider.provider.token_requests.load(Ordering::SeqCst), 1);
    }
}

#[tokio::test]
async fn failed_authenticated_create_and_rotation_cannot_be_retried_by_middleware() {
    for operation in [StoreOperation::Create, StoreOperation::Delete] {
        for after_write in [false, true] {
            let (provider, app, faults) = with_faults().await;
            let attempt = login_attempt(&app, &provider, "%2Fstored").await;
            faults.fail_next(operation, after_write);
            let failed = callback_request(&app, &attempt, "").await;
            assert_eq!(
                response_location(&failed),
                "/login?error=provider_unavailable&return_to=%2Fstored"
            );
            assert_unauthenticated(&app, &attempt.cookie).await;
            if let Some(cookie) = failed.headers().get(header::SET_COOKIE) {
                let cookie = cookie.to_str().unwrap().split(';').next().unwrap();
                assert_unauthenticated(&app, cookie).await;
                assert!(
                    cookie.ends_with('='),
                    "no provisional auth cookie may escape"
                );
            }
            assert_eq!(provider.provider.token_requests.load(Ordering::SeqCst), 1);
        }
    }
}

#[tokio::test]
async fn initial_binding_and_load_failures_do_not_leave_usable_orphans() {
    for after_write in [false, true] {
        let (provider, app, faults) = with_faults().await;
        faults.fail_next(StoreOperation::Create, after_write);
        let failed = request(&app, "GET", "/auth/login?return_to=%2Frequested", "").await;
        assert_eq!(
            response_location(&failed),
            "/login?error=provider_unavailable&return_to=%2Frequested"
        );
        assert!(failed.headers().get(header::SET_COOKIE).is_none());
        let attempt = login_attempt(&app, &provider, "%2Fprior").await;
        faults.fail_next(StoreOperation::Load, false);
        let failed = request(
            &app,
            "GET",
            "/auth/login?return_to=%2Frequested",
            &attempt.cookie,
        )
        .await;
        assert_eq!(
            response_location(&failed),
            "/login?error=provider_unavailable&return_to=%2Frequested"
        );
        assert_eq!(
            response_location(&callback_request(&app, &attempt, "").await),
            "/prior"
        );
    }
}

#[tokio::test]
async fn logout_delete_failures_return_500_and_expire_cookie_without_resurrection() {
    for after_write in [false, true] {
        let (provider, app, faults) = with_faults().await;
        let attempt = login_attempt(&app, &provider, "%2Fsettings").await;
        let authenticated = callback_request(&app, &attempt, "").await;
        let cookie = response_cookie(&authenticated);
        faults.fail_next(StoreOperation::Delete, after_write);
        let failed = request(&app, "POST", "/auth/logout", &cookie).await;
        assert_eq!(failed.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert!(failed.headers().get(header::LOCATION).is_none());
        assert_matching_removal_cookie(&failed, response_set_cookie(&authenticated));
        assert_eq!(failed.headers()[header::CACHE_CONTROL], "no-store");
        assert_unauthenticated(&app, &cookie).await;
        assert_unauthenticated(&app, &attempt.cookie).await;
    }
}

#[tokio::test]
async fn failed_logout_revokes_pending_and_inflight_completion_ownership() {
    for after_write in [false, true] {
        for exchanging in [false, true] {
            let (provider, app, faults) = with_faults().await;
            let attempt = login_attempt(&app, &provider, "%2Fstored").await;
            let exchange = if exchanging {
                let gate = TokenGate::install(&provider).await;
                let task = {
                    let app = app.clone();
                    let attempt = attempt.clone();
                    tokio::spawn(async move { callback_request(&app, &attempt, "").await })
                };
                bounded(gate.entered()).await;
                Some((gate, task))
            } else {
                None
            };
            faults.fail_next(StoreOperation::Delete, after_write);
            let failed = bounded(request(&app, "POST", "/auth/logout", &attempt.cookie)).await;
            assert_eq!(failed.status(), StatusCode::INTERNAL_SERVER_ERROR);
            assert!(failed.headers().get(header::LOCATION).is_none());
            assert_matching_removal_cookie(&failed, &attempt.cookie);
            let callback = if let Some((gate, task)) = exchange {
                gate.release();
                bounded(task).await.unwrap()
            } else {
                callback_request(&app, &attempt, "").await
            };
            let target = if exchanging { "%2Fstored" } else { "%2F" };
            assert_eq!(
                response_location(&callback),
                format!("/login?error=authentication_failed&return_to={target}")
            );
            assert_unauthenticated(&app, &attempt.cookie).await;
            let repeated = request(&app, "POST", "/auth/logout", &attempt.cookie).await;
            assert_eq!(response_location(&repeated), "/login?logged_out=1");
            assert_eq!(
                provider.provider.token_requests.load(Ordering::SeqCst),
                usize::from(exchanging)
            );
        }
    }
}

fn assert_matching_removal_cookie(response: &Response, issued: &str) {
    use tower_sessions::cookie::{Cookie, SameSite};
    let issued = Cookie::parse(issued.to_owned()).unwrap();
    let cookies: Vec<_> = response
        .headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .filter_map(|value| {
            let cookie = Cookie::parse(value.to_str().unwrap().to_owned()).unwrap();
            (cookie.name() == crate::auth::SESSION_COOKIE_NAME).then_some(cookie)
        })
        .collect();
    assert_ne!(cookies.len(), 0);
    for cookie in &cookies {
        assert_eq!(cookie.name(), issued.name());
        assert_eq!(cookie.value(), "");
        assert_eq!(cookie.domain(), None);
        assert_eq!(cookie.path(), Some("/"));
        assert_eq!(cookie.max_age(), Some(time::Duration::ZERO));
    }
    // tower-sessions can append its own bare removal. The explicit logout
    // expiry must also survive, with the same attributes as the issued cookie.
    let explicit = cookies
        .iter()
        .find(|cookie| cookie.secure() == Some(true))
        .expect("matching explicit logout expiry");
    assert_eq!(explicit.http_only(), Some(true));
    assert_eq!(explicit.same_site(), Some(SameSite::Lax));
    assert_eq!(
        explicit.expires_datetime(),
        Some(time::OffsetDateTime::UNIX_EPOCH)
    );
}

#[derive(Clone, Copy)]
enum PostHandlerFailureMode {
    SaveBeforeWrite,
    SaveAfterWrite,
    Callback500PreserveExtensions,
    Callback500ReplaceResponse,
}

#[derive(Clone)]
struct PostHandlerFailure {
    faults: SessionFaults,
    mode: PostHandlerFailureMode,
    provisional: Arc<StdMutex<Option<tower_sessions::Session>>>,
}

async fn mutate_after_callback(
    State(failure): State<PostHandlerFailure>,
    session: tower_sessions::Session,
    request: Request<Body>,
    next: axum::middleware::Next,
) -> Response {
    let callback = request.uri().path() == "/auth/callback";
    let mut response = next.run(request).await;
    if callback
        && session
            .get::<serde_json::Value>(crate::auth::session::AUTHENTICATED_SESSION_KEY)
            .await
            .unwrap()
            .is_some()
    {
        *failure.provisional.lock().unwrap() = Some(session.clone());
        match failure.mode {
            PostHandlerFailureMode::SaveBeforeWrite | PostHandlerFailureMode::SaveAfterWrite => {
                // A REAL nonduplicate write, not the manager's idempotent auto-save.
                session.insert("test_post_handler", true).await.unwrap();
                failure.faults.fail_next(
                    StoreOperation::Save,
                    matches!(failure.mode, PostHandlerFailureMode::SaveAfterWrite),
                );
            }
            PostHandlerFailureMode::Callback500PreserveExtensions => {
                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                response.headers_mut().remove(header::LOCATION);
            }
            PostHandlerFailureMode::Callback500ReplaceResponse => {
                // Drop CallbackIntent and all other handler response extensions.
                response = StatusCode::INTERNAL_SERVER_ERROR.into_response();
                assert!(response.extensions().is_empty());
            }
        }
        response.headers_mut().append(
            header::SET_COOKIE,
            HeaderValue::from_static("unrelated=kept; Path=/"),
        );
    }
    response
}

#[tokio::test]
async fn middleware_failure_preserves_callback_target_and_revokes_provisional_id() {
    for mode in [
        PostHandlerFailureMode::SaveBeforeWrite,
        PostHandlerFailureMode::SaveAfterWrite,
    ] {
        assert_middleware_failure(mode).await;
    }
}

#[tokio::test]
async fn callback_500_with_preserved_extensions_revokes_provisional_id() {
    assert_middleware_failure(PostHandlerFailureMode::Callback500PreserveExtensions).await;
}

#[tokio::test]
async fn callback_500_replacing_all_extensions_preserves_target_and_revokes_provisional_id() {
    assert_middleware_failure(PostHandlerFailureMode::Callback500ReplaceResponse).await;
}

async fn assert_middleware_failure(mode: PostHandlerFailureMode) {
    // RSA fixture setup is CPU-bound and may contend with other provider tests.
    // Keep it outside the deadline for actual lifecycle work.
    let provider = RunningProvider::start(TokenAuthMethod::Basic).await;
    bounded(assert_middleware_failure_with_provider(mode, provider)).await;
}

async fn assert_middleware_failure_with_provider(
    mode: PostHandlerFailureMode,
    provider: RunningProvider,
) {
    let failure = PostHandlerFailure {
        faults: SessionFaults::default(),
        mode,
        provisional: Arc::default(),
    };
    let auth = Arc::new(
        AuthState::with_session_faults(
            EnabledAuthConfig::loopback_test(&provider.provider.base, "http://127.0.0.1"),
            Arc::new(TestClock {
                time: StdMutex::new(SystemTime::now()),
            }),
            Arc::new(crate::auth::app::SystemMonotonicClock),
            Arc::new(ReqwestAuthHttpClient::new().unwrap()),
            failure.faults.clone(),
        )
        .await
        .unwrap(),
    );
    let protected = server_app(Arc::clone(&auth));
    let app = protocol_routes(&auth)
        .layer(axum::middleware::from_fn_with_state(
            failure.clone(),
            mutate_after_callback,
        ))
        .layer(auth.session_layer());
    let attempt = login_attempt(&app, &provider, "%2Fsettings%3Ftab%3Ddata%23export").await;
    let failed = callback_request(&app, &attempt, "").await;
    assert_eq!(failed.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        response_location(&failed),
        "/login?error=provider_unavailable&return_to=%2Fsettings%3Ftab%3Ddata%23export"
    );
    let cookies: Vec<_> = failed
        .headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .map(|c| c.to_str().unwrap())
        .collect();
    assert!(cookies.contains(&"unrelated=kept; Path=/"));
    assert!(
        cookies
            .iter()
            .any(|c| c.starts_with("flowl.sid=;") && c.contains("Max-Age=0"))
    );
    assert!(
        cookies
            .iter()
            .filter(|c| c.starts_with("flowl.sid="))
            .all(|c| { c.starts_with("flowl.sid=;") && c.contains("Max-Age=0") }),
        "no provisional auth cookie may escape"
    );
    assert_unauthenticated(&protected, &attempt.cookie).await;
    let provisional = failure.provisional.lock().unwrap().take().unwrap();
    let id = provisional.id().unwrap();
    provisional.load().await.unwrap();
    assert!(
        provisional
            .get::<serde_json::Value>(crate::auth::session::AUTHENTICATED_SESSION_KEY)
            .await
            .unwrap()
            .is_none()
    );
    assert_ne!(
        provisional.id(),
        Some(id),
        "the proposed identifier has no loadable record"
    );
    assert_eq!(provider.provider.token_requests.load(Ordering::SeqCst), 1);
}

async fn replace_logout_response(request: Request<Body>, next: axum::middleware::Next) -> Response {
    let logout = request.uri().path() == "/auth/logout";
    let response = next.run(request).await;
    if !logout {
        return response;
    }
    // Discard both LogoutExpiry and the handler's explicit removal cookie.
    let mut response = StatusCode::INTERNAL_SERVER_ERROR.into_response();
    assert!(response.extensions().is_empty());
    response.headers_mut().append(
        header::SET_COOKIE,
        HeaderValue::from_static("unrelated=kept; Path=/"),
    );
    response
}

#[tokio::test]
async fn logout_500_response_replacement_still_expires_cookie_and_revokes_authentication() {
    let provider = RunningProvider::start(TokenAuthMethod::Basic).await;
    bounded(async {
        let auth = Arc::new(
            AuthState::with_session_faults(
                EnabledAuthConfig::loopback_test(&provider.provider.base, "http://127.0.0.1"),
                Arc::new(TestClock {
                    time: StdMutex::new(SystemTime::now()),
                }),
                Arc::new(crate::auth::app::SystemMonotonicClock),
                Arc::new(ReqwestAuthHttpClient::new().unwrap()),
                SessionFaults::default(),
            )
            .await
            .unwrap(),
        );
        let protected = server_app(Arc::clone(&auth));
        let app = protocol_routes(&auth)
            .layer(axum::middleware::from_fn(replace_logout_response))
            .layer(auth.session_layer());
        let attempt = login_attempt(&app, &provider, "%2Fsettings").await;
        let authenticated = callback_request(&app, &attempt, "").await;
        assert_eq!(authenticated.status(), StatusCode::SEE_OTHER);
        let cookie = response_cookie(&authenticated);
        assert_eq!(
            request(&protected, "GET", "/api/info", &cookie)
                .await
                .status(),
            StatusCode::OK
        );

        let failed = request(&app, "POST", "/auth/logout", &cookie).await;
        assert_eq!(failed.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert!(failed.headers().get(header::LOCATION).is_none());
        let cookies: Vec<_> = failed
            .headers()
            .get_all(header::SET_COOKIE)
            .iter()
            .map(|c| c.to_str().unwrap())
            .collect();
        assert!(cookies.contains(&"unrelated=kept; Path=/"));
        assert!(
            cookies
                .iter()
                .any(|c| { c.starts_with("flowl.sid=;") && c.contains("Max-Age=0") })
        );
        assert_unauthenticated(&protected, &cookie).await;
        assert_unauthenticated(&protected, &attempt.cookie).await;
        assert_eq!(provider.provider.token_requests.load(Ordering::SeqCst), 1);
    })
    .await;
}

#[tokio::test]
async fn middleware_duplicate_save_is_not_a_second_backing_write() {
    let (provider, app, faults) = with_faults().await;
    let attempt = login_attempt(&app, &provider, "%2Fsettings").await;
    faults.fail_next(StoreOperation::Save, true);
    let success = callback_request(&app, &attempt, "").await;
    assert_eq!(success.status(), StatusCode::SEE_OTHER);
    assert_eq!(response_location(&success), "/settings");
    let cookie = response_cookie(&success);
    assert_eq!(
        request(&app, "GET", "/api/info", &cookie).await.status(),
        StatusCode::OK
    );
    assert_unauthenticated(&app, &attempt.cookie).await;
}

#[tokio::test]
async fn successful_session_retains_only_local_authentication_timestamps() {
    use crate::auth::session::AUTHENTICATED_SESSION_KEY;
    use crate::auth::session::OWNER_KEY;
    use tower_sessions::{Session, SessionStore};

    let provider = RunningProvider::start(TokenAuthMethod::Basic).await;
    let now = SystemTime::now();
    let clock = Arc::new(TestClock {
        time: StdMutex::new(now),
    });
    let auth = Arc::new(
        AuthState::with_session_faults(
            EnabledAuthConfig::loopback_test(&provider.provider.base, "http://127.0.0.1"),
            clock,
            Arc::new(crate::auth::app::SystemMonotonicClock),
            Arc::new(ReqwestAuthHttpClient::new().unwrap()),
            SessionFaults::default(),
        )
        .await
        .unwrap(),
    );
    let captured = Arc::new(StdMutex::new(None::<Session>));
    let capture = Arc::clone(&captured);
    let app = protocol_routes(&auth)
        .layer(axum::middleware::from_fn(
            move |session: Session, request: Request<Body>, next: axum::middleware::Next| {
                let capture = Arc::clone(&capture);
                async move {
                    if request.uri().path() == "/auth/callback" {
                        for (key, value) in [
                            ("oidc.nonce", "seeded-nonce"),
                            ("oidc.code", "seeded-code"),
                            ("oidc.token", "seeded-token"),
                            ("oidc.claims", "seeded-claims"),
                            ("oidc.secret", "seeded-secret"),
                        ] {
                            session.insert(key, value).await.unwrap();
                        }
                        session.save().await.unwrap();
                    }
                    let response = next.run(request).await;
                    *capture.lock().unwrap() = Some(session);
                    response
                }
            },
        ))
        .layer(auth.session_layer());
    let attempt = login_attempt(&app, &provider, "%2Fplants").await;
    let old_id = captured.lock().unwrap().as_ref().unwrap().id().unwrap();
    let response = callback_request(&app, &attempt, "").await;
    assert_eq!(response_location(&response), "/plants");
    let session = captured.lock().unwrap().take().unwrap();
    let id = session.id().unwrap();
    assert_ne!(id, old_id);
    let record = auth.sessions.store.load(&id).await.unwrap().unwrap();
    assert!(auth.sessions.store.current(&session).await);
    assert!(auth.sessions.store.load(&old_id).await.unwrap().is_none());
    assert_eq!(record.data.len(), 2);
    assert!(record.data.contains_key(OWNER_KEY));
    let timestamp = time::OffsetDateTime::from(now).unix_timestamp();
    assert_eq!(
        record.data[AUTHENTICATED_SESSION_KEY],
        serde_json::json!({
            "authenticated_at": timestamp,
            "expires_at": timestamp + 43_200,
        })
    );
    assert_eq!(
        record.expiry_date,
        time::OffsetDateTime::from(now + crate::auth::app::SESSION_TTL)
    );
    let data = serde_json::to_string(&record.data).unwrap();
    for secret in [
        attempt.state.as_str(),
        attempt.nonce.as_str(),
        "seeded-nonce",
        "seeded-code",
        "seeded-token",
        "seeded-claims",
        "seeded-secret",
        "local-access-token",
        "local-provider-user",
        CLIENT_SECRET,
    ] {
        assert!(!data.contains(secret), "session retained {secret}");
    }
}

#[derive(Clone)]
struct LoginAttempt {
    cookie: String,
    state: String,
    nonce: String,
    challenge: String,
}

fn protocol_routes(auth: &Arc<AuthState>) -> Router {
    auth.protocol_routes()
}

fn server_app(auth: Arc<AuthState>) -> Router {
    // These requests do not serve uploads; the ordinary Flowl server and owned
    // session layer are still used for protected-route assertions.
    let (state, _directory) = test_state(auth);
    server::router(state)
}

fn response_location(response: &Response) -> String {
    response.headers()[header::LOCATION]
        .to_str()
        .unwrap()
        .to_owned()
}

fn response_set_cookie(response: &Response) -> &str {
    response
        .headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .find(|value| value.to_str().unwrap().starts_with("flowl.sid="))
        .unwrap()
        .to_str()
        .unwrap()
}

fn response_cookie(response: &Response) -> String {
    response_set_cookie(response)
        .split(';')
        .next()
        .unwrap()
        .to_owned()
}

async fn login_attempt(app: &Router, provider: &RunningProvider, return_to: &str) -> LoginAttempt {
    let response = request(
        app,
        "GET",
        &format!("/auth/login?return_to={return_to}"),
        "",
    )
    .await;
    attempt_from_response(provider, &response, response_cookie(&response)).await
}

async fn callback_request(app: &Router, attempt: &LoginAttempt, suffix: &str) -> Response {
    request(
        app,
        "GET",
        &format!(
            "/auth/callback?state={}&code={}{}",
            attempt.state, attempt.state, suffix
        ),
        &attempt.cookie,
    )
    .await
}
