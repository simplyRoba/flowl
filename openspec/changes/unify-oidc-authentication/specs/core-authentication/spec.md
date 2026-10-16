# Spec Delta

## MODIFIED Requirements

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

#### Scenario: Non-Unicode authentication configuration

- **WHEN** `FLOWL_AUTH_ENABLED` is present but not Unicode, or authentication is enabled and an authentication configuration field being read is present but not Unicode
- **THEN** startup fails with a safe configuration error
- **AND** unrelated configuration retains its existing parsing behavior

#### Scenario: Explicit false ignores other OIDC configuration

- **WHEN** `FLOWL_AUTH_ENABLED=false`
- **AND** another OIDC configuration value is malformed or non-Unicode
- **THEN** authentication remains disabled without reading or validating that value

#### Scenario: Provider display name normalization

- **WHEN** an enabled configuration supplies `FLOWL_OIDC_PROVIDER_NAME`
- **THEN** Flowl trims surrounding whitespace before displaying the name
- **AND** rejects an empty trimmed name, any remaining Unicode control character, or a trimmed name longer than 80 Unicode scalar values
- **AND** the limit is not measured in UTF-8 bytes


### Requirement: Startup discovery and confidential-client authentication

When authentication is enabled, Flowl SHALL perform standards-compliant OIDC discovery and initial JWKS retrieval during startup, cache the validated metadata and keys, refuse HTTP redirects, and fail startup if discovery, initial JWKS retrieval/parsing, issuer verification, standalone Authorization Code response support, compatible advertised grants, a non-empty permitted ID-token signing-algorithm set with a compatible signing key, required HTTPS authorization/token/JWKS endpoints, or compatible token-endpoint authentication cannot be established. Flowl SHALL support `client_secret_basic` and `client_secret_post` according to discovery metadata, preferring Basic when both are supported; omission of `token_endpoint_auth_methods_supported` SHALL mean `client_secret_basic`.

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

#### Scenario: Standalone code response required

- **WHEN** discovery advertises only hybrid response types such as `code id_token` and no standalone `code` response type
- **THEN** startup fails

#### Scenario: Authorization code grant compatibility

- **WHEN** discovery supplies `grant_types_supported` without `authorization_code`
- **THEN** startup fails
- **AND** omission of `grant_types_supported` alone does not cause rejection

#### Scenario: Signing metadata and keys must be compatible

- **WHEN** metadata advertises `none`, or the initial JWKS has no signing key compatible with the permitted advertised ID-token signing algorithms
- **THEN** startup fails before the server accepts requests


### Requirement: Authorization Code flow transaction

For an unauthenticated browser, `GET /auth/login` SHALL start a generic OIDC Authorization Code flow using PKCE S256, cryptographically random state and nonce, and the callback URI derived from `FLOWL_EXTERNAL_URL`. Pending login transactions SHALL be backend-only, bound to the initiating browser's opaque pre-authentication session, one-time, expire after five minutes, and be invalidated by a process restart. Flowl SHALL allow at most 1,024 pending login transactions across the running instance, pruning expired entries before checking capacity.

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

#### Scenario: Capacity boundary accepts the final slot

- **WHEN** 1,023 valid pending transactions remain after expired entries are pruned
- **AND** a new login is requested
- **THEN** the new transaction is created as the 1,024th pending transaction
- **AND** Flowl redirects to the OIDC provider

#### Scenario: Full capacity rejects new login

- **WHEN** 1,024 valid pending transactions remain after expired entries are pruned
- **AND** a new login is requested
- **THEN** Flowl does not create or bind the new transaction
- **AND** does not redirect to the OIDC provider
- **AND** redirects through the existing generic `/login?error=provider_unavailable&return_to=<safe-target>` path

#### Scenario: Expired entries free capacity before rejection

- **WHEN** the pending transaction capacity has been reached
- **AND** at least one entry is expired when a new login is requested
- **THEN** Flowl prunes expired entries before checking capacity
- **AND** creates the new transaction when the remaining valid count is below 1,024

#### Scenario: Saturation does not evict active transactions

- **WHEN** the pending transaction capacity is full of valid transactions
- **AND** a new login is rejected for capacity
- **THEN** every previously valid transaction remains present and usable
- **AND** no existing transaction is evicted, including a prior transaction bound to the initiating session

#### Scenario: Login succeeds after capacity becomes available

- **WHEN** a login was rejected because the registry was full
- **AND** a valid entry is later consumed or expires
- **THEN** a subsequent login can create a transaction and redirect to the OIDC provider

#### Scenario: State transaction replay

- **WHEN** a callback state has already been consumed once
- **THEN** a later callback using that state fails without exchanging the code

#### Scenario: Authenticated login bypass

- **WHEN** a browser with a valid authenticated session requests `/auth/login`
- **THEN** Flowl responds with HTTP 303 to the validated `return_to` or `/` when absent or invalid
- **AND** creates no transaction and sends no provider redirect
- **AND** does not mutate the session, rotate its identifier, or shorten or extend its absolute deadline

#### Scenario: Replacement login is saved before becoming usable

- **WHEN** a browser with a pending login successfully starts a replacement login
- **THEN** only the replacement transaction is bound to that browser after the binding is saved
- **AND** the previous pending or in-flight attempt cannot subsequently authenticate that browser

#### Scenario: Failed replacement binding preserves prior login

- **WHEN** saving a new or replacement pre-authentication binding fails
- **THEN** Flowl does not redirect to the provider or leave the new transaction usable
- **AND** preserves any prior transaction and browser binding unless a newer login or logout has invalidated them
- **AND** redirects to the generic provider-unavailable login state with the validated target

#### Scenario: Concurrent callbacks consume once

- **WHEN** callbacks concurrently present the same valid browser-bound state
- **THEN** at most one callback consumes it and proceeds to token exchange
- **AND** other callbacks cannot establish a session from that state

#### Scenario: Older binding update cannot replace a newer login

- **WHEN** login starts, callbacks, or persistence completions overlap for the same browser
- **THEN** an older attempt cannot overwrite a newer successfully saved login binding or authenticated session
- **AND** failed saves do not restore superseded bindings over newer state


### Requirement: Callback and token validation

`GET /auth/callback` SHALL reject ambiguous or polluted callback parameters, atomically consume and validate the pending transaction before exchanging an authorization code with its original PKCE verifier, and require an ID token in the token response. Flowl SHALL use maintained OIDC and cryptographic libraries to verify the ID-token signature and permitted algorithm, exact issuer, client audience, expiration, required positive issued-at claim, stored nonce, and authorized-party semantics when applicable. An `azp` claim, when present, SHALL equal the configured client identifier; additional audiences SHALL be rejected without introducing audience trust exceptions. If a signed ID token supplies an applicable `at_hash`, Flowl SHALL verify it against the returned access token. Any failed check SHALL prevent session authentication.

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

- **WHEN** an ID token is missing, has an invalid signature, disallowed algorithm, mismatched exact issuer, wrong or multiple untrusted audiences, missing or non-positive issued-at, invalid authorized party, or expired claims
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

#### Scenario: Strict callback form decoding

- **WHEN** the callback query contains malformed percent escapes or invalid UTF-8 after form decoding
- **THEN** Flowl rejects it before token exchange
- **AND** does not consume any otherwise valid pending transaction

#### Scenario: Duplicate recognized callback parameter

- **WHEN** any of `state`, `code`, `error`, `error_description`, or `error_uri` occurs more than once after form decoding
- **THEN** Flowl rejects the callback before token exchange
- **AND** does not consume a valid transaction

#### Scenario: Mixed or empty success callback

- **WHEN** a callback supplies success `code` together with `error`, `error_description`, or `error_uri`, or supplies an empty success `code` or `state`
- **THEN** Flowl rejects the callback before token exchange
- **AND** does not consume a valid transaction

#### Scenario: Incomplete callback shape

- **WHEN** a callback lacks a non-empty `state`, lacks both a success `code` and a non-empty provider `error`, or supplies error details without `error`
- **THEN** Flowl rejects it before token exchange
- **AND** does not consume any valid transaction

#### Scenario: Valid provider error with both detail fields

- **WHEN** a well-formed callback supplies a non-empty `error` and a matching browser-bound `state`, with distinct optional `error_description` and `error_uri` fields
- **THEN** Flowl accepts the callback as an authentication failure and consumes the matching transaction once
- **AND** ignores both detail values and preserves only the transaction's stored return target
- **AND** performs no token exchange and reflects no provider-supplied content

#### Scenario: Wrong browser leaves transaction usable

- **WHEN** a well-formed success or provider-error callback presents a valid state from another browser
- **THEN** Flowl rejects it without consuming the matching transaction
- **AND** the initiating browser can still use that transaction


### Requirement: Non-durable authenticated sessions

Authenticated sessions SHALL use an opaque, cryptographically random session ID with non-durable server-side session data. The session cookie SHALL be host-only, `HttpOnly`, `Secure`, `SameSite=Lax`, scoped to `/`, and SHALL contain no OIDC token, identity claim, code, state, nonce, PKCE value, or client secret. Sessions SHALL have a non-sliding absolute lifetime of twelve hours from successful authentication and SHALL be invalidated by process restart.

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

#### Scenario: Authentication is persisted before success

- **WHEN** token verification succeeds and the callback still owns the browser's current login attempt
- **THEN** Flowl rotates the session identifier, clears pre-authentication state, and saves the authenticated session before sending the success redirect

#### Scenario: Authenticated session save fails

- **WHEN** saving the verified and rotated authenticated session fails
- **THEN** no valid authenticated state is established
- **AND** Flowl redirects to the generic provider-unavailable login state with only the trusted stored target

#### Scenario: Callback completion after invalidation

- **WHEN** a callback completes provider processing after successful logout or replacement has invalidated its login attempt
- **THEN** it cannot establish an authenticated session or overwrite a newer session or login binding
- **AND** provider requests already sent need not be cancelled


### Requirement: Authentication route policy

When authentication is enabled, only `/health`, `/login`, `/auth/config`, `/auth/login`, `/auth/callback`, `POST /auth/logout`, and exact non-data static resources required for login/PWA operation (including the service worker, manifest, icons, offline page, and immutable build assets) SHALL be public. `index.html` and normal application documents SHALL not become public merely because they are embedded files. All `/api` and `/uploads` namespace roots and descendants, and normal application SPA routes SHALL require an authenticated session. The public `GET /auth/config` response SHALL be exactly `{ "enabled": true, "provider_name": <string> }` when enabled and exactly `{ "enabled": false }` when disabled. Every `/auth` and `/auth/*` response, including errors and disabled-mode responses, and every authentication-required response SHALL be marked `Cache-Control: no-store`.

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
- **THEN** the enabled response contains exactly `enabled: true` and the string `provider_name`, or the disabled response contains exactly `enabled: false`
- **AND** it contains no issuer, client identifier, client secret, endpoint, token, session, or identity value

#### Scenario: Disabled auth configuration omits provider name

- **WHEN** authentication is disabled and a client requests `/auth/config`
- **THEN** the body is exactly `{ "enabled": false }`
- **AND** `provider_name` is absent rather than null
- **AND** the response has `Cache-Control: no-store`

#### Scenario: Authentication errors are not cacheable

- **WHEN** a request to `/auth` or any `/auth/*` path produces an error, including when authentication is disabled
- **THEN** the response has `Cache-Control: no-store`
- **AND** authentication-required HTTP 401 responses and document login redirects also retain `Cache-Control: no-store`


### Requirement: Safe local return targets

All backend and frontend authentication navigation SHALL apply equivalent validation for a bounded local `return_to`. A valid target SHALL begin with exactly one `/`, contain a well-formed local path, and support valid query and fragment components. Flowl SHALL reject absolute or protocol-relative URLs, backslashes, malformed or encoded path confusion, Unicode controls U+0000–U+001F and U+007F–U+009F, auth/login/API/health/upload namespace roots and descendants, and serialized targets longer than 2048 UTF-8 bytes, using `/` as the fallback. Backend-originated redirects SHALL preserve path and query; SPA-originated reauthentication SHALL additionally preserve a valid `location.hash` when present.

#### Scenario: Valid backend navigation target

- **WHEN** an unauthenticated browser requests `/plants/42?tab=care`
- **THEN** the login redirect preserves `/plants/42?tab=care` as `return_to`

#### Scenario: Valid SPA target with fragment

- **WHEN** the SPA handles authentication expiry at `/plants/42?tab=care#entry-7`
- **THEN** it preserves the complete valid local path, query, and fragment as `return_to`

#### Scenario: External target rejected

- **WHEN** `return_to` is `https://attacker.example/`, `//attacker.example/`, or another non-local form
- **THEN** Flowl uses `/` instead

#### Scenario: Auth route target rejected

- **WHEN** `return_to` names `/login`, `/auth`, any path below them, or an encoded or normalized equivalent
- **THEN** Flowl uses `/` instead

#### Scenario: Malformed or oversized target rejected

- **WHEN** `return_to` contains a control character, backslash, malformed escape, unsafe path normalization, or exceeds 2048 bytes
- **THEN** Flowl uses `/` instead

#### Scenario: Serialized target is measured after one form decode

- **WHEN** an authentication query supplies a `return_to` value
- **THEN** Flowl form-decodes the query exactly once to obtain the serialized target and checks its 2048-UTF-8-byte limit before validation-only decoding
- **AND** wire query encoding does not count toward that limit
- **AND** a valid accepted serialized target is returned unchanged rather than decoded or normalized

#### Scenario: Invalid encodings and controls rejected

- **WHEN** the serialized target has malformed percent escapes, invalid UTF-8, raw or decoded backslashes, or raw or decoded Unicode controls U+0000–U+001F or U+007F–U+009F
- **THEN** Flowl uses `/` instead

#### Scenario: Path confusion rejected

- **WHEN** validation of the path finds dot segments, encoded `/`, backslash, `?`, or `#` delimiters, or residual `%` indicating nested path escaping
- **THEN** Flowl uses `/` instead
- **AND** validation does not decode and return a different target

#### Scenario: Ordinary query and fragment escapes preserved

- **WHEN** `/plants/42?label=a%20b&ratio=100%25#entry%20one` is supplied as the serialized target
- **THEN** it is accepted unchanged
- **AND** a decoded percent sign in a valid query value alone does not cause rejection

#### Scenario: Reserved non-document targets rejected

- **WHEN** a target names `/auth`, `/login`, `/api`, `/health`, or `/uploads`, or any descendant or encoded equivalent of those paths
- **THEN** Flowl uses `/` instead
- **AND** names that merely share a prefix, such as `/apiary`, are not rejected for that reason alone

#### Scenario: Equivalent frontend and backend validation

- **WHEN** frontend and backend validate the same serialized return-target vectors, including C1 controls, nested path escapes, and valid escaped query values
- **THEN** they agree on acceptance or fallback and preserve the same accepted representation
- **AND** backend request navigation preserves only path and query while SPA-originated navigation can preserve a valid fragment


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

#### Scenario: Parsed OAuth rejection is an authentication failure

- **WHEN** a valid provider-error callback or a parsed OAuth token-endpoint error from a non-5xx response rejects authentication
- **THEN** Flowl redirects to `/login?error=authentication_failed&return_to=<safe-target>`
- **AND** exposes no provider error values

#### Scenario: Provider service failures remain unavailable

- **WHEN** a required provider request has a transport failure, HTTP 5xx response, unparseable response, or unexpected service response
- **THEN** Flowl uses the generic `provider_unavailable` state rather than treating it as a parsed OAuth rejection
- **AND** redirects with only the trusted transaction target or `/` when none is trustworthy


### Requirement: Local logout

`POST /auth/logout` SHALL invalidate any presented local authenticated session and pending or in-flight login attempt, expire the session cookie with matching security attributes, and on success respond with HTTP 303 to `/login?logged_out=1`. Logout SHALL be local and SHALL NOT require provider-specific end-session behavior.

#### Scenario: Authenticated logout

- **WHEN** an authenticated browser posts to `/auth/logout`
- **THEN** the local session can no longer access protected routes
- **AND** the session cookie is cleared
- **AND** the response redirects to `/login?logged_out=1`

#### Scenario: Repeated logout

- **WHEN** a browser without a valid session posts to `/auth/logout`
- **THEN** the operation remains safe and idempotent
- **AND** redirects to `/login?logged_out=1`

#### Scenario: Logout persistence failure

- **WHEN** local session or associated-attempt invalidation or its persistence fails
- **THEN** Flowl responds with HTTP 500 and expires the presented session cookie with matching security attributes
- **AND** does not send a successful signed-out redirect

#### Scenario: Logout invalidates pending login

- **WHEN** a browser successfully logs out with a pending or in-flight login attempt
- **THEN** that attempt cannot later establish an authenticated session even if provider processing completes successfully
- **AND** requests already sent to the provider need not be cancelled


### Requirement: Secret handling and safe diagnostics

OIDC tokens and all client secret, authorization code, state, nonce, PKCE, login transaction, and session values SHALL remain backend-only and SHALL NOT be logged. Discovery, token, and JWKS failures SHALL retain useful safe context and underlying error categories without logging credentials or provider response bodies. Flowl SHALL use maintained OIDC and cryptographic libraries rather than implementing protocol cryptography or token validation primitives itself.

#### Scenario: Sensitive callback is logged safely

- **WHEN** callback or token processing fails
- **THEN** logs may identify the operation and safe error category
- **AND** do not contain the callback query, code, state, nonce, PKCE verifier, token, secret, cookie, session ID, or provider response body

#### Scenario: Discovery diagnostic is useful and safe

- **WHEN** startup discovery fails
- **THEN** the startup diagnostic identifies discovery as the failing operation and preserves a fixed safe transport, status, parsing, or validation category
- **AND** omits response bodies and credentials

#### Scenario: Provider validation text is not a diagnostic

- **WHEN** a provider or validation failure includes arbitrary error text
- **THEN** diagnostics contain only the operation, fixed safe category, and safe HTTP status when applicable
- **AND** do not include arbitrary provider-supplied or validation-library error strings


## ADDED Requirements

### Requirement: Guarded outbound OIDC destinations

Flowl SHALL validate every outbound OIDC URL before making a request, including discovery, initial JWKS, token exchange, and JWKS refresh. Production destinations SHALL use HTTPS and contain no URL credentials or fragments. Flowl SHALL refuse redirects. HTTP provider fixtures SHALL be restricted to tests, not enabled as a production loopback exception.

#### Scenario: Unsafe initial JWKS is never contacted

- **WHEN** discovery identifies an HTTP, credential-bearing, or fragment-bearing JWKS URL in production
- **THEN** startup fails without contacting that JWKS destination

#### Scenario: Every provider request is guarded

- **WHEN** discovery, token exchange, or signing-key refresh would use an invalid URL
- **THEN** Flowl rejects it before network contact
- **AND** initial discovery failure stops startup while runtime provider failures use the generic provider-unavailable result

#### Scenario: Redirect response is not followed

- **WHEN** a discovery, initial JWKS, token, or refresh request receives a redirect
- **THEN** Flowl does not contact the redirect destination
- **AND** startup fails or runtime authentication reports provider unavailable as applicable

#### Scenario: Authorization URL is validated before use

- **WHEN** discovery supplies an authorization endpoint with credentials, a fragment, or a non-HTTPS production scheme
- **THEN** startup fails and no browser is redirected to that endpoint
