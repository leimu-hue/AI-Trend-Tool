# Tokens Management Page

## Purpose

API Token 管理页面 — 表格展示所有 Token，支持生成（一次性明文展示）、复制和吊销操作，使用自定义 CSS 组件匹配原型暗色主题。

## ADDED Requirements

### Requirement: Tokens table display
系统 SHALL 以表格形式展示所有 API Token，每行显示名称、最后使用时间、过期时间、状态和操作按钮。

#### Scenario: Table renders with token data
- **WHEN** 页面加载且 `GET /api/v1/tokens` 返回 Token 列表
- **THEN** 表格每行显示：名称（`color: var(--fg)`）、最后使用（mono 字体）、过期时间（mono 字体，「永久」或日期）、状态（`.badge-success` 有效 / `.badge-danger` 已吊销）、操作按钮（有效 Token 显示「吊销」，已吊销显示「—」）

#### Scenario: Empty state
- **WHEN** Token 列表为空
- **THEN** 显示 Empty 组件，描述为"暂无 API 令牌"

### Requirement: Generate token modal
系统 SHALL 通过 Modal 弹窗提供生成新 Token 的表单，创建成功后展示一次性明文。

#### Scenario: Modal opens with form
- **WHEN** 用户点击「+ 生成令牌」按钮
- **THEN** Modal 显示标题"生成 API 令牌"，表单字段：令牌名称/用途（必填）、过期时间（可选 date 输入，留空为永久），以及「取消」和「生成」按钮

#### Scenario: Token created and shown once
- **WHEN** 用户填写名称并点击「生成」
- **THEN** 调用 `POST /api/v1/tokens`，Modal 内容切换为成功展示：显示完整 Token 明文、名称、过期时间，提供「复制」按钮，警告文字"请立即复制并安全保存此令牌，关闭后将无法再次查看"

#### Scenario: Copy token to clipboard
- **WHEN** 用户在成功展示界面点击「复制」按钮
- **THEN** Token 明文被复制到剪贴板，显示"令牌已复制"Toast

#### Scenario: Close after token generation refreshes list
- **WHEN** 用户关闭成功展示的 Modal
- **THEN** Token 列表刷新，新 Token 出现在列表中（不含明文）

### Requirement: Revoke token
系统 SHALL 支持吊销有效 Token，操作前需确认。

#### Scenario: Revoke with confirmation
- **WHEN** 用户点击有效 Token 行的「吊销」按钮
- **THEN** 弹出 Confirm 确认弹窗，确认后调用 `POST /api/v1/tokens/revoke/{id}`，刷新列表，显示"令牌已吊销"Toast

#### Scenario: Revoked token shows no actions
- **WHEN** Token 已吊销（`revoked: true`）
- **THEN** 操作列显示「—」，无任何按钮

### Requirement: Custom CSS styling without Ant Design
页面 SHALL 使用自定义 CSS 组件类，不导入 antd 组件。
