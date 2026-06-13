## Context

TrendAITool 项目全面审查发现 7 个 P0 级缺陷：3 个安全漏洞（Token 明文泄露）、2 个数据一致性问题（Filter 事务缺失、keyword_mentions 无 UNIQUE 约束）、2 个功能可用性问题（CSP/axios 端口不匹配导致生产环境不可用）。当前后端为 Rust 单进程（API + 3 个后台模块），前端为 Electron + React，数据库为 SQLite WAL 模式。

所有 fix 均为现有代码的局部修改，不引入新依赖、不改变架构。

## Goals / Non-Goals

**Goals:**
- 消除启动日志和数据库中的 Token 明文存储，仅首次创建时通过 `warn` 日志一次性输出
- CORS 从 `permissive()` 收紧为显式方法白名单
- Filter 关键写操作包裹在 SQLite 事务中，保证 hot_event + push_record 原子性
- keyword_mentions 添加 UNIQUE 约束，清理重复数据
- 修复前端 CSP 和 axios 端口与后端默认 3000 一致

**Non-Goals:**
- P1/P2 级别问题（Task 5-6, 17-31）
- 数据库路径迁移（Task 7，用户保留现有路径）
- CORS origin 限制为具体域名（Electron file:// 无 origin）
- 直接 DROP token 列（SQLite ALTER TABLE 限制）

## Decisions

### Decision 1: Token 掩码而非完全静默

**选择**：已有 token 时打印掩码 `前4位...后4位`，仅首次创建时打印完整明文（warn 级别）。

**备选**：完全移除日志中的 token 信息。
**理由**：完全静默导致管理员无法确认 token 是否已存在或数量，掩码提供可辨识的最小信息量。

### Decision 2: INSERT 时置占位符而非 DROP 列

**选择**：`create_token` 和 `insert_initial_token` INSERT 时将 `token` 列设为 `***REDACTED***`，明文通过函数返回值一次性传递。新建迁移清空已有明文。

**备选 A**：DROP token 列。SQLite 3.35.0+ 才支持 `ALTER TABLE DROP COLUMN`，且表重建方式有外键风险。
**备选 B**：保留状态不变。违背安全原则。

**理由**：占位符方案兼容所有 SQLite 版本，auth 中间件已通过 `token_hash` 查询，不受影响。迁移不可逆但明确告知。

### Decision 3: Filter 事务粒度 — 仅包裹写操作

**选择**：在 `run_filter_once` 中仅将 upsert hot_event + insert push_records 包裹在事务中，mark_processed 在事务提交成功后执行。

**备选**：将整个 `run_filter_once` 包裹在事务中。
**理由**：整个流程包含大量读操作（批处理文章、构建 Aho-Corasick、统计计算），长事务会阻塞其他模块的读写。仅包裹写操作即可保证原子性，且事务外的 mark_processed 失败时文章会在下次重新处理（processed_at 仍为 NULL）。

### Decision 4: keyword_mentions 唯一索引而非复合主键

**选择**：创建 `UNIQUE INDEX idx_mentions_unique ON keyword_mentions(keyword_id, article_id)`。

**备选**：修改表结构使用复合主键 `PRIMARY KEY (keyword_id, article_id)`。
**理由**：唯一索引与 `INSERT OR IGNORE` 配合即可实现去重，无需修改已有表结构。SQLite 修改主键需要重建表。

### Decision 5: CSP 通配符端口而非固定端口

**选择**：`connect-src 'self' http://localhost:*`。

**备选**：`connect-src 'self' http://localhost:3000`。
**理由**：通配符允许用户修改 config.toml 端口后前端自动适配，不需要编译时耦合。

## Risks / Trade-offs

- **Token 迁移不可逆**：已有 token 明文被清空，所有用户需重新生成 token。→ 迁移前通过启动日志 warn 用户备份；首次创建的 token 明文已通过函数返回值传递给用户。
- **`***REDACTED***` 列有歧义风险**：如果代码某处仍通过 `SELECT *` 读取 `token` 列并直接使用，会拿到占位符。→ 已确认 auth 中间件使用 `token_hash` 查询，不受影响；`get_first_active_token` 返回的占位符被 Task 1 的掩码逻辑处理。
- **事务失败时 mark_processed 不执行**：文章保持 `processed_at = NULL`，下次 filter 运行会重新处理。→ 可能导致少量重复 keyword_mentions，但 UNIQUE 索引会忽略重复。
- **CORS `allow_origin(Any)` 未收紧**：Electron `file://` 协议不发送 Origin 头，无法限制为具体域名。→ 备注说明如果部署为 Web 应用需改为白名单。

## Migration Plan

1. 部署前通知所有管理员：Token 将全部失效，需重新创建
2. 部署新版本，启动时自动运行两个新迁移：
   - `20260610000001_drop_token_plaintext.sql` — 清空已有 token 明文
   - `20260610000002_mentions_unique_index.sql` — 清理重复 + 创建唯一索引
3. 管理员使用首次启动 warn 日志中的新 token（或通过数据库直接插入）登录
4. 回滚：迁移不可逆，回滚需从备份恢复数据库

## Open Questions

- 无
