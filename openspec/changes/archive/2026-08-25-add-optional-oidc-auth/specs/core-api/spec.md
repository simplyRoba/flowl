## ADDED Requirements

### Requirement: API authentication-required response

When OIDC authentication is enabled, every unauthenticated request under `/api/*` SHALL return HTTP 401 in the existing JSON error format with code `AUTHENTICATION_REQUIRED` and its fixed safe default message. API authentication failures SHALL include `Cache-Control: no-store` and SHALL NOT redirect to `/login`, return SPA HTML, or redirect to the OIDC provider.

#### Scenario: Unauthenticated API request

- **WHEN** authentication is enabled
- **AND** a request without a valid authenticated session is made to any `/api/*` route
- **THEN** the API responds with HTTP 401
- **AND** the body is `{"code":"AUTHENTICATION_REQUIRED","message":"Authentication is required"}`
- **AND** the response includes `Cache-Control: no-store`

#### Scenario: API request is never redirected for login

- **WHEN** an unauthenticated API request would otherwise encounter the browser login guard
- **THEN** the response remains the JSON `AUTHENTICATION_REQUIRED` error
- **AND** has no redirect to HTML or an OIDC endpoint

#### Scenario: Arbitrary unauthorized error remains distinct

- **WHEN** an API response has HTTP 401 for a reason other than a missing or expired Flowl session
- **THEN** it SHALL NOT use `AUTHENTICATION_REQUIRED`

#### Scenario: Disabled mode has no authentication error

- **WHEN** authentication is disabled
- **THEN** the API does not require a session
- **AND** existing endpoint status and JSON behavior remains unchanged
