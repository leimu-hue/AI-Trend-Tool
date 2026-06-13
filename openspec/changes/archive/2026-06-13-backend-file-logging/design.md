## Context

当前后端使用 `tracing` + `tracing-subscriber`，日志仅输出到 stdout。`src/main.rs:95` 处通过 `tracing_subscriber::fmt().with_env_filter("info").init()` 初始化，无文件持久化能力。系统中有约 25 处 `tracing::error!`/`tracing::warn!` 调用分布在 `services/parser.rs`、`services/filter.rs`、`services/pusher.rs`、`error.rs`。

需要在保持现有控制台输出的同时，增加文件日志持久化，支持自动轮转和基于时间/大小/数量的保留策略。

## Goals / Non-Goals

**Goals:**
- 日志同时输出到 stdout 和文件（可配置关闭控制台输出）
- 日志文件按天轮转，文件名格式 `app-YYYY-MM-DD.log`
- 启动时及每 6 小时自动清理过期日志（按天数、总大小、文件数三条规则）
- 所有日志参数通过 `config.toml` 的 `[logging]` 段配置
- 缺失 `[logging]` 段时使用合理默认值，向后兼容
- `logs/` 目录加入 `.gitignore`

**Non-Goals:**
- 结构化 JSON 日志输出（保持纯文本格式）
- 远程日志采集/聚合（如 Elasticsearch、Loki）
- 日志搜索或分析 UI
- 前端日志管理界面
- 按日志级别分离文件
- 日志内容加密或压缩

## Decisions

### 1. 选择 `tracing-appender` 而非 `log4rs`/`fern`

`tracing-appender` 是 tracing 生态官方配套 crate，提供 `RollingFileAppender`。与已使用的 `tracing-subscriber` 无缝组合。`log4rs` 需要适配 tracing 的 log 桥接层，引入额外复杂度；`fern` 不内置轮转能力。

### 2. 按天轮转（Daily Rotation）

使用 `tracing-appender::rolling::Rotation::DAILY`。相比按大小轮转，按天轮转的日志文件命名可预测（`app-YYYY-MM-DD.log`），排查问题时易于按日期定位。日志量预估不高（错误和警告为主），单日文件不会过大。

### 3. 双输出：Layer 组合模式

使用 `tracing_subscriber::registry()` 组合多个 `fmt::Layer`：文件 layer 使用 `RollingFileAppender` 作为 writer（关闭 ANSI 颜色），控制台 layer 使用 `std::io::stdout`。调用 `init()` 一次性完成全局 subscriber 注册。

### 4. 保留策略：tracing-appender 原生 + 自定义清理任务

`tracing-appender` 的 `max_log_files` 仅作用于其内部计数器（限制滚动文件数，但重启后计数器重置且不处理日期/大小维度）。额外实现 `cleanup_old_logs()` 函数处理三条规则：

- **Rule 1（天数）**: 删除 mtime 距今超过 `max_days` 的文件
- **Rule 2（大小）**: 按文件 mtime 从旧到新删除，直到总大小 ≤ `max_total_size_mb`
- **Rule 3（数量）**: 按文件 mtime 从旧到新删除，直到文件数 ≤ `max_files`

清理在启动时执行一次，然后每 6 小时通过 `tokio::spawn` 的后台任务执行。使用 `tracing::debug!` 记录删除操作，`tracing::info!` 总结清理数量。

备选方案：仅依赖 OS 的 logrotate。但本项目需跨平台（Windows/Linux/macOS），logrotate 依赖操作系统，不可靠。

### 5. 配置设计

新增 `[logging]` 段，所有字段有 `#[serde(default)]` 默认值。`AppConfig` 中 `logging` 字段标记 `#[serde(default)]`，缺失整个段时使用默认值，确保旧配置文件直接可用。

```toml
[logging]
dir = "./logs"               # 日志目录
level = "info"               # 日志级别: trace, debug, info, warn, error
max_files = 30               # 最多保留文件数
max_days = 30                # 最多保留天数（基于文件 mtime）
max_total_size_mb = 500      # 日志总大小上限（MB）
console_output = true        # 是否同时输出到控制台
```

### 6. 日志级别处理

使用 `EnvFilter::try_new(&log_cfg.level)` 从配置构造过滤层，解析失败时回退到 `"info"`。**与现有 `RUST_LOG` 环境变量的互动**：`tracing_subscriber::registry()` 中 filter layer 的优先级由添加顺序决定。配置级别作为默认，如果未来需要 `RUST_LOG` 覆盖，可以通过 `EnvFilter::from_env("RUST_LOG")` 作为独立 layer 添加或合并。

## Risks / Trade-offs

| Risk | Mitigation |
|------|-----------|
| 日志目录无写权限 | `init_logging()` 中 `create_dir_all` 失败会 panic（启动时快速失败）。生产部署应确保目录权限正确 |
| 磁盘空间耗尽 | 三条保留规则 + 启动时清理。极端情况（短时间内大量日志）可能在清理间隔内占满磁盘 → 可调小 `max_total_size_mb` 或调短清理间隔 |
| 清理操作阻塞 | `cleanup_old_logs()` 是同步函数，清理由 tokio 异步任务调用。在繁忙系统上遍历数百个日志文件仅需毫秒级，不构成阻塞风险 |
| `tracing-appender` 内部缓冲丢失 | 默认使用非缓冲 writer（`RollingFileAppender`），每条日志立即 `write()` + `flush()`。权衡性能换取可靠性（本系统日志量低，性能损失可忽略） |
| 多进程写同一日志文件 | 当前系统为单进程架构，不涉及多进程并发写。若未来扩展，需改用 `tracing-appender` 的 `non_blocking` 模式 |
