# config-validation

## Purpose

Configuration validation that runs immediately after TOML deserialization, rejecting invalid configurations before the server starts.

## Requirements

### Requirement: Config validated on load

The system SHALL validate all config fields after deserializing from TOML and SHALL return an error if any field is invalid. The process SHALL exit before starting the HTTP server or any background module.

#### Scenario: Valid config passes validation

- **WHEN** `AppConfig::load()` is called with a valid `config.toml`
- **THEN** `validate()` SHALL return `Ok(())`
- **THEN** the server SHALL proceed to start

#### Scenario: Port zero rejected

- **WHEN** `server.port` is `0`
- **THEN** the system SHALL return `Err("server.port must be > 0")`

#### Scenario: Empty database path rejected

- **WHEN** `database.path` is an empty string
- **THEN** the system SHALL return `Err("database.path must not be empty")`

#### Scenario: Zero parser interval rejected

- **WHEN** `parser.interval_seconds` is `0`
- **THEN** the system SHALL return `Err("parser.interval_seconds must be > 0")`

#### Scenario: Zero parser max_concurrent_fetches rejected

- **WHEN** `parser.max_concurrent_fetches` is `0`
- **THEN** the system SHALL return `Err("parser.max_concurrent_fetches must be > 0")`

#### Scenario: Zero filter batch_size rejected

- **WHEN** `filter.batch_size` is `0`
- **THEN** the system SHALL return `Err("filter.batch_size must be > 0")`

#### Scenario: Zero filter interval rejected

- **WHEN** `filter.interval_seconds` is `0`
- **THEN** the system SHALL return `Err("filter.interval_seconds must be > 0")`

#### Scenario: Zero pusher interval rejected

- **WHEN** `pusher.interval_seconds` is `0`
- **THEN** the system SHALL return `Err("pusher.interval_seconds must be > 0")`

#### Scenario: Zero pusher max_retries rejected

- **WHEN** `pusher.max_retries` is `0`
- **THEN** the system SHALL return `Err("pusher.max_retries must be > 0")`

#### Scenario: Zero pusher retry_max_seconds rejected

- **WHEN** `pusher.retry_max_seconds` is `0`
- **THEN** the system SHALL return `Err("pusher.retry_max_seconds must be > 0")`

#### Scenario: Zero pusher stale_timeout_minutes rejected

- **WHEN** `pusher.stale_timeout_minutes` is `0`
- **THEN** the system SHALL return `Err("pusher.stale_timeout_minutes must be > 0")`

### Requirement: PusherConfig 新增字段及校验

`PusherConfig` struct SHALL 新增 `retry_max_seconds: u64`（退避上限，默认 3600）和 `stale_timeout_minutes: u64`（陈旧超时，默认 10）。`config.toml` 的 `[pusher]` 段 SHALL 包含对应配置项。

#### Scenario: retry_max_seconds 默认值

- **WHEN** `config.toml` 中未指定 `retry_max_seconds`
- **THEN** `PusherConfig.retry_max_seconds` SHALL 为 3600（1 小时）

#### Scenario: stale_timeout_minutes 默认值

- **WHEN** `config.toml` 中未指定 `stale_timeout_minutes`
- **THEN** `PusherConfig.stale_timeout_minutes` SHALL 为 10（10 分钟）

#### Scenario: config.toml pusher 段包含新字段

- **WHEN** 查看 `config.toml` 的 `[pusher]` 段
- **THEN** SHALL 包含 `retry_max_seconds = 3600`
- **THEN** SHALL 包含 `stale_timeout_minutes = 10`

### Requirement: Config 验证单元测试

`src/config.rs` SHALL 包含 `#[cfg(test)]` 测试模块，至少覆盖有效配置通过和无效配置（port=0）拒绝两个场景。

#### Scenario: 有效配置通过验证
- **WHEN** `cargo test` 运行
- **THEN** 加载有效 `config.toml` 的测试 SHALL 通过

#### Scenario: port=0 被拒绝
- **WHEN** 测试中构造 port=0 的配置
- **THEN** `validate()` SHALL 返回 `Err`
