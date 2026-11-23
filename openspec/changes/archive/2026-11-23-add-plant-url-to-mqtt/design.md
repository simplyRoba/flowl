# Design

## Context

Home Assistant discovery in `src/mqtt.rs` already points each sensor to its retained JSON attributes topic. The state topic carries plain text, and all plants under a topic prefix share one Home Assistant device. The Flowl browser route is `/plants/{id}`; `/api/plants/{id}` returns data, not a page. Some publishers run without an HTTP request.

## Goals / Non-Goals

- Use the same link on every publish for a plant, whether triggered by a request or by background synchronization.
- Do not add per-plant Home Assistant devices, modify discovery metadata, or introduce Flowl subpath hosting. The URL is available to Home Assistant templates; it does not create a built-in link on the device page.

## Decisions

- **Use the existing attributes topic.** Add `plant_url` in `attributes_topic_and_payload` so every replacement JSON object carries the link when configured. Do not put it in plain-text state or `device.configuration_url`, which would assign a plant-specific link to the shared device.
- **Reuse the existing browser-facing origin.** When OIDC is enabled, take the validated `FLOWL_EXTERNAL_URL` from the OIDC configuration, preserving its existing fail-closed HTTPS policy. With OIDC disabled and MQTT enabled, read the same variable as an optional HTTP(S) origin using the existing `url` dependency. Check the raw input before URL normalization can hide a supplied path. Warn without echoing malformed or non-Unicode input; do not warn when unset. Do not derive the origin from the bind address, broker hostname, or request headers.
- **Share the validated value across publishers.** Pass the optional origin from `src/main.rs` into `AppState` and the state checker. Pass it through the existing publish paths in `src/api/plants.rs` and `src/api/care_events.rs`, plus full synchronization in `src/mqtt.rs` (initial connection, reconnect, and repair via `src/api/mqtt.rs` or `src/api/restore.rs`). Build the `/plants/{id}` URL in the shared attributes builder from the same ID as the topic. Do not persist the URL in the database.

## Risks / Trade-offs

- **An origin can be valid but unreachable from a browser.** Document using the address where Home Assistant users actually open Flowl, such as a reverse proxy hostname rather than a container-internal address. The link does not bypass Flowl authentication.
- **Retained links may outlive their configuration.** On startup or restart, the existing initial-connect full sync populates, replaces, or removes `plant_url` for current plants by overwriting their retained attributes object. MQTT repair can force a refresh when connected. No new topic or cleanup path is needed.
