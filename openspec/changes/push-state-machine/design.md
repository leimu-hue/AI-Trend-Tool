## Context

当前推送管道（Parser → Filter → Pusher）存在两个层级的可靠性缺口：

1. **Article 层**: 仅用 `processed_at IS NULL/NOT NULL` 二元判断，无法区分"匹配到关键字"和"未匹配"的文章；Filter 无并发保护，多次 `run_filter_once` 可能同时处理同一批文章
2. **PushRecord 层**: 线性退避 `delay = n * base` 而非指数退避；耗尽重试的记录以 `failed + next_retry_at=NULL` 存放，语义不清；Pusher 崩溃后 `processing` 记录永久卡死；无错误信息字段

本设计参考 `docs/plans/13-push-state-machine.md`，在两个层级各引入状态机，并增强并发安全和错误可观测性。所有变更仅涉及后端 Rust 代码 + SQLite migration，不涉及前端。

## Goals / Non-Goals

**Goals:**
- Article 引入 `pending → processing → matched/skipped` 状态流转，含原子 claim 防并发重复处理
- Filter 通过 `UPDATE WHERE status='pending'` 原子领取文章，替换无保护的 `SELECT WHERE processed_at IS NULL`
- Pusher 退避从线性 `n * base` 改为指数 `base * 2^(n-1)`，增加 `retry_max_seconds` 上限
- PushRecord 新增 `dead` 终态，与可重试 `failed` 分离
- PushRecord 新增 `last_error` 字段，记录每次失败原因
- Pusher 每次运行时恢复卡死在 `processing` 超过 N 分钟的记录
- 配置新增 `retry_max_seconds` 和 `stale_timeout_minutes`，带合理默认值

**Non-Goals:**
- 不改变 Parser 的取文章或去重逻辑
- 不改变 PushRecord 的 optimistic locking（`update_push_status_optimistic`）机制，仅扩展参数
- 不移除 `processed_at` 字段，保留向后兼容
- 不修改前端页面（Dashboard/Article 列表适配后续单独变更）
- 不引入第三方消息队列或分布式锁（单进程 SQLite 写锁已足够）

## Decisions

### 1. Article 状态机: 保留 processed_at 字段，新增 status 列

**选择**: 添加 `status TEXT NOT NULL DEFAULT 'pending'` 列，`processed_at` 保留不动，matched/skipped 时同步更新 `processed_at`。

**备选方案**: 完全移除 `processed_at`，仅用 `status`。
**否决理由**: `processed_at` 已被前端和现有查询代码依赖，一次性移除风险大。保留该字段实现平滑迁移，后续版本可废弃。

**注意**: SQLite 不支持 `ALTER TABLE DROP COLUMN`（需重建表），因此移除 `processed_at` 只能在主版本迁移中进行。

### 2. Filter 原子 claim: 两步操作（UPDATE + SELECT），非单条 RETURNING

**选择**: 
```sql
-- Step 1: 原子标记
UPDATE articles SET status = 'processing' 
WHERE id IN (SELECT id FROM articles WHERE status = 'pending' ... LIMIT ?)
-- Step 2: 读取已标记的
SELECT * FROM articles WHERE status = 'processing' ORDER BY fetched_at ASC LIMIT ?
```

**备选方案**: 使用 SQLite 3.35+ 的 `UPDATE ... RETURNING *` 单步完成。
**否决理由**: SQLite `RETURNING` 仅返回被 UPDATE 的行，但此处 LIMIT 位于子查询中，`RETURNING` 不能直接与子查询 LIMIT 配合。需要两步操作。两步之间没有并发风险——Step1 的原子性由 SQLite 写锁保证，相同 status='pending' 的行已全部转为 processing，后续调用者不会重复领取。

### 3. 退避策略: 指数退避 + 上限裁剪

**选择**: `delay = min(base * 2^(retry_count-1), retry_max_seconds)`，默认 `base=60s, max=3600s`。

**备选方案**: 加随机抖动（jitter）避免惊群。
**否决理由**: 单进程场景不存在惊群问题。PushRecord 是按条独立重试的，不是批量同时重试。

**退避表**（base=60s, max=3600s, max_retries=5）:
| retry_count | 退避延迟 | 实际延迟（含上限） |
|------------|---------|-----------------|
| 1 | 60s | 60s |
| 2 | 120s | 120s |
| 3 | 240s | 240s |
| 4 | 480s | 480s |
| 5 | 960s | 960s |
| 6+ | 1920s… | → dead |

### 4. 陈旧 processing 恢复: 每次 run_pusher_once 开头执行

**选择**: `UPDATE push_records SET status='pending' WHERE status='processing' AND updated_at < datetime('now', '-N minutes')`，默认 N=10。

**备选方案**: 使用独立的定时 recovery 任务。
**否决理由**: 增加复杂度但收益不大。PushRecord processing 状态持有时间极短（webhook POST 通常在秒级完成）。唯一需要恢复的场景是进程崩溃，而 Pusher 重启后第一次 `run_pusher_once` 就会触发恢复。

### 5. last_error 字段: 可选 TEXT 列，非结构化

**选择**: `last_error TEXT` 存储简短的错误描述（如 `"HTTP 500"`、`"Network error: timeout"`）。

**备选方案**: 结构化错误码 + 枚举。
**否决理由**: 错误来源多样化（HTTP状态码、网络错误、序列化错误），枚举维护成本高。文本字段灵活且满足排障需求。

## Risks / Trade-offs

**[风险] Article migration 存量数据标记不准确**: 迁移脚本将 `processed_at IS NOT NULL` 的文章一律标记为 `matched`，可能覆盖了本应标记为 `skipped` 的文章
→ **缓解**: 早期版本中 Filter 仅处理成功才标记 processed_at，实际不会有 processed 但未命中的情况。若担忧，可先标记为 `matched` 并在日志中提示用户可手动审计。

**[风险] processing 状态残留**: 如果 Filter 在 claim 后、mark_matched/skipped 前崩溃，文章将永久卡在 processing
→ **缓解**: Filter 作为单次 `run_filter_once` 调用，所有操作在单次调用内完成。即使崩溃，下次 `run_filter_once` 启动时会重新 claim 新的 pending 文章（processing 的文章不会被重新 claim）。未来可添加类似 Pusher 的陈旧恢复机制。

**[风险] API 参数 `processed` → `status` 破坏前端兼容性**
→ **缓解**: 后端暂时同时支持 `processed` 和 `status` 参数（`processed=true` 映射为 `status=matched`，`processed=false` 映射为 `status=pending`），前端后续适配后再移除 `processed` 支持。

**[权衡] 额外 DB 列和索引**: 增加 `status`、`last_error` 列和各一个索引
→ **接受**: SQLite 文件大小增加可忽略（约数KB），索引维护开销极小。

## Migration Plan

1. **Deploy 步骤**:
   - 停止旧版本进程
   - 部署新二进制
   - 启动 → `sqlx::migrate!()` 自动运行两个新 migration
   - Migration 1: `ALTER TABLE articles ADD COLUMN status` + 存量数据迁移 + 创建索引
   - Migration 2: `ALTER TABLE push_records ADD COLUMN last_error` + 存量 dead 标记
   - 验证: `cargo test` 全部通过

2. **Rollback**: 
   - Migration 不可逆（SQLite 不支持 DROP COLUMN）
   - 向前修复（forward fix）替代回滚
   - 如果需要回退到旧二进制：旧代码不读取 `status`/`last_error` 列，不会报错（sqlx `FromRow` 按列名匹配）

3. **兼容性**:
   - 旧二进制 + 新数据库 → 正常（sqlx 不检查额外的列）
   - 新二进制 + 旧数据库 → migration 自动升级
   - `processed_at` 字段保留，旧查询不受影响

## Open Questions

1. Article 的 processing 状态是否需要类似 Pusher 的陈旧恢复？当前设计中 Filter 的 claim→mark 在同一函数调用内完成，风险较低。若后续 Filter 执行时间变得很长（大批量文章处理），可考虑添加。
2. `dead` 状态的 PushRecord 是否需要人工重试 API？当前设计仅标记 dead + 记录 last_error，无手动重试端点。可后续按需添加 `POST /api/v1/push-records/{id}/retry`。
3. 文章查询 API 的 `processed` 参数过渡期多长？建议 1-2 个小版本后移除。
