## MODIFIED Requirements

### Requirement: Project compiles and runs

The project SHALL compile with `cargo build` and start an HTTP server with `cargo run`. The server SHALL bind to the host and port specified in `config.toml`. On startup, the system SHALL always spawn all three background modules (Parser, Filter, Pusher) alongside the API server.

#### Scenario: Server starts all modules by default

- **WHEN** the server is started with `cargo run -- config.toml`
- **THEN** the server SHALL spawn Parser, Filter, and Pusher background tasks
- **AND** the server SHALL listen on the configured host:port
- **AND** `GET /health` SHALL return HTTP 200 with JSON body `{"status": "ok"}`

### Requirement: Configuration parsing

The system SHALL load configuration from a TOML file specified as the first positional argument (or default to `config.toml`). The configuration SHALL include sections for server, database, auth, parser, filter, and pusher. Invalid or missing config SHALL cause the process to exit with an error message.

#### Scenario: Load valid config file

- **WHEN** `config.toml` exists with all required sections and valid values
- **THEN** `AppConfig::load("config.toml")` SHALL return `Ok(AppConfig)` with all fields populated

#### Scenario: Missing config file

- **WHEN** the specified config file does not exist
- **THEN** `AppConfig::load()` SHALL return an error

#### Scenario: Invalid config values

- **WHEN** the config file contains a string value where an integer is expected
- **THEN** `AppConfig::load()` SHALL return a deserialization error

#### Scenario: ParserConfig requires interval_seconds

- **WHEN** `config.toml` does not contain `interval_seconds` in the `[parser]` section
- **THEN** `AppConfig::load()` SHALL return a deserialization error unless a default value is provided via `#[serde(default = "...")]`

## REMOVED Requirements

### Requirement: CLI mode selection

**Reason**: 事件驱动架构下 Parser → Filter → Pusher 形成依赖链路，单独运行某个模块失去意义。移除 CLI mode 简化启动命令和 `main.rs` 入口逻辑。

**Migration**: 启动命令从 `cargo run -- --config config.toml <mode>` 改为 `cargo run -- <config_path>`（config 路径可选，默认 `config.toml`）。Dockerfile 中 `CMD ["hotspot", "all"]` 改为 `CMD ["hotspot"]`。
