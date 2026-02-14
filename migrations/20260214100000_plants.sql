CREATE TABLE locations (
    id   INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE plants (
    id                     INTEGER PRIMARY KEY,
    name                   TEXT    NOT NULL,
    species                TEXT,
    icon                   TEXT    NOT NULL DEFAULT '🪴',
    location_id            INTEGER REFERENCES locations(id),
    watering_interval_days INTEGER NOT NULL DEFAULT 7,
    light_needs            TEXT    NOT NULL DEFAULT 'indirect',
    notes                  TEXT,
    created_at             TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at             TEXT    NOT NULL DEFAULT (datetime('now'))
);
