# Spec Delta

## MODIFIED Requirements

### Requirement: Public authentication configuration

The frontend SHALL accept public `GET /auth/config` only as exactly `{ "enabled": true, "provider_name": <string> }` or exactly `{ "enabled": false }`. It SHALL obtain enablement and provider display name from this response and SHALL NOT receive or retain OIDC endpoints, identifiers, secrets, tokens, claims, or session values.

#### Scenario: Authentication enabled

- **WHEN** `/auth/config` returns exactly `enabled: true` and a string `provider_name`
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

- **WHEN** `/auth/config` returns non-boolean enablement, a missing or non-string enabled provider name, unexpected fields, or disabled configuration containing `provider_name` even as null
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

#### Scenario: Provider action is manual

- **WHEN** an unauthenticated user opens the login page with valid enabled configuration
- **THEN** the page waits for explicit activation of the provider action before starting `/auth/login`
- **AND** does not automatically redirect to the provider

#### Scenario: Result states do not restart authentication

- **WHEN** the login page displays authentication-failed, provider-unavailable, or logged-out state
- **THEN** it waits for a deliberate provider-action activation to retry authentication
- **AND** neither immediately nor after a delay automatically starts login
- **AND** displays only generic translated status text, never callback or provider error details


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


## ADDED Requirements

### Requirement: Logout failures are not signed-out success

The frontend SHALL treat an HTTP 500 response from `POST /auth/logout` as a logout failure, not as a successful signed-out result. It SHALL retain the existing requirement to await protected-cache purge acknowledgement before posting logout and SHALL NOT change service-worker caching policy.

#### Scenario: Backend logout fails after cache purge

- **WHEN** protected cache purge has completed with acknowledgement and the subsequent logout POST returns HTTP 500
- **THEN** the frontend exposes a generic logout failure instead of navigating as though logout succeeded
- **AND** does not show a signed-out confirmation or automatically restart provider login
