# Event-Driven Pipeline Refactor

## 1. Background & Problem Statement

当前三个后台服务（Parser、Filter、Pusher）均采用相同的 `loop { sleep; do_work; }` 模式：

```rust
// src/services/parser.rs (line 98)
pub async fn start_parser_loop(pool: SqlitePool, config: ParserConfig) {
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(30)).await; // 硬编码 30s
        // ... fetch due sources ...
    }
}

// src/services/filter.rs (line 270)
pub async fn start_filter_loop(pool: SqlitePool, config: FilterConfig) {
    let interval = std::time::Duration::from_secs(config.interval_seconds);
    loop {
        tokio::time::sleep(interval).await;
        run_filter_once(&pool, &config).await;
    }
}

// src/services/pusher.rs (line 252)
pub async fn start_pusher_loop(pool: SqlitePool, config: PusherConfig) {
    let interval = std::time::Duration::from_secs(config.interval_seconds);
    loop {
        tokio::time::sleep(interval).await;
        run_pusher_once(&pool, &config).await;
    }
}
```

**问题清单：**

| 问题 | 影响 |
|------|------|
| 无法优雅关闭 | Ctrl+C 时后台任务被强制杀死，可能丢失进行中的操作 |
| 盲目轮询 | 无论是否有新数据，都按固定间隔唤醒，浪费 CPU 和 I/O |
| 模块间无协调 | Parser 刚插入新文章，Filter 要等到下一次轮询才知道（最延迟 filter.interval_seconds） |
| 无错误恢复 | 某个 loop panic 后整个后台任务静默消失，无重启机制 |
| Parser 间隔硬编码 | 30 秒间隔写死在代码中，不可通过 config.toml 配置 |
| CLI mode 冗余 | `main.rs` 通过 CLI 参数选择启动模式（all/api/parser/filter/pusher），增加了代码复杂度和维护成本 |

---

## 2. Architecture Overview

### 核心设计：事件驱动 + 兜底轮询 + 优雅关闭

每个后台模块同时监听三种信号（通过 `tokio::select!`）：

1. **事件通知**（`tokio::sync::mpsc` channel）— 上游模块有新数据时立即触发，实现近实时处理
2. **兜底 interval**（`tokio::time::interval`）— 防止消息丢失或延迟，定期轮询作为安全网
3. **取消信号**（`tokio_util::sync::CancellationToken`）— 支持优雅关闭

### 新增依赖

```toml
# Cargo.toml
tokio-util = "0.7"    # 提供 CancellationToken
```

移除不再需要的依赖：

```toml
# 移除
clap = { version = "4", features = ["derive"] }  # CLI mode 选择不再需要
```

---

## 3. Data Flow Diagram

```
+----------+    articles_ready     +----------+    push_ready      +----------+
|  Parser  | ----[mpsc channel]--> |  Filter  | ----[mpsc channel]--> |  Pusher  |
|          |                       |          |                       |          |
| interval |                       | interval |                       | interval |
| (兜底)   |                       | (兜底)   |                       | (兜底)   |
| cancel   |                       | cancel   |                       | cancel   |
+----------+                       +----------+                       +----------+
      ^                                 ^                                  ^
      |                                 |                                  |
      +---- CancellationToken (全局共享，Ctrl+C 触发) -----------------------+
```

**信号流向：**

```
Parser 插入新文章 --> articles_ready_tx.send(NewData)
                             |
                             v
                     Filter 收到信号 --> 立即执行 run_filter_once()
                                         |
                                         v (产生了新 hot_event)
                               push_ready_tx.send(NewData)
                                         |
                                         v
                               Pusher 收到信号 --> 立即执行 run_pusher_once()
```

**手动触发 API 信号流：**

```
POST /api/v1/trigger/filter --> run_filter_once() --> push_ready_tx.send(NewData)
POST /api/v1/trigger/pusher --> run_pusher_once()
POST /api/v1/sources/{id}/fetch --> reset_last_fetched() --> 立即执行一次 parser fetch
```

---

## 4. Detailed Module Changes

### 4.1 New File: `src/pipeline.rs`

创建 Pipeline 事件总线模块，定义跨模块通信的 channel 和共享上下文：

```rust
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

/// Pipeline 事件信号 — 轻量通知，不携带数据载荷
#[derive(Debug, Clone, Copy)]
pub enum PipelineEvent {
    /// 上游通知：有新数据需要处理
    NewData,
}

/// 所有后台任务共享的管道上下文
#[derive(Clone)]
pub struct Pipeline {
    /// Parser -> Filter 的通知通道
    pub articles_ready_tx: mpsc::Sender<PipelineEvent>,
    /// Filter -> Pusher 的通知通道
    pub push_ready_tx: mpsc::Sender<PipelineEvent>,
    /// 全局取消令牌
    pub cancel: CancellationToken,
}

impl Pipeline {
    /// 创建新 Pipeline，返回 (Pipeline, articles_ready_rx, push_ready_rx)
    pub fn new() -> (Self, mpsc::Receiver<PipelineEvent>, mpsc::Receiver<PipelineEvent>) {
        let cancel = CancellationToken::new();
        let (articles_ready_tx, articles_ready_rx) = mpsc::channel(16);
        let (push_ready_tx, push_ready_rx) = mpsc::channel(16);

        let pipeline = Pipeline {
            articles_ready_tx,
            push_ready_tx,
            cancel,
        };

        (pipeline, articles_ready_rx, push_ready_rx)
    }
}
```

**设计决策：**
- Channel 容量 16 足够（仅传递信号，不传数据）
- `CancellationToken` 支持 clone，各模块持有自己的 clone
- Receiver 不放在 Pipeline 中（不可 Clone），通过 `Pipeline::new()` 返回值传递

**模块注册：**

需要在 `src/main.rs` 的 mod 声明中增加 `mod pipeline;`。

---

### 4.2 Config Changes: `src/config.rs` + `config.toml`

**`src/config.rs`** — `ParserConfig` 新增 `interval_seconds` 字段：

```rust
#[derive(Debug, Deserialize, Clone)]
pub struct ParserConfig {
    pub interval_seconds: u64,          // 新增：轮询间隔（原硬编码 30s）
    pub max_concurrent_fetches: usize,
    pub default_user_agent: String,
    pub default_timeout_seconds: u64,
}
```

**`config.toml`** — `[parser]` 增加配置项：

```toml
[parser]
interval_seconds = 30              # 新增
max_concurrent_fetches = 10
default_user_agent = "HotspotMonitor/1.0"
default_timeout_seconds = 30
```

---

### 4.3 Refactor Parser: `src/services/parser.rs`

**函数签名变更：**

```rust
// Before
pub async fn start_parser_loop(pool: SqlitePool, config: ParserConfig)

// After
pub async fn start_parser_loop(pool: SqlitePool, config: ParserConfig, pipeline: Pipeline)
```

**循环体改造：**

```rust
pub async fn start_parser_loop(pool: SqlitePool, config: ParserConfig, pipeline: Pipeline) {
    let parser = Arc::new(RssParser::new(&config));
    let semaphore = Arc::new(Semaphore::new(config.max_concurrent_fetches));
    let mut interval = tokio::time::interval(
        std::time::Duration::from_secs(config.interval_seconds)
    );

    loop {
        tokio::select! {
            _ = pipeline.cancel.cancelled() => {
                tracing::info!("Parser: shutting down gracefully");
                break;
            }
            _ = interval.tick() => {
                // 执行原有的 fetch + insert 逻辑：
                // 1. db::source::list_due_sources()
                // 2. for each source: spawn task -> fetch_and_parse -> insert articles
                // 3. 成功插入文章后，通知 Filter：
                //    if total_inserted > 0 {
                //        let _ = pipeline.articles_ready_tx.try_send(PipelineEvent::NewData);
                //    }
            }
        }
    }
}
```

**关键细节：**
- 使用 `try_send` 而非 `send().await`，避免在 channel 满时阻塞（信号可丢弃，兜底 interval 会补偿）
- `interval.tick()` 替换 `sleep()`，避免时间漂移
- 原有的 fetch + insert 逻辑保持不变，仅在插入成功后增加 channel 通知

---

### 4.4 Refactor Filter: `src/services/filter.rs`

**`run_filter_once` 签名变更：**

```rust
// Before
pub async fn run_filter_once(pool: &SqlitePool, config: &FilterConfig)

// After — 返回是否有新 hot_event/push_record 产生
pub async fn run_filter_once(pool: &SqlitePool, config: &FilterConfig) -> bool
```

返回值逻辑：在现有的 burst detection 阶段，如果创建了新 push_record 则返回 `true`，否则返回 `false`。

**`start_filter_loop` 签名变更：**

```rust
// Before
pub async fn start_filter_loop(pool: SqlitePool, config: FilterConfig)

// After
pub async fn start_filter_loop(
    pool: SqlitePool,
    config: FilterConfig,
    pipeline: Pipeline,
    mut articles_rx: mpsc::Receiver<PipelineEvent>,
)
```

**循环体改造：**

```rust
pub async fn start_filter_loop(
    pool: SqlitePool,
    config: FilterConfig,
    pipeline: Pipeline,
    mut articles_rx: mpsc::Receiver<PipelineEvent>,
) {
    let mut interval = tokio::time::interval(
        std::time::Duration::from_secs(config.interval_seconds)
    );

    loop {
        tokio::select! {
            _ = pipeline.cancel.cancelled() => {
                tracing::info!("Filter: shutting down gracefully");
                break;
            }
            _ = interval.tick() => {
                let has_new = run_filter_once(&pool, &config).await;
                if has_new {
                    let _ = pipeline.push_ready_tx.try_send(PipelineEvent::NewData);
                }
            }
            Some(_) = articles_rx.recv() => {
                // Parser 通知有新文章到达，立即执行 filter
                let has_new = run_filter_once(&pool, &config).await;
                if has_new {
                    let _ = pipeline.push_ready_tx.try_send(PipelineEvent::NewData);
                }
            }
        }
    }
}
```

**`run_filter_once` 返回值改造要点：**

```rust
pub async fn run_filter_once(pool: &SqlitePool, config: &FilterConfig) -> bool {
    let mut created_push_records = false;

    // ... 现有步骤 1-3 不变 ...

    // 步骤 5: Burst detection（修改点）
    for kw in &keywords {
        // ... 现有逻辑 ...
        if is_hotspot {
            // ... 现有逻辑 ...
            if db::push_record::insert_push_records_for_event(...).await.is_ok() {
                created_push_records = true;  // 标记有新 push record
            }
        }
    }

    // 步骤 6: Mark processed（不变）
    // ...

    created_push_records
}
```

---

### 4.5 Refactor Pusher: `src/services/pusher.rs`

**函数签名变更：**

```rust
// Before
pub async fn start_pusher_loop(pool: SqlitePool, config: PusherConfig)

// After
pub async fn start_pusher_loop(
    pool: SqlitePool,
    config: PusherConfig,
    pipeline: Pipeline,
    mut push_rx: mpsc::Receiver<PipelineEvent>,
)
```

**循环体改造：**

```rust
pub async fn start_pusher_loop(
    pool: SqlitePool,
    config: PusherConfig,
    pipeline: Pipeline,
    mut push_rx: mpsc::Receiver<PipelineEvent>,
) {
    let mut interval = tokio::time::interval(
        std::time::Duration::from_secs(config.interval_seconds)
    );

    loop {
        tokio::select! {
            _ = pipeline.cancel.cancelled() => {
                tracing::info!("Pusher: shutting down gracefully");
                break;
            }
            _ = interval.tick() => {
                run_pusher_once(&pool, &config).await;
            }
            Some(_) = push_rx.recv() => {
                run_pusher_once(&pool, &config).await;
            }
        }
    }
}
```

`run_pusher_once` 签名和内部逻辑不变。

---

### 4.6 Simplify main.rs

**移除 CLI mode 参数，移除 `clap` 依赖：**

```rust
// Before: 使用 clap 解析 CLI 参数，通过 mode 选择启动模块
#[derive(Parser)]
struct Cli {
    #[arg(long, default_value = "config.toml")]
    config: String,
    #[arg(default_value = "all")]
    mode: String,
}

// After: 仅保留 config 路径参数（可用 std::env::args 或保留极简 clap）
```

**简化后的 main 函数：**

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    // 加载配置（简化：从命令行第一个参数或默认 config.toml）
    let config_path = std::env::args().nth(1).unwrap_or_else(|| "config.toml".to_string());
    let config = config::AppConfig::load(&config_path)?;

    // 初始化数据库
    let db_dir = std::path::Path::new(&config.database.path).parent().unwrap();
    std::fs::create_dir_all(db_dir)?;
    let pool = db::init_pool(&config.database.path).await?;
    sqlx::migrate!("./docs/migrations").run(&pool).await?;
    ensure_initial_token(&pool, &config).await?;

    // 创建 Pipeline 事件总线
    let (pipeline, articles_rx, push_rx) = pipeline::Pipeline::new();

    // Spawn Parser
    tokio::spawn(services::parser::start_parser_loop(
        pool.clone(), config.parser.clone(), pipeline.clone(),
    ));

    // Spawn Filter
    tokio::spawn(services::filter::start_filter_loop(
        pool.clone(), config.filter.clone(), pipeline.clone(), articles_rx,
    ));

    // Spawn Pusher
    tokio::spawn(services::pusher::start_pusher_loop(
        pool.clone(), config.pusher.clone(), pipeline.clone(), push_rx,
    ));

    tracing::info!("All background services started (parser + filter + pusher)");

    // Ctrl+C 优雅关闭
    let cancel = pipeline.cancel.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        tracing::info!("Received Ctrl+C, initiating graceful shutdown...");
        cancel.cancel();
    });

    // 启动 API server（带优雅关闭）
    let app = routes::create_router(pool.clone(), config.clone(), pipeline.clone());
    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port).parse()?;
    tracing::info!("Server listening on {}", addr);
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            pipeline.cancel.cancelled_owned().await;
        })
        .await?;

    Ok(())
}
```

---

### 4.7 Update Routes: `src/routes.rs`

**AppState 增加 Pipeline：**

```rust
#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub config: AppConfig,
    pub pipeline: Pipeline,  // 新增
}
```

**`create_router` 签名变更：**

```rust
// Before
pub fn create_router(pool: SqlitePool, config: AppConfig) -> Router

// After
pub fn create_router(pool: SqlitePool, config: AppConfig, pipeline: Pipeline) -> Router
```

---

### 4.8 Update Trigger Handlers: `src/handlers/query.rs` + `src/handlers/source.rs`

**`query.rs` — trigger_filter 增加下游通知：**

```rust
pub async fn trigger_filter(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let has_new = crate::services::filter::run_filter_once(&state.pool, &state.config.filter).await;
    if has_new {
        let _ = state.pipeline.push_ready_tx.try_send(PipelineEvent::NewData);
    }
    Ok(ApiResponse::ok(json!({"message": "Filter executed"})))
}
```

`trigger_pusher` 不需要变化（无下游模块）。

**`source.rs` — trigger_fetch 增加即时通知：**

`trigger_fetch` 目前只 reset `last_fetched_at`，等待 Parser 下一次轮询。可选优化：通过 channel 通知 Parser 立即执行。但这需要 Parser 也监听一个 `fetch_now_rx` 通道，增加复杂度。

**建议**：当前阶段不增加此功能，保持原有行为（reset last_fetched_at，等下一次 interval tick）。如后续需要即时 fetch，可作为增量改进。

---

## 5. Migration Notes

### 向后兼容性

| 项目 | 影响 |
|------|------|
| `config.toml` | 需要在 `[parser]` 下新增 `interval_seconds = 30`。**不兼容旧配置**（缺少该字段会解析失败） |
| CLI 参数 | 移除 `mode` 参数。启动命令从 `hotspot all` 简化为 `hotspot` 或 `hotspot config.toml` |
| API 接口 | 无变化，所有 REST endpoint 保持兼容 |
| 数据库 | 无 schema 变更 |

### 回退方案

如果新架构出现问题，可以：
1. 恢复 `loop { sleep }` 模式（代码已在 git 历史中）
2. 重新添加 `clap` 依赖
3. 从 `config.toml` 移除 `parser.interval_seconds`

### 配置迁移

旧 `config.toml` 需要手动添加一行：

```diff
 [parser]
+interval_seconds = 30
 max_concurrent_fetches = 10
 default_user_agent = "HotspotMonitor/1.0"
 default_timeout_seconds = 30
```

---

## 6. Testing Strategy

### 单元测试

| 测试目标 | 测试内容 |
|----------|----------|
| `Pipeline::new()` | 验证 channel 创建成功，clone 后 sender/receiver 数量正确 |
| `run_filter_once` 返回值 | 模拟有/无 hotspot 场景，验证 bool 返回值正确 |
| `CancellationToken` | 验证 cancel 后各 loop 正常退出 |

### 集成测试

| 场景 | 预期行为 |
|------|----------|
| Parser 插入文章 | Filter 在 < 1s 内被触发（而非等待 interval） |
| Filter 创建 push record | Pusher 在 < 1s 内被触发 |
| Ctrl+C | 所有后台任务日志 "shutting down gracefully"，进程干净退出 |
| Channel 满（16 条未消费） | `try_send` 返回错误但不阻塞，兜底 interval 继续工作 |
| 手动 trigger API | `POST /trigger/filter` 执行后 Pusher 被通知 |

### 压力场景

| 场景 | 预期行为 |
|------|----------|
| 大量 source 同时到期 | Parser 并发限制（Semaphore）不变，Filter 收到多次信号但合并处理 |
| Filter 处理慢于 Parser 产出 | Channel 缓冲吸收突发，interval 兜底保证最终处理 |

---

## 7. File Change Summary

| 文件 | 变更类型 | 说明 |
|------|---------|------|
| `Cargo.toml` | 修改 | 新增 `tokio-util = "0.7"`，移除 `clap` |
| `src/pipeline.rs` | **新建** | Pipeline 事件总线 + PipelineEvent 枚举 |
| `src/config.rs` | 修改 | `ParserConfig` 增加 `interval_seconds` |
| `config.toml` | 修改 | `[parser]` 增加 `interval_seconds = 30` |
| `src/services/parser.rs` | 修改 | 重构循环 + channel 通知 + interval 替换 sleep |
| `src/services/filter.rs` | 修改 | 重构循环 + channel 监听/通知 + `run_filter_once` 返回 bool |
| `src/services/pusher.rs` | 修改 | 重构循环 + channel 监听 |
| `src/main.rs` | 修改 | 移除 CLI mode，添加 Pipeline 初始化 + 优雅关闭 |
| `src/routes.rs` | 修改 | AppState 增加 Pipeline 字段 |
| `src/handlers/query.rs` | 修改 | trigger API 发送 channel 信号 |
