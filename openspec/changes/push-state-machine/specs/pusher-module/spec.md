# pusher-module (delta)

## MODIFIED Requirements

### Requirement: Exponential backoff retry

系统 SHALL 实现指数退避计算重试延迟：`delay = min(base * 2^(retry_count - 1), retry_max_seconds)`，其中 `base = config.pusher.retry_base_seconds`，`retry_max_seconds` 为退避上限。当 `retry_count >= max_retries` 时，记录 SHALL 标记为 `dead` 终态。原线性退避 `delay = retry_count * retry_base_seconds` SHALL 被替换。

#### Scenario: First retry scheduled

- **WHEN** a push fails and `retry_count` becomes 1
- **THEN** `next_retry_at` SHALL be set to `now + retry_base_seconds`（即 `base * 2^0`）

#### Scenario: Second retry with longer backoff

- **WHEN** a push fails and `retry_count` becomes 2
- **THEN** `next_retry_at` SHALL be set to `now + base * 2^1 = now + 2 * retry_base_seconds`

#### Scenario: Third retry further doubled

- **WHEN** a push fails and `retry_count` becomes 3
- **THEN** `next_retry_at` SHALL be set to `now + base * 2^2 = now + 4 * retry_base_seconds`

#### Scenario: Backoff capped at retry_max_seconds

- **WHEN** the computed backoff delay exceeds `retry_max_seconds`（e.g., base=60s, retry_count=8, delay=7680s > max=3600s）
- **THEN** `next_retry_at` SHALL be set to `now + retry_max_seconds`

#### Scenario: Max retries exhausted — dead

- **WHEN** `retry_count` reaches `config.pusher.max_retries`
- **THEN** `next_retry_at` SHALL be NULL
- **THEN** `status` SHALL be `dead`（而非 `failed`）

### Requirement: Mark failed on non-2xx response

系统 SHALL 在 HTTP 非 2xx 响应时将 `last_error` 设为 `"HTTP <status_code>"`，然后调用 `mark_failed` 进行指数退避调度。

#### Scenario: Mark failed with error message on non-2xx

- **WHEN** the webhook POST returns a non-2xx status
- **THEN** the system SHALL set `last_error` to `"HTTP <status_code>"`
- **THEN** the system SHALL increment `retry_count` and set status to `failed`（retry_count < max_retries）or `dead`（retry_count >= max_retries）

### Requirement: Mark failed on network error

系统 SHALL 在网络错误时将 `last_error` 设为 `"Network error: <简要描述>"`，然后调用 `mark_failed`。

#### Scenario: Mark failed with error message on network error

- **WHEN** the webhook POST fails with a network error
- **THEN** the system SHALL set `last_error` to `"Network error: <error>"` with a brief description
- **THEN** the system SHALL increment `retry_count` and apply exponential backoff

### Requirement: Mark success on HTTP 2xx

系统 SHALL 在推送成功时将 `last_error` 设为 NULL。

#### Scenario: Mark success and clear last_error

- **WHEN** the webhook POST returns a 2xx status
- **THEN** the system SHALL update the push record status to `success`
- **THEN** `last_error` SHALL be set to NULL

### Requirement: Skip channel with no webhook URL

系统 SHALL 在 channel 无有效 webhook URL 时将 `last_error` 设为类似 `"Channel N has no valid webhook URL"` 的描述。

#### Scenario: Skip channel and record error

- **WHEN** a push channel's config JSON does not contain a valid `url` field
- **THEN** the system SHALL set `last_error` to `"Channel N has no valid webhook URL"`
- **THEN** the system SHALL mark the push record as `failed` and log an error

## ADDED Requirements

### Requirement: Pending and retry-due record polling with stale recovery

在领取记录之前，Pusher SHALL 先调用 `recover_stale_processing_records` 恢复卡死在 `processing` 超时的记录。然后原子领取 pending 和 retry-due 记录（不变）。领取查询 SHALL 排除 `status='dead'` 的记录。

#### Scenario: Stale recovery before claiming

- **WHEN** `run_pusher_once` executes
- **THEN** it SHALL first call `recover_stale_processing_records(pool, config.stale_timeout_minutes)`
- **THEN** it SHALL then atomically claim pending/retry-due records (existing behavior)

#### Scenario: Dead records not claimed

- **WHEN** push records have `status='dead'`
- **THEN** the atomic claim UPDATE SHALL NOT select them for processing
