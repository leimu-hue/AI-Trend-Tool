# data-models (delta)

## MODIFIED Requirements

### Requirement: Article model and ArticleQuery

系统 SHALL 在 `Article` struct 中新增 `status: String` 字段（取值: `pending`、`processing`、`matched`、`skipped`），与 `articles.status` 列对应。`ArticleQuery` struct SHALL 新增 `status: Option<String>` 字段，替代 `processed: Option<bool>`。`processed` 参数 SHALL 保留为过渡期兼容字段。

#### Scenario: Article status field mapped from DB

- **WHEN** `sqlx::query_as::<_, Article>("SELECT * FROM articles WHERE ...")` 执行
- **THEN** 返回的 `Article` SHALL 包含 `status` 字段，值对应 `articles.status` 列

#### Scenario: Query articles by status

- **WHEN** `ArticleQuery { page: Some(1), status: Some("matched".to_string()), ... }` 被构造
- **THEN** 查询参数 SHALL 包含 `status = 'matched'` 过滤条件

#### Scenario: processed 参数向后兼容

- **WHEN** `ArticleQuery` 的 `processed` 字段为 `Some(true)`
- **THEN** 系统 SHALL 内部将其映射为 `status = 'matched'` 过滤
- **WHEN** `processed` 为 `Some(false)`
- **THEN** 系统 SHALL 内部将其映射为 `status = 'pending'` 过滤

### Requirement: PushRecord model

`PushRecord` struct 的 `status` 字段 SHALL 扩展支持 `dead` 值（原有 `pending`、`processing`、`success`、`failed`）。新增 `last_error: Option<String>` 字段对应 `push_records.last_error` 列。`PushRecordWithChannel` struct SHALL 同步新增 `last_error: Option<String>` 字段。

#### Scenario: PushRecord with dead status

- **WHEN** `sqlx::query_as::<_, PushRecord>("SELECT * FROM push_records WHERE status = 'dead'")` 执行
- **THEN** 结果 SHALL 包含 `status = "dead"` 的记录
- **THEN** `last_error` SHALL 包含失败原因（如果有）

#### Scenario: PushRecord serialized with last_error

- **WHEN** 一个 `PushRecord` 被序列化为 JSON
- **THEN** JSON SHALL 包含 `"last_error": "HTTP 500"` 或 `"last_error": null`
- **THEN** `status` SHALL 可能为 `"dead"`

#### Scenario: PushRecordWithChannel includes last_error

- **WHEN** push records for a hotspot are queried via JOIN
- **THEN** 每个 `PushRecordWithChannel` SHALL 包含 `last_error` 字段
- **THEN** 对应的 SQL SHALL 选择 `pr.last_error`
