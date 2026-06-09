## Why

当前三个后台模块（Parser、Filter、Pusher）均使用固定间隔的 `loop { sleep; do_work; }` 模式，存在盲目轮询浪费资源、模块间无协调导致延迟、无法优雅关闭、Parser 间隔硬编码不可配置等问题。此次重构将轮询模式升级为事件驱动 + 兜底轮询 + 优雅关闭，实现近实时处理和可控生命周期。

## What Changes

- 新增 `src/pipeline.rs` 事件总线模块，定义 `PipelineEvent` 和 `Pipeline` 结构体，通过 `mpsc` channel 实现 Parser → Filter → Pusher 的事件通知链路
- 新增 `tokio-util` 依赖，提供 `CancellationToken` 实现优雅关闭
- Parser 新增 `interval_seconds` 配置项（替换硬编码的 30s），插入新文章后通过 channel 通知 Filter
- Filter 的 `run_filter_once` 返回 `bool`（是否产生了新 push_record），收到 Parser 通知后立即执行
- Pusher 收到 Filter 通知后立即执行
- 移除 CLI mode 选择（`clap` 依赖），启动命令简化为 `hotspot` 或 `hotspot <config_path>`
- **BREAKING**: `config.toml` 的 `[parser]` 段需新增 `interval_seconds` 字段，缺少会导致启动失败
- **BREAKING**: 不再支持 `hotspot parser|filter|pusher|api` 单独启动模式，始终同时运行所有模块 + API server
- 手动触发 API（`POST /trigger/filter`）执行后通过 channel 通知 Pusher

### Non-goals

- 不改变数据库 schema
- 不改变 API 接口（REST endpoint 保持不变）
- 不改变 Parser 的 fetch + insert 核心逻辑
- 不改变 Filter 的 Aho-Corasick 匹配 + burst detection 算法
- 不改变 Pusher 的 webhook 发送 + 指数退避重试逻辑
- 不为 `trigger_fetch` 增加即时 channel 通知（保持原有行为，等下一次 interval tick）

## Capabilities

### New Capabilities

- `event-driven-pipeline`: Pipeline 事件总线，通过 `tokio::sync::mpsc` channel 实现模块间低延迟事件通知（Parser → Filter → Pusher），搭配 `tokio::time::interval` 兜底轮询防止消息丢失
- `graceful-shutdown`: 基于 `tokio_util::sync::CancellationToken` 的优雅关闭机制，Ctrl+C 后各模块完成当前工作再退出，axum server 通过 `with_graceful_shutdown` 等待关闭

### Modified Capabilities

- `parser-module`: 循环体从 `loop { sleep; work }` 改为 `tokio::select! { cancel, interval, ... }` 三路监听；`interval_seconds` 从硬编码 30s 改为 `config.toml` 可配置
- `filter-module`: 循环体从 `loop { sleep; work }` 改为 `tokio::select! { cancel, interval, articles_rx }` 三路监听；`run_filter_once` 返回 `bool` 表示是否产生了新 push_record；收到 Parser 通知时立即执行
- `pusher-module`: 循环体从 `loop { sleep; work }` 改为 `tokio::select! { cancel, interval, push_rx }` 三路监听；收到 Filter 通知时立即执行
- `trigger-apis`: `trigger_filter` 执行后通过 `push_ready_tx` channel 通知 Pusher，减少热点推送延迟
- `backend-project-scaffold`: 移除 `clap` CLI mode 参数解析，`main.rs` 始终同时启动所有模块 + API server；启动命令从 `hotspot all` 简化为 `hotspot [config_path]`

## Impact

- **依赖**: 新增 `tokio-util = "0.7"`；移除 `clap`
- **文件变更**: 1 个新文件 (`src/pipeline.rs`)，8 个修改文件 (`Cargo.toml`, `src/config.rs`, `config.toml`, `src/services/parser.rs`, `src/services/filter.rs`, `src/services/pusher.rs`, `src/main.rs`, `src/routes.rs`, `src/handlers/query.rs`)
- **配置迁移**: 所有 `config.toml` 需在 `[parser]` 下新增 `interval_seconds` 字段
- **部署影响**: Dockerfile 中的 `CMD ["hotspot", "all"]` 需改为 `CMD ["hotspot"]`
- **前端**: 无影响
