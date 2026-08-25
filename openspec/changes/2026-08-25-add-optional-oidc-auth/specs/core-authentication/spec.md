## Purpose

Defines Flowl's optional generic OpenID Connect relying-party boundary, including strict configuration, protocol validation, process-local sessions, route protection, safe navigation, logout, and recoverable signing-key rotation.

## ADDED Requirements

### Requirement: Optional fail-closed authentication configuration

OIDC authentication SHALL be disabled by default. `FLOWL_AUTH_ENABLED` SHALL be parsed strictly, and when it is `true` Flowl SHALL require non-empty valid values for `FLOWL_EXTERNAL_URL`, `FLOWL_OIDC_ISSUER`, `FLOWL_OIDC_CLIENT_ID`, and `FLOWL_OIDC_CLIENT_SECRET`; `FLOWL_OIDC_PROVIDER_NAME` SHALL default to `OpenID Connect`. Invalid enabled-mode configuration SHALL stop startup rather than silently disable or weaken authentication.

#### Scenario: Authentication disabled by default

- **WHEN** Flowl starts without `FLOWL_AUTH_ENABLED`
- **THEN** authentication is disabled
- **AND** existing API, upload, and SPA behavior remains unchanged

#### Scenario: Explicitly disabled authentication

- **WHEN** Flowl starts with `FLOWL_AUTH_ENABLED=false`
- **THEN** authentication is disabled
- **AND** OIDC configuration and provider availability are not required

#### Scenario: Invalid enabled flag

- **WHEN** `FLOWL_AUTH_ENABLED` contains a value other than a valid boolean
- **THEN** startup fails with a configuration error

#### Scenario: Missing enabled-mode configuration

- **WHEN** `FLOWL_AUTH_ENABLED=true`
- **AND** any required OIDC or external URL value is absent or empty
- **THEN** startup fails before the server accepts requests

#### Scenario: Provider name default

- **WHEN** authentication is enabled without `FLOWL_OIDC_PROVIDER_NAME`
- **THEN** the provider display name is `OpenID Connect`

### Requirement: Exact issuer and external URL validation

Flowl SHALL retain `FLOWL_OIDC_ISSUER` as its exact configured string and separately validate it as an absolute HTTPS issuer URL without changing that representation. `IssuerUrl` SHALL be constructed directly from the retained raw string, never from a reserialized `url::Url`; discovery and signature-verified ID-token issuer values SHALL also be compared to the retained raw string as a defense-in-depth exact check. URL-normalized alternatives SHALL NOT be accepted. `FLOWL_EXTERNAL_URL` SHALL be an explicit absolute HTTPS origin without credentials, query, fragment, or a non-root path, and SHALL be the sole basis for callback and same-origin security decisions.

#### Scenario: Exact issuer representation is preserved

- **WHEN** `FLOWL_OIDC_ISSUER=https://issuer.example` is configured
- **THEN** discovery and token verification use exactly `https://issuer.example`
- **AND** Flowl does not serialize it as `https://issuer.example/`

#### Scenario: Discovery issuer slash mismatch is rejected

- **WHEN** the configured issuer is `https://issuer.example`
- **AND** discovery metadata identifies the issuer as `https://issuer.example/`
- **THEN** startup fails
- **AND** Flowl does not accept the slash-normalized issuer

#### Scenario: Signed token issuer slash mismatch is rejected

- **WHEN** the configured issuer is `https://issuer.example`
- **AND** an otherwise valid signed ID token identifies the issuer as `https://issuer.example/`
- **THEN** authentication fails
- **AND** Flowl does not accept the slash-normalized issuer

#### Scenario: Invalid issuer URL

- **WHEN** authentication is enabled with a malformed issuer, a non-HTTPS issuer, or an issuer containing a query or fragment
- **THEN** startup fails with a safe configuration diagnostic

#### Scenario: Invalid external URL

- **WHEN** authentication is enabled with an external URL that is malformed, non-HTTPS, contains credentials, query, fragment, or a non-root path
- **THEN** startup fails with a safe configuration diagnostic

#### Scenario: Forwarded headers cannot change callback origin

- **WHEN** a request supplies `Host`, `Forwarded`, or `X-Forwarded-*` values different from `FLOWL_EXTERNAL_URL`
- **THEN** Flowl still uses `FLOWL_EXTERNAL_URL` to construct `https://<configured-origin>/auth/callback`

### Requirement: Startup discovery and confidential-client authentication

When authentication is enabled, Flowl SHALL perform standards-compliant OIDC discovery and initial JWKS retrieval during startup, cache the validated metadata and keys, refuse HTTP redirects, and fail startup if discovery, initial JWKS retrieval/parsing, issuer verification, Authorization Code support, a non-empty permitted ID-token signing-algorithm set, required HTTPS authorization/token/JWKS endpoints, or compatible token-endpoint authentication cannot be established. Flowl SHALL support `client_secret_basic` and `client_secret_post` according to discovery metadata, preferring Basic when both are supported; omission of `token_endpoint_auth_methods_supported` SHALL mean `client_secret_basic`.

#### Scenario: Basic client authentication advertised

- **WHEN** discovery metadata advertises `client_secret_basic`
- **THEN** token exchange authenticates the client with HTTP Basic authentication
- **AND** the client secret is not duplicated in the request body

#### Scenario: Post client authentication advertised

- **WHEN** discovery metadata omits `client_secret_basic`
- **AND** advertises `client_secret_post`
- **THEN** token exchange sends the client credentials in the form-encoded request body

#### Scenario: Client authentication metadata omitted

- **WHEN** discovery metadata omits `token_endpoint_auth_methods_supported`
- **THEN** Flowl uses `client_secret_basic` as the OIDC default

#### Scenario: No supported client authentication method

- **WHEN** discovery metadata advertises neither `client_secret_basic` nor `client_secret_post`
- **THEN** startup fails before the server accepts requests

#### Scenario: Discovery unavailable at startup

- **WHEN** authentication is enabled and valid discovery metadata or initial JWKS cannot be retrieved or parsed
- **THEN** startup fails closed with a safe diagnostic

#### Scenario: Unsafe or incompatible discovered metadata

- **WHEN** discovery supplies a non-HTTPS authorization, token, or JWKS endpoint, does not support the Authorization Code response type, or has no permitted ID-token signing algorithm
- **THEN** startup fails before the server accepts requests

### Requirement: Authorization Code flow transaction

`GET /auth/login` SHALL start a generic OIDC Authorization Code flow using PKCE S256, cryptographically random state and nonce, and the callback URI derived from `FLOWL_EXTERNAL_URL`. Pending login transactions SHALL be backend-only, bound to the initiating browser's opaque pre-authentication session, one-time, bounded in number and size, and expire after five minutes.

#### Scenario: Login authorization redirect

- **WHEN** an unauthenticated browser requests `/auth/login` with a valid local `return_to`
- **THEN** Flowl creates a pending transaction containing the state, nonce, PKCE verifier, callback URI, expiry, and validated target
- **AND** responds with a redirect to the discovered authorization endpoint
- **AND** the request contains `response_type=code`, a random state, a random nonce, a PKCE challenge, and `code_challenge_method=S256`

#### Scenario: Transaction bound to browser

- **WHEN** a callback presents a valid state but not the pre-authentication session cookie that initiated it
- **THEN** the callback fails without exchanging the code

#### Scenario: Expired transaction

- **WHEN** a callback uses a transaction five minutes or more after it was created
- **THEN** the callback fails without exchanging the code

#### Scenario: State transaction replay

- **WHEN** a callback state has already been consumed once
- **THEN** a later callback using that state fails without exchanging the code

### Requirement: Callback and token validation

`GET /auth/callback` SHALL reject ambiguous or polluted callback parameters, atomically consume and validate the pending transaction before exchanging an authorization code with its original PKCE verifier, and require an ID token in the token response. Flowl SHALL use maintained OIDC and cryptographic libraries to verify the ID-token signature and permitted algorithm, exact issuer, client audience, expiration, required issued-at claim, stored nonce, and authorized-party semantics when applicable. If a signed ID token supplies an applicable `at_hash`, Flowl SHALL verify it against the returned access token. Any failed check SHALL prevent session authentication.

#### Scenario: Successful callback

- **WHEN** the callback cookie, state, code, PKCE verifier, nonce, token response, signature, issuer, audience, and expiry are valid
- **AND** any supplied applicable `at_hash` is valid
- **THEN** Flowl creates an authenticated session
- **AND** responds with HTTP 303 to the transaction's validated `return_to`

#### Scenario: PKCE mismatch

- **WHEN** the token endpoint cannot validate the stored PKCE verifier for the authorization code
- **THEN** authentication fails
- **AND** no authenticated session is created

#### Scenario: State or nonce mismatch

- **WHEN** callback state or ID-token nonce does not match the one-time transaction
- **THEN** authentication fails
- **AND** no authenticated session is created

#### Scenario: Invalid ID-token claims

- **WHEN** an ID token is missing, has an invalid signature, disallowed algorithm, mismatched exact issuer, wrong or multiple untrusted audiences, missing issued-at, invalid authorized party, or expired claims
- **THEN** authentication fails
- **AND** no authenticated session is created

#### Scenario: Ambiguous callback parameters

- **WHEN** a callback repeats `code` or `state`, or supplies both success and error parameters
- **THEN** authentication fails without creating a session
- **AND** untrusted ambiguous values are not used for token exchange

#### Scenario: Invalid access-token hash

- **WHEN** an ID token supplies an applicable `at_hash` that does not match the returned access token
- **THEN** authentication fails
- **AND** no authenticated session is created

### Requirement: Process-local authenticated sessions

Authenticated sessions SHALL use an opaque, cryptographically random session ID whose data exists only in process memory. The session cookie SHALL be host-only, `HttpOnly`, `Secure`, `SameSite=Lax`, scoped to `/`, and SHALL contain no OIDC token, identity claim, code, state, nonce, PKCE value, or client secret. Sessions SHALL have a non-sliding absolute lifetime of twelve hours from successful authentication and SHALL be invalidated by process restart.

#### Scenario: Session ID rotation after login

- **WHEN** authentication succeeds from a pre-authentication session
- **THEN** Flowl invalidates the pre-authentication session ID
- **AND** issues a new unrelated authenticated session ID

#### Scenario: Valid authenticated session

- **WHEN** a protected request presents an unexpired authenticated session cookie
- **THEN** the request proceeds without exposing OIDC tokens to the browser

#### Scenario: Absolute session expiry

- **WHEN** twelve hours have elapsed since authentication
- **THEN** the session is invalid even if it was recently used
- **AND** requests do not extend its expiry

#### Scenario: Restart invalidates sessions

- **WHEN** the Flowl process restarts
- **THEN** all prior authenticated sessions and pending login transactions are invalid

### Requirement: Authentication route policy

When authentication is enabled, only `/health`, `/login`, `/auth/config`, `/auth/login`, `/auth/callback`, `POST /auth/logout`, and exact non-data static resources required for login/PWA operation (including the service worker, manifest, icons, offline page, and immutable build assets) SHALL be public. `index.html` and normal application documents SHALL not become public merely because they are embedded files. All `/api/*`, `/uploads/*`, and normal application SPA routes SHALL require an authenticated session. The public `GET /auth/config` response SHALL expose only whether auth is enabled and the provider display name. Every `/auth/*` response and authentication-required response SHALL be marked `Cache-Control: no-store`.

#### Scenario: Public health and login resources

- **WHEN** an unauthenticated client requests a listed public route or required login asset
- **THEN** the request is served without an authenticated session

#### Scenario: Protected upload request

- **WHEN** an unauthenticated client requests `/uploads/photo.jpg`
- **THEN** Flowl returns HTTP 401
- **AND** does not serve or redirect to the protected photo

#### Scenario: Protected browser navigation

- **WHEN** an unauthenticated browser navigates online to a normal application SPA route
- **THEN** Flowl redirects it to `/login?return_to=<validated-path-and-query>`

#### Scenario: Auth config contains no sensitive values

- **WHEN** a client requests `/auth/config`
- **THEN** the response contains only `enabled` and `provider_name`
- **AND** it contains no issuer, client identifier, client secret, endpoint, token, session, or identity value

### Requirement: Safe local return targets

All backend and frontend authentication navigation SHALL use one shared behavioral policy for a bounded local `return_to`. A valid target SHALL begin with one `/`, contain a well-formed local path, and MAY contain query and fragment components. Flowl SHALL reject absolute or protocol-relative URLs, backslashes, malformed or encoded path confusion, control characters, auth/login routes, and targets longer than 2048 bytes, using `/` as the fallback. Backend-originated redirects SHALL preserve path and query; SPA reauthentication MAY additionally preserve `location.hash`.

#### Scenario: Valid backend navigation target

- **WHEN** an unauthenticated browser requests `/plants/42?tab=care`
- **THEN** the login redirect preserves `/plants/42?tab=care` as `return_to`

#### Scenario: Valid SPA target with fragment

- **WHEN** the SPA handles authentication expiry at `/plants/42?tab=care#entry-7`
- **THEN** it may preserve the complete local path, query, and fragment as `return_to`

#### Scenario: External target rejected

- **WHEN** `return_to` is `https://attacker.example/`, `//attacker.example/`, or another non-local form
- **THEN** Flowl uses `/` instead

#### Scenario: Auth route target rejected

- **WHEN** `return_to` names `/login`, `/auth`, any path below them, or an encoded or normalized equivalent
- **THEN** Flowl uses `/` instead

#### Scenario: Malformed or oversized target rejected

- **WHEN** `return_to` contains a control character, backslash, malformed escape, unsafe path normalization, or exceeds 2048 bytes
- **THEN** Flowl uses `/` instead

### Requirement: Safe callback failure behavior

Callback and provider failures SHALL return the browser to `/login` with only a generic authentication-failed or provider-unavailable state and a validated local `return_to`. Flowl SHALL NOT expose protocol, token, claim, provider response body, or credential details to the browser.

#### Scenario: Invalid callback returns generic failure

- **WHEN** callback validation fails because of state, PKCE, nonce, token, claim, parameter ambiguity, or replay validation
- **THEN** Flowl responds with HTTP 303 to `/login` with a generic authentication-failed state
- **AND** a consumed trustworthy transaction preserves only its previously validated `return_to`
- **AND** a callback without a trustworthy transaction falls back to `/`
- **AND** no sensitive failure details appear in the location

#### Scenario: Provider unavailable during callback

- **WHEN** token exchange or a required signing-key refresh fails because the provider cannot be reached or returns an unusable service response
- **THEN** Flowl responds with HTTP 303 to `/login` with a generic provider-unavailable state

### Requirement: Local logout

`POST /auth/logout` SHALL invalidate any presented local authenticated session, expire the session cookie with matching security attributes, and respond with HTTP 303 to `/login?logged_out=1`. Logout SHALL be local and SHALL NOT require provider-specific end-session behavior.

#### Scenario: Authenticated logout

- **WHEN** an authenticated browser posts to `/auth/logout`
- **THEN** the local session can no longer access protected routes
- **AND** the session cookie is cleared
- **AND** the response redirects to `/login?logged_out=1`

#### Scenario: Repeated logout

- **WHEN** a browser without a valid session posts to `/auth/logout`
- **THEN** the operation remains safe and idempotent
- **AND** redirects to `/login?logged_out=1`

### Requirement: Recoverable JWKS rotation

Flowl SHALL retain the last valid JWKS in memory. When ID-token verification fails for a reason that a signing-key rotation could resolve, Flowl SHALL coordinate at most one refresh across concurrent callbacks, retry verification once with a newly fetched JWKS, and never refresh for issuer, audience, expiry, nonce, or other non-key claim failures. A failed refresh SHALL retain the last valid keys, impose a thirty-second retry cooldown, and allow a later callback to attempt refresh again without restarting Flowl.

#### Scenario: New signing key succeeds after refresh

- **WHEN** a valid ID token uses a newly rotated signing key absent from the cached JWKS
- **AND** refreshed JWKS contains that key
- **THEN** Flowl refreshes JWKS once
- **AND** retries and completes token verification successfully

#### Scenario: Concurrent rotation failures are deduplicated

- **WHEN** concurrent callbacks fail against the same cached JWKS generation for a key-rotation reason
- **THEN** only one JWKS network refresh occurs
- **AND** the other callbacks retry against the resulting generation rather than starting duplicate refreshes

#### Scenario: Failed refresh enters cooldown

- **WHEN** a key-triggered JWKS refresh fails
- **THEN** Flowl keeps the last valid cached JWKS
- **AND** further key-triggered requests during the next thirty seconds do not start another refresh

#### Scenario: Authentication recovers after failed refresh

- **WHEN** a JWKS refresh previously failed
- **AND** the thirty-second cooldown has elapsed
- **AND** the provider now returns valid rotated keys
- **THEN** a later callback refreshes JWKS and can authenticate successfully without a Flowl restart

### Requirement: Secret handling and safe diagnostics

OIDC tokens and all client secret, authorization code, state, nonce, PKCE, login transaction, and session values SHALL remain backend-only and SHALL NOT be logged. Discovery, token, and JWKS failures SHALL retain useful safe context and underlying error categories without logging credentials or provider response bodies. Flowl SHALL use maintained OIDC and cryptographic libraries rather than implementing protocol cryptography or token validation primitives itself.

#### Scenario: Sensitive callback is logged safely

- **WHEN** callback or token processing fails
- **THEN** logs may identify the operation and safe error category
- **AND** do not contain the callback query, code, state, nonce, PKCE verifier, token, secret, cookie, session ID, or provider response body

#### Scenario: Discovery diagnostic is useful and safe

- **WHEN** startup discovery fails
- **THEN** the startup diagnostic identifies discovery as the failing operation and preserves a safe underlying transport, status, parsing, or validation cause
- **AND** omits response bodies and credentials

### Requirement: Authenticated identity scope

Any identity whose OIDC authentication passes all configured-provider validation SHALL be allowed to use the same Flowl application data. Flowl SHALL NOT add local users, passwords, registration, roles, groups, permissions, claim mapping, userinfo lookup, or per-user data, and SHALL discard received OIDC tokens and identity claims after completing validation.

#### Scenario: Any valid provider identity is accepted

- **WHEN** an identity from the configured issuer completes valid OIDC authentication
- **THEN** Flowl creates an authenticated session without applying role, group, email, subject allowlist, or other claim-based authorization

#### Scenario: Tokens are not retained for application use

- **WHEN** successful token and ID-token validation completes
- **THEN** Flowl retains only local session authentication state and timestamps
- **AND** does not persist or expose the provider tokens or identity claims
