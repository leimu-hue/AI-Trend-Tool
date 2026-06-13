## 1. 数据库 Migration

- [x] 1.1 创建 `docs/migrations/20260613000001_article_status.sql`：ALTER TABLE articles ADD COLUMN status，存量数据迁移（processed_at IS NOT NULL → matched），创建 idx_articles_status 索引，删除 idx_articles_processed 索引
- [x] 1.2 创建 `docs/migrations/20260613000002_push_record_enhancements.sql`：ALTER TABLE push_records ADD COLUMN last_error，存量 dead 标记（failed + next_retry_at IS NULL + retry_count > 0 → dead）
- [x] 1.3 更新 `build.rs`：确认 `rerun-if-changed` 包含 `docs/migrations/` 目录（或确保 migration 宏自动检测新文件）
- [x] 1.4 验证：`cargo build` 编译通过，migration 文件被嵌入

## 2. 模型层更新

- [x] 2.1 更新 `src/models/article.rs`：`Article` struct 新增 `status: String` 字段；`ArticleQuery` 新增 `status: Option<String>` 字段（保留 `processed: Option<bool>` 过渡期兼容）
- [x] 2.2 更新 `src/models/push_record.rs`：`PushRecord` struct 新增 `last_error: Option<String>` 字段；`PushRecordWithChannel` struct 新增 `last_error: Option<String>` 字段
- [x] 2.3 验证：`cargo check` 无编译错误（可能会有尚未适配的引用告警）

## 3. 配置层更新

- [x] 3.1 更新 `src/config.rs`：`PusherConfig` struct 新增 `retry_max_seconds: u64`（默认 3600）和 `stale_timeout_minutes: u64`（默认 10）字段，添加 `#[serde(default)]` + 默认值函数
- [x] 3.2 更新 `config.toml`：`[pusher]` 段新增 `retry_max_seconds = 3600` 和 `stale_timeout_minutes = 10`
- [x] 3.3 更新 `src/config.rs` 的 `validate()` 函数：校验 `retry_max_seconds > 0` 和 `stale_timeout_minutes > 0`
- [x] 3.4 验证：`cargo check` 无编译错误

## 4. DB 层更新 — Article

- [x] 4.1 在 `src/db/article.rs` 新增 `claim_pending_articles(pool, limit)` 函数：原子 UPDATE pending→processing + SELECT processing 返回
- [x] 4.2 在 `src/db/article.rs` 新增 `mark_articles_matched(pool, ids: &[i64])` 函数：批量更新 processing→matched + processed_at（每 100 分批）
- [x] 4.3 在 `src/db/article.rs` 新增 `mark_articles_skipped(pool, ids: &[i64])` 函数：批量更新 processing→skipped + processed_at（每 100 分批）
- [x] 4.4 更新 `src/db/article.rs` 的 `build_article_filter`：支持 `status` 查询参数；`processed` 参数内部映射为 status
- [x] 4.5 更新现有 `get_unprocessed_articles` 或相关查询函数适配新 status 字段
- [x] 4.6 验证：`cargo check` 无编译错误

## 5. DB 层更新 — PushRecord

- [x] 5.1 在 `src/db/push_record.rs` 新增 `recover_stale_processing_records(pool, timeout_minutes)` 函数
- [x] 5.2 更新 `src/db/push_record.rs` 的 `update_push_status` 函数：增加 `last_error: Option<&str>` 参数
- [x] 5.3 更新 `src/db/push_record.rs` 的 `update_push_status_optimistic` 函数：增加 `last_error: Option<&str>` 参数
- [x] 5.4 更新 `PushRecordWithChannel` 查询 SQL：增加 `pr.last_error` 字段选择
- [x] 5.5 更新 `claim_pending_records` 查询：排除 `status='dead'` 的记录
- [x] 5.6 验证：`cargo check` 无编译错误

## 6. 服务层更新 — Filter

- [x] 6.1 更新 `src/services/filter.rs` 的 `run_filter_once`：将 `get_unprocessed_articles` 替换为 `claim_pending_articles`
- [x] 6.2 在匹配循环中收集 `matched_ids: Vec<i64>` 和 `skipped_ids: Vec<i64>`（按文章是否命中关键字分类）
- [x] 6.3 在事务提交后分别调用 `mark_articles_matched` 和 `mark_articles_skipped`
- [x] 6.4 处理"无启用关键字"场景：将所有 processing 文章标记为 `skipped`
- [x] 6.5 验证：`cargo check` 无编译错误

## 7. 服务层更新 — Pusher

- [x] 7.1 在 `src/services/pusher.rs` 的 `run_pusher_once` 开头添加 `recover_stale_processing_records` 调用
- [x] 7.2 实现/重构 `mark_failed` 函数：支持指数退避计算（`base * 2^(retry_count-1)`）、上限裁剪、dead 终态判定
- [x] 7.3 更新成功推送路径：`update_push_status_optimistic` 调用传入 `last_error = None`
- [x] 7.4 更新失败推送路径（HTTP 非 2xx）：传入 `last_error = "HTTP <status_code>"`
- [x] 7.5 更新失败推送路径（网络错误）：传入 `last_error = "Network error: <简述>"`
- [x] 7.6 更新失败推送路径（无 webhook URL）：传入 `last_error = "Channel N has no valid webhook URL"`
- [x] 7.7 验证：`cargo check` 无编译错误

## 8. API 层更新

- [x] 8.1 更新 `src/handlers/query.rs` 的文章列表 handler：支持 `status` 查询参数，实现 `processed` → `status` 向后兼容映射
- [x] 8.2 添加 `status` 参数校验：仅接受 `pending`、`processing`、`matched`、`skipped`，无效值返回 `ApiError::InvalidStatus`
- [x] 8.3 更新 `src/error.rs`：新增 `InvalidStatus` 错误变体
- [x] 8.4 更新 `docs/apis/query-api.md`：文档化 `status` 参数和 `processed` 废弃说明
- [x] 8.5 验证：`cargo check` 无编译错误

## 9. 测试与最终验证

- [x] 9.1 更新 `src/services/filter.rs` 的现有单元测试适配新 API（claim 替代 get_unprocessed）
- [x] 9.2 为 `claim_pending_articles` 添加测试：正常领取、无记录、并发保护
- [x] 9.3 为 `mark_articles_matched` / `mark_articles_skipped` 添加测试
- [x] 9.4 为指数退避计算添加测试：验证各种 retry_count 下的 delay 值
- [x] 9.5 为 `recover_stale_processing_records` 添加测试
- [x] 9.6 为 `ArticleQuery` 的 status/processed 参数映射添加测试
- [x] 9.7 运行 `cargo test`：全部测试通过
- [x] 9.8 运行 `cargo build`：无编译错误和警告
