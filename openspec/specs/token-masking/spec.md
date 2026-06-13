# Token Masking

## Purpose

启动日志中 token 的掩码显示逻辑：已有 token 时仅显示部分字符，首次创建时通过 warn 级别打印完整明文。

## Requirements

### Requirement: Token 掩码显示

已有活跃 token 时，系统在启动日志中 SHALL 仅显示 token 的掩码形式（前 4 位 + `...` + 后 4 位），而非完整明文。当 token 长度不足 8 位时 SHALL 显示 `****`。

#### Scenario: 已有 token 显示掩码

- **WHEN** 系统启动且 `api_tokens` 表中有活跃 token
- **THEN** 启动日志 SHALL 显示格式 `Active token: abcd...wxyz`
- **THEN** 日志 SHALL 同时显示 token 总数

#### Scenario: Token 长度不足 8 位

- **WHEN** token 长度不足 8 个字符
- **THEN** 日志 SHALL 显示 `Active token: ****`

### Requirement: 首次创建 Token 明文输出

系统仅在首次自动创建初始 token 时，通过 `warn` 级别日志输出完整明文，确保管理员能够获取初始凭证。

#### Scenario: 空数据库首次创建 token

- **WHEN** `api_tokens` 表为空且系统自动创建初始 token
- **THEN** `tracing::warn!` SHALL 以显著格式打印完整 token 明文
- **THEN** 日志级别 SHALL 为 `WARN` 以确保在生产日志中可见

#### Scenario: 已有 token 时不输出明文

- **WHEN** `api_tokens` 表不为空
- **THEN** 系统 SHALL NOT 打印任何 token 明文
