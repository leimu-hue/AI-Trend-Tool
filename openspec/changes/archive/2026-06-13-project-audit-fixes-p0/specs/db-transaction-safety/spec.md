## ADDED Requirements

### Requirement: Filter 写操作事务保护

`run_filter_once` 中 hot_event 的 upsert 和 push_record 的 insert 操作 SHALL 在同一个数据库事务中执行。事务提交成功后才标记文章为已处理。

#### Scenario: 事务成功 — 全部提交

- **WHEN** hot_event upsert 和 push_records insert 均在事务中成功执行
- **THEN** 事务 SHALL COMMIT
- **THEN** 标记文章为已处理的操作 SHALL 在事务提交后执行

#### Scenario: push_records insert 失败 — 继续处理其他关键词

- **WHEN** 某个关键词的 push_records insert 失败（如外键违反）
- **THEN** 系统 SHALL 记录错误日志
- **THEN** 该关键词的 hot_event 仍被持久化
- **THEN** 其他关键词的 hot_event 和 push_records SHALL 正常处理
- **THEN** 事务成功后文章 SHALL 被标记为已处理

#### Scenario: 事务失败不标记已处理

- **WHEN** 事务中的任何操作失败
- **THEN** 该批次的文章 `processed_at` SHALL 保持 NULL
- **THEN** 文章将在下次 filter 运行时重新处理
