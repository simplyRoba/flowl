## ADDED Requirements

### Requirement: Authentication-aware SPA document access

When authentication is enabled, the server SHALL keep `/login` and exact non-data resources required for login/PWA rendering public while requiring an authenticated session before serving `index.html` for root, normal application routes, or unknown SPA fallbacks. Authentication-disabled SPA and static-file behavior SHALL remain as defined by the canonical SPA Static File Serving requirement.

#### Scenario: Public login document

- **WHEN** authentication is enabled
- **AND** an unauthenticated browser requests `/login`
- **THEN** the server responds with the SPA document needed for the public login route

#### Scenario: Public non-document resources

- **WHEN** authentication is enabled
- **AND** an unauthenticated client requests an exact immutable build asset, service worker, manifest, icon, favicon, or offline page required for login/PWA operation
- **THEN** the server serves that exact resource without authentication
- **AND** does not make `index.html` or an unknown document fallback public

#### Scenario: Protected SPA document

- **WHEN** authentication is enabled
- **AND** an unauthenticated browser navigates to `/`, `/index.html`, a normal application route, or an unknown document fallback
- **THEN** the server redirects to `/login?return_to=<safe-local-path-and-query>`
- **AND** does not serve the protected SPA document at that target

### Requirement: Authentication-aware upload access

When authentication is enabled, every `/uploads/*` request SHALL require a valid authenticated session. Authentication-disabled upload serving SHALL remain as defined by the canonical Upload File Serving requirement.

#### Scenario: Authenticated uploaded file served

- **WHEN** authentication is enabled
- **AND** a GET request with a valid session is made to `/uploads/abc.jpg`
- **AND** the file exists in the upload directory
- **THEN** the server responds with the file contents

#### Scenario: Unauthenticated upload is protected

- **WHEN** authentication is enabled
- **AND** a request without a valid session is made to `/uploads/abc.jpg`
- **THEN** the server responds with HTTP 401
- **AND** does not return file bytes or a login/provider redirect
