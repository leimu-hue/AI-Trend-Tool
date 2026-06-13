# config-validation (delta)

## MODIFIED Requirements

### Requirement: PusherConfig 新增字段及校验

`PusherConfig` struct SHALL 新增 `retry_max_seconds: u64`（退避上限，默认 3600）和 `stale_timeout_minutes: u64`（陈旧超时，默认 10）。`config.toml` 的 `[pusher]` 段 SHALL 包含对应配置项。

#### Scenario: retry_max_seconds 默认值

- **WHEN** `config.toml` 中未指定 `retry_max_seconds`
- **THEN** `PusherConfig.retry_max_seconds` SHALL 为 3600（1 小时）

#### Scenario: stale_timeout_minutes 默认值

- **WHEN** `config.toml` 中未指定 `stale_timeout_minutes`
- **THEN** `PusherConfig.stale_timeout_minutes` SHALL 为 10（10 分钟）

#### Scenario: 校验 retry_max_seconds 必须大于 0

- **WHEN** config 中 `retry_max_seconds` 为 0
- **THEN** `validate()` SHALL 返回 `Err("pusher.retry_max_seconds must be > 0")`

#### Scenario: 校验 stale_timeout_minutes 必须大于 0

- **WHEN** config 中 `stale_timeout_minutes` 为 0
- **THEN** `validate()` SHALL 返回 `Err("pusher.stale_timeout_minutes must be > 0")`

#### Scenario: config.toml pusher 段包含新字段

- **WHEN** 查看 `config.toml` 的 `[pusher]` 段
- **THEN** SHALL 包含 `retry_max_seconds = 3600`
- **THEN** SHALL 包含 `stale_timeout_minutes = 10`
