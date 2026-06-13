## 1. 数据库迁移

- [x] 1.1 创建迁移文件 `docs/migrations/20260610000001_drop_token_plaintext.sql` — 将已有 token 明文置为占位符
- [x] 1.2 创建迁移文件 `docs/migrations/20260610000002_mentions_unique_index.sql` — 清理重复数据并创建 `(keyword_id, article_id)` 唯一索引
- [x] 1.3 验证迁移通过编译 — `cargo build`

## 2. Token 安全修复

- [x] 2.1 修改 `src/main.rs` — `ensure_initial_token`：已有 token 时打印掩码（前4位...后4位），仅首次创建时 warn 级别输出完整明文
- [x] 2.2 修改 `src/db/token.rs` — `create_token`：INSERT 时 token 列写 `***REDACTED***`，明文回填到返回对象
- [x] 2.3 修改 `src/db/token.rs` — `insert_initial_token`：INSERT 时 token 列写 `***REDACTED***`
- [x] 2.4 验证 — `cargo build`，确认无编译错误

## 3. Filter 事务保护

- [x] 3.1 修改 `src/services/filter.rs` — `run_filter_once`：将 hot_event upsert + push_records insert 包裹在 `pool.begin().await?` 事务中
- [x] 3.2 事务成功 COMMIT 后再调用 `mark_processed_batch`
- [x] 3.3 验证 — `cargo build`

## 4. CORS 安全配置

- [x] 4.1 修改 `src/routes.rs` — 替换 `CorsLayer::permissive()` 为显式方法白名单（GET/POST/PUT/DELETE/OPTIONS）
- [x] 4.2 验证 — `cargo build`

## 5. 前端端口修复

- [x] 5.1 修改 `web/src/main/index.ts` — 生产环境 CSP `connect-src` 从 `http://localhost:8080` 改为 `http://localhost:*`
- [x] 5.2 修改 `web/src/renderer/src/api/client.ts` — baseURL fallback 端口从 8080 改为 3000
- [x] 5.3 修改 `web/src/renderer/src/pages/Settings.tsx` — DEFAULTS 中 `server.port` 从 8080 改为 3000
- [x] 5.4 验证 — `cd web && npm run build`

## 6. 最终验证

- [x] 6.1 后端编译 + 测试 — `cargo build && cargo test`
- [x] 6.2 前端编译 — `cd web && npm run build`
- [x] 6.3 手动验证 token 创建流程 — 启动后端，创建新 token，确认日志仅显示掩码
- [x] 6.4 手动验证 CSP 配置 — 生产构建前端，确认 API 请求不被 CSP 拦截
