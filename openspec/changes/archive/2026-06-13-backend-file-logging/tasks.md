## 1. 添加依赖

- [x] 1.1 在 `Cargo.toml` 的 `[dependencies]` 的 `# Logging` 区域添加 `tracing-appender = "0.2"`
- [x] 1.2 运行 `cargo check` 确认依赖解析成功

## 2. 配置段 — config.toml + src/config.rs

- [x] 2.1 在 `config.toml` 末尾追加 `[logging]` 段，包含 `dir`、`level`、`max_files`、`max_days`、`max_total_size_mb`、`console_output` 六个字段
- [x] 2.2 在 `src/config.rs` 中定义默认函数（`default_log_dir`、`default_log_level`、`default_max_files`、`default_max_days`、`default_max_total_size_mb`、`default_console_output`）
- [x] 2.3 在 `src/config.rs` 中定义 `LoggingConfig` 结构体，所有字段标注 `#[serde(default = "...")]`
- [x] 2.4 在 `AppConfig` 结构体中添加 `#[serde(default)] pub logging: LoggingConfig` 字段
- [x] 2.5 在 `AppConfig::validate()` 中添加日志配置验证（dir 非空、level 合法值、max_files/max_days/max_total_size_mb > 0）
- [x] 2.6 运行 `cargo build` 确认编译通过

## 3. 创建日志模块 — src/logging.rs

- [x] 3.1 创建 `src/logging.rs`，实现 `init_logging(config: &AppConfig)` 函数：创建日志目录 → 构建 `RollingFileAppender`（按天轮转）→ 组合 file layer + 可选 console layer → 初始化 global subscriber
- [x] 3.2 实现 `cleanup_old_logs(config: &LoggingConfig)` 函数：遍历 `.log` 文件 → 按 mtime 排序 → 依次执行天数/大小/数量三条清理规则 → 使用 `tracing::debug!`/`tracing::info!` 记录
- [x] 3.3 运行 `cargo build` 确认编译通过

## 4. 接入 main.rs

- [x] 4.1 在 `src/main.rs` 顶部添加 `mod logging;` 模块声明
- [x] 4.2 调整 `main()` 启动顺序：将配置加载移到日志初始化之前，用 `logging::init_logging(&config)` 替换原有的 `tracing_subscriber::fmt()` 初始化
- [x] 4.3 在配置加载后、DB 初始化前调用 `logging::cleanup_old_logs(&config.logging)` 执行首次清理
- [x] 4.4 在后台任务区域添加 tokio::spawn 定期清理任务（每 6 小时调用 `cleanup_old_logs`）
- [x] 4.5 运行 `cargo build` 确认编译通过

## 6. 补全关键路径日志（12 处缺口）

- [x] 6.1 `src/middleware/auth.rs`：认证失败时添加 `tracing::warn!`（缺失 Header、格式错误、token 无效、token 过期）
- [x] 6.2 `src/services/filter.rs`：keywords 为空时添加 `tracing::warn!` 告警
- [x] 6.3 `src/services/filter.rs`：batch_insert_keyword_mentions 后添加 `tracing::info!` 记录 mention 总数
- [x] 6.4 `src/services/filter.rs`：filter run 末尾添加汇总日志（processed articles、mentions、hotspots created、push sent）
- [x] 6.5 `src/services/parser.rs`：发送 articles_ready 事件后添加 `tracing::debug!`
- [x] 6.6 `src/services/pusher.rs`：claim_pending_records 后添加 `tracing::debug!` 记录声明数量
- [x] 6.7 `src/services/pusher.rs`：批量处理后添加 `tracing::info!` 汇总（success/failed/gave_up）
- [x] 6.8 `src/handlers/query.rs`：trigger_filter / trigger_pusher 添加 `tracing::info!`
- [x] 6.9 `src/error.rs`：NotFound/BadRequest/Unauthorized/Conflict/Internal 变体添加 `tracing::warn!`（Database 保持 `tracing::error!`）
- [x] 6.10 `src/main.rs`：数据库迁移开始/完成添加 `tracing::info!`
- [x] 6.11 `src/main.rs`：token 引导完成添加 `tracing::info!`（确保写入文件日志而非仅 stdout print）
- [x] 6.12 运行 `cargo build` + `cargo test` 确认编译通过且现有测试未破坏

## 7. 更新 .gitignore

- [x] 7.1 在 `.gitignore` 末尾追加 `logs/` 条目（项目根目录下的日志目录）
- [x] 7.2 运行 `git status` 确认 `logs/` 不被列为未跟踪文件

## 8. 验证

- [x] 8.1 运行 `cargo build --release` 确认 release 构建成功
- [x] 8.2 运行 `cargo test` 确认现有测试全部通过
- [ ] 8.3 启动应用，确认 `./logs/app-YYYY-MM-DD.log` 文件生成，内容包含启动日志
- [ ] 8.4 触发认证失败请求，确认日志文件包含认证失败记录
- [ ] 8.5 手动调用 trigger API，确认日志文件包含手动触发记录
- [ ] 8.6 手动在 `logs/` 目录创建过期 `.log` 文件，重启应用确认清理生效
