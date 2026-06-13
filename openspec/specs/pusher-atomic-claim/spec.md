# pusher-atomic-claim

## Purpose

Prevent duplicate webhook sends when multiple Pusher triggers (timer tick + event-driven) fire concurrently, by atomically claiming push_records via UPDATE before SELECT.

## Requirements

### Requirement: Pusher 原子领取待处理记录

`run_pusher_once` 在处理之前 SHALL 先通过原子 UPDATE 将满足条件的 pending 记录标记为 processing，再 SELECT 这些 processing 记录进行处理。防止定时器 tick 与事件触发并发时重复处理同一批记录。

#### Scenario: 原子领取成功

- **WHEN** `run_pusher_once` 开始执行且存在 status='pending' 的 push_records
- **THEN** 系统 SHALL 执行 `UPDATE push_records SET status='processing' WHERE status='pending' AND (next_retry_at IS NULL OR next_retry_at <= datetime('now'))`
- **THEN** 系统 SHALL 接着 SELECT `WHERE status='processing'` 并处理这些记录

#### Scenario: 并发调用时第二批无记录可领

- **WHEN** `run_pusher_once` (A) 已原子领取了所有 pending 记录
- **AND** `run_pusher_once` (B) 在 (A) 提交前触发
- **THEN** (B) 的 UPDATE 影响 0 行
- **THEN** (B) 的 SELECT 返回空结果集
- **THEN** (B) SHALL 提前返回，不发送任何 webhook

#### Scenario: processing 记录正常完成

- **WHEN** 记录被领取为 processing 状态
- **AND** webhook 发送成功
- **THEN** 记录状态 SHALL 更新为 'success'
