## Purpose

Defines Flowl's public authentication experience, safe session-expiry navigation, and logout coordination without exposing provider tokens or adding local identity management.

## Requirements

### Requirement: Public authentication configuration

The frontend SHALL obtain authentication enablement and provider display name from public `GET /auth/config` and SHALL NOT receive or retain OIDC endpoints, identifiers, secrets, tokens, claims, or session values.

#### Scenario: Authentication enabled

- **WHEN** `/auth/config` returns `enabled: true`
- **THEN** the frontend enables the login and logout experiences
- **AND** displays the returned provider name as untrusted text

#### Scenario: Authentication configuration is loading

- **WHEN** the public auth-config request is pending
- **THEN** the login page SHALL render the branded login card with a translated loading state
- **AND** SHALL NOT render or enable an OIDC provider action

#### Scenario: Authentication disabled

- **WHEN** `/auth/config` returns `enabled: false`
- **THEN** the frontend does not show authentication-only controls
- **AND** normal Flowl behavior remains unchanged

#### Scenario: Authentication configuration is unavailable

- **WHEN** the public auth-config request fails
- **THEN** the login page SHALL show a generic provider-unavailable state
- **AND** SHALL NOT render or enable an OIDC provider action

### Requirement: Public login page

The `/login` Svelte route SHALL render outside the normal protected application shell using Flowl's branding, logo, theme, and translated visual language. It SHALL show a short authentication-required message and exactly one provider action labeled `Continue with {provider_name}`, with no username, password, registration, user-list, or provider-specific fields.

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

### Requirement: Responsive two-area login composition

The `/login` page SHALL use Gazel's established login composition structurally while using only Flowl's own colors, spacing tokens, typography tokens, surfaces, shadows, and owl/sprout branding. The viewport SHALL center one login card vertically and horizontally.

#### Scenario: Mobile login card

- **WHEN** the viewport is narrower than `48rem`
- **THEN** the page uses a single-column card with `width: min(100%, 400px)`
- **AND** the card is centered within a `100dvh` minimum-height page with Flowl-token outer padding
- **AND** branding/copy appears above the action area with centered text

#### Scenario: Desktop two-column login card

- **WHEN** the viewport is at least `48rem` wide
- **THEN** the centered card uses `width: min(100%, 880px)` and `min-height: 380px`
- **AND** it uses two columns sized `minmax(0, 1.15fr) minmax(300px, 0.85fr)`
- **AND** text alignment changes to left within the left column

#### Scenario: Left branding and copy column

- **WHEN** the login card renders
- **THEN** its first area contains the Flowl logo/wordmark, login heading, and short authentication-required copy in that order
- **AND** at desktop width the area vertically centers those elements, uses a Flowl feature/surface background, and has a separating right border

#### Scenario: Right raised action panel

- **WHEN** the login card renders
- **THEN** its second area contains one full-width raised panel using Flowl surface, border, and medium-shadow tokens
- **AND** the panel contains the optional generic status message above exactly one full-width provider button
- **AND** at desktop width the action area centers the raised panel vertically within the right column

#### Scenario: Status and provider action ordering

- **WHEN** authentication-failed, provider-unavailable, or logged-out state is present
- **THEN** the translated status appears inside the raised action panel before the provider action
- **AND** the provider button remains full width and is the only authentication action

### Requirement: Centralized authentication-expiry handling

All frontend programmatic requests for protected API, export, import, AI, upload, protected-photo, and route-load resources SHALL pass non-success responses through one authentication-required handler. Only a response with both HTTP 401 and JSON code `AUTHENTICATION_REQUIRED` SHALL navigate to `/login?return_to=<current-local-path-query-hash>` during normal online operation.

#### Scenario: Authentication-required API response

- **WHEN** a protected frontend request receives HTTP 401 with code `AUTHENTICATION_REQUIRED`
- **THEN** the browser navigates to `/login`
- **AND** preserves the validated current path, query, and hash as `return_to`

#### Scenario: Arbitrary 401 response

- **WHEN** a request receives HTTP 401 without code `AUTHENTICATION_REQUIRED`
- **THEN** the central handler does not start login navigation

#### Scenario: Other API failure

- **WHEN** a request receives a non-401 API error
- **THEN** the central handler does not start login navigation
- **AND** existing localized error behavior remains available

#### Scenario: Network failure

- **WHEN** a request fails without an HTTP response because of temporary network loss
- **THEN** the central handler does not start login navigation
- **AND** existing connectivity/offline handling runs

#### Scenario: Direct request paths use the handler

- **WHEN** export, AI identify, AI chat, protected-photo conversion, upload, or a plant route loader receives `401 AUTHENTICATION_REQUIRED`
- **THEN** it performs the same login navigation as an ordinary API request
- **AND** does not silently convert the response into a download, stream, blob, or generic page error

### Requirement: Frontend return target safety

The frontend SHALL apply the same bounded safe-local-target rules as the backend before sending or following `return_to`. It MAY preserve the current `location.hash`, but SHALL fall back to `/` for external, protocol-relative, malformed, control-character, backslash, oversized, login, or auth targets.

#### Scenario: Current SPA location preserved

- **WHEN** authentication expires at `/plants/42?tab=care#entry-7`
- **THEN** the generated local target preserves `/plants/42?tab=care#entry-7`

#### Scenario: Unsafe login target discarded

- **WHEN** the login page receives an unsafe or auth-recursive `return_to`
- **THEN** the provider action uses `/` as its target

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

The frontend SHALL use only the provider display name and generic result states. It SHALL NOT store OIDC tokens, authorization codes, state, nonce, PKCE values, client secrets, identity claims, or session IDs in JavaScript-readable storage, URLs it constructs, service-worker caches, logs, or analytics.

#### Scenario: Successful login leaves no frontend token

- **WHEN** the OIDC callback succeeds
- **THEN** the browser receives only the secure session cookie and redirect location from the backend
- **AND** no provider token or identity claim is exposed to frontend JavaScript
