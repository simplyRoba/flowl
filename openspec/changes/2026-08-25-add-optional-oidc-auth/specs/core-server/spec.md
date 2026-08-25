## MODIFIED Requirements

### Requirement: SPA Static File Serving

The server SHALL serve embedded SvelteKit build assets with correct MIME types. When authentication is disabled, any request that does not match an API route SHALL retain the existing `index.html` SPA fallback. When authentication is enabled, `/login` and exact non-document static assets required to render it SHALL remain public, while root, `index.html`, unknown paths, and all normal application SPA navigations SHALL require an authenticated session before receiving the SPA document.

#### Scenario: Root path serves SPA

- **WHEN** authentication is disabled
- **AND** a GET request is made to `/`
- **THEN** the server responds with the SvelteKit `index.html`

#### Scenario: Static asset served

- **WHEN** a GET request is made to a path matching an embedded non-document static file such as `/_app/immutable/entry/start.js`
- **THEN** the server responds with the file contents and correct MIME type
- **AND** login/PWA resources such as the service worker, manifest, icons, offline page, and immutable build assets remain available without authentication
- **AND** `index.html` remains a protected application document except when served specifically for `/login`

#### Scenario: Unknown path falls back to SPA

- **WHEN** authentication is disabled
- **AND** a GET request is made to a path that does not match any API route or static file
- **THEN** the server responds with the SvelteKit `index.html` for client-side routing

#### Scenario: Public login document with auth enabled

- **WHEN** authentication is enabled
- **AND** an unauthenticated browser requests `/login`
- **THEN** the server responds with the SPA document needed for the public login route

#### Scenario: Protected SPA document with auth enabled

- **WHEN** authentication is enabled
- **AND** an unauthenticated browser navigates to `/`, `index.html`, an unknown path, or a normal application SPA route
- **THEN** the server redirects to `/login?return_to=<safe-local-path-and-query>`
- **AND** does not serve the protected SPA document at that target

#### Scenario: API routes handled by API router

- **WHEN** a request is made to a path starting with `/api`
- **THEN** the request is handled by the nested API router
- **AND** does not fall through to the SPA handler

### Requirement: Upload File Serving

The server SHALL serve existing files from the upload directory at `/uploads/*` using the static file service. When authentication is enabled, every `/uploads/*` request SHALL require a valid authenticated session and unauthenticated requests SHALL return HTTP 401 without serving or redirecting to the file. When authentication is disabled, existing public upload behavior SHALL remain unchanged.

#### Scenario: Uploaded file served

- **WHEN** authentication is disabled
- **AND** a GET request is made to `/uploads/abc.jpg`
- **AND** the file exists in the upload directory
- **THEN** the server responds with the file contents

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
