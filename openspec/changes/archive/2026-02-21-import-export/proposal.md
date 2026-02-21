## Why

There is no way to back up or restore plant data. If the database file is lost or corrupted, all plants, care history, locations, and photos are gone. Users also need a migration path when moving to a new host or reinstalling the container.

## What Changes

- Add a backend API endpoint to export all data (plants, locations, care events, photos) as a single ZIP archive download.
- Add a backend API endpoint to import a previously exported ZIP archive, restoring all data and photos.
- Add export and import controls to the Data section of the settings page.

## Capabilities

### New Capabilities
- `core/backup`: ZIP export containing a JSON manifest of all user data plus photo files.
- `core/restore`: ZIP import that replaces all existing data and photos from a previously exported archive.

### Modified Capabilities
- `ui/settings`: Add export/import buttons to the existing Data section.

## Impact

- Backend: new `src/api/backup.rs` and `src/api/restore.rs` modules with handlers, two new routes in `src/api/mod.rs`.
- New dependency: `zip` crate for archive creation/extraction.
- UI: updated settings page with download and upload controls in the Data section.
- No schema changes — reads and writes existing tables (locations, plants, care_events) and the uploads directory.
