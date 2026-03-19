## MODIFIED Requirements

### Requirement: Add Plant Form

The route `/plants/new` SHALL display a form to create a new plant (all other scenarios remain unchanged from the base spec docs).

#### Scenario: Watering interval

- **WHEN** the user selects a preset (3d, 5d, 7d, 14d) or uses the custom stepper
- **THEN** the watering interval is set accordingly
