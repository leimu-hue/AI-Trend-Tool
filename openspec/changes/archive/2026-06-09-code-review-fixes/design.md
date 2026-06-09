## Context

代码审查计划（`docs/plans/10-code-review-fixes.md`）列出了 13 个修复项，按 P0-P3 优先级排列。这些修复覆盖安全、数据完整性、性能和可靠性四个维度。当前系统数据量小、用户少，是执行修复的最佳时机——迁移成本低，且避免了规模增长后的数据修复痛苦。

核心技术约束：
- SQLite 不支持 `ALTER TABLE ADD CONSTRAINT`，需要重建表来添加 UNIQUE 约束
- SQLx 0.7 编译期检查 SQL，动态 SQL（如 `build_article_filter`）需要用 `sqlx::query(&sql)` 而非 `sqlx::query_as!`
- Axum 0.8 已经修复了 `Path<T>` + `from_fn_with_state` 的兼容问题

## Goals / Non-Goals

**Goals:**
- 消除 P0 数据完整性 bug（外键断裂、硬编码配置）
- 完成 token 哈希迁移，消除明文存储风险
- 优化关键路径性能（批量 SQL、并发推送、连接复用）
- 增加输入校验、配置校验、health check 等防御性措施
- 所有修改向后兼容，迁移可逆

**Non-Goals:**
- 不引入新的校验框架（validator/deserialize），手动校验
- 不修改前端代码
- 不重构 filter/pusher 的整体调度循环架构
- 不在本次变更中删除 token 明文列（留到后续版本）
- 不加性能 benchmark 或集成测试

## Decisions

### 1. Token 哈希算法：SHA-256

**选择**：SHA-256 (via `sha2` crate)

**替代方案**：
- bcrypt/argon2：专为密码设计，慢哈希抗暴力。但 API token 是 64 字符随机 hex 字符串（128 位熵），暴力破解不可行。慢哈希反而不必要增加每次 API 请求的延迟。
- 不哈希：当前状态，数据库读权限者可直接获取所有有效 token。

**理由**：API token ≠ 用户密码。token 本身已经是足够长的随机字符串，哈希的目的是防止数据库泄露时 token 被直接复用。SHA-256 快且安全，适合每次 API 请求都会执行的路径。

### 2. hot_events 迁移：重建表

**选择**：创建新表 → 迁移数据 → DROP 旧表 → RENAME 新表

**替代方案**：
- 在应用层检查重复后 UPDATE：并发窗口期内可能产生竞态，WHERE + UPDATE 需要事务，不如 SQL 层 UNIQUE 约束可靠。
- 用 `sqlx::query` 动态判断 INSERT/UPDATE：同样有竞态问题。

**理由**：SQLite 不支持 `ALTER TABLE ADD CONSTRAINT`。ON CONFLICT UPSERT 是最简洁的并发安全方案。数据表小（每小时最多 keyword_count 条），迁移 SQL 执行快速。

### 3. Pusher 并发度：8

**选择**：`futures::stream::for_each_concurrent(8, ...)`

**替代方案**：
- `join_all` 无限制并发：大量 pending push_records 时会一次性发起所有 HTTP 请求，可能压垮 webhook 接收方。
- 单线程串行：当前方案，延迟线性叠加。

**理由**：8 是合理的默认值。对于 3 个渠道 × 10 个热点的场景，8 路并发足够并行所有推送。对于更大规模也不至于打爆接收方。

### 4. 输入校验方式：手动校验

**选择**：在每个 handler 函数入口手动检查字段，返回 `AppError::BadRequest`

**替代方案**：
- `validator` crate + derive macro：引入新依赖，增加宏复杂度，且 `validator` 的错误消息定制不如手动灵活。
- serde deserialize 时校验：无法区分"缺失字段"和"字段为空"，错误信息不够友好。

**理由**：校验规则简单（非空、URL 前缀、JSON 合法性），手动校验不超过 10 行每 handler。与项目现有 `AppError` 错误模式一致。

### 5. keyword_mentions 批量插入：chunk 100

**选择**：每 100 条一组，构建 `INSERT OR IGNORE INTO ... VALUES (?, ?), (?, ?), ...`

**替代方案**：
- SQLite 事务包裹逐条 INSERT：比原始逐条好，但仍有多轮 round-trip。
- 全部塞进一个 SQL：SQLite 默认最大变量数为 999，1000 个 mentions 就是 2000 个变量，会超过限制。

**理由**：100 条 = 200 个绑定变量，远低于 999 限制。`INSERT OR IGNORE` 保留去重语义，与原始 `INSERT OR IGNORE` 一致。

### 6. Health check 路由结构调整

**选择**：将 `/health` 放在带 `State` 的外层 Router 上，与 `api` Router 同级，保持在 auth middleware 之外。

**理由**：health check 需要 `AppState` 来访问 `SqlitePool` 执行 `SELECT 1`。但 health check 必须不受 auth middleware 保护。拆分两层 Router 是最小改动方案。

### 7. Config 校验时机：反序列化后、使用前

**选择**：`AppConfig::load()` 中在 `toml::from_str` 后调用 `validate()` 方法

**替代方案**：
- 在 clap 解析时用 `validator`：clap 的 validator 设计用于 CLI 参数，不适合 TOML 配置结构体。
- 在使用处各自校验：分散在各模块，重复且容易遗漏。

**理由**：集中校验，Fail-fast。配置错误在启动时立即报告，而非运行时在某个模块的 panic。

## Risks / Trade-offs

- **[迁移风险] hot_events 表重建**：涉及 DROP TABLE。迁移脚本在事务内执行，失败会自动回滚。生产部署前先在 staging 验证。
  - 缓解：迁移文件编号确保在其他迁移之后执行。数据量小，重建耗时 < 1 秒。

- **[向后兼容] token_hash 列**：迁移添加 `DEFAULT ''` 列。现有 token 的 hash 列为空字符串，查询时会找不到。需要在迁移执行后重新创建所有 token（或手动 UPDATE hash 列）。
  - 缓解：部署后通过 API 重新创建 token，或执行一次性的 hash 回填脚本。

- **[并发安全] Pusher 并发推送**：`process_one` 内部更新 push_records 时依赖乐观锁（`WHERE status = ...`），并发不会产生重复推送。
  - 缓解：已有乐观锁机制，本次不改动 `process_one` 内部逻辑。

- **[性能退化] Health check SELECT 1**：每次 health check 请求会执行一次 DB 查询。对于监控系统每 10 秒的探活频率，成本可忽略。
  - 缓解：`SELECT 1` 是 SQLite 最快查询。

## Open Questions

无。所有技术决策已有明确方案，来自审查计划文档。
