## 1. 数据源文章计数

- [x] 1.1 新增 `src/db/source.rs` — `SourceWithCount` 结构体和 `list_sources_with_count` 查询函数（LEFT JOIN + COUNT + GROUP BY）
- [x] 1.2 修改 `src/handlers/source.rs` — `list_sources` handler 调用 `list_sources_with_count`
- [x] 1.3 验证 — `cargo build`

## 2. Pusher HTTP 客户端复用

- [x] 2.1 修改 `src/services/pusher.rs` — `start_pusher_loop`：在 loop 外创建 `reqwest::Client`，传入 `run_pusher_once`
- [x] 2.2 修改 `src/services/pusher.rs` — `run_pusher_once`：签名增加 `client: &reqwest::Client` 参数，删除内部 `Client::new()`
- [x] 2.3 修改 `src/handlers/query.rs` — `trigger_pusher`：手动触发时创建临时 client 传入
- [x] 2.4 验证 — `cargo build`

## 3. Pusher 原子领取

- [x] 3.1 修改 `src/services/pusher.rs` — `run_pusher_once`：在 SELECT 前增加 `UPDATE push_records SET status='processing' WHERE status='pending' AND (...)` 
- [x] 3.2 修改 SELECT 条件：`WHERE status='processing'` 替代 `WHERE status='pending'`
- [x] 3.3 验证 — `cargo build`

## 4. 分页 per_page 响应修正

- [x] 4.1 修改 `src/handlers/query.rs` — `list_articles`：handler 层 clamp `per_page` 后再传入 DB 和响应
- [x] 4.2 修改 `src/handlers/query.rs` — `list_hotspots`：同理 clamp `per_page`
- [x] 4.3 验证 — `cargo build`

## 5. push_records 错误日志

- [x] 5.1 修改 `src/db/push_record.rs` — `insert_push_records_for_event`：区分 `Ok(None)`（UNIQUE 冲突）和 `Err(e)`（真实错误），对后者记录 `tracing::error!`
- [x] 5.2 验证 — `cargo build`

## 6. 前端修复

- [x] 6.1 修改 `web/src/renderer/src/pages/Settings.tsx` — DEFAULTS 对象：`max_concurrent_fetches`、`batch_size`、`port` 对齐 config.toml
- [x] 6.2 修改 `web/src/renderer/src/components/Toast.tsx` — `useRef<Set>` 收集 timer ID，`useEffect` cleanup 统一 `clearTimeout`
- [x] 6.3 修改 `web/src/renderer/src/components/Layout.tsx` — 侧边栏页脚文案改为 "监控中"（移除 "每5分钟自动刷新"）
- [x] 6.4 验证 — `cd web && npm run build`

## 7. 最终验证

- [x] 7.1 后端编译 + 测试 — `cargo build && cargo test`
- [x] 7.2 前端编译 — `cd web && npm run build`
- [x] 7.3 手动验证数据源列表 — 确认 article_count 正确显示
- [x] 7.4 手动验证 Settings 页面 — 确认 DEFAULTS 与 config.toml 一致
