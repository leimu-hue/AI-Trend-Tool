## ADDED Requirements

### Requirement: Project compiles and runs

The project SHALL compile with `cargo build` and start an HTTP server with `cargo run`. The server SHALL bind to the host and port specified in `config.toml`.

#### Scenario: Server starts and responds to health check

- **WHEN** the server is started with `cargo run -- --config config.toml all`
- **THEN** the server SHALL listen on the configured host:port
- **THEN** `GET /health` SHALL return HTTP 200 with JSON body `{"status": "ok"}`

### Requirement: Configuration parsing

The system SHALL load configuration from a TOML file specified by the `--config` CLI argument. The configuration SHALL include sections for server, database, auth, parser, filter, and pusher. Invalid or missing config SHALL cause the process to exit with an error message.

#### Scenario: Load valid config file

- **WHEN** `config.toml` exists with all required sections and valid values
- **THEN** `AppConfig::load("config.toml")` SHALL return `Ok(AppConfig)` with all fields populated

#### Scenario: Missing config file

- **WHEN** the specified config file does not exist
- **THEN** `AppConfig::load()` SHALL return an error

#### Scenario: Invalid config values

- **WHEN** the config file contains a string value where an integer is expected
- **THEN** `AppConfig::load()` SHALL return a deserialization error

### Requirement: SQLite connection pool

The system SHALL initialize a SQLite connection pool with WAL journal mode and foreign key enforcement enabled. The database file SHALL be created automatically if it does not exist.

#### Scenario: First startup creates database file

- **WHEN** the server starts and no database file exists at the configured path
- **THEN** a new SQLite database SHALL be created
- **THEN** WAL journal mode SHALL be enabled
- **THEN** foreign key enforcement SHALL be enabled

#### Scenario: Subsequent startup reuses existing database

- **WHEN** the server starts and the database file already exists
- **THEN** the connection pool SHALL connect to the existing database
- **THEN** data SHALL persist from previous runs

### Requirement: Unified error handling

All API error responses SHALL follow the format `{"error": {"code": "<ERROR_CODE>", "message": "<description>"}}`. The system SHALL define error variants for 404 (Not Found), 400 (Bad Request), 401 (Unauthorized), 409 (Conflict), and 500 (Internal Server Error). Database errors SHALL auto-convert to 500 responses via `From<sqlx::Error>`.

#### Scenario: Resource not found

- **WHEN** an API endpoint cannot find the requested resource
- **THEN** the response SHALL be HTTP 404 with body `{"error": {"code": "NOT_FOUND", "message": "<details>"}}`

#### Scenario: Bad request

- **WHEN** a request contains invalid parameters
- **THEN** the response SHALL be HTTP 400 with body `{"error": {"code": "BAD_REQUEST", "message": "<details>"}}`

#### Scenario: Unauthorized access

- **WHEN** a request lacks valid authentication
- **THEN** the response SHALL be HTTP 401 with body `{"error": {"code": "UNAUTHORIZED", "message": "<details>"}}`

#### Scenario: Database error

- **WHEN** a database operation fails unexpectedly
- **THEN** the response SHALL be HTTP 500 with body `{"error": {"code": "DATABASE_ERROR", "message": "Internal server error"}}`
- **THEN** the error details SHALL be logged via tracing

### Requirement: Unified success response

The system SHALL provide an `ApiResponse` helper with methods for 200 OK, 201 Created, and 204 No Content. The 200 and 201 responses SHALL wrap data in `{"data": <value>}`.

#### Scenario: Successful GET response

- **WHEN** an API endpoint successfully retrieves data
- **THEN** `ApiResponse::ok(data)` SHALL return HTTP 200 with body `{"data": <data>}`

#### Scenario: Successful creation response

- **WHEN** an API endpoint successfully creates a resource
- **THEN** `ApiResponse::created(data)` SHALL return HTTP 201 with body `{"data": <data>}`

#### Scenario: Successful deletion with no content

- **WHEN** an API endpoint successfully deletes a resource
- **THEN** `ApiResponse::no_content()` SHALL return HTTP 204 with no body

### Requirement: CLI mode selection

The system SHALL accept a positional CLI argument to select the run mode. Valid modes are `all`, `api`, `parser`, `filter`, and `pusher`. The default mode SHALL be `all`.

#### Scenario: Default mode

- **WHEN** the server is started with `cargo run -- --config config.toml`
- **THEN** the system SHALL run in `all` mode

#### Scenario: Explicit mode

- **WHEN** the server is started with `cargo run -- --config config.toml api`
- **THEN** the system SHALL run only the API server

### Requirement: CORS support

The HTTP server SHALL include a permissive CORS layer to allow frontend development without cross-origin issues.

#### Scenario: Cross-origin request

- **WHEN** the frontend on a different origin makes an API request
- **THEN** the server SHALL respond with appropriate CORS headers allowing the request

### Requirement: Initial token is optional

The `auth.initial_token` config field SHALL be optional. When not set, the system SHALL start without auto-creating any token.

#### Scenario: No initial token configured

- **WHEN** `config.toml` has no `initial_token` in the `[auth]` section
- **THEN** the system SHALL start normally without creating any token
