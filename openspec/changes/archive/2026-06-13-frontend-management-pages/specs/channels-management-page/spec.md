# Channels Management Page

## Purpose

推送渠道管理页面 — 表格展示所有推送渠道，支持添加、编辑、删除和测试 Webhook 连接，使用自定义 CSS 组件匹配原型暗色主题。

## ADDED Requirements

### Requirement: Channels table display
系统 SHALL 以表格形式展示所有推送渠道，每行显示名称、类型、Webhook URL（脱敏）、推送次数、上次推送时间、启用状态和操作按钮。

#### Scenario: Table renders with channel data
- **WHEN** 页面加载且 `GET /api/v1/channels` 返回渠道列表
- **THEN** 表格每行显示：名称（`color: var(--fg)`）、类型（`.badge-neutral`）、Webhook URL（mono 字体 truncate，脱敏显示）、推送次数、上次推送、状态（`.badge-success` / `.badge-neutral`）、操作按钮（「测试」「编辑」）

#### Scenario: Webhook URL masked display
- **WHEN** 渠道 config 中包含 `url` 字段
- **THEN** URL 以脱敏形式显示（仅显示协议和域名，如 `https://hooks.example.com/****`）

#### Scenario: Empty state
- **WHEN** 渠道列表为空
- **THEN** 显示 Empty 组件，描述为"暂无推送渠道"

### Requirement: Add channel modal
系统 SHALL 通过 Modal 弹窗提供添加推送渠道的表单，包含名称、类型和 Webhook URL 字段。

#### Scenario: Modal opens with empty form
- **WHEN** 用户点击「+ 添加渠道」按钮
- **THEN** Modal 显示标题"添加推送渠道"，表单字段：名称（必填）、类型（select: webhook）、Webhook URL（必填，URL 格式），以及「取消」和「确认添加」按钮

#### Scenario: Config is assembled from form fields
- **WHEN** 用户提交表单
- **THEN** Webhook URL 被组装为 `config: { url: "<webhook_url>" }` JSON 字符串发送至 API

#### Scenario: Successful channel creation
- **WHEN** 用户填写完整表单并点击「确认添加」
- **THEN** 调用 `POST /api/v1/channels`，关闭 Modal，刷新列表，显示"推送渠道已添加"Toast

### Requirement: Edit channel
系统 SHALL 支持编辑已有渠道，从 config JSON 中提取 Webhook URL 预填到表单。

#### Scenario: Edit modal extracts URL from config
- **WHEN** 用户点击某行的「编辑」按钮
- **THEN** Modal 标题为"编辑推送渠道"，Webhook URL 字段预填当前 config.url 值

#### Scenario: Successful channel update
- **WHEN** 用户修改字段并提交
- **THEN** 调用 `POST /api/v1/channels/update/{id}`，关闭 Modal，刷新列表，显示"推送渠道已更新"Toast

### Requirement: Delete channel
系统 SHALL 支持从表格行内删除渠道。

#### Scenario: Delete with confirmation
- **WHEN** 用户在表格行内点击「删除」按钮
- **THEN** 弹出 Confirm 确认弹窗，确认后调用 `POST /api/v1/channels/delete/{id}`，刷新列表，显示"推送渠道已删除"Toast

### Requirement: Test channel webhook
系统 SHALL 支持测试推送渠道的 Webhook 连接。

#### Scenario: Test triggered
- **WHEN** 用户点击某行的「测试」按钮
- **THEN** 调用 `POST /api/v1/channels/test/{id}`，显示"测试消息已发送"Toast

#### Scenario: Test endpoint unavailable
- **WHEN** 后端未实现 test 端点且返回错误
- **THEN** 显示"测试功能暂不可用"Toast 提示

### Requirement: Custom CSS styling without Ant Design
页面 SHALL 使用自定义 CSS 组件类，不导入 antd 组件。
