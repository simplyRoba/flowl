# Spec Delta

## MODIFIED Requirements

### Requirement: Authentication-aware SPA document access

When authentication is enabled, the server SHALL keep `/login` accessible without authentication and exact non-data resources required for login/PWA rendering public while requiring an authenticated session before serving `index.html` for root, normal application routes, or unknown browser UI document fallbacks. Authentication-disabled browser UI document and asset behavior SHALL remain as defined by the canonical Browser UI document and asset serving requirement.

#### Scenario: Public login document

- **WHEN** authentication is enabled
- **AND** an unauthenticated browser requests `/login`
- **THEN** the server responds with the SPA document needed for the public login route

#### Scenario: Public non-document resources

- **WHEN** authentication is enabled
- **AND** an unauthenticated client requests an exact app asset, service worker, manifest, icon, favicon, or offline page required for login/PWA operation
- **THEN** the server serves that exact resource without authentication
- **AND** does not make `index.html` or an unknown browser UI document fallback public

#### Scenario: Protected SPA document

- **WHEN** authentication is enabled
- **AND** an unauthenticated browser navigates to `/`, `/index.html`, a normal application route, or an unknown document fallback
- **THEN** the server redirects to `/login?return_to=<safe-local-path-and-query>`
- **AND** does not serve the protected SPA document at that target

#### Scenario: Authenticated login document bypass

- **WHEN** authentication is enabled
- **AND** a browser with a valid authenticated session requests `/login`, including with result-state or return-target query parameters
- **THEN** the server responds with HTTP 303 to `/` without serving login HTML
- **AND** does not initiate provider authentication, create a transaction, mutate the session, rotate its identifier, or shorten or extend its absolute deadline
