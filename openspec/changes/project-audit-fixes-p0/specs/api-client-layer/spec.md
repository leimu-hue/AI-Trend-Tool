## MODIFIED Requirements

### Requirement: Axios instance with base configuration
The system SHALL create and export a configured Axios instance at `src/api/client.ts` with the base URL from environment variable `VITE_API_BASE_URL`, 30-second timeout, and JSON content type header.

#### Scenario: Axios instance created with defaults
- **WHEN** the client module is imported
- **THEN** an Axios instance exists with `baseURL` matching `VITE_API_BASE_URL` (default `http://localhost:3000/api/v1`), `timeout: 30000`, and `Content-Type: application/json` header
