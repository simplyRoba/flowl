## ADDED Requirements

### Requirement: Image store service

The system SHALL provide an `ImageStore` service that manages image file storage on the local filesystem. The service SHALL accept image bytes and a content-type, validate the input, save the file with a UUID-based filename, and return the generated filename. The service SHALL support deletion of stored files by filename.

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

#### Scenario: Delete existing file

- **WHEN** `delete` is called with a filename that exists on disk
- **THEN** the file SHALL be removed from the upload directory

#### Scenario: Delete non-existent file

- **WHEN** `delete` is called with a filename that does not exist on disk
- **THEN** the operation SHALL log a warning and complete without error

### Requirement: Startup orphan cleanup

On application startup, after database migrations have run, the `ImageStore` SHALL scan the uploads directory and delete any files not referenced by `plants.photo_path` or `care_events.photo_path`. This provides self-healing cleanup for files orphaned by crashes or CASCADE deletes.

#### Scenario: Orphaned file removed

- **WHEN** the application starts
- **AND** the uploads directory contains a file `abc.jpg` not referenced by any `plants.photo_path` or `care_events.photo_path`
- **THEN** the file SHALL be deleted from disk

#### Scenario: Referenced file preserved

- **WHEN** the application starts
- **AND** the uploads directory contains a file `def.jpg` referenced by a plant's `photo_path`
- **THEN** the file SHALL NOT be deleted

#### Scenario: No orphans

- **WHEN** the application starts
- **AND** all files in the uploads directory are referenced by database records
- **THEN** no files SHALL be deleted

#### Scenario: Empty uploads directory

- **WHEN** the application starts
- **AND** the uploads directory is empty
- **THEN** the cleanup completes without error
