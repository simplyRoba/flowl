# Spec Delta

## ADDED Requirements

### Requirement: MQTT Plant Link Origin

Flowl SHALL use `FLOWL_EXTERNAL_URL` as the browser-facing origin for MQTT plant links. With OIDC enabled, its existing required HTTPS origin rules apply. With only MQTT enabled, Flowl SHALL accept an optional HTTP or HTTPS origin with a host and optional port, with or without a trailing slash, and no credentials, subpath, query, or fragment. If unset, publish without links; if invalid in MQTT-only mode, warn without logging the value and publish without links.

#### Scenario: Local HTTP origin without OIDC

- **GIVEN** OIDC is disabled and MQTT is enabled
- **AND** `FLOWL_EXTERNAL_URL` is `http://flowl.lan:4100/`
- **WHEN** MQTT attributes for plant 7 are synchronized
- **THEN** `plant_url` is `http://flowl.lan:4100/plants/7`

#### Scenario: Optional origin not set

- **GIVEN** OIDC is disabled
- **WHEN** MQTT attributes are synchronized with `FLOWL_EXTERNAL_URL` unset
- **THEN** the payload omits `plant_url`
- **AND** the existing watering attributes are still published

#### Scenario: Invalid optional origin

- **GIVEN** OIDC is disabled and MQTT is enabled
- **WHEN** Flowl starts with `FLOWL_EXTERNAL_URL` set to an empty, non-Unicode, relative, non-HTTP(S), credential-bearing, non-root, query-bearing, or fragment-bearing value
- **THEN** Flowl warns that the setting is invalid without logging the supplied value
- **AND** MQTT synchronization continues without `plant_url`

#### Scenario: OIDC and MQTT share the same origin

- **GIVEN** OIDC and MQTT are enabled with a valid OIDC configuration and `FLOWL_EXTERNAL_URL=https://flowl.example.org`
- **WHEN** Flowl starts and synchronizes plant 7
- **THEN** the OIDC callback URL is `https://flowl.example.org/auth/callback`
- **AND** `plant_url` is `https://flowl.example.org/plants/7`

## MODIFIED Requirements

### Requirement: MQTT Attributes Publishing

The application SHALL publish plant watering attributes as a retained JSON object to a dedicated attributes topic whenever MQTT is enabled. With a valid MQTT plant link origin, every non-empty attributes payload SHALL also contain `plant_url`, an absolute string URL pointing to the browser detail route `/plants/{id}` for the same plant.

#### Scenario: Attributes published

- **GIVEN** a plant with id 1, `last_watered` = `2026-02-13T14:30:00`, `watering_interval_days` = 7, `next_due` = `2026-02-20`
- **AND** the MQTT topic prefix is `flowl`
- **AND** MQTT is enabled
- **WHEN** attributes are synchronized
- **THEN** a retained JSON message is published to `flowl/plant/1/attributes`
- **AND** the payload contains `next_due`, `last_watered`, and `watering_interval_days`

#### Scenario: Attributes for never-watered plant

- **GIVEN** a plant with `last_watered` = NULL
- **AND** MQTT is enabled
- **WHEN** attributes are synchronized
- **THEN** the payload contains `next_due` = null, `last_watered` = null, and the `watering_interval_days` value

#### Scenario: Each plant links to its own detail page

- **GIVEN** plants 7 and 42 exist and plant 7 has never been watered
- **AND** MQTT is enabled with `FLOWL_EXTERNAL_URL=https://flowl.example.org`
- **WHEN** the broker’s initial connection triggers a full synchronization
- **THEN** the retained attributes for plants 7 and 42 contain `plant_url` values `https://flowl.example.org/plants/7` and `https://flowl.example.org/plants/42`, respectively
- **AND** the discovery payloads, topic names, and plain-text watering state format are unchanged

#### Scenario: Attribute updates preserve the link

- **GIVEN** MQTT is enabled with a valid MQTT plant link origin
- **WHEN** a plant’s attributes are republished after a mutation, periodic reconciliation, reconnect, repair, or restore
- **THEN** the replacement retained attributes still contain the corresponding `plant_url`
