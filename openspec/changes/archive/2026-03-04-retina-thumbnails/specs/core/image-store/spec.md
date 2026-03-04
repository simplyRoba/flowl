## MODIFIED Requirements

### Requirement: Image store service

The system SHALL provide an `ImageStore` service that manages image file storage on the local filesystem. The service SHALL accept image bytes and a content-type, validate the input, save the file with a UUID-based filename, and return the generated filename. The service SHALL support deletion of stored files by filename. When deleting a file, the service SHALL also delete any associated thumbnail variants (`{stem}_200.jpg`, `{stem}_600.jpg`, and `{stem}_1000.jpg`).

#### Scenario: Save valid JPEG image

- **WHEN** `save` is called with valid JPEG bytes and content-type `image/jpeg`
- **THEN** the file SHALL be written to the upload directory with a UUID filename ending in `.jpg`
- **AND** the generated filename SHALL be returned

#### Scenario: Save valid PNG image

- **WHEN** `save` is called with valid PNG bytes and content-type `image/png`
- **THEN** the file SHALL be written with a UUID filename ending in `.png`

#### Scenario: Save valid WebP image

- **WHEN** `save` is called with valid WebP bytes and content-type `image/webp`
- **THEN** the file SHALL be written with a UUID filename ending in `.webp`

#### Scenario: Reject invalid content type

- **WHEN** `save` is called with content-type `text/plain`
- **THEN** the service SHALL return an `InvalidContentType` error
- **AND** no file SHALL be written to disk

#### Scenario: Reject oversized file

- **WHEN** `save` is called with image bytes exceeding 5 MB
- **THEN** the service SHALL return a `TooLarge` error
- **AND** no file SHALL be written to disk

#### Scenario: Delete existing file and its thumbnails

- **WHEN** `delete` is called with filename `abc.jpg` that exists on disk
- **THEN** `abc.jpg` SHALL be removed from the upload directory
- **AND** `abc_200.jpg`, `abc_600.jpg`, and `abc_1000.jpg` SHALL also be removed if they exist

#### Scenario: Delete non-existent file

- **WHEN** `delete` is called with a filename that does not exist on disk
- **THEN** the operation SHALL log a warning and complete without error

#### Scenario: Delete when thumbnail missing

- **WHEN** `delete` is called with filename `abc.jpg`
- **AND** `abc.jpg` exists but `abc_200.jpg` does not
- **THEN** `abc.jpg` SHALL be deleted
- **AND** the missing thumbnail SHALL be silently ignored

### Requirement: Thumbnail generation on save

After writing an original image to disk, `ImageStore::save()` SHALL decode the image and generate three JPEG thumbnail variants (quality 80) with the longest edge fitting within 200px, 600px, and 1000px, preserving aspect ratio. Thumbnails SHALL be written alongside the original using the naming convention `{stem}_{size}.jpg`.

Thumbnail generation SHALL run on `spawn_blocking` to avoid blocking the Tokio runtime. If generation fails (e.g., corrupt or unsupported image data), the error SHALL be logged as a warning and the original save SHALL still succeed.

#### Scenario: Save generates 200px, 600px, and 1000px thumbnails

- **WHEN** `save` is called with a valid image and produces filename `a1b2c3.jpg`
- **THEN** the original file SHALL be written as `a1b2c3.jpg`
- **AND** a 200px thumbnail SHALL be written as `a1b2c3_200.jpg` (JPEG, quality 80, longest edge <= 200px)
- **AND** a 600px thumbnail SHALL be written as `a1b2c3_600.jpg` (JPEG, quality 80, longest edge <= 600px)
- **AND** a 1000px thumbnail SHALL be written as `a1b2c3_1000.jpg` (JPEG, quality 80, longest edge <= 1000px)

#### Scenario: PNG original produces JPEG thumbnails

- **WHEN** `save` is called with a valid PNG image producing filename `d4e5f6.png`
- **THEN** the original file SHALL be written as `d4e5f6.png`
- **AND** thumbnails SHALL be written as `d4e5f6_200.jpg`, `d4e5f6_600.jpg`, and `d4e5f6_1000.jpg` (JPEG format)

#### Scenario: Aspect ratio preserved

- **WHEN** the original image is 3000x2000 pixels
- **THEN** the 1000px thumbnail SHALL be 1000x667 pixels
- **AND** the 600px thumbnail SHALL be 600x400 pixels
- **AND** the 200px thumbnail SHALL be 200x133 pixels

#### Scenario: Thumbnail generation fails gracefully

- **WHEN** `save` is called with image data the `image` crate cannot decode
- **THEN** the original file SHALL still be saved successfully
- **AND** a warning SHALL be logged
- **AND** no thumbnail files SHALL be created

### Requirement: Startup orphan cleanup

On application startup, after database migrations have run, the `ImageStore` SHALL scan the uploads directory and delete any files not referenced by `plants.photo_path` or `care_events.photo_path`. Thumbnail files (`{stem}_200.jpg`, `{stem}_600.jpg`, and `{stem}_1000.jpg`) whose corresponding original stem matches a referenced `photo_path` SHALL NOT be treated as orphans. This provides self-healing cleanup for files orphaned by crashes or CASCADE deletes.

#### Scenario: Orphaned file removed

- **WHEN** the application starts
- **AND** the uploads directory contains a file `abc.jpg` not referenced by any `plants.photo_path` or `care_events.photo_path`
- **THEN** the file SHALL be deleted from disk

#### Scenario: Referenced file preserved

- **WHEN** the application starts
- **AND** the uploads directory contains a file `def.jpg` referenced by a plant's `photo_path`
- **THEN** the file SHALL NOT be deleted

#### Scenario: Thumbnail of referenced file preserved

- **WHEN** the application starts
- **AND** the uploads directory contains `def_200.jpg`, `def_600.jpg`, and `def_1000.jpg`
- **AND** `def.jpg` is referenced by a `photo_path` in the database
- **THEN** `def_200.jpg`, `def_600.jpg`, and `def_1000.jpg` SHALL NOT be deleted

#### Scenario: Thumbnail of orphaned file removed

- **WHEN** the application starts
- **AND** the uploads directory contains `orphan_200.jpg`, `orphan_600.jpg`, and `orphan_1000.jpg`
- **AND** no `photo_path` references `orphan.jpg`
- **THEN** `orphan_200.jpg`, `orphan_600.jpg`, and `orphan_1000.jpg` SHALL be deleted as orphans

#### Scenario: No orphans

- **WHEN** the application starts
- **AND** all files in the uploads directory are referenced by database records or are thumbnails of referenced files
- **THEN** no files SHALL be deleted

#### Scenario: Empty uploads directory

- **WHEN** the application starts
- **AND** the uploads directory is empty
- **THEN** the cleanup completes without error

### Requirement: Startup thumbnail migration

On application startup, after orphan cleanup, the system SHALL scan all `photo_path` values referenced in the database and generate any missing thumbnail variants (200px, 600px, 1000px).

#### Scenario: Missing thumbnails generated on startup

- **WHEN** the application starts
- **AND** `plants.photo_path` references `abc.jpg` but `abc_1000.jpg` does not exist on disk
- **THEN** `abc_200.jpg`, `abc_600.jpg`, and `abc_1000.jpg` SHALL be generated from `abc.jpg`

#### Scenario: Existing thumbnails skipped

- **WHEN** the application starts
- **AND** `abc_200.jpg`, `abc_600.jpg`, and `abc_1000.jpg` already exist alongside `abc.jpg`
- **THEN** no regeneration SHALL occur for `abc.jpg`

#### Scenario: Original missing on disk

- **WHEN** the application starts
- **AND** `plants.photo_path` references `missing.jpg` but the file does not exist on disk
- **THEN** thumbnail generation SHALL be skipped for that entry
- **AND** a warning SHALL be logged
