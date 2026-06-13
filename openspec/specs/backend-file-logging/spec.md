# backend-file-logging

## Purpose

后端文件日志系统，提供日志文件持久化、按天轮转、双输出（控制台 + 文件）、基于保留策略的自动清理，以及关键路径追踪日志覆盖。

## Requirements

### Requirement: File log initialization

系统 SHALL 在启动时将 tracing 日志同时写入按天轮转的文件和 stdout（可配置关闭 stdout 输出）。
日志 SHALL 使用 `tracing_subscriber::registry()` 组合多个 `fmt::Layer`，以 `EnvFilter` 控制日志级别。

#### Scenario: 双输出模式启动

- **WHEN** `config.toml` 中 `logging.console_output` 为 `true`
- **AND** 系统启动
- **THEN** `tracing::info!`、`tracing::warn!`、`tracing::error!` 宏的输出 SHALL 同时出现在 stdout 和 `./logs/app-YYYY-MM-DD.log` 文件中
- **AND** 文件日志 SHALL 不含 ANSI 颜色转义码

#### Scenario: 仅文件输出模式

- **WHEN** `config.toml` 中 `logging.console_output` 为 `false`
- **AND** 系统启动
- **THEN** 日志 SHALL 仅写入 `./logs/app-YYYY-MM-DD.log`，不输出到 stdout

#### Scenario: 日志目录自动创建

- **WHEN** `./logs/` 目录不存在
- **AND** 系统启动
- **THEN** 系统 SHALL 自动创建 `./logs/` 目录
- **AND** 日志写入正常进行

### Requirement: Daily log rotation

系统 SHALL 使用 `tracing-appender` 的 `RollingFileAppender` 实现按天日志轮转。
日志文件名格式 SHALL 为 `app-YYYY-MM-DD.log`。

#### Scenario: 跨天轮转

- **WHEN** 系统运行跨越 UTC 午夜
- **THEN** 新一天的日志 SHALL 写入新的 `app-<新日期>.log` 文件
- **AND** 前一天的日志文件 SHALL 保留在日志目录中

### Requirement: Log retention cleanup

系统 SHALL 在启动时执行一次日志清理，之后每 6 小时执行一次。
清理逻辑 SHALL 按以下三条规则依次执行：

1. **按天数清理**：删除 mtime 距今超过 `max_days` 的文件
2. **按总大小清理**：按文件 mtime 从旧到新删除，直到所有日志文件总大小 ≤ `max_total_size_mb`
3. **按数量清理**：按文件 mtime 从旧到新删除，直到文件数 ≤ `max_files`

每条规则执行时 SHALL 通过 `tracing::debug!` 记录被删除的文件路径。清理完成后，若删除了文件，SHALL 通过 `tracing::info!` 记录删除总数。

#### Scenario: 按天数清理旧文件

- **WHEN** 存在 35 天前的 `.log` 文件
- **AND** `max_days` 配置为 `30`
- **THEN** 清理任务 SHALL 删除该文件
- **AND** SHALL 记录 `tracing::debug!` 日志

#### Scenario: 按总大小清理

- **WHEN** 日志目录总大小超过 `max_total_size_mb`
- **AND** 没有超过 `max_days` 的旧文件
- **THEN** 清理任务 SHALL 从最旧的文件开始删除
- **AND** 直到总大小 ≤ `max_total_size_mb`

#### Scenario: 按数量清理

- **WHEN** 日志文件数超过 `max_files`
- **AND** 总大小和天数均未超限
- **THEN** 清理任务 SHALL 从最旧的文件开始删除
- **AND** 直到文件数 ≤ `max_files`

#### Scenario: 无文件需清理

- **WHEN** 日志目录中所有文件均满足保留策略
- **THEN** 清理任务 SHALL 不删除任何文件
- **AND** SHALL 不输出 `info` 级别的清理日志

#### Scenario: 仅处理 .log 文件

- **WHEN** 日志目录中包含非 `.log` 后缀的文件（如 `.tmp`、`.lock`）
- **THEN** 清理任务 SHALL 忽略这些文件，不做任何操作

#### Scenario: 日志目录不存在时跳过清理

- **WHEN** `logs.dir` 指定的目录不存在
- **THEN** 清理任务 SHALL 直接返回，不报错

### Requirement: Logging configuration

`LoggingConfig` SHALL 包含以下字段，所有字段 SHALL 有默认值：

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `dir` | `String` | `"./logs"` | 日志文件输出目录 |
| `level` | `String` | `"info"` | 日志级别 |
| `max_files` | `u32` | `30` | 最多保留日志文件数 |
| `max_days` | `u32` | `30` | 日志文件保留最大天数 |
| `max_total_size_mb` | `u32` | `500` | 日志文件总大小上限（MB） |
| `console_output` | `bool` | `true` | 是否同时输出到控制台 |

`config.toml` 中整个 `[logging]` 段缺失时，系统 SHALL 使用所有默认值正常启动。

#### Scenario: 完整配置加载

- **WHEN** `config.toml` 包含完整的 `[logging]` 段
- **THEN** `AppConfig::load()` SHALL 返回 `LoggingConfig` 包含配置文件中指定的值

#### Scenario: 缺失 logging 段

- **WHEN** `config.toml` 不包含 `[logging]` 段
- **THEN** `AppConfig::load()` SHALL 返回带有全部默认值的 `LoggingConfig`
- **THEN** 系统正常启动

### Requirement: Logging config validation

系统 SHALL 在 `AppConfig::validate()` 中验证日志配置字段：

- `dir` 不得为空字符串
- `level` 必须为 `trace`、`debug`、`info`、`warn`、`error` 之一
- `max_files` 必须 > 0
- `max_days` 必须 > 0
- `max_total_size_mb` 必须 > 0

#### Scenario: 有效日志配置通过验证

- **WHEN** `logging.dir` 为 `"./logs"`、`level` 为 `"info"`、`max_files` 为 `30`、`max_days` 为 `30`、`max_total_size_mb` 为 `500`
- **THEN** `validate()` SHALL 返回 `Ok(())`

#### Scenario: 空目录路径被拒绝

- **WHEN** `logging.dir` 为空字符串
- **THEN** `validate()` SHALL 返回 `Err("logging.dir must not be empty")`

#### Scenario: 无效日志级别被拒绝

- **WHEN** `logging.level` 为 `"verbose"`
- **THEN** `validate()` SHALL 返回 `Err("logging.level must be one of: [\"trace\", \"debug\", \"info\", \"warn\", \"error\"]")`

#### Scenario: max_files 为零被拒绝

- **WHEN** `logging.max_files` 为 `0`
- **THEN** `validate()` SHALL 返回 `Err("logging.max_files must be > 0")`

#### Scenario: max_days 为零被拒绝

- **WHEN** `logging.max_days` 为 `0`
- **THEN** `validate()` SHALL 返回 `Err("logging.max_days must be > 0")`

#### Scenario: max_total_size_mb 为零被拒绝

- **WHEN** `logging.max_total_size_mb` 为 `0`
- **THEN** `validate()` SHALL 返回 `Err("logging.max_total_size_mb must be > 0")`

### Requirement: Logs directory excluded from git

`.gitignore` SHALL 包含 `logs/` 条目，确保日志目录不纳入版本控制。

#### Scenario: logs 目录被忽略

- **WHEN** 在仓库根目录执行 `git status`
- **THEN** `./logs/` 目录及其内容 SHALL 不出现在未跟踪文件列表中

### Requirement: Critical-path tracing instrumentation

系统 SHALL 在以下关键路径上通过 `tracing` 宏输出日志，确保排查问题时能追踪完整执行链路：

**认证模块** (`src/middleware/auth.rs`)：
- 认证失败时 SHALL 输出 `tracing::warn!`，记录失败原因（缺失 Header、格式错误、token 无效/过期）

**Filter 模块** (`src/services/filter.rs`)：
- 当启用的关键词列表为空时 SHALL 输出 `tracing::warn!`，提示所有文章将被标记为已处理而不做匹配
- 批量插入 keyword_mentions 后 SHALL 输出 `tracing::info!`，记录插入的 mention 总数
- 每次 filter run 结束时 SHALL 输出 `tracing::info!` 汇总：处理文章数、匹配 mention 数、创建 hotspot 数

**Parser 模块** (`src/services/parser.rs`)：
- 当有新文章插入并通知 Filter 时 SHALL 输出 `tracing::debug!`，记录发送的事件及插入文章数

**Pusher 模块** (`src/services/pusher.rs`)：
- `claim_pending_records` 成功后 SHALL 输出 `tracing::debug!`，记录声明了多少条记录
- 批量处理完成后 SHALL 输出 `tracing::info!` 汇总：成功数、失败数、放弃数

**手动触发端点** (`src/handlers/query.rs`)：
- `trigger_filter` 和 `trigger_pusher` 被调用时 SHALL 输出 `tracing::info!`，记录触发行为

**错误处理** (`src/error.rs`)：
- 所有 `AppError` 变体（NotFound、BadRequest、Unauthorized、Conflict、Internal、Database）在转换为 HTTP 响应时 SHALL 至少输出 `tracing::warn!`（Database 保持 `tracing::error!`）

**启动流程** (`src/main.rs`)：
- 数据库迁移开始和完成时 SHALL 输出 `tracing::info!`
- API token 引导完成时 SHALL 输出 `tracing::info!`，记录 token 数量（已有部分日志，需确保写入文件日志）

#### Scenario: 认证失败被记入日志

- **WHEN** API 请求的 Authorization header 缺失或包含无效 token
- **THEN** 日志文件 SHALL 包含 `tracing::warn!` 级别的认证失败记录
- **AND** 日志内容 SHALL 包含失败原因（如 "Missing Authorization header"、"Invalid or revoked token"）

#### Scenario: Filter 无关键词时告警

- **WHEN** Filter 加载到待处理文章但系统中没有启用的关键词
- **THEN** 日志文件 SHALL 包含 `tracing::warn!` 级别记录，说明所有文章被标记为已处理

#### Scenario: Filter run 汇总

- **WHEN** 一次 filter run 完成
- **THEN** 日志文件 SHALL 包含 `tracing::info!` 级别汇总，包含处理文章数、mention 总数、hotspot 创建数

#### Scenario: 手动触发被记录

- **WHEN** `POST /api/v1/trigger/filter` 或 `POST /api/v1/trigger/pusher` 被调用
- **THEN** 日志文件 SHALL 包含 `tracing::info!` 级别记录

#### Scenario: 错误响应记入日志

- **WHEN** API 返回 4xx 或 5xx 错误响应
- **THEN** 日志文件 SHALL 包含该错误的 tracing 记录，级别不低于 `tracing::warn!`
