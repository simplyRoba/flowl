# Proposal

## Why

Home Assistant receives watering data for each plant but has no direct link back to that plant in Flowl. A per-plant URL lets dashboards and notifications link to the correct detail page.

## What Changes

- Use the existing `FLOWL_EXTERNAL_URL` as the browser-facing origin for plant links, including when OIDC is disabled. Do not add a second setting.
- Include `plant_url` in each plant’s retained MQTT attributes when the setting is valid. Keep existing state values, topics, and Home Assistant discovery identities unchanged.
- Document how to use the sensor attribute as a link in Home Assistant.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `core-mqtt`: Configure the browser-facing origin and publish the corresponding plant detail URL in MQTT attributes.

## Impact

- Affects backend configuration, MQTT attributes publishing, and Home Assistant dashboard or automation templates that opt into the new attribute.
- No API, database, UI, or OIDC configuration changes.
