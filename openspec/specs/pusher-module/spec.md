# pusher-module

## Purpose

Background service that polls pending and retry-due push records, sends webhook POST requests to configured push channels (DingTalk/Feishu), and manages retry with exponential backoff and optimistic locking.

## Requirements

### Requirement: Pusher background scheduling

The system SHALL run the Pusher module as an asynchronous background task that uses `tokio::select!` to listen for three signals: cancellation token (graceful shutdown), configurable interval tick (fallback polling), and `push_ready_rx` channel (event-driven trigger from Filter). A single `reqwest::Client` SHALL be created in `start_pusher_loop` and passed as a reference to each `run_pusher_once` call. The system SHALL expose a shared `run_pusher_once` function for manual triggering.

#### Scenario: Pusher runs on interval

- **WHEN** the Pusher loop is running and the interval ticks
- **THEN** it SHALL call `run_pusher_once` with the shared `reqwest::Client` reference

#### Scenario: Pusher runs on Filter notification

- **WHEN** the Pusher loop receives `PipelineEvent::NewData` via `push_rx.recv()`
- **THEN** it SHALL immediately call `run_pusher_once` with the shared `reqwest::Client` reference

#### Scenario: Manual pusher trigger calls same function

- **WHEN** the `POST /api/v1/trigger/pusher` endpoint is called
- **THEN** it SHALL create a temporary `reqwest::Client` and call `run_pusher_once`

#### Scenario: Pusher shuts down gracefully

- **WHEN** the global `CancellationToken` is cancelled
- **THEN** the Pusher SHALL log a shutdown message and break out of its loop

### Requirement: Pending and retry-due record polling with stale recovery

在领取记录之前，Pusher SHALL 先调用 `recover_stale_processing_records` 恢复卡死在 `processing` 超时的记录。然后原子领取 pending 和 retry-due 记录。领取查询 SHALL 排除 `status='dead'` 的记录。

#### Scenario: Stale recovery before claiming

- **WHEN** `run_pusher_once` executes
- **THEN** it SHALL first call `recover_stale_processing_records(pool, config.stale_timeout_minutes)`
- **THEN** it SHALL then atomically claim pending/retry-due records

#### Scenario: Atomically claim pending records

- **WHEN** `run_pusher_once` executes after stale recovery
- **THEN** it SHALL first execute `UPDATE push_records SET status='processing' WHERE status IN ('pending', 'failed') AND (next_retry_at IS NULL OR next_retry_at <= datetime('now'))`
- **THEN** it SHALL then SELECT records WHERE `status='processing'` for webhook delivery

#### Scenario: Dead records not claimed

- **WHEN** push records have `status='dead'`
- **THEN** the atomic claim UPDATE SHALL NOT select them for processing

#### Scenario: No pushable records — early return

- **WHEN** no processing or retry-due records exist
- **THEN** the pusher SHALL return immediately without error

### Requirement: Webhook push delivery

The system SHALL send a POST request to the channel's webhook URL with a JSON payload describing the hotspot event.

#### Scenario: Send webhook for pending record

- **WHEN** a push record is pending
- **THEN** the system SHALL fetch the associated `push_channel`, `hot_event`, and `keyword`
- **AND** SHALL POST a JSON payload to the channel's webhook URL
- **AND** SHALL include the keyword word and hotspot count in the message

#### Scenario: Mark success on HTTP 2xx

- **WHEN** the webhook POST returns a 2xx status
- **THEN** the system SHALL update the push record status to `success`
- **THEN** `last_error` SHALL be set to NULL

#### Scenario: Mark failed on non-2xx response

- **WHEN** the webhook POST returns a non-2xx status
- **THEN** the system SHALL set `last_error` to `"HTTP <status_code>"`
- **THEN** the system SHALL increment `retry_count` and set status to `failed`（retry_count < max_retries）or `dead`（retry_count >= max_retries）

#### Scenario: Mark failed on network error

- **WHEN** the webhook POST fails with a network error
- **THEN** the system SHALL set `last_error` to `"Network error: <error>"` with a brief description
- **THEN** the system SHALL increment `retry_count` and apply exponential backoff

#### Scenario: Skip channel with no webhook URL

- **WHEN** a push channel's config JSON does not contain a valid `url` field
- **THEN** the system SHALL set `last_error` to `"Channel N has no valid webhook URL"`
- **THEN** the system SHALL mark the push record as `failed` and log an error

### Requirement: Concurrent webhook push delivery

The system SHALL process pushable records concurrently with a maximum of 8 simultaneous webhook POST requests.

#### Scenario: Multiple records pushed concurrently

- **WHEN** multiple push records are pending
- **THEN** the system SHALL send webhook POST requests concurrently (max 8 at a time)
- **THEN** each push SHALL use optimistic locking to prevent duplicate sends

#### Scenario: Single record still works

- **WHEN** only one push record is pending
- **THEN** the system SHALL process it without error (concurrent stream of 1)

#### Scenario: Concurrent pushes share reqwest client

- **WHEN** concurrent pushes are in flight
- **THEN** all push tasks SHALL use clones of the same `reqwest::Client` instance
- **THEN** HTTP connection pooling SHALL be shared across all concurrent tasks

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

### Requirement: Optimistic locking for push records

The system SHALL use optimistic locking when updating push record status to prevent duplicate sends from concurrent pusher runs.

#### Scenario: Optimistic update succeeds

- **WHEN** the pusher updates a record from `pending` to a new status
- **THEN** it SHALL update only if current status matches expected status

#### Scenario: Optimistic update skipped

- **WHEN** another process already changed the record's status
- **THEN** the update SHALL affect zero rows and the pusher SHALL skip that record
