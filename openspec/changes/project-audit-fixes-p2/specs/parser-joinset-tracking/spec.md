## ADDED Requirements

### Requirement: Parser 使用 JoinSet 跟踪子任务

Parser 模块 SHALL 使用 `tokio::task::JoinSet` 管理 `tokio::spawn` 的 fetch 子任务。当取消信号触发时，Parser SHALL 等待所有进行中的子任务完成后再退出。

#### Scenario: JoinSet 收集 spawn 的任务

- **WHEN** Parser 为到期数据源 spawn fetch 子任务
- **THEN** 每个子任务 SHALL 通过 `JoinSet::spawn` 启动
- **THEN** JoinSet SHALL 持有所有进行中任务的句柄

#### Scenario: 取消信号后等待子任务完成

- **WHEN** CancellationToken 被触发
- **THEN** Parser SHALL 停止 spawn 新任务
- **THEN** Parser SHALL 循环 `join_next()` 等待所有进行中子任务完成
- **THEN** 所有子任务完成后 Parser SHALL 退出

#### Scenario: 无进行中子任务时立即退出

- **WHEN** CancellationToken 被触发且无进行中的子任务
- **THEN** Parser SHALL 立即退出
