## Purpose

Managed image behavior — validate accepted uploads, preserve originals, provide canonical responsive renditions, and recover referenced media.

## Requirements

### Requirement: Managed image acceptance and originals

The system SHALL accept JPEG, PNG, and WebP image uploads no larger than 5 MB. The declared content type SHALL match the image bytes. For a valid upload, the system SHALL persist the original image and return an opaque media reference. The owner SHALL retain that reference for association, URL derivation, deletion, reconciliation, and archive preservation. An original identified by `{reference}` SHALL remain available at `/uploads/{reference}`.

#### Scenario: Save a valid supported image

- **WHEN** a valid JPEG, PNG, or WebP image no larger than 5 MB is submitted with a matching content type
- **THEN** the original image is persisted
- **AND** an opaque media reference is returned for association with the owner
- **AND** the original is available at `/uploads/{reference}`

#### Scenario: Reject an unsupported declared content type

- **WHEN** an image upload declares a content type other than JPEG, PNG, or WebP
- **THEN** the upload is rejected
- **AND** no original image is persisted

#### Scenario: Reject a mismatched content type

- **WHEN** an image upload's declared content type does not match its bytes
- **THEN** the upload is rejected
- **AND** no original image is persisted

#### Scenario: Reject an oversized image

- **WHEN** an image upload exceeds 5 MB
- **THEN** the upload is rejected
- **AND** no original image is persisted

### Requirement: Canonical rendition URLs

For an original image URL `/uploads/{stem}.{ext}`, the system SHALL provide three canonical derived JPEG rendition URLs: `/uploads/{stem}_200.jpg`, `/uploads/{stem}_600.jpg`, and `/uploads/{stem}_1000.jpg`. These URLs are the cross-layer contract for managed responsive image renditions.

#### Scenario: Canonical URLs for a JPEG original

- **WHEN** an original is available at `/uploads/a1b2c3.jpg`
- **THEN** its canonical rendition URLs are `/uploads/a1b2c3_200.jpg`, `/uploads/a1b2c3_600.jpg`, and `/uploads/a1b2c3_1000.jpg`

#### Scenario: Canonical URLs for a non-JPEG original

- **WHEN** an original is available at `/uploads/d4e5f6.png` or `/uploads/d4e5f6.webp`
- **THEN** its canonical rendition URLs use the same stem and the `.jpg` extension

### Requirement: Generated renditions

The system SHALL generate the three canonical renditions as JPEG images with maximum longest-edge dimensions of 200, 600, and 1000 pixels while preserving aspect ratio. Rendition generation SHALL not degrade request-serving responsiveness.

If rendition generation fails, the original SHALL remain available and the failure SHALL be logged as a warning.

#### Scenario: Generate all rendition sizes

- **WHEN** a valid original image is persisted
- **THEN** its 200, 600, and 1000 pixel canonical JPEG renditions are generated

#### Scenario: Preserve aspect ratio

- **WHEN** an original image is 3000 by 2000 pixels
- **THEN** its 1000, 600, and 200 pixel renditions are 1000 by 667, 600 by 400, and 200 by 133 pixels respectively

#### Scenario: Rendition generation fails

- **WHEN** rendition generation cannot process a persisted original
- **THEN** the original remains available
- **AND** the failure is logged as a warning

### Requirement: Managed media deletion

Deleting managed media SHALL delete its original and associated canonical renditions. An absent original SHALL be logged as a warning and SHALL NOT cause the operation to fail. An absent rendition SHALL be silently ignored. Any other deletion failure SHALL be logged at error level without causing the operation to fail, so later cleanup can retry it.

#### Scenario: Delete associated media

- **WHEN** managed media with an original and canonical renditions is deleted
- **THEN** the original and all associated canonical renditions are removed

#### Scenario: Original is already absent

- **WHEN** deletion is requested for managed media whose original is already absent
- **THEN** a warning is logged
- **AND** the operation completes without error

#### Scenario: A rendition is already absent

- **WHEN** managed media is deleted while an associated canonical rendition is already absent
- **THEN** the original and any other present canonical renditions are removed
- **AND** the absent rendition is silently ignored

#### Scenario: Deletion failure is recoverable

- **WHEN** deleting an original or rendition fails for an unexpected reason
- **THEN** the failure is logged at error level
- **AND** the operation completes without error
- **AND** later cleanup can retry the deletion

### Requirement: Media recovery and cleanup

At application startup, once logical media references are available, the system SHALL reconcile managed media before regenerating renditions. Reconciliation SHALL remove unreferenced originals and unreferenced or invalid rendition lookalikes while never removing a referenced original or its canonical renditions. The system SHALL then regenerate missing canonical renditions for referenced originals. If a referenced original is absent, regeneration SHALL be skipped and a warning logged.

#### Scenario: Unreferenced media is removed

- **WHEN** startup reconciliation finds an original with no logical media reference
- **THEN** that original and any associated rendition lookalikes are removed

#### Scenario: Referenced media is preserved

- **WHEN** startup reconciliation finds a logically referenced original and its canonical renditions
- **THEN** none of them are removed

#### Scenario: Invalid rendition lookalike is removed

- **WHEN** startup reconciliation finds a rendition-like asset that is not a canonical rendition of a referenced original
- **THEN** the asset is removed

#### Scenario: Missing renditions are regenerated

- **WHEN** startup reconciliation has completed
- **AND** a logically referenced original is present but one or more canonical renditions are missing
- **THEN** the missing canonical renditions are regenerated

#### Scenario: Existing renditions are retained

- **WHEN** a logically referenced original and all canonical renditions are present at startup
- **THEN** no renditions are regenerated for that original

#### Scenario: Referenced original is absent

- **WHEN** a logical media reference has no available original
- **THEN** rendition regeneration is skipped
- **AND** a warning is logged

#### Scenario: No managed media exists

- **WHEN** startup reconciliation finds no managed media
- **THEN** reconciliation completes without error
