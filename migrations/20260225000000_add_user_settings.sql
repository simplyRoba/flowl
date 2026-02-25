CREATE TABLE user_settings (
    id INTEGER PRIMARY KEY DEFAULT 1 CHECK (id = 1),
    theme TEXT NOT NULL DEFAULT 'system',
    locale TEXT NOT NULL DEFAULT 'en'
);

INSERT INTO user_settings (id) VALUES (1);
