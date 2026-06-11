## ADDED Requirements

### Requirement: Config 验证单元测试

系统 SHALL 在 `src/config.rs` 底部添加 `#[cfg(test)] mod tests`，至少包含有效配置通过和无效配置拒绝两个测试。

#### Scenario: 有效配置通过验证

- **WHEN** `cargo test` 运行 config 测试
- **THEN** 加载有效 `config.toml` 的测试 SHALL 通过（`validate().is_ok()`）

#### Scenario: 无效端口被拒绝

- **WHEN** `cargo test` 运行 config 测试
- **THEN** `port = 0` 的配置 SHALL 验证失败（`validate().is_err()`）

### Requirement: Error 模块单元测试

系统 SHALL 在 `src/error.rs` 底部添加单元测试，验证 NotFound 和 Database 错误变体的 HTTP 状态码。

#### Scenario: NotFound 返回 404

- **WHEN** `AppError::NotFound("test".into()).into_response()` 被调用
- **THEN** 响应状态码 SHALL 为 404

#### Scenario: Database 错误返回 500 且不泄露细节

- **WHEN** `AppError::Database(sqlx::Error::PoolTimedOut).into_response()` 被调用
- **THEN** 响应状态码 SHALL 为 500
- **THEN** 响应体 SHALL NOT 包含原始错误字符串

### Requirement: Filter 统计函数单元测试

系统 SHALL 提取 `compute_stats` 纯函数（计算均值和标准差），并添加单元测试验证边界情况。

#### Scenario: 空数组返回零

- **WHEN** `compute_stats(&[])` 被调用
- **THEN** 返回值 SHALL 为 `(0.0, 0.0)`

#### Scenario: 单元素数组

- **WHEN** `compute_stats(&[5])` 被调用
- **THEN** 均值 SHALL 为 5.0，标准差 SHALL 为 0.0

#### Scenario: 正常数据计算

- **WHEN** `compute_stats(&[2, 4, 4, 4, 5, 5, 7, 9])` 被调用
- **THEN** 均值 SHALL 约等于 5.0（误差 < 0.01）
- **THEN** 标准差 SHALL 约等于 2.0（误差 < 0.01）
