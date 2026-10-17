## Purpose

Defines Flowl's public authentication experience, safe session-expiry navigation, and logout coordination without exposing provider tokens or adding local identity management.

## Requirements

### Requirement: Public authentication configuration

The frontend SHALL accept public `GET /auth/config` only as exactly `{ "enabled": true, "provider_name": <string>, "auto_redirect": <boolean> }` or exactly `{ "enabled": false }`. It SHALL obtain enablement, provider display name, and the automatic-login flag from this response and SHALL NOT receive or retain OIDC endpoints, identifiers, secrets, tokens, claims, or session values.

#### Scenario: Authentication enabled

- **WHEN** `/auth/config` returns exactly `enabled: true`, a string `provider_name`, and a boolean `auto_redirect`
- **THEN** the frontend enables the login and logout experiences
- **AND** displays the returned provider name as untrusted text

#### Scenario: Authentication configuration is loading

- **WHEN** the public auth-config request is pending
- **THEN** the login page SHALL render the branded login card with a translated loading state
- **AND** SHALL NOT render or enable an OIDC provider action

#### Scenario: Authentication disabled

- **WHEN** `/auth/config` returns exactly `{ "enabled": false }`
- **THEN** the frontend does not show authentication-only controls
- **AND** normal Flowl behavior remains unchanged

#### Scenario: Authentication configuration is unavailable

- **WHEN** the public auth-config request fails
- **THEN** the login page SHALL show a generic provider-unavailable state
- **AND** SHALL NOT render or enable an OIDC provider action

#### Scenario: Invalid configuration shape fails closed

- **WHEN** `/auth/config` returns missing or non-boolean enablement, a missing or non-string enabled provider name, missing or non-boolean enabled `auto_redirect` (including the old enabled shape), unexpected fields, or disabled configuration containing any extra field even as null
- **THEN** the frontend treats configuration as unavailable
- **AND** shows the generic provider-unavailable state without rendering or enabling the provider action

### Requirement: Public login page

For unauthenticated users, the `/login` Svelte route SHALL render outside the normal protected application shell using Flowl's branding, logo, theme, and translated visual language. It SHALL show a short authentication-required message and exactly one provider action labeled `Continue with {provider_name}`, with no username, password, registration, user-list, or provider-specific fields.

#### Scenario: Default login page

- **WHEN** an unauthenticated user opens `/login` without a result state
- **THEN** the page displays Flowl branding and an authentication-required message
- **AND** shows one button labeled with the configured provider name
- **AND** the button starts `GET /auth/login` with the validated local `return_to`

#### Scenario: Authentication failed state

- **WHEN** `/login` is opened with the generic authentication-failed state
- **THEN** the page displays a translated generic authentication-failed message
- **AND** does not expose protocol or provider diagnostics

#### Scenario: Provider unavailable state

- **WHEN** `/login` is opened with the generic provider-unavailable state
- **THEN** the page displays a translated provider-unavailable message
- **AND** retains the provider action for a later retry

#### Scenario: Logged-out state

- **WHEN** `/login?logged_out=1` is opened
- **THEN** the page displays a translated signed-out confirmation
- **AND** retains the provider action

#### Scenario: Login page while auth is disabled

- **WHEN** `/login` loads and public auth configuration reports `enabled: false`
- **THEN** the frontend navigates to `/`

#### Scenario: Authenticated login page bypass

- **WHEN** a browser with a valid authenticated session requests `/login`
- **THEN** it receives HTTP 303 to `/` without login HTML
- **AND** no provider authentication starts and the session identifier and absolute deadline remain unchanged

#### Scenario: Provider action is manual by default

- **WHEN** an unauthenticated user opens the login page with valid enabled configuration and `auto_redirect: false`
- **THEN** the page waits for explicit activation of the provider action before starting `/auth/login`
- **AND** does not automatically redirect to the provider

#### Scenario: Result states do not restart authentication

- **WHEN** the login page displays authentication-failed, provider-unavailable, or logged-out state
- **THEN** it waits for a deliberate provider-action activation to retry authentication
- **AND** neither immediately nor after a delay automatically starts login
- **AND** displays only generic translated status text, never callback or provider error details

### Requirement: Optional automatic login uses the validated provider action

On ordinary `/login` entry with valid enabled public configuration and `auto_redirect: true`, the mounted page SHALL perform exactly one top-level navigation to `/auth/login`, replacing the login history entry and using the same validated, serialized `return_to` as the manual button. This SHALL apply to direct browser entry and SPA/API-expiry entry. The page SHALL NOT navigate automatically when the query contains `error` or `logged_out`, regardless of value, while configuration is loading, failed, or malformed, or after unmount.

#### Scenario: Ordinary automatic-login entry

- **WHEN** a direct browser entry or SPA/API-expiry navigation reaches `/login` without `error` or `logged_out` and valid enabled config reports `auto_redirect: true`
- **THEN** the page replaces the login history entry with exactly one top-level navigation to the manual button's `/auth/login` target
- **AND** preserves the same validated `return_to`, including `/` for absent or unsafe targets

#### Scenario: Result query suppresses automatic login

- **WHEN** `/login` contains `error` or `logged_out`, including an empty or unknown value, even with `auto_redirect: true`
- **THEN** the page does not automatically start OIDC
- **AND** existing generic result messages and the manual retry action remain unchanged

#### Scenario: Configuration cannot trigger navigation after unmount

- **WHEN** config is pending, fails, is malformed, or completes after the login page unmounts
- **THEN** the page does not automatically navigate to `/auth/login`

### Requirement: Responsive two-area login composition

The `/login` page SHALL use Flowl's established login composition and branding. It SHALL present one login card centered in the available viewport with a clear, readable hierarchy.

#### Scenario: Mobile login card

- **WHEN** the viewport is below `48rem` wide
- **THEN** the page presents a single-area login card that fits the available width without clipping or horizontal overflow
- **AND** the card remains centered and visibly separated from the viewport edges
- **AND** branding and explanatory copy appear above the action area with centered text
- **AND** all copy and authentication actions remain visible and reachable

#### Scenario: Desktop two-column login card

- **WHEN** the viewport is `48rem` wide or wider
- **THEN** the centered card presents separate branding and action areas side by side
- **AND** their content does not clip, overlap, or cause horizontal overflow
- **AND** the branding-area text is left-aligned

#### Scenario: Left branding and copy column

- **WHEN** the login card renders
- **THEN** its first area contains the Flowl logo/wordmark, login heading, and short authentication-required copy in that order
- **AND** at desktop width those elements are vertically centered and the branding area is visually distinct from the action area

#### Scenario: Right raised action panel

- **WHEN** the login card renders
- **THEN** its second area contains one visually elevated action panel that uses the available area width
- **AND** the panel contains the optional generic status message above exactly one provider button that uses the available panel width
- **AND** at desktop width the action panel is vertically centered within the action area

#### Scenario: Status and provider action ordering

- **WHEN** authentication-failed, provider-unavailable, or logged-out state is present
- **THEN** the translated status appears inside the raised action panel before the provider action
- **AND** the provider button remains full width and is the only authentication action

### Requirement: Consistent authentication-expiry handling

All frontend programmatic requests for protected API, export, import, AI, upload, protected-photo, and route-load resources SHALL handle authentication failures consistently. Only a response with both HTTP 401 and JSON code `AUTHENTICATION_REQUIRED` SHALL navigate to `/login?return_to=<current-local-path-query-hash>` during normal online operation.

#### Scenario: Authentication-required API response

- **WHEN** a protected frontend request receives HTTP 401 with code `AUTHENTICATION_REQUIRED`
- **THEN** the browser navigates to `/login`
- **AND** preserves the validated current path, query, and hash as `return_to`

#### Scenario: Arbitrary 401 response

- **WHEN** a request receives HTTP 401 without code `AUTHENTICATION_REQUIRED`
- **THEN** the frontend does not start login navigation

#### Scenario: Other API failure

- **WHEN** a request receives a non-401 API error
- **THEN** the frontend does not start login navigation
- **AND** existing localized error behavior remains available

#### Scenario: Network failure

- **WHEN** a request fails without an HTTP response because of temporary network loss
- **THEN** the frontend does not start login navigation
- **AND** existing connectivity/offline handling runs

#### Scenario: Direct request paths handle authentication consistently

- **WHEN** export, AI identify, AI chat, protected-photo conversion, upload, or a plant route loader receives `401 AUTHENTICATION_REQUIRED`
- **THEN** it performs the same login navigation as an ordinary API request
- **AND** does not silently convert the response into a download, stream, blob, or generic page error

### Requirement: Frontend return target safety

The frontend SHALL apply the same bounded safe-local-target rules as the backend before sending or following `return_to`, preserving accepted serialized targets unchanged. SPA-originated targets SHALL preserve a valid current `location.hash`. Invalid targets SHALL fall back to `/`, including external, protocol-relative, malformed, control-character, backslash, oversized, or reserved namespace targets.

#### Scenario: Current SPA location preserved

- **WHEN** authentication expires at `/plants/42?tab=care#entry-7`
- **THEN** the generated local target preserves `/plants/42?tab=care#entry-7`

#### Scenario: Unsafe login target discarded

- **WHEN** the login page receives an unsafe or auth-recursive `return_to`
- **THEN** the provider action uses `/` as its target

#### Scenario: Decode query once and measure serialized target

- **WHEN** the frontend receives `return_to` in the login query
- **THEN** it form-decodes the query exactly once to obtain the serialized target
- **AND** checks that target is at most 2048 UTF-8 bytes before validation-only decoding
- **AND** does not count wire query encoding toward the limit or decode the accepted target before passing it on

#### Scenario: Unsafe path representations rejected

- **WHEN** a target lacks exactly one leading `/`, has dot segments, has encoded path delimiters `/`, backslash, `?`, or `#`, or has residual `%` from nested path escaping
- **THEN** the frontend uses `/` instead

#### Scenario: Malformed encoding and Unicode controls rejected

- **WHEN** a serialized target has malformed percent escapes, invalid UTF-8, raw or decoded backslashes, or raw or decoded controls U+0000–U+001F or U+007F–U+009F
- **THEN** the frontend uses `/` instead

#### Scenario: Reserved namespace roots and descendants rejected

- **WHEN** the target names `/auth`, `/login`, `/api`, `/health`, or `/uploads`, or any descendant or encoded equivalent
- **THEN** the frontend uses `/` instead
- **AND** does not reject an unrelated path such as `/apiary` merely because it shares a prefix

#### Scenario: Valid escaped query and fragment preserved

- **WHEN** the serialized target is `/plants/42?label=a%20b&ratio=100%25#entry%20one`
- **THEN** the frontend accepts it unchanged
- **AND** a decoded percent sign in a valid query value alone does not cause rejection

#### Scenario: Shared backend and frontend vectors

- **WHEN** frontend and backend validate the same serialized targets, including C1 controls, nested path escapes, valid query escapes, and the UTF-8 byte-length boundary
- **THEN** acceptance, fallback, and preservation of the accepted representation agree
- **AND** only SPA-originated navigation preserves a fragment that was not present in the backend request

### Requirement: Explicit logout coordination

When authentication is enabled, the Settings Sign out action SHALL clear protected offline API, photo, and runtime application caches, then POST to `/auth/logout` for local session invalidation and allow the backend's 303 redirect to `/login?logged_out=1`. Public login assets, theme, locale, manifest, icons, and service-worker update metadata SHALL remain available.

#### Scenario: Sign out succeeds

- **WHEN** the user activates Sign out while online
- **THEN** protected caches are cleared with completion acknowledgement
- **AND** the browser POSTs to `/auth/logout`
- **AND** reaches `/login?logged_out=1`

#### Scenario: Logout cache purge does not remove preferences

- **WHEN** explicit logout clears protected offline content
- **THEN** locally stored theme and locale preferences remain
- **AND** the public login page can still render

### Requirement: Authentication UI privacy

The frontend SHALL use only public authentication enablement, the provider display name, the automatic-login boolean, and generic result states. It SHALL NOT store OIDC tokens, authorization codes, state, nonce, PKCE values, client secrets, identity claims, or session IDs in JavaScript-readable storage, URLs it constructs, service-worker caches, logs, or analytics.

#### Scenario: Successful login leaves no frontend token

- **WHEN** the OIDC callback succeeds
- **THEN** the browser receives only the secure session cookie and redirect location from the backend
- **AND** no provider token or identity claim is exposed to frontend JavaScript

### Requirement: Logout failures are not signed-out success

The frontend SHALL treat an HTTP 500 response from `POST /auth/logout` as a logout failure, not as a successful signed-out result. It SHALL retain the existing requirement to await protected-cache purge acknowledgement before posting logout and SHALL NOT change service-worker caching policy.

#### Scenario: Backend logout fails after cache purge

- **WHEN** protected cache purge has completed with acknowledgement and the subsequent logout POST returns HTTP 500
- **THEN** the frontend exposes a generic logout failure instead of navigating as though logout succeeded
- **AND** does not show a signed-out confirmation or automatically restart provider login
