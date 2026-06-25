## MODIFIED Requirements

### Requirement: Translation dictionaries

Each supported locale SHALL have a TypeScript translation object with identical keys.

#### Scenario: Dictionary structure

- **GIVEN** the English, German, and Spanish translation dictionaries
- **THEN** all three dictionaries SHALL have identical key structures
- **AND** keys are organized in shallow nested groups (e.g., `nav`, `dashboard`, `plant`, `status`, `settings`, `care`, `form`, `identify`, `dialog`)

#### Scenario: English as canonical type

- **GIVEN** the translation dictionaries
- **THEN** the English dictionary SHALL serve as the canonical TypeScript type definition
- **AND** the German and Spanish dictionaries SHALL satisfy the same type
