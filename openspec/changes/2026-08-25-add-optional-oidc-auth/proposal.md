## Why

Flowl currently assumes a trusted network or an external authentication proxy, which makes safe direct exposure and consistent protection of its API, uploads, SPA routes, and offline data difficult. Add an optional, standards-compliant built-in OIDC relying party while preserving the current zero-auth behavior by default and retaining Flowl's useful offline experience.

## What Changes

- Add disabled-by-default generic OIDC authentication using Authorization Code flow with PKCE S256, one-time state, nonce, startup discovery, strict issuer and ID-token validation, supported confidential-client authentication, and recoverable JWKS rotation handling.
- Add process-local opaque sessions with backend-only tokens, secure cookies, absolute twelve-hour expiry, session-ID rotation after login, and local POST logout; do not add users, passwords, registration, authorization policy, claim mapping, per-user data, or provider-specific behavior.
- Add strict fail-closed auth configuration through `FLOWL_AUTH_ENABLED`, `FLOWL_EXTERNAL_URL`, `FLOWL_OIDC_ISSUER`, `FLOWL_OIDC_CLIENT_ID`, `FLOWL_OIDC_CLIENT_SECRET`, and optional provider display name.
- Make `/health`, login/auth endpoints, and login-rendering assets public while protecting `/api/*`, `/uploads/*`, and normal SPA routes when auth is enabled. API authentication failures remain JSON `401 AUTHENTICATION_REQUIRED`; browser navigations use validated local `return_to` redirects.
- Add a branded, translated public `/login` page and a translated Settings Authentication section with Sign out when auth is enabled.
- Centralize frontend handling of `401 AUTHENTICATION_REQUIRED` across ordinary, export, AI, upload/photo, and route-load fetch paths without treating arbitrary failures or network outages as session expiry.
- Harden service-worker caching so auth responses and unsuccessful protected responses are never cached, online navigation reaches the backend auth boundary, temporary outages retain stale offline data, and explicit logout purges protected API/photo/runtime application caches without removing public login resources.
- Add local/mock-provider automated coverage for configuration, OIDC protocol and token validation, session lifecycle, route protection, safe redirects, JWKS recovery, frontend login/expiry behavior, logout cache purge, and existing offline behavior.

## Capabilities

### New Capabilities

- `core-authentication`: Optional OIDC configuration, discovery, protocol validation, process-local sessions, route policy, login/callback/logout behavior, safe redirects, and JWKS refresh recovery.
- `ui-authentication`: Public login experience, auth configuration consumption, centralized session-expiry navigation, and logout coordination.

### Modified Capabilities

- `core-api`: Add the stable JSON `AUTHENTICATION_REQUIRED` 401 contract for protected APIs.
- `core-server`: Make SPA fallback and upload serving conditional on authentication while retaining public health and required static assets.
- `ui-shell`: Render the public login route outside the protected application shell and avoid protected bootstrap work there.
- `ui-settings`: Add the conditional translated Authentication/Sign out section.
- `ui-i18n`: Add matching authentication strings to all supported locale dictionaries.
- `ui-pwa`: Prevent authentication/error response caching, preserve authenticated offline fallback, use backend-aware online navigation, and purge protected caches only on explicit logout or existing version lifecycle events.

## Impact

- Backend: `src/config.rs`, startup composition, `AppState`, Axum router/middleware, API error catalog, and new authentication/session/OIDC modules and endpoints.
- Frontend: root layout, new `/login` route, Settings, translations, centralized fetch/error handling, route loaders, direct AI/export/photo fetches, service worker, and cache message protocol.
- Dependencies: maintained Rust OIDC/OAuth validation and cookie/session primitives plus their transitive cryptography; no auth database, Redis, or external session service.
- Operations/docs: README and deployment examples gain optional OIDC variables and reverse-proxy/TLS guidance; auth-enabled startup performs provider discovery and fails closed on invalid configuration or discovery.
- Compatibility: auth-disabled deployments retain existing route and offline behavior. Auth-enabled process restarts invalidate all local sessions and in-flight login transactions by design.
