-- Recreate plants table without DEFAULT on timestamp columns.
-- This ensures timestamps are always provided explicitly from Rust.
CREATE TABLE plants_new (
    id                     INTEGER PRIMARY KEY,
    name                   TEXT    NOT NULL,
    species                TEXT,
    icon                   TEXT    NOT NULL DEFAULT '🪴',
    photo_path             TEXT,
    location_id            INTEGER REFERENCES locations(id),
    watering_interval_days INTEGER NOT NULL DEFAULT 7,
    light_needs            TEXT    NOT NULL DEFAULT 'indirect',
    difficulty             TEXT,
    pet_safety             TEXT,
    growth_speed           TEXT,
    soil_type              TEXT,
    soil_moisture          TEXT,
    notes                  TEXT,
    created_at             TEXT    NOT NULL,
    updated_at             TEXT    NOT NULL
);

INSERT INTO plants_new SELECT
    id, name, species, icon, photo_path, location_id,
    watering_interval_days, light_needs, difficulty, pet_safety,
    growth_speed, soil_type, soil_moisture, notes, created_at, updated_at
FROM plants;

DROP TABLE plants;
ALTER TABLE plants_new RENAME TO plants;

-- Recreate care_events table without DEFAULT on timestamp columns.
CREATE TABLE care_events_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    plant_id INTEGER NOT NULL REFERENCES plants(id) ON DELETE CASCADE,
    event_type TEXT NOT NULL,
    notes TEXT,
    photo_path TEXT,
    occurred_at TEXT NOT NULL,
    created_at TEXT NOT NULL
);

INSERT INTO care_events_new SELECT
    id, plant_id, event_type, notes, photo_path, occurred_at, created_at
FROM care_events;

DROP TABLE care_events;
ALTER TABLE care_events_new RENAME TO care_events;
