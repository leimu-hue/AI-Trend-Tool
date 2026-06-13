## Context

P1 级修复涉及 8 个问题：功能缺陷（article_count、分页 per_page）、性能（Pusher client 复用）、并发安全（Pusher 竞态）、错误处理（push_records 静默吞错）、前端体验（Settings 默认值、Toast 内存泄漏、Layout 页脚）。所有修复均为现有代码的局部优化，不改变架构。

## Goals / Non-Goals

**Goals:**
- 数据源列表返回实际文章计数
- Pusher HTTP 客户端在 loop 级别复用
- 消除 Pusher 定时器/事件并发触发的重复发送窗口
- 分页响应 per_page 与实际返回条数一致
- push_records 插入时区分 UNIQUE 冲突和真实错误
- Settings fallback 值与 config.toml 对齐
- Toast 组件卸载时清理定时器
- Layout 页脚移除误导性自动刷新文案

**Non-Goals:**
- P0 安全修复（已在独立变更中处理）
- P2 代码改进（将在独立变更中处理）
- 实际实现 Layout 自动刷新机制

## Decisions

### Decision 1: LEFT JOIN + COUNT 而非子查询

**选择**：`SELECT ds.*, COUNT(a.id) as article_count FROM data_sources ds LEFT JOIN articles a ON a.source_id = ds.id GROUP BY ds.id`

**备选**：子查询 `(SELECT COUNT(*) FROM articles WHERE source_id = ds.id)`
**理由**：单次 JOIN 比每行子查询效率高，SQLite 优化器对 GROUP BY 有良好支持。

### Decision 2: 原子领取 (UPDATE→SELECT) 而非乐观锁

**选择**：Pusher 处理前先 `UPDATE push_records SET status='processing' WHERE status='pending'`，再 `SELECT ... WHERE status='processing'`

**备选**：当前乐观锁（处理后再 UPDATE WHERE status=expected）。已在 design 中但有竞态窗口。
**理由**：原子领取消除了两轮 run_pusher_once 同时 SELECT 同一批 pending 记录的窗口。配合原乐观锁形成双重保护。

### Decision 3: Handler 层 clamp per_page

**选择**：在 handler 中 `let per_page = query.per_page.unwrap_or(20).min(100)`，再传入 DB 层和 PaginatedResponse

**备选**：DB 层返回实际值。需要修改返回值签名，增加复杂度。
**理由**：handler 层 clamp 最直观，DB 层和响应使用同一个值，确保一致。

### Decision 4: 区分 Ok(None) vs Err

**选择**：`INSERT OR IGNORE ... RETURNING *` 返回 `Ok(None)` 时正常跳过，`Err(e)` 时记录 `tracing::error!`

**理由**：`INSERT OR IGNORE` 会忽略所有错误（包括磁盘满），必须区分 UNIQUE 冲突和真实故障。

## Risks / Trade-offs

- **原子领取增加一次 UPDATE**：每条 pending 记录多一次写操作，但 Pusher 运行频率低（10s），性能影响可忽略。
- **LEFT JOIN 计数**：数据源列表页面加载时多一次聚合查询，但数据源数量通常很少（<100），性能可接受。
