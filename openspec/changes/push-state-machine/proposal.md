## Why

当前推送管道存在 6 个痛点：(1) Article 无状态区分，无法识别"匹配到关键字"和"未匹配"的文章；(2) Filter 无并发保护，两次 `run_filter_once` 可能同时处理同一批文章；(3) Pusher 使用线性退避而非指数退避；(4) 耗尽重试次数的记录语义不清（`failed` + `next_retry_at=NULL`）；(5) Pusher 崩溃后 `processing` 记录永久卡死；(6) 推送失败原因无处记录。本次变更为推送管道引入双层状态机，一揽子解决上述问题，提升系统可靠性和可观测性。

## What Changes

- **Article 状态机（新增）**: 引入 `pending → processing → matched/skipped` 四态流转，区分命中与未命中文章
- **Filter 原子 claim（新增）**: `claim_pending_articles` 通过原子 UPDATE + WHERE status='pending' 防止并发重复处理
- **PushRecord 指数退避**: 线性退避替换为 `base * 2^(n-1)` 指数退避，新增 `retry_max_seconds` 上限
- **PushRecord dead 状态（新增）**: 耗尽重试次数的记录标记为 `dead` 终态，与可重试 `failed` 明确区分
- **陈旧 processing 恢复（新增）**: Pusher 每次运行时检测并恢复卡死超过 N 分钟的 `processing` 记录
- **last_error 字段（新增）**: push_records 新增 `last_error` 列，记录推送失败原因
- **查询 API 参数变更**: 文章列表的 `processed` 参数改为 `status`（**BREAKING**，但保留向后兼容过渡期）
- **PusherConfig 扩展**: 新增 `retry_max_seconds` 和 `stale_timeout_minutes` 配置项
- **两套数据库 migration**: article 添加 `status` 列 + push_records 添加 `last_error` 列并迁移存量数据

## Capabilities

### New Capabilities

- `article-status-machine`: Article 状态机 — pending/processing/matched/skipped 四态，含原子 claim 防并发、分类标记 matched/skipped
- `push-record-dead-letter`: PushRecord dead letter + 陈旧恢复 — dead 终态、last_error 记录、processing 超时自动恢复为 pending

### Modified Capabilities

- `filter-module`: 文章获取从 `SELECT WHERE processed_at IS NULL` 改为原子 claim `pending→processing`；处理完成后分类标记为 `matched` 或 `skipped`
- `pusher-module`: 退避策略从线性改为指数退避（含上限配置）；推送记录增加 `dead` 终态和 `last_error`；新增陈旧 processing 恢复步骤
- `data-models`: Article 模型新增 `status: String` 字段；PushRecord 模型新增 `last_error: Option<String>` 字段；ArticleQuery 新增 `status: Option<String>` 替代 `processed`；PushRecordWithChannel 新增 `last_error`
- `database-schema`: articles 表新增 `status` 列 + 索引；push_records 表新增 `last_error` 列；存量数据迁移
- `query-apis`: 文章列表 API 参数从 `?processed=true/false` 改为 `?status=pending|processing|matched|skipped`
- `config-validation`: PusherConfig 新增 `retry_max_seconds`（退避上限）和 `stale_timeout_minutes`（陈旧超时）字段及默认值校验

## Non-goals

- 不改变 Parser 模块的取文章或去重逻辑
- 不改变 PushRecord 的 optimistic locking 机制（已有 `pusher-atomic-claim`）
- 不改变前端 Dashboard 或 Article 列表页面（后续单独变更）
- 不移除 `processed_at` 字段（保留向后兼容）

## Impact

- **数据库**: 新增 2 个 migration 文件；`build.rs` 需包含新 migration 路径
- **后端模块**: `src/db/article.rs`、`src/db/push_record.rs`、`src/services/filter.rs`、`src/services/pusher.rs`
- **模型层**: `src/models/article.rs`、`src/models/push_record.rs`
- **配置**: `src/config.rs`（PusherConfig 结构体）、`config.toml`（默认配置）
- **API**: `src/handlers/query.rs`（文章列表查询参数变更，BREAKING）
- **测试**: `src/services/filter.rs` 的单元测试需更新
- **前端**: 文章列表 API 参数变更，前端调用方需适配（本期不做，后续单独变更）
