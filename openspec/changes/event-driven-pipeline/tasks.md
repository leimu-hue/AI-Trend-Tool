## 1. 依赖与配置变更

- [x] 1.1 在 `Cargo.toml` 中新增 `tokio-util = "0.7"` 依赖，移除 `clap` 依赖
- [x] 1.2 在 `src/config.rs` 的 `ParserConfig` 中新增 `interval_seconds: u64` 字段（可加 `#[serde(default = "default_interval")]` 提供默认值 30，降低迁移风险）
- [x] 1.3 更新根目录 `config.toml`，在 `[parser]` 段新增 `interval_seconds = 30`
- [x] 1.4 验证：`cargo check` 确认依赖解析和配置结构编译通过

## 2. 新建 Pipeline 事件总线模块

- [x] 2.1 创建 `src/pipeline.rs`，定义 `PipelineEvent` 枚举（`NewData` 变体）和 `Pipeline` 结构体（含 `articles_ready_tx`、`push_ready_tx`、`cancel` 字段）
- [x] 2.2 实现 `Pipeline::new()` 构造函数，创建两个容量为 16 的 mpsc channel 和一个 `CancellationToken`，返回 `(Pipeline, Receiver, Receiver)`
- [x] 2.3 实现 `Clone` for `Pipeline`（derive 或手动 impl）
- [x] 2.4 在 `src/main.rs` 中新增 `mod pipeline;`
- [x] 2.5 验证：`cargo check` 确认新模块编译通过

## 3. 重构 Parser 模块

- [x] 3.1 修改 `src/services/parser.rs` — `start_parser_loop` 签名新增 `pipeline: Pipeline` 参数
- [x] 3.2 将 `loop { sleep; work }` 改为 `tokio::select!` 三路监听：`cancel.cancelled()`、`interval.tick()`（使用 `config.interval_seconds`）、当前无额外 channel
- [x] 3.3 在成功插入文章后，调用 `pipeline.articles_ready_tx.try_send(PipelineEvent::NewData)`（忽略 `try_send` 错误）
- [x] 3.4 验证：`cargo check` 确认 Parser 编译通过（暂时忽略 main.rs 调用处不匹配，后续步骤修复）

## 4. 重构 Filter 模块

- [x] 4.1 修改 `src/services/filter.rs` — `run_filter_once` 返回值改为 `bool`，在 burst detection 创建 push_record 时标记 `true`
- [x] 4.2 修改 `start_filter_loop` 签名：新增 `pipeline: Pipeline` 和 `articles_rx: mpsc::Receiver<PipelineEvent>` 参数
- [x] 4.3 将 `loop { sleep; work }` 改为 `tokio::select!` 三路监听：`cancel.cancelled()`、`interval.tick()`、`articles_rx.recv()`
- [x] 4.4 在 interval tick 和 articles_rx 分支中，若 `run_filter_once` 返回 `true`，调用 `pipeline.push_ready_tx.try_send(PipelineEvent::NewData)`
- [x] 4.5 验证：`cargo check` 确认 Filter 编译通过

## 5. 重构 Pusher 模块

- [x] 5.1 修改 `src/services/pusher.rs` — `start_pusher_loop` 签名：新增 `pipeline: Pipeline` 和 `push_rx: mpsc::Receiver<PipelineEvent>` 参数
- [x] 5.2 将 `loop { sleep; work }` 改为 `tokio::select!` 三路监听：`cancel.cancelled()`、`interval.tick()`、`push_rx.recv()`
- [x] 5.3 验证：`cargo check` 确认 Pusher 编译通过

## 6. 简化 main.rs

- [x] 6.1 移除 `clap` 相关的 `#[derive(Parser)]` 结构体和 mode 匹配逻辑
- [x] 6.2 config 路径从 `std::env::args().nth(1)` 获取，默认 `"config.toml"`
- [x] 6.3 创建 `Pipeline::new()` 获取 `(pipeline, articles_rx, push_rx)`
- [x] 6.4 使用 `pipeline.cancel.clone()` 启动 Ctrl+C 监听 task
- [x] 6.5 spawn 三个后台任务，传入各自需要的 receiver 和 pipeline clone
- [x] 6.6 axum `serve` 改用 `with_graceful_shutdown(pipeline.cancel.cancelled_owned())`
- [x] 6.7 移除 `src/main.rs` 中不再需要的 `use clap::Parser;` 等导入
- [x] 6.8 验证：`cargo check` 确认整个项目编译通过

## 7. 更新 Routes 和 Handlers

- [x] 7.1 修改 `src/routes.rs` — `AppState` 新增 `pipeline: Pipeline` 字段
- [x] 7.2 修改 `create_router` 签名，接收并注入 `pipeline` 到 `AppState`
- [x] 7.3 修改 `src/handlers/query.rs` — `trigger_filter` 调用 `run_filter_once` 后检查返回值，若 `true` 则 `push_ready_tx.try_send(PipelineEvent::NewData)`
- [x] 7.4 验证：`cargo check` 确认全项目编译通过

## 8. 更新部署配置

- [x] 8.1 更新 `Dockerfile` — `CMD` 从 `["hotspot", "all"]` 改为 `["hotspot"]`
- [x] 8.2 检查所有 `config.toml` 变体（如 `config.example.toml`）是否包含 `parser.interval_seconds`

## 9. 编译与基础验证

- [x] 9.1 运行 `cargo build` 确认全项目编译成功（无 warning 更佳）
- [x] 9.2 运行 `cargo test` 确认已有测试全部通过
- [ ] 9.3 手动启动验证：`cargo run -- config.toml`，确认日志显示三个模块启动 + server listening
- [ ] 9.4 按 Ctrl+C，确认日志显示三个模块 "shutting down gracefully"，进程干净退出
