## Why

当前后端所有 tracing 日志仅输出到 stdout，进程重启后历史日志全部丢失。生产环境中排查问题时无法回溯历史错误和警告，也无法追踪突发性问题的时序。需要引入文件持久化日志，支持自动轮转和保留策略，确保问题排查时有据可查。

## What Changes

- 新增 `tracing-appender` 依赖，提供滚动文件日志能力
- 新增 `[logging]` 配置段，支持日志目录、级别、保留天数、最大文件数、最大总大小、是否同时输出控制台等配置
- 新增 `src/logging.rs` 模块：日志初始化（文件 + 控制台双输出） + 定期清理过期日志文件
- 修改 `src/main.rs`：将日志初始化提前到配置加载之后，替换原有的 `tracing_subscriber::fmt().with_env_filter("info").init()`
- 修改 `src/config.rs`：新增 `LoggingConfig` 结构体及默认值、验证逻辑
- 修改 `.gitignore`：排除 `logs/` 目录
- 日志文件按天轮转（`app-YYYY-MM-DD.log`），启动时及每 6 小时按 `max_days`/`max_total_size_mb`/`max_files` 清理旧文件
- 补全关键路径上的追踪日志（12 处缺口）：认证失败、Filter 无关键词跳过、mention 批量插入、filter run 汇总、事件通知、手动触发、错误响应、迁移/引导等

## Capabilities

### New Capabilities

- `backend-file-logging`: 后端文件日志系统，支持按天轮转、双输出（控制台 + 文件）、基于保留策略的自动清理

### Modified Capabilities

_（无现有 capability 的需求变更）_

## Impact

- **依赖**: Cargo.toml 新增 `tracing-appender = "0.2"`
- **配置**: config.toml 新增 `[logging]` 段（向后兼容，缺失时使用默认值）
- **后端代码**: 新增 `src/logging.rs`，修改 `src/main.rs`（日志初始化流程 + 补全日志）、`src/config.rs`（新增结构体和验证）、`src/services/filter.rs`（补全关键日志）、`src/services/parser.rs`（补全事件通知日志）、`src/services/pusher.rs`（补全处理日志）、`src/handlers/query.rs`（手动触发日志）、`src/middleware/auth.rs`（认证失败日志）、`src/error.rs`（全错误类型日志）
- **文件系统**: 运行时在 `./logs/` 目录下生成日志文件，需 gitignore
- **无 API 变更、无前端影响**
- **非 BREAKING**：所有变更向后兼容，`[logging]` 段缺失时使用合理默认值
