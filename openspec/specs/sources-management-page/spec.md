# Sources Management Page

## Purpose

数据源管理页面 — 表格展示所有数据源，支持添加、编辑、删除和手动触发抓取，使用自定义 CSS 组件匹配原型暗色主题。

## Requirements

### Requirement: Data sources table display
系统 SHALL 以表格形式展示所有数据源，每行显示名称、类型、URL、拉取间隔、文章数、上次抓取时间、启用状态和操作按钮。

#### Scenario: Table renders with source data
- **WHEN** 页面加载且 `GET /api/v1/sources` 返回数据源列表
- **THEN** 表格每行显示：名称（`color: var(--fg)`）、类型（`.badge-neutral`）、URL（mono 字体 truncate）、间隔秒数、文章数、上次抓取时间、状态（`.badge-success` / `.badge-danger`）、操作按钮（「抓取」「编辑」）

#### Scenario: Empty state
- **WHEN** 数据源列表为空
- **THEN** 显示 Empty 组件，描述为"暂无数据源"，并提供"添加数据源"操作按钮

#### Scenario: Loading state
- **WHEN** 数据正在请求中
- **THEN** 显示"加载中..."文字（`color: var(--muted)`）

### Requirement: Add data source modal
系统 SHALL 通过 Modal 弹窗提供添加数据源的表单，包含名称、类型（RSS/API/Atom）、URL 和拉取间隔字段。

#### Scenario: Modal opens with empty form
- **WHEN** 用户点击「+ 添加数据源」按钮
- **THEN** Modal 弹窗显示标题"添加数据源"，表单字段：名称（必填）、类型（select: RSS/API/Atom）、URL（必填）、拉取间隔（默认 300 秒，最小 30），以及「取消」和「确认添加」按钮

#### Scenario: Form validation on submit
- **WHEN** 用户提交表单但名称为空或 URL 为空
- **THEN** 显示 Toast 错误提示，不发送请求

#### Scenario: Successful source creation
- **WHEN** 用户填写完整表单并点击「确认添加」
- **THEN** 调用 `POST /api/v1/sources` 创建数据源，关闭 Modal，刷新列表，显示"数据源已添加"Toast

#### Scenario: Overlay click closes modal
- **WHEN** 用户点击 Modal 背景遮罩
- **THEN** Modal 关闭，表单状态重置

### Requirement: Edit data source
系统 SHALL 支持编辑已有数据源，预填当前值到 Modal 表单。

#### Scenario: Edit modal opens with pre-filled values
- **WHEN** 用户点击某行的「编辑」按钮
- **THEN** Modal 显示标题"编辑数据源"，表单字段预填当前名称、类型、URL、间隔值

#### Scenario: Successful source update
- **WHEN** 用户修改字段并提交
- **THEN** 调用 `POST /api/v1/sources/update/{id}`，关闭 Modal，刷新列表，显示"数据源已更新"Toast

### Requirement: Delete data source
系统 SHALL 支持从表格行内删除数据源。

#### Scenario: Delete with confirmation
- **WHEN** 用户在表格行内点击「删除」按钮
- **THEN** 弹出 Confirm 确认弹窗，确认后调用 `POST /api/v1/sources/delete/{id}`，刷新列表，显示"数据源已删除"Toast

### Requirement: Manual fetch trigger
系统 SHALL 支持手动触发数据源抓取。

#### Scenario: Fetch triggered successfully
- **WHEN** 用户点击某行的「抓取」按钮
- **THEN** 调用 `POST /api/v1/sources/fetch/{id}`，显示"手动抓取已触发"Toast

### Requirement: Custom CSS styling without Ant Design
页面 SHALL 使用自定义 CSS 组件类（`.panel`、`.btn`、`.badge`、`.modal`、`.table-wrap`、`.field`），不导入 antd 组件。

#### Scenario: Panel layout
- **WHEN** 页面渲染
- **THEN** 根容器使用 `.panel` 类，包含 `.panel-header`（标题 + 操作按钮）和 `.table-wrap` 表格容器

#### Scenario: Modal uses custom overlay
- **WHEN** Modal 打开
- **THEN** 使用 `.modal-overlay.open` + `.modal` 自定义结构，不依赖 antd Modal
