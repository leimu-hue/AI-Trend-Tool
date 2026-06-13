# Frontend Article Status Display

## Purpose

TBD - Article status badge rendering and status-based filtering for the Articles page.

## Requirements

### Requirement: 文章状态 Badge 工具函数

系统 SHALL 提供 `articleStatusBadge(status?: string, processedAt?: string | null)` 工具函数，位于 `utils/statusBadge.ts`，返回 `{ cls: string, label: string }` 用于渲染文章处理状态的 Badge。

#### Scenario: status 为 matched

- **WHEN** 调用 `articleStatusBadge('matched')`
- **THEN** 返回 `{ cls: 'badge-success', label: '已匹配' }`

#### Scenario: status 为 pending

- **WHEN** 调用 `articleStatusBadge('pending')`
- **THEN** 返回 `{ cls: 'badge-warn', label: '待处理' }`

#### Scenario: status 为 processing

- **WHEN** 调用 `articleStatusBadge('processing')`
- **THEN** 返回 `{ cls: 'badge-info', label: '处理中' }`

#### Scenario: status 为 skipped

- **WHEN** 调用 `articleStatusBadge('skipped')`
- **THEN** 返回 `{ cls: 'badge-muted', label: '已跳过' }`

#### Scenario: status 缺失，processedAt 有值（向后兼容）

- **WHEN** 调用 `articleStatusBadge(undefined, '2026-01-01T00:00:00Z')`
- **THEN** 返回 `{ cls: 'badge-success', label: '已处理' }`

#### Scenario: status 缺失，processedAt 为 null（向后兼容）

- **WHEN** 调用 `articleStatusBadge(undefined, null)`
- **THEN** 返回 `{ cls: 'badge-warn', label: '待处理' }`

#### Scenario: status 为未知值

- **WHEN** 调用 `articleStatusBadge('unknown_status')`
- **THEN** 返回 `{ cls: 'badge-warn', label: '未知' }`

### Requirement: Articles 页面状态筛选器

Articles 页面 SHALL 提供基于 `status` 字段的多值枚举筛选器，替代原有的 boolean `processed` 筛选器。

#### Scenario: 默认显示全部状态

- **WHEN** Articles 页面首次加载
- **THEN** 状态筛选下拉框 SHALL 选中「全部状态」（值为空字符串）
- **THEN** 表格 SHALL 显示所有状态的文章

#### Scenario: 按 pending 筛选

- **WHEN** 用户选择状态筛选「待处理」（值为 `pending`）
- **THEN** 系统 SHALL 调用 `getArticles({ page: 1, status: 'pending' })`
- **THEN** 表格 SHALL 仅显示 status 为 `pending` 的文章

#### Scenario: 筛选选项列表

- **WHEN** 状态筛选下拉框展开
- **THEN** 选项 SHALL 包含：全部状态、待处理 (pending)、处理中 (processing)、已匹配 (matched)、已跳过 (skipped)

### Requirement: 文章状态 Badge 渲染

系统 SHALL 在文章列表中渲染带颜色和状态圆点的 Badge，明确区分 4 种处理状态。

#### Scenario: pending 状态 Badge

- **WHEN** 文章 `status` 为 `pending`
- **THEN** 渲染黄色 Badge（`badge badge-warn`），含黄色圆点（`badge-dot dot-show warn-dot`），文字「待处理」

#### Scenario: processing 状态 Badge

- **WHEN** 文章 `status` 为 `processing`
- **THEN** 渲染蓝色 Badge（`badge badge-info`），含蓝色圆点（`badge-dot dot-show info-dot`），文字「处理中」

#### Scenario: matched 状态 Badge

- **WHEN** 文章 `status` 为 `matched`
- **THEN** 渲染绿色 Badge（`badge badge-success`），含绿色圆点（`badge-dot dot-show success-dot`），文字「已匹配」

#### Scenario: skipped 状态 Badge

- **WHEN** 文章 `status` 为 `skipped`
- **THEN** 渲染灰色 Badge（`badge badge-muted`），含灰色圆点（`badge-dot dot-show muted-dot`），文字「已跳过」
