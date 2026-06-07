## MODIFIED Requirements

### Requirement: Initial token is optional

The `auth.initial_token` config field SHALL be optional. When set to a non-empty value, the system SHALL use it as the initial admin token on first startup. When not set or empty, the system SHALL auto-generate a random 64-character hex token on first startup and log it via `tracing::warn!` for the administrator to capture. If the `api_tokens` table already contains rows, no token SHALL be created regardless of config.

#### Scenario: No initial token configured

- **WHEN** `config.toml` has no `initial_token` in the `[auth]` section
- **AND** the `api_tokens` table is empty
- **THEN** the system SHALL generate a random 64-character hex token
- **THEN** the generated token SHALL be logged via `tracing::warn!` with prominent formatting
- **THEN** the system SHALL insert the token with name "Initial Admin Token"

#### Scenario: Initial token configured

- **WHEN** `config.toml` has `initial_token = "my-secret-token"` in `[auth]`
- **AND** the `api_tokens` table is empty
- **THEN** the system SHALL insert `"my-secret-token"` as the initial token with name "Initial Admin Token"

#### Scenario: Table already has tokens

- **WHEN** `config.toml` has any `initial_token` value
- **AND** the `api_tokens` table already contains one or more rows
- **THEN** the system SHALL skip token creation entirely

#### Scenario: Empty string initial_token

- **WHEN** `config.toml` has `initial_token = ""`
- **AND** the `api_tokens` table is empty
- **THEN** the system SHALL treat it as if no token was configured and auto-generate one
