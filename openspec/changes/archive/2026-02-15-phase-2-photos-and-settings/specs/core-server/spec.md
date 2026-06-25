## Purpose

Server — MODIFIED to use `AppState` and serve uploaded files.

## Requirements

### Requirement: AppState

The server SHALL use an `AppState` struct containing `SqlitePool` and `PathBuf` (upload directory) as the shared state for all routes.

#### Scenario: AppState constructed

- **WHEN** the application starts
- **THEN** `AppState` is created with the database pool and upload directory path
- **AND** the upload directory is created if it does not exist

### Requirement: Upload File Serving

The server SHALL serve files from the upload directory at `/uploads/*` using `tower-http::ServeDir`.

#### Scenario: Uploaded file served

- **WHEN** a GET request is made to `/uploads/abc.jpg`
- **AND** the file exists in the upload directory
- **THEN** the server responds with the file contents
