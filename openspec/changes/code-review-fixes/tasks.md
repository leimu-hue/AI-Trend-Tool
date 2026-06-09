## 1. 配置校验与代码质量

- [x] 1.1 `src/config.rs`: 为 `AppConfig` 添加 `validate()` 方法，校验 port、database.path、parser/filter/pusher 的 interval_seconds、max_concurrent_fetches、batch_size、max_retries 均 > 0。在 `load()` 中反序列化后调用 `validate()`。验证：`cargo check`
- [x] 1.2 `src/handlers/token.rs`: 修改 `revoke_token` 函数，先调用 `db::token::get_token_by_id()` 检查存在性，不存在返回 404，存在再执行 `db::token::revoke_token()`。验证：`cargo check`
- [x] 1.3 `src/db/article.rs`: 重构 `list_articles` 和 `count_articles`，提取 `build_article_filter` 辅助函数，用动态绑定替代 4 路 match 分支。验证：`cargo check`

## 2. P0 Bug 修复

- [x] 2.1 `src/db/push_record.rs`: 修改 `list_retry_due_records` 函数签名，接受 `max_retries: u32` 参数，SQL 中用 `?` 绑定替代硬编码 `< 3`。验证：`cargo check`
- [x] 2.2 `src/services/pusher.rs`: 调用 `list_retry_due_records` 处传入 `config.max_retries`。验证：`cargo check`
- [x] 2.3 `docs/migrations/20260609000001_hot_events_unique.sql`: 创建迁移文件，重建 `hot_events` 表添加 `UNIQUE(keyword_id, hour_bucket)` 约束，迁移去重数据。验证：迁移 SQL 语法正确
- [x] 2.4 `src/services/filter.rs`: 将 `upsert_hot_event_record` 改为使用 `ON CONFLICT(keyword_id, hour_bucket) DO UPDATE` 语法，删除旧的 DELETE+INSERT 逻辑。验证：`cargo check`
- [x] 2.5 删除 `src/db/hot_event.rs` 中的 `insert_hot_event` 函数（确认无其他调用点后删除）。验证：`cargo check`

## 3. P1 安全加固 — Token 哈希存储

- [x] 3.1 `Cargo.toml`: 在 `[dependencies]` 中添加 `sha2 = "0.10"`。验证：`cargo check`
- [x] 3.2 `docs/migrations/20260609000002_token_hash.sql`: 创建迁移文件，`ALTER TABLE api_tokens ADD COLUMN token_hash TEXT NOT NULL DEFAULT ''`，并创建 UNIQUE INDEX。验证：迁移 SQL 语法正确
- [x] 3.3 `src/models/token.rs`: 在 `ApiToken` 和 `ApiTokenInfo` 结构体中添加 `token_hash: String` 字段，更新 `From<ApiToken>` 实现。验证：`cargo check`
- [x] 3.4 `src/db/token.rs`: 添加 `hash_token` 函数（SHA-256 via `sha2`）；修改 `create_token` 和 `insert_initial_token` 同时写入 `token` 和 `token_hash`；修改 `get_token_by_value` 按 `token_hash` 查询。验证：`cargo check`
- [x] 3.5 `src/middleware/auth.rs`: 修改 token 验证逻辑，对请求中的 Bearer token 先哈希再查询 `api_tokens WHERE token_hash = ?`。验证：`cargo check`

## 4. P1 性能与安全 — reqwest 复用 + 输入校验

- [x] 4.1 `src/services/parser.rs`: 将 `reqwest::Client` 作为 `RssParser` 的成员字段，在 `new()` 中构造，`fetch_and_parse` 中使用 `self.client`。验证：`cargo check`
- [x] 4.2 `src/handlers/source.rs`: 在 `create_source` 和 `update_source` 函数开头添加 name/url 校验（非空、URL scheme）。验证：`cargo check`
- [x] 4.3 `src/handlers/keyword.rs`: 在 `create_keyword` 和 `update_keyword` 函数开头添加 word/std_multiplier/min_hot_count 校验。验证：`cargo check`
- [x] 4.4 `src/handlers/channel.rs`: 在 `create_channel` 和 `update_channel` 函数开头添加 name/config/JSON 校验。验证：`cargo check`
- [x] 4.5 `src/handlers/token.rs`: 在 `create_token` 函数开头添加 name 非空校验。验证：`cargo check`

## 5. Health Check 数据库探活

- [x] 5.1 `src/routes.rs`: 修改 `health_check` 函数接受 `State<AppState>`，执行 `SELECT 1` 探测数据库，返回 `{"status": "ok"/"degraded", "database": "ok"/"error"}`。添加 `use axum::extract::State`。验证：`cargo check`
- [x] 5.2 `src/routes.rs`: 调整路由结构，将 `/health` 放在带 `State` 的外层 Router 上与 `nest("/api/v1", api)` 同级，确保 health 在 auth middleware 之外。验证：`cargo check`

## 6. P2 性能优化 — 批量操作与并发

- [x] 6.1 `src/db/keyword_mention.rs`: 添加 `batch_insert_keyword_mentions(pool, mentions: &[(i64, i64)])` 函数，每 100 条 chunk 用单条 `INSERT OR IGNORE INTO ... VALUES (?, ?), ...`。验证：`cargo check`
- [x] 6.2 `src/services/filter.rs`: 修改匹配循环，先收集所有 mentions 到 Vec，循环结束后调用 `batch_insert_keyword_mentions`。验证：`cargo check`
- [x] 6.3 `src/db/hot_event.rs`: 添加 `get_all_hourly_counts(pool, hours: i32) -> Result<Vec<(i64, String, i32)>>` 批量查询函数。验证：`cargo check`
- [x] 6.4 `src/services/filter.rs`: 在关键词循环前调用 `get_all_hourly_counts` 一次性加载所有历史数据，在内存中按 keyword_id 分组计算 mean/stddev，替换逐关键词 `compute_historical_stats` 查询。验证：`cargo check`
- [x] 6.5 `Cargo.toml`: 在 `[dependencies]` 中添加 `futures = "0.3"`。验证：`cargo check`
- [x] 6.6 `src/services/pusher.rs`: 将串行推送循环改为 `futures::stream::for_each_concurrent(8, ...)`，每个任务 clone pool/config/client。验证：`cargo check`

## 7. 优雅关闭

- [x] 7.1 `src/main.rs`: 保存 `parser_handle`、`filter_handle`、`pusher_handle` 三个 `JoinHandle`；在 `axum::serve(...).await` 之后用 `tokio::join!` 等待三个后台任务完成。验证：`cargo check`

## 8. 最终验证

- [x] 8.1 `cargo build`: 确保全部修改编译通过，无 warning
- [x] 8.2 `cargo sqlx migrate run`: 验证迁移能在空数据库上成功执行
- [x] 8.3 手动测试：启动 `cargo run -- all`，验证健康检查端点 `/health` 返回数据库状态；创建 source/keyword/channel 时提交空 name 验证 400 响应；创建 token 后验证可通过认证
