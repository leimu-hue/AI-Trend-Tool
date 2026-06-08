## Context

当前后台三大模块（Parser、Filter、Pusher）均采用 `loop { sleep; do_work; }` 模式：
- Parser 硬编码 30s 间隔，不可通过配置调整
- Filter 和 Pusher 虽可通过 `config.toml` 配置间隔，但无法响应上游模块的新数据——Parser 刚插入文章后，Filter 必须等到下一次 interval tick 才能处理（最长延迟 `filter.interval_seconds`）
- Ctrl+C 时后台任务被强制杀死，无法优雅完成进行中的操作
- CLI 通过 `clap` 提供 `all|api|parser|filter|pusher` 模式选择，增加了入口复杂度

此次重构将轮询模式升级为 **事件驱动 + 兜底轮询 + 优雅关闭**，通过 `tokio::sync::mpsc` channel 实现 Parser → Filter → Pusher 的近实时事件通知链路，同时保留 interval 作为兜底安全网。

## Goals / Non-Goals

**Goals:**
- 实现 Parser → Filter → Pusher 的事件驱动通知链路，降低模块间延迟
- 实现基于 `CancellationToken` 的优雅关闭，确保 Ctrl+C 后各模块完成当前工作再退出
- 将 Parser 间隔从硬编码改为 `config.toml` 可配置
- 简化 `main.rs`，移除 CLI mode 选择，始终同时运行所有模块 + API server
- 保持所有现有 API 接口不变

**Non-Goals:**
- 不改变数据库 schema
- 不改变核心业务逻辑（RSS 抓取、Aho-Corasick 匹配、burst detection、webhook 推送）
- 不增加 `trigger_fetch` 的即时 channel 通知（保持原有行为）
- 不引入分布式消息队列（如 Redis/Kafka）——保持单进程内通信

## Decisions

### Decision 1: mpsc channel 而非 broadcast

**选择**: `tokio::sync::mpsc`（多生产者单消费者）

**原因**:
- 每个下游模块只需一个消费者（Filter 消费 Parser 信号，Pusher 消费 Filter 信号）
- `broadcast` channel 的每条消息会唤醒所有接收者，但此处只需要点对点通知
- `mpsc` 语义更简单，`try_send` 在 channel 满时不阻塞

**备选方案**: `tokio::sync::broadcast` — 若未来需要一对多通知（如多个 Filter 实例），可切换。当前无此需求。

### Decision 2: try_send 而非 send().await

**选择**: `try_send`（非阻塞发送）

**原因**:
- 事件信号是"提示"，不是必须送达的"命令"——channel 满时可丢弃
- 兜底 interval 保证最终会处理，信号丢失只是延迟增加而非数据丢失
- 避免上游模块因下游消费慢而被阻塞

**风险**: 高频场景下信号可能连续丢失，导致下游完全依赖 interval 兜底。缓解：channel 容量 16 提供足够缓冲；interval 保证最终一致性。

### Decision 3: CancellationToken 而非自定义信号

**选择**: `tokio_util::sync::CancellationToken`

**原因**:
- 标准库方案，API 稳定，与 tokio 生态集成良好
- `cancelled()` 返回 `Future`，可直接用于 `tokio::select!`
- `cancelled_owned()` 支持 `with_graceful_shutdown` 闭包
- 只需新增一个轻量依赖（`tokio-util = "0.7"`）

**备选方案**: `tokio::sync::watch` channel — 需要手动管理初始值和更新，不如 `CancellationToken` 语义清晰。

### Decision 4: 移除 CLI mode，始终运行全部模块

**选择**: 移除 `clap` 依赖和 mode 参数，`main.rs` 始终 spawn 所有后台任务 + API server

**原因**:
- 事件驱动架构下，Parser → Filter → Pusher 形成链路，单独运行某个模块失去意义
- 简化启动命令：`hotspot [config_path]` 替代 `hotspot --config config.toml all`
- 减少 `main.rs` 中 `match mode` 分支，降低维护成本

**备选方案**: 保留 mode 参数但废弃 `parser|filter|pusher` 单独模式，仅支持 `all` 和 `api`。但 `api` 模式也可通过后续配置项（如 `server.only = true`）实现，故当前直接移除。

**迁移**: Dockerfile 中 `CMD ["hotspot", "all"]` 需改为 `CMD ["hotspot"]`。

### Decision 5: Channel 容量 16

**选择**: `mpsc::channel(16)`

**原因**:
- 每条信号仅 `PipelineEvent::NewData` 枚举值，无数据载荷，内存占用极小
- 16 条缓冲足够吸收突发（如 Parser 一次抓取多个 source，发送多个信号）
- 下游模块的 `run_*_once` 执行时间通常在毫秒到秒级，16 条缓冲不会被长时间占满

**备选方案**: 更小的容量（如 4 或 8）——信号丢失概率更高，收益微乎其微。更大的容量（如 64）——无实际收益。

### Decision 6: interval.tick() 替换 sleep()

**选择**: `tokio::time::interval` + `.tick()`

**原因**:
- `sleep().await` 在循环中会累积漂移：如果 `do_work` 耗时 1s，实际间隔变成 `sleep + work` = 31s
- `interval.tick()` 以固定周期触发，自动补偿漂移
- `.tick()` 返回的第一次触发是即时的（burst 模式），适合启动时立即执行一次检查

## Risks / Trade-offs

| 风险 | 缓解措施 |
|------|----------|
| Channel 信号丢失导致下游延迟增加 | 兜底 interval 保证最终处理；Parser/Filter 间隔配置应合理（≤30s） |
| 移除 CLI mode 后无法单独调试某模块 | 可通过日志级别和 trigger API 进行针对性测试；开发阶段仍可临时修改 main.rs |
| `CancellationToken` 取消后 tokio::spawn 的 task 可能未完成 | 每个循环在 `cancelled()` 分支执行 `break`，`with_graceful_shutdown` 等待 task 自然结束；tokio runtime drop 时会等待所有 spawn 完成 |
| `config.toml` 缺少 `interval_seconds` 导致启动失败 | **BREAKING** — 文档中明确标注；可在 `AppConfig::load` 中为 `ParserConfig` 提供 `#[serde(default)]` 兜底值（30s），降低迁移风险 |

## Open Questions

暂无。所有关键决策已在上述分析中确定。
