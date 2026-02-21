## Context

flowl stores all data in a single SQLite database (locations, plants, care_events) and photo files in an uploads directory on disk. There is no backup or restore mechanism. Users risk losing all data and photos if the volume is lost. The settings page already has a "Data" section showing plant/event counts but no actions.

## Goals / Non-Goals

**Goals:**
- Provide a ZIP export containing a JSON manifest of all user data plus photo files via a single API call.
- Provide a ZIP import that restores data and photos from a previously exported archive.
- Add export/import controls to the settings page Data section.

**Non-Goals:**
- Incremental or differential backups.
- Automatic scheduled backups.
- CSV or other format support.

## Decisions

- Export as a ZIP archive containing `data.json` at the root and photo files in a `photos/` directory.
  - **Alternative:** JSON-only with base64-encoded photos. **Rejected** — bloats file size ~33%, hard to inspect, and photos can be large.
  - **Alternative:** JSON-only without photos. **Rejected** — users would lose photos on migration, defeating the purpose of a complete backup.

- The `data.json` manifest contains `version`, `exported_at`, and arrays for `locations`, `plants`, and `care_events`. Plant entries include `photo_path` (the filename) which maps to a file in the `photos/` directory of the archive.

- Import replaces all existing data (clear + insert) and all existing photos (clear uploads directory + extract) within a single transaction for the database portion.
  - **Alternative:** Merge/upsert by matching IDs. **Rejected** — IDs are local auto-incremented integers with no stable identity across instances.

- Require a confirmation step in the UI before import to prevent accidental data loss.

- Use the crate version (`env!("CARGO_PKG_VERSION")`) as the `version` field in the export JSON. Import requires the major and minor version to match the running server — patch differences are allowed since patch bumps are bug fixes only and don't change the schema. Users must re-export after a major or minor upgrade.
  - **Alternative:** A separate integer version for the export format. **Rejected** — the crate version already tracks schema changes and avoids maintaining a parallel versioning scheme.

- Use `GET /api/data/export` for download (returns ZIP with `Content-Disposition: attachment`) and `POST /api/data/import` for upload (accepts ZIP as multipart form data). The import endpoint has a 100 MB body size limit to accommodate photo-heavy archives.

- Preserve original `created_at`/`updated_at`/`occurred_at` timestamps from the export. Do not regenerate them on import.

- Validate-before-modify: the import handler fully validates the ZIP (structure, JSON parsing, version check, filename sanitization) before touching any existing data. This avoids partial state if validation fails late.

- After a successful import, call MQTT repair to clear orphaned retained topics left by pre-import plants and republish fresh discovery configs, state, and attributes for all imported plants.

- Add `zip` crate (with `deflate` feature) as a dependency for archive creation and extraction. This is a pure-Rust implementation with no system dependencies.

## Risks / Trade-offs

- Import replaces all data and photos → Mitigation: UI shows a confirmation dialog warning that existing data will be replaced. The export button is prominently placed so users can back up first.
- Large photo collections could make the ZIP large and slow to transfer → Mitigation: practical plant collections are small (tens to low hundreds of plants, each with at most one photo). ZIP compression helps with JSON but photos are already compressed (JPEG/PNG).
- Schema evolution breaks old exports → Mitigation: strict version check rejects mismatches with a clear error. Users export before upgrading. A round-trip integration test catches any drift between the database schema and the export structs.
- ZIP extraction of untrusted files (path traversal) → Mitigation: validate all filenames in the archive, reject paths with `..` or absolute paths. Only extract files from the `photos/` prefix.

## Migration Plan

No database migration needed. New API routes, new crate dependency, and UI controls only. Deploy with standard build.
