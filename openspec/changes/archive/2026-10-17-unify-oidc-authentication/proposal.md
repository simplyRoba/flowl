# Proposal

## Why

Flowl and Gazel implement the same identity-neutral OIDC access gate, but independent implementations have accumulated different security guarantees and edge-case behavior. Consolidate them before adding optional automatic provider redirection so future authentication changes can be reviewed and tested against one common contract.

## What Changes

- Converge both repositories on synchronized authentication modules and common behavioral test vectors, retaining standalone builds rather than introducing a shared crate or sibling-path dependency.
- Adopt Gazel's guarded outbound transport and explicit component boundaries without losing Flowl's bounded transactions, coordinated session persistence, explicit claim checks, cache protection, or offline behavior.
- Validate every backend OIDC destination before network contact, require standalone Authorization Code response support and compatible advertised grants, and reject fragments in provider endpoints.
- Bypass `/login` and `/auth/login` for valid authenticated sessions without initiating OIDC, changing the session identifier, or shortening its absolute deadline.
- Reject malformed, duplicate, or mixed callback parameters before token exchange; accept both distinct standard provider-error detail fields. Specify session persistence, replacement, and logout/callback concurrency outcomes.
- **BREAKING**: Standardize disabled `GET /auth/config` to exactly `{ "enabled": false }` rather than including `provider_name: null`; update frontend parsing in the same release.
- **BREAKING**: Trim provider labels, reject control characters, and use an 80-Unicode-scalar limit instead of the existing 128-byte rule. Reject present non-Unicode auth configuration rather than treating it as absent.
- **BREAKING**: Apply the common return-target contract, including API/health and managed-media destination exclusions; rejected targets fall back to `/`.
- Preserve production HTTPS-only configuration, the Flowl cookie name, protected APIs/uploads, exact public login/PWA resources, and explicit cache purge before local logout. Preserve Gazel's distinct HTTP-loopback support in the coordinated peer change.
- Defer optional automatic provider redirection, distributed sessions, local users, provider logout, shared-crate packaging, and configurable network resource limits.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `core-authentication`: Common configuration/display validation, pre-request transport guards, discovery, authenticated-login bypass, callback/session lifecycle, public config shape, safe return targets, and logout failure/concurrency guarantees.
- `core-server`: Authenticated login-document bypass while preserving protected document/media and public PWA boundaries.
- `ui-authentication`: Strict public config parsing, authenticated-login bypass, equivalent safe-return rules, and manual login/result-state behavior without changing offline-cache privacy policy.

## Impact

- Backend: `src/auth/**`, auth-specific parsing in `src/config.rs`, and auth integration in `src/server.rs`; existing API error codes and fallback text remain compatible.
- Frontend: `ui/src/lib/auth.ts`, login page/config consumers, and associated authentication tests. Existing service-worker policy and acknowledged logout purge remain unchanged.
- Tests: local provider fixtures, transport no-contact assertions, authenticated-login expiry regression, callback/session concurrency and failure injection, common vectors, and cross-repository synchronization checks.
- Documentation: public config compatibility and provider-label validation changes. No database migration, new service, new auth framework, or runtime cross-repository dependency.
- Coordinated peer: Gazel's `openspec/changes/unify-oidc-authentication/`; implementation is separately reviewable in each repository and must not merge or push without human approval.
