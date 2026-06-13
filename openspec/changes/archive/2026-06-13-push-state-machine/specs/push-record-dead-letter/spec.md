# push-record-dead-letter

## Purpose

为 PushRecord 引入 `dead` 终态区分重试耗尽记录，新增 `last_error` 字段记录失败原因，并实现陈旧 `processing` 记录的自动恢复机制。

## ADDED Requirements

### Requirement: PushRecord dead 终态

系统 SHALL 支持 `dead` 作为 PushRecord 的终态。当 `retry_count >= config.pusher.max_retries` 时，推送记录 SHALL 从 `failed` 转为 `dead`，`next_retry_at` SHALL 设为 NULL，系统 SHALL 不再尝试重试该记录。

#### Scenario: 重试耗尽标记为 dead

- **WHEN** push record 的 `retry_count` 在失败后达到 `max_retries`（例如 max_retries=5，retry_count 从 4 增至 5）
- **THEN** `status` SHALL 更新为 `dead`
- **THEN** `next_retry_at` SHALL 为 NULL
- **THEN** Pusher SHALL 在后续运行中不再处理该记录

#### Scenario: dead 记录与可重试 failed 区分

- **WHEN** 查询 push_records
- **THEN** `status='failed'` 且有 `next_retry_at IS NOT NULL` 的记录 SHALL 仍可被 Pusher 重试
- **THEN** `status='dead'` 的记录 SHALL 不被 Pusher 的正常领取查询返回

### Requirement: last_error 字段

`push_records` 表 SHALL 包含 `last_error TEXT` 字段，记录最近一次推送失败的简要错误描述。推送成功时 `last_error` SHALL 设为 NULL。

#### Scenario: HTTP 错误记录到 last_error

- **WHEN** webhook POST 返回 HTTP 500
- **THEN** push record 的 `last_error` SHALL 设为 `"HTTP 500"`
- **THEN** `status` SHALL 更新为 `failed`（retry_count < max_retries）或 `dead`（retry_count >= max_retries）

#### Scenario: 网络错误记录到 last_error

- **WHEN** webhook POST 因网络超时或连接失败而失败
- **THEN** push record 的 `last_error` SHALL 设为类似 `"Network error: <简述>"` 的描述

#### Scenario: 推送成功清除 last_error

- **WHEN** webhook POST 返回 2xx 且 push record 之前有 `last_error` 值
- **THEN** `last_error` SHALL 更新为 NULL
- **THEN** `status` SHALL 更新为 `success`

#### Scenario: 配置错误记录到 last_error

- **WHEN** push channel 的 config JSON 中无有效 webhook URL
- **THEN** push record 的 `last_error` SHALL 设为类似 `"Channel N has no valid webhook URL"` 的描述
- **THEN** `status` SHALL 更新为 `failed`

### Requirement: 陈旧 processing 记录自动恢复

Pusher 在每次 `run_pusher_once` 开头 SHALL 调用 `db::push_record::recover_stale_processing_records(pool, timeout_minutes)` 将卡在 `processing` 超过指定分钟数的记录重置为 `pending` 状态。

#### Scenario: 恢复卡死的 processing 记录

- **WHEN** 存在 `status='processing'` 且 `updated_at < datetime('now', '-N minutes')` 的记录（N=配置的 stale_timeout_minutes）
- **AND** `run_pusher_once` 开始执行
- **THEN** 这些记录的 `status` SHALL 更新为 `pending`
- **THEN** 系统 SHALL 通过 tracing::warn 记录恢复数量

#### Scenario: 正常 processing 记录不被误恢复

- **WHEN** 存在 `status='processing'` 且 `updated_at` 在超时窗口内的记录
- **THEN** 这些记录 SHALL 不受影响
- **THEN** Pusher SHALL 正常处理这些记录

#### Scenario: 无陈旧记录时静默通过

- **WHEN** 不存在需要恢复的陈旧记录
- **THEN** `recover_stale_processing_records` SHALL 返回 0
- **THEN** 不输出任何 warning 日志

### Requirement: 指数退避替换线性退避

Pusher SHALL 使用指数退避计算失败后的重试延迟：`delay = min(base * 2^(retry_count - 1), retry_max_seconds)`，其中 `base = config.pusher.retry_base_seconds`，`retry_max_seconds` 为退避上限。原线性退避 `delay = retry_count * retry_base_seconds` SHALL 被移除。

#### Scenario: 第一次失败使用 base 退避

- **WHEN** push 失败且 `retry_count` 变为 1
- **THEN** `next_retry_at` SHALL 为 `now + retry_base_seconds`

#### Scenario: 第三次失败加倍退避

- **WHEN** push 失败且 `retry_count` 变为 3
- **THEN** `next_retry_at` SHALL 为 `now + base * 2^2 = now + base * 4`

#### Scenario: 退避超过上限被裁剪

- **WHEN** 计算出的退避延迟超过 `retry_max_seconds`
- **THEN** `next_retry_at` SHALL 为 `now + retry_max_seconds`
