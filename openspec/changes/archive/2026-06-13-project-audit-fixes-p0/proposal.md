## Why

项目全面审查发现 7 个 P0 级别问题：启动日志泄露 Token 明文、CORS 过度宽松、Token 明文持久化列、Filter 数据写操作缺乏事务保护、keyword_mentions 表缺少 UNIQUE 约束导致数据重复、CSP/axios 端口与后端不匹配导致生产环境不可用。这些问题涉及安全漏洞、数据一致性和功能可用性，需立即修复。

## What Changes

- **启动日志 Token 掩码**：已有 token 时仅打印前 4 位 + 后 4 位掩码，不再泄露明文
- **CORS 显式方法白名单**：替换 `CorsLayer::permissive()` 为明确的方法限制，保留 origin 为 Any（Electron file:// 协议兼容）
- **Token 明文列废弃**：`api_tokens.token` 列写入时置为占位符 `***REDACTED***`，明文仅通过函数返回值一次性传递；新增数据迁移清空已有明文
- **Filter 事务保护**：hot_event upsert 和 push_record insert 包裹在同一数据库事务中，防止部分写入导致数据丢失
- **keyword_mentions UNIQUE 索引**：新建迁移添加 `(keyword_id, article_id)` 唯一索引，清理已有重复数据，防止 `INSERT OR IGNORE` 无限膨胀
- **CSP connect-src 端口修复**：生产环境 CSP 改用 `http://localhost:*` 通配符，匹配后端任意端口
- **axios baseURL fallback 修复**：默认端口从 8080 改为 3000，与 `config.toml` 一致

## Capabilities

### New Capabilities

- `token-masking`: 启动日志中 token 的掩码显示逻辑，已有 token 时仅显示部分字符，首次创建时通过 warn 级别打印完整明文
- `db-transaction-safety`: Filter 模块关键写操作的事务保护，确保 hot_event 和 push_record 的原子性写入
- `unique-mentions-constraint`: keyword_mentions 表的 `(keyword_id, article_id)` 唯一约束，防止重复提及记录

### Modified Capabilities

- `initial-token-bootstrap`: 启动日志不再打印 token 明文，改为掩码显示
- `auth-middleware`: CORS 配置从 `permissive()` 改为显式方法白名单
- `token-api`: `create_token` 和 `insert_initial_token` 不再将明文写入数据库 token 列
- `database-schema`: api_tokens.token 列迁移为占位符；新增 keyword_mentions 唯一索引
- `filter-module`: `run_filter_once` 的写操作包裹在数据库事务中
- `frontend-project-scaffold`: 生产环境 CSP `connect-src` 从固定端口改为通配符
- `api-client-layer`: axios 默认 baseURL 端口从 8080 改为 3000

## Non-goals

- Task 4（401 hash 路由跳转）：经分析非 Bug，无需修复
- Task 7（数据库路径迁移）：用户选择保留现有路径 `./docs/data/`
- Tasks 5-6, 17-31（P1/P2 问题）：将在后续独立变更中处理
- 非 RESTful 路由（POST update/delete）：按项目规范不做更改

## Impact

- **后端**：`src/main.rs`、`src/routes.rs`、`src/db/token.rs`、`src/services/filter.rs`、`src/db/push_record.rs`（Task 19 连带修复）、`src/handlers/query.rs`（Task 18 连带修复）
- **数据库**：2 个新迁移文件（token 明文清理、keyword_mentions 唯一索引）
- **前端**：`web/src/main/index.ts`（CSP）、`web/src/renderer/src/api/client.ts`（baseURL）、`web/src/renderer/src/pages/Settings.tsx`（默认端口）
- **破坏性变更**：token 迁移不可逆，已持久化的 token 明文将被清空为 `***REDACTED***`；需要重新生成所有 API token
