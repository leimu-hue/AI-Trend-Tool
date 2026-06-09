# Keywords Management Page

## Purpose

关键词管理页面 — 表格展示所有关键词，支持添加、编辑、删除和行内启用/暂停切换，使用自定义 CSS 组件匹配原型暗色主题。

## ADDED Requirements

### Requirement: Keywords table display
系统 SHALL 以表格形式展示所有关键词，每行显示关键词文本、大小写敏感设置、标准差倍数、最小触发计数、24h 命中数、启用状态和操作按钮。

#### Scenario: Table renders with keyword data
- **WHEN** 页面加载且 `GET /api/v1/keywords` 返回关键词列表
- **THEN** 表格每行显示：关键词（mono 字体，`color: var(--fg)`）、大小写（「是」/「否」）、标准差倍数、最小计数、24h 命中（绿色 mono）、状态（`.badge-success` 启用 / `.badge-neutral` 暂停）、操作按钮（「暂停/启用」「编辑」）

#### Scenario: Empty state
- **WHEN** 关键词列表为空
- **THEN** 显示 Empty 组件，描述为"暂无关键词"

#### Scenario: Loading state
- **WHEN** 数据正在请求中
- **THEN** 显示"加载中..."文字

### Requirement: Add keyword modal
系统 SHALL 通过 Modal 弹窗提供添加关键词的表单，包含关键词文本、大小写敏感、标准差倍数和最小触发计数字段。

#### Scenario: Modal opens with default values
- **WHEN** 用户点击「+ 添加关键词」按钮
- **THEN** Modal 显示标题"添加关键词"，表单字段：关键词（必填）、大小写敏感（select: 是/否，默认"否"）、标准差倍数（默认 2.0，步长 0.1）、最小触发计数（默认 3），以及「取消」和「确认添加」按钮

#### Scenario: Successful keyword creation
- **WHEN** 用户填写关键词并点击「确认添加」
- **THEN** 调用 `POST /api/v1/keywords`，关闭 Modal，刷新列表，显示"关键词已添加"Toast

### Requirement: Edit keyword
系统 SHALL 支持编辑已有关键词，预填当前值到 Modal 表单。

#### Scenario: Edit modal opens with pre-filled values
- **WHEN** 用户点击某行的「编辑」按钮
- **THEN** Modal 显示标题"编辑关键词"，表单字段预填当前关键词、大小写敏感、标准差倍数、最小计数

#### Scenario: Successful keyword update
- **WHEN** 用户修改字段并提交
- **THEN** 调用 `POST /api/v1/keywords/update/{id}`，关闭 Modal，刷新列表，显示"关键词已更新"Toast

### Requirement: Delete keyword
系统 SHALL 支持从编辑 Modal 中删除关键词。

#### Scenario: Delete with confirmation
- **WHEN** 用户在编辑 Modal 中点击「删除」按钮
- **THEN** 弹出 `window.confirm` 确认，确认后调用 `POST /api/v1/keywords/delete/{id}`，关闭 Modal，刷新列表，显示"关键词已删除"Toast

### Requirement: Toggle keyword enabled state
系统 SHALL 支持行内切换关键词启用/暂停状态，无需打开 Modal。

#### Scenario: Enable paused keyword
- **WHEN** 用户点击暂停关键词的「启用」按钮
- **THEN** 调用 `POST /api/v1/keywords/update/{id}` 设置 `enabled: true`，刷新列表，显示"关键词已启用"Toast

#### Scenario: Pause enabled keyword
- **WHEN** 用户点击启用关键词的「暂停」按钮
- **THEN** 调用 `POST /api/v1/keywords/update/{id}` 设置 `enabled: false`，刷新列表，显示"关键词已暂停"Toast

### Requirement: Custom CSS styling without Ant Design
页面 SHALL 使用自定义 CSS 组件类，不导入 antd 组件，与数据源管理页保持一致的视觉风格。
