## Context

See `proposal.md` for motivation. Today `Config::load_from` is non-fallible and silently defaults invalid parsed values; `main.rs` builds one `AppState`; `server::router` publicly mounts `/health`, `/api/info`, `/api/*`, `/uploads/*`, then an embedded-SPA fallback. There is no request identity boundary.

The static SvelteKit UI uses same-origin cookies naturally, but HTTP access is split between `ui/src/lib/api.ts`, export/AI streaming helpers, route-load `fetch`, and a protected-photo fetch in `IdentifyPanel`. The service worker stores selected API responses and thumbnails by URL, currently including any fulfilled HTTP status, and uses cached navigation before network. Cache keys do not vary by cookie. The root layout also fetches protected settings and starts shell/PWA behavior for every frontend route.

Authentication therefore crosses startup, routing, token verification, process memory, frontend navigation, and offline privacy. It must preserve auth-disabled behavior and useful offline reads while making explicit logout remove locally cached protected content.

## Goals / Non-Goals

**Goals:**

- Make security-relevant configuration and state explicit typed boundaries that fail closed only when auth is enabled.
- Delegate OIDC parsing, discovery, PKCE generation, signature/claim validation, token-hash computation, cookies, and session primitives to maintained crates.
- Keep provider tokens and identity data ephemeral and backend-only.
- Make API, upload, document-navigation, and offline-cache behavior distinguish authentication expiry from network failure.
- Keep protocol, session, JWKS cooldown, and cache lifecycle logic deterministic under automated tests.

**Non-Goals:**

- Persistent or distributed sessions, login transactions, provider tokens, or identity records.
- Local accounts, passwords, registration, account linking, role/group/claim policy, userinfo, per-user data, or provider logout.
- Bearer tokens in the frontend, provider-specific parameters, dynamic client registration, device flow, implicit/hybrid flow, or an OAuth completion HTML workaround.
- Base-path hosting; Flowl's existing routes remain rooted at `/`.
- Changes to pull-to-refresh behavior.

## Decisions

### 1. Use the established fixed OIDC and session libraries

The OIDC library is fixed to `openidconnect` 4.0.1 and the session library is fixed to `tower-sessions` 0.15, with the established feature sets:

```toml
openidconnect = { version = "4.0.1", default-features = false, features = ["reqwest", "rustls-tls", "timing-resistant-secret-traits"] }
tower-sessions = { version = "0.15", default-features = false, features = ["axum-core", "memory-store", "private"] }
```

Do not research or substitute alternative OIDC, auth, or session frameworks. Small supporting utility dependencies may be added only when the chosen implementation requires them, such as OS randomness, time/URL handling, or local-provider test support; they must not replace protocol/session behavior owned by the fixed libraries.

`openidconnect` supplies typed issuer URLs, discovery, authorization requests, PKCE S256, nonce/state types, confidential token exchange, ID-token verification, JWK types, and timing-resistant secret handling. `tower-sessions` supplies Axum integration, private cookies, an in-process memory store, deletion, and session-ID cycling. No SQL auth tables or external store are added.

The OIDC crate validates standard ID-token claims and signatures. Flowl orchestrates one-time transaction consumption and uses the crate's access-token-hash helper to compare a supplied applicable `at_hash`; it does not implement digest selection or comparison itself.

### 2. Split ordinary and security-strict configuration parsing

Keep existing lenient parsing for unrelated variables to avoid a drive-by behavior change. Introduce a fallible auth configuration parser:

- `FLOWL_AUTH_ENABLED`: absent means `false`; only valid boolean text is accepted when present.
- Disabled mode does not require, validate, or contact an OIDC provider.
- Enabled mode requires non-empty external URL, issuer, client ID, and client secret; provider name defaults to `OpenID Connect` and is bounded for safe UI use.
- `FLOWL_EXTERNAL_URL` must be an HTTPS origin with no userinfo, query, fragment, or non-root path. The callback is formed by joining the fixed `/auth/callback` path. Forwarded and request host headers are never consulted.
- `FLOWL_OIDC_ISSUER` is retained as its exact configured `String`. Validation constructs `IssuerUrl::new(raw.clone())` directly; code must never parse to `url::Url` and later serialize into `IssuerUrl`. `openidconnect` 4.0.1's URL wrapper stores both the parsed URL and original string and compares/serializes the original, but Flowl additionally compares signature-verified discovery/token issuer text to the retained raw string as defense in depth. Additional checks require HTTPS and no query or fragment while allowing an issuer path.

Configuration errors return safe typed startup diagnostics. Authentication initialization happens before binding the listener. A test-only constructor may use a loopback HTTP issuer for a local mock provider; no production environment path bypasses HTTPS validation.

**Alternative considered:** inferring enabled mode from partially present OIDC variables was rejected because typoed or partial configuration could silently change the security boundary.

### 3. Discover once at startup and select token endpoint auth from metadata

Build a redirect-disabled Rustls HTTP client and call `CoreProviderMetadata::discover_async` with the exact `IssuerUrl`. In `openidconnect` 4.0.1 this call explicitly retrieves both metadata and the associated initial JWKS before returning and compares issuer URL wrappers by their retained original strings. Cache endpoints, permitted ID-token algorithms, `jwks_uri`, and the initial key set in immutable/shared auth state. Before listening, additionally require HTTPS authorization, token, and JWKS endpoints, Authorization Code response support, and a non-empty permitted ID-token signing-algorithm set; reject redirects, unavailable/malformed initial JWKS, and incompatible metadata.

Read `token_endpoint_auth_methods_supported` before constructing the client:

1. Omitted metadata selects `AuthType::BasicAuth`, the OIDC default.
2. Advertised Basic selects `BasicAuth`, including when Post is also advertised.
3. If Basic is absent and Post is advertised, select `AuthType::RequestBody`.
4. If neither supported confidential method is available, startup fails.

Discovery, token, and JWKS clients refuse redirects. Startup diagnostics identify safe categories such as transport, HTTP status, parsing, missing metadata, or issuer mismatch, but wrappers must not print response bodies. The client secret and request bodies are never included in debug output.

**Alternative considered:** retrying discovery after startup was rejected because the requested model says discovery happens at startup and enabled startup fails closed. Runtime recovery is reserved for JWKS rotation after a valid initial configuration.

### 4. Bind one-time login transactions to a private pre-auth session

`GET /auth/login` validates/bounds `return_to`, creates a private pre-auth `tower-sessions` session, and generates authorization URL values through `openidconnect`: random CSRF state, nonce, and `PkceCodeChallenge::new_random_sha256` plus verifier. Reuse the established transaction model: a process-local locked map keyed by state stores nonce, PKCE verifier, validated target, and absolute five-minute expiry, while the private tower session stores only that state binding. Starting a fresh login removes that session's prior pending transaction. Registry operations prune expired entries.

The callback compares returned state with the initiating session's stored state, atomically removes the matching registry entry while locked, and releases the lock before provider I/O. This makes success, provider error, validation failure, and concurrent replay one-time outcomes. Missing, expired, mismatched, or consumed transactions never reach the token endpoint.

Reject duplicate `code`/`state`, mixed success/error parameters, and other ambiguous callback pollution before token use. After token exchange, require an ID token, pass the original PKCE verifier, and verify it with the stored nonce. Keep the library's strict additional-audience rejection and explicitly reject invalid `azp` when applicable; require the standard issued-at claim. Verify any supplied applicable `at_hash` with `openidconnect`'s hash helper against the returned access token. Discard authorization code, tokens, claims, nonce, state, verifier, and transaction immediately after completion.

On success call the session cycle/rotation primitive, remove all pre-auth fields, and write only an authenticated marker plus immutable authentication and expiry timestamps. The callback then returns 303 to the transaction's validated target. Invalid protocol/token results return 303 to a generic authentication-failed login state; token endpoint/JWKS transport availability failures return the generic provider-unavailable state. Neither URL contains diagnostics.

**Alternatives considered:** state in a browser-readable cookie was rejected because transaction secrets must be backend-only; a state map unbound to the browser was rejected because it permits login transaction swapping; an intermediate callback HTML page was rejected because it is unnecessary.

### 5. Enforce a local absolute session lifetime in addition to cookie expiry

Use `tower_sessions::MemoryStore` only. Configure the cookie name as a Flowl-specific host-only cookie with Path `/`, `HttpOnly`, `Secure`, and `SameSite=Lax`; private-cookie support prevents readable/tamperable session identifiers and Lax permits the top-level provider callback. Generate the private-cookie key at process startup, matching the intentionally process-local lifecycle.

Every authenticated-session check compares an immutable `expires_at = authenticated_at + 12 hours` against an injectable monotonic/wall clock boundary. Requests never move that timestamp. Cookie max-age is at most twelve hours, but server-side timestamp enforcement is authoritative even if middleware cookie behavior changes. Expired sessions are deleted and their cookie is cleared. Process restart loses both memory-store data and the private-cookie key.

Use `Session::cycle_id()` (or the crate's equivalent rotation API) only after all callback checks succeed. Tests assert the pre-auth cookie/session ID cannot authenticate and differs from the post-auth session.

**Alternative considered:** inactivity expiry was rejected because it violates the required absolute lifetime.

### 6. Compose explicit public, API, upload, and document auth boundaries

Build auth-disabled and auth-enabled router composition from the same application routers:

- Always public: `/health`.
- Public when enabled: `/login`, `/auth/config`, `/auth/login`, `/auth/callback`, POST-only `/auth/logout`, `/service-worker.js`, `/manifest.json`, PWA icons/favicon, `/offline.html`, and exact immutable embedded non-document JS/CSS/assets needed by login/PWA operation.
- Protected API: `/api/info` and the complete nested `/api` router. Its guard returns `ApiError::Unauthorized("AUTHENTICATION_REQUIRED")`, preserving JSON and never redirecting.
- Protected uploads: the complete `/uploads` `ServeDir`; its guard returns 401 without file bytes or redirects.
- Protected documents: root, `index.html`, known normal SPA routes, and unknown fallback documents. An unauthenticated GET receives a safe local `/login?return_to=...` redirect.

Exact static assets may be public because they contain application code but no plant/user data; serving them does not grant API, upload, or document access. `index.html` is not treated as a generally public exact asset: `/login` is the single public document entry. Explicit `/auth/*` routes are registered ahead of fallback so unknown auth paths never become SPA HTML. With auth disabled, use the existing router behavior and do not create session work on requests.

`GET /auth/config` is always public so the compiled UI can hide/show auth UX. It returns only `enabled` and `provider_name`. Every `/auth/*` response and every `AUTHENTICATION_REQUIRED` response receives `Cache-Control: no-store`; the service worker independently excludes them as a second layer.

**Alternative considered:** one heuristic middleware that redirects every unauthenticated request was rejected because API and image fetches would follow HTML/provider redirects and corrupt typed callers.

### 7. Use one strict, bounded local-return parser on both sides

Define backend and frontend utilities with shared test vectors. A target is accepted only when it is at most 2048 bytes, starts with exactly one `/`, has no backslash/control character, parses as a same-origin relative path, and cannot normalize or percent-decode into `/login`, `/auth`, or their descendants. Reject absolute URLs, protocol-relative URLs, userinfo/authority forms, malformed percent escapes, encoded separators/path confusion, and unsafe normalization. Fallback is `/`.

Backend guards preserve request path and query; URL fragments never reach the backend. The frontend expiry handler constructs from `location.pathname + location.search + location.hash`, validates it, then URL-encodes it as the login query value. The login page validates again before linking to `/auth/login`. Callback redirects only to the already validated transaction value, never directly to callback query input.

**Alternative considered:** parsing against the request Host was rejected because host/forwarded headers are untrusted and because an explicit external URL already exists.

### 8. Reuse the cached-CoreClient JWKS refresh model

Store the normal configured `CoreClient` in a read/write-locked cache state with a generation counter and optional process-monotonic `refresh_retry_at`. The runtime also retains the validated provider metadata, client ID/secret, callback URL, selected Basic/Post auth method, and guarded HTTP client needed to rebuild that same client. Authorization URL creation, code exchange, and ID-token verification all clone and use the cached `CoreClient`; do not introduce a separate keyset cache or custom verifier architecture.

Only `NoMatchingKey` and signature crypto failures that rotated keys could resolve enter refresh logic. Issuer, audience, expiry, nonce, algorithm-policy, `at_hash`, parsing, and other claim failures never refresh.

A separate Tokio mutex serializes targeted refresh:

1. Record the cached-client generation used by failed verification.
2. Acquire the refresh mutex and re-read cache state. If another callback advanced the generation, reuse its cached client and retry verification once without fetching.
3. If the same generation is still inside its failed-refresh cooldown, return provider unavailable without fetching.
4. Otherwise fetch only the already validated discovery `jwks_uri`, validate usable signing keys, clone the retained provider metadata with the refreshed JWKS, and rebuild `CoreClient::from_provider_metadata`, reapplying the selected auth type and callback URL.
5. On success atomically replace the cached client, increment generation, clear cooldown, and retry verification exactly once.
6. On failure or unusable keys retain the last-known-good cached client/generation and set a 30-second process-monotonic cooldown. A later callback after cooldown may retry without restart.

This is the established cached-client model and needs no Flowl-specific deviation. Tests control the clock and provider request count.

### 9. Centralize frontend response classification without confusing offline state

Refactor `ui/src/lib/api.ts` around a shared response/error function that can use either `window.fetch` or SvelteKit's injected `fetch`. It parses a non-success JSON error once and checks the exact pair `status === 401 && code === "AUTHENTICATION_REQUIRED"`. On that pair it validates the current local path/query/hash and assigns the login URL. All other statuses continue through `ApiError`; transport rejection alone invokes `recheckHealth`.

Route loaders call the same helper with their injected fetch. Export, identify, chat, and protected-photo-to-blob flows share response classification before reading blobs/streams. Ordinary multipart upload methods already pass through the central request helper. Browser-managed `<img>` requests remain cookie-authenticated; API/page bootstrap produces the controlled redirect if a session has expired, while image 401s are never converted into login HTML.

Guard against repeated navigation and never redirect from `/login` or `/auth/*`; unsafe recursion falls back to `/`. No auth state is persisted in JavaScript.

**Alternative considered:** globally monkey-patching `window.fetch` was rejected because it is difficult to test, does not cover injected SvelteKit fetch cleanly, and can affect service-worker/network health calls.

### 10. Render login outside the application shell and leave pull-to-refresh unchanged

The root layout branches on the route: `/login` renders only public children with locally cached theme/locale initialization. It skips `/api/settings`, protected stores, sidebar/bottom nav, network monitor, service-worker update notifications, and pull-to-refresh initialization. The login page fetches `/auth/config`, displays Flowl's existing `Logo`, and treats recognized query flags only as generic translated states. A failed config request displays provider unavailable without conflating it with an authenticated app session.

The login page reuses Gazel's established structure without copying Gazel's visual identity:

- page: `min-height: 100dvh`, centered grid, Flowl-token outer padding;
- mobile `< 48rem`: one centered single-column card, `width: min(100%, 400px)`, centered branding/copy above the action area;
- desktop `>= 48rem`: `width: min(100%, 880px)`, `min-height: 380px`, grid columns `minmax(0, 1.15fr) minmax(300px, 0.85fr)`;
- left desktop column: Flowl logo/wordmark, heading, and authentication-required copy, vertically centered on a Flowl feature/surface background with a separating border;
- right desktop column: vertically centered action area containing a full-width raised Flowl-surface panel with border and medium shadow; optional status appears above exactly one full-width provider button.

Use Flowl's existing color, spacing, typography, surface, border, shadow, and logo tokens throughout. Normal routes retain current shell behavior. Do not modify `pull-to-refresh.ts` or its allowlist. Any necessary route-derived Svelte effects must use event handlers or `untrack`/one-directional state flow; no effect may read state that it or its cleanup mutates. Tests cover both sides of the `48rem` breakpoint, exact card constraints/areas, fail on `effect_update_depth_exceeded`, and confirm existing pull-to-refresh tests remain unchanged.

### 11. Keep protected offline content across outages/expiry, purge it on explicit logout

Change service-worker response rules:

- `/auth/*` is always network-only and never cached.
- Cacheable API network responses are stored only for status 200. Any received 401 or other HTTP response is returned directly; stale fallback happens only when `fetch` rejects without an HTTP response.
- Thumbnails become network-first: store only status-200 image responses, return every received auth/error/redirect response even when a stale thumbnail exists, and use stale thumbnail fallback only when the network rejects without an HTTP response.
- Normal application navigation becomes network-first before every static/precache lookup, including cached `index.html`, so an online unauthenticated request reaches the backend redirect. On transport rejection, use the precached SPA shell if available, then `offline.html`.
- Keep public immutable build assets/login resources separate from protected API/thumbnails and any runtime navigation cache.

This intentionally permits previously cached protected data to remain usable during a genuine outage, including after the backend's twelve-hour session would have expired. A received expiry 401 redirects to login but does not delete those caches. This is the accepted balance required to avoid destroying Flowl's offline value merely because time passed or connectivity flapped.

Explicit Settings logout is the privacy boundary. Before submitting a real browser POST to `/auth/logout`, a cache utility sends `PURGE_PROTECTED_CACHES` to the controlling worker and waits for an acknowledgement; without a controller it directly deletes matching protected Flowl caches. The purge covers current and obsolete API/photo/runtime protected cache names, but retains immutable public login assets, manifest/icons, service-worker version metadata, and local theme/locale preferences. After acknowledgement, submit a same-origin form so the backend's 303 performs the required navigation. `/auth/logout` itself is never cached.

**Alternatives considered:** clearing on every 401 was rejected because expiry must not blindly destroy offline data; partitioning by user was rejected because Flowl has no user model and retains no stable subject; disabling all protected offline caching under auth was rejected because it would unnecessarily remove an existing core feature.

### 12. Test protocol behavior through a local generic mock provider

Add a local Axum/mock OIDC server fixture with configurable discovery metadata, token auth expectation, token claims/signature, JWKS versions/failures, and request counters. Use fixed test signing keys through a maintained JWT library. Production code receives injectable HTTP client, clock, and auth/session state; tests do not sleep twelve hours or thirty seconds. A test-only configuration constructor permits the local HTTP issuer while production environment validation remains HTTPS-only.

Backend integration tests manually preserve secure cookies across in-process requests and cover full login/callback routing. Unit tests cover exact raw issuer preservation, return-target vectors, expiry, transaction atomicity, auth method selection, and JWKS generation/cooldown. Existing `common::test_app()` remains auth-disabled to prove compatibility.

Frontend tests use Vitest/jsdom fetch, location, Cache API, MessageChannel, and service-worker mocks. Extract service-worker policy functions where necessary so status caching, network rejection fallback, auth exclusions, purge, and worker install/update while unauthenticated can be tested without a browser worker runtime.

## Risks / Trade-offs

- **[Risk] Process restart logs out every browser and invalidates callbacks** → This is explicit process-local behavior; the login page provides a direct retry.
- **[Risk] Auth-enabled replicas have independent sessions and login transactions** → Document that deployments require one process/instance or external sticky routing; distributed sessions are out of scope.
- **[Risk] Cached protected data remains readable offline after session expiry** → Accepted to preserve offline behavior; online responses expose expiry, login hides the app, and explicit logout is the deliberate cache-destruction boundary.
- **[Risk] A user closes the page during logout purge/POST sequencing** → Purge is completed and acknowledged before navigation; repeated logout is idempotent and old protected cache names are included.
- **[Risk] A custom provider name is unavailable on a first offline login load** → Public login assets still render and show a generic provider-unavailable/default OIDC action; auth config itself remains uncached as required.
- **[Risk] Discovery/JWKS dependencies increase binary size** → Disable default crate features and use the existing Rustls/Reqwest stack; no database or daemon dependency is introduced.
- **[Risk] Provider metadata advertises unusual algorithms or endpoints** → Rely on `openidconnect`'s typed validation and discovery algorithm allowlist, refuse redirects, and fail closed instead of weakening validation.
- **[Risk] Access logging accidentally expands to include query strings later** → Keep access logs path-only and add redaction tests/guard comments around callback and token HTTP handling.

## Migration Plan

1. Ship authentication disabled by default; existing deployments require no new variables and keep current sessions/routes/offline caches.
2. Document the six variables, HTTPS/reverse-proxy requirement, exact issuer semantics, callback URI (`<FLOWL_EXTERNAL_URL>/auth/callback`), process-local restart behavior, and provider client setup. Update Docker Compose examples and replace README's statement that built-in authentication does not exist.
3. Operators enable all required values together and register the exact callback URI at a generic OIDC provider. Startup validates configuration and discovery before listening.
4. Existing browser service workers update normally. Protected caches become subject to the new response checks and explicit-logout purge; no database migration occurs.
5. Rollback by disabling `FLOWL_AUTH_ENABLED` and restarting. This restores existing public behavior; it does not restore process-local sessions, and existing offline caches continue under the normal version lifecycle.
