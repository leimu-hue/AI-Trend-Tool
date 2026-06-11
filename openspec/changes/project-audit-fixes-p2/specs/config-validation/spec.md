## ADDED Requirements

### Requirement: Config 验证单元测试

`src/config.rs` SHALL 包含 `#[cfg(test)]` 测试模块，至少覆盖有效配置通过和无效配置（port=0）拒绝两个场景。

#### Scenario: 有效配置通过验证

- **WHEN** `cargo test` 运行
- **THEN** 加载有效 `config.toml` 的测试 SHALL 通过

#### Scenario: port=0 被拒绝

- **WHEN** 测试中构造 port=0 的配置
- **THEN** `validate()` SHALL 返回 `Err`
