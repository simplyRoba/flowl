## MODIFIED Requirements

### Requirement: SPA Static File Serving

The server SHALL serve the embedded SvelteKit build output as static files. Any request that does not match an API route SHALL fall back to the SPA's `index.html`.

#### Scenario: API routes handled by API router

- **WHEN** a GET request is made to a path starting with `/api`
- **THEN** the request is handled by the nested API router
- **AND** does not fall through to the SPA handler
