## MODIFIED Requirements

### Requirement: Project compiles and runs

The project SHALL compile with `cargo build` and start an HTTP server with `cargo run`. The server SHALL bind to the host and port specified in `config.toml`. On startup, the system SHALL always spawn all three background modules (Parser, Filter, Pusher) alongside the API server. The tracing subscriber SHALL use `EnvFilter` that reads `RUST_LOG` environment variable, falling back to `info` level.

#### Scenario: Server starts all modules by default

- **WHEN** the server is started with `cargo run -- config.toml`
- **THEN** the server SHALL spawn Parser, Filter, and Pusher background tasks
- **AND** the server SHALL listen on the configured host:port
- **AND** `GET /health` SHALL return HTTP 200 with JSON body `{"status": "ok"}`

#### Scenario: RUST_LOG overrides log level

- **WHEN** the server is started with `RUST_LOG=debug cargo run -- config.toml`
- **THEN** tracing SHALL emit debug-level logs in addition to info and above

#### Scenario: No RUST_LOG defaults to info

- **WHEN** the server is started without `RUST_LOG` set
- **THEN** tracing SHALL filter at `info` level

### Requirement: SQLite connection pool

The system SHALL initialize a SQLite connection pool with WAL journal mode and foreign key enforcement enabled. The pool SHALL have at least `max_concurrent_fetches + 5` connections (minimum 15). The database file SHALL be created automatically if it does not exist.

#### Scenario: Connection pool sized for concurrent fetches

- **WHEN** `config.parser.max_concurrent_fetches` is 10
- **THEN** `max_connections` SHALL be at least 15
- **THEN** all concurrent fetch tasks SHALL be able to acquire a database connection without starvation

#### Scenario: Database path unwrap safety

- **WHEN** `database.path` has no parent directory (e.g., root path `/`)
- **THEN** the system SHALL return an error instead of panicking
- **THEN** the error message SHALL describe the issue clearly
