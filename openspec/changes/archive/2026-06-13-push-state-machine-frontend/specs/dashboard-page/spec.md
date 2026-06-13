## MODIFIED Requirements

### Requirement: 活跃热点表格

仪表盘页面 SHALL 在双栏布局的右侧面板显示活跃热点表格，包含关键词、热度（次/小时）、偏离度（σ 值）、推送状态列。

#### Scenario: 显示热点列表

- **WHEN** 仪表盘加载完成且存在热点事件
- **THEN** 表格 SHALL 显示每个热点的关键词名称、当前计数（格式 `{count} 次/小时`）、偏离度（格式 `{deviation}σ`）
- **THEN** 热度列 SHALL 使用 mono 字体
- **THEN** 偏离度 SHALL 使用 `.severity` 样式类，根据 σ 值显示不同颜色（≥3.0 critical、≥2.0 high、其他 medium）

#### Scenario: 显示推送状态

- **WHEN** 热点所有 channel 推送记录状态均为 `success`
- **THEN** 状态列 SHALL 显示绿色 `.badge-success` 徽章「已推送」
- **WHEN** 热点任一 channel 推送记录状态为 `dead`
- **THEN** 状态列 SHALL 显示红色 `.badge-dead` 徽章「已放弃」，title 属性 SHALL 包含各 channel 的 `last_error` 信息
- **WHEN** 热点任一 channel 推送记录状态为 `failed` 且无 `dead` 记录
- **THEN** 状态列 SHALL 显示黄色 `.badge-warn` 徽章「推送失败」，title 属性 SHALL 包含错误详情
- **WHEN** 热点所有 channel 推送记录状态为 `pending`
- **THEN** 状态列 SHALL 显示蓝色 `.badge-info` 徽章「待推送」
- **WHEN** 推送记录请求失败或状态未知
- **THEN** 状态列 SHALL 显示灰色 `.badge-muted` 徽章「未知」

#### Scenario: 面板头部包含操作按钮

- **WHEN** 活跃热点面板渲染
- **THEN** 面板头部 SHALL 显示"活跃热点"标题
- **THEN** 面板头部右侧 SHALL 显示"手动扫描"按钮（`.btn.btn-primary.btn-sm`）
- **WHEN** 用户点击"手动扫描"
- **THEN** 系统 SHALL 调用 `POST /api/v1/trigger/filter` 并通过 Toast 通知结果

#### Scenario: 无热点数据

- **WHEN** 热点列表为空
- **THEN** 表格区域 SHALL 显示 `<Empty description="暂无热点事件" />`

### Requirement: 最新文章表格

仪表盘页面 SHALL 在下方面板显示最新 5 篇文章的表格，包含来源、标题（可点击外链）、匹配关键词、发布时间、处理状态列。

#### Scenario: 显示最新文章

- **WHEN** 仪表盘加载完成且存在文章
- **THEN** 表格 SHALL 显示最近 5 篇文章，每行包含：来源名（mono 字体 11px）、标题（truncate 截断，max-width: 320px，可点击打开原文链接）、匹配关键词（mono 字体绿色）、发布时间（mono 字体 11px）、处理状态（使用 `articleStatusBadge` 工具函数渲染 4 态 Badge）

#### Scenario: pending 状态文章

- **WHEN** 文章 `status` 为 `pending`
- **THEN** 状态列 SHALL 显示黄色 Badge「待处理」

#### Scenario: matched 状态文章

- **WHEN** 文章 `status` 为 `matched`
- **THEN** 状态列 SHALL 显示绿色 Badge「已匹配」

#### Scenario: 旧数据无 status 字段

- **WHEN** 文章无 `status` 字段但 `processed_at` 有值
- **THEN** 状态列 SHALL 显示绿色 Badge「已处理」（向后兼容 fallback）

#### Scenario: 面板头部包含跳转链接

- **WHEN** 最新文章面板渲染
- **THEN** 面板头部右侧 SHALL 显示「查看全部 →」按钮（`.btn.btn-ghost.btn-sm`）
- **WHEN** 用户点击「查看全部 →」
- **THEN** 系统 SHALL 导航到 `/articles` 页面

#### Scenario: 无文章数据

- **WHEN** 文章列表为空
- **THEN** 表格区域 SHALL 显示 `<Empty description="暂无文章" />`

#### Scenario: 文章标题为空

- **WHEN** 文章的 `title` 字段为 null 或空字符串
- **THEN** 标题列 SHALL 显示「(无标题)」作为占位文本，仍保留原文链接

## ADDED Requirements

### Requirement: pushStatusMap 支持聚合状态和错误信息

仪表盘页面的 `pushStatusMap` SHALL 从 `Record<number, string>` 扩展为 `Record<number, { status: string; errors: string[] }>`，跨多个 channel 聚合热点推送状态。

#### Scenario: 多个 channel 全部成功

- **WHEN** 热点有 2 个 channel 的推送记录，状态均为 `success`
- **THEN** `pushStatusMap[hotspotId]` 为 `{ status: 'success', errors: [] }`

#### Scenario: 存在 dead 记录

- **WHEN** 热点有 2 个 channel 的推送记录，状态分别为 `success` 和 `dead`，dead 记录的 `last_error` 为 `"timeout"`
- **THEN** `pushStatusMap[hotspotId]` 为 `{ status: 'dead', errors: ['channel_name: timeout'] }`

#### Scenario: 存在 failed 但无 dead

- **WHEN** 热点有 2 个 channel 的推送记录，状态分别为 `pending` 和 `failed`，failed 记录的 `last_error` 为 `"DNS error"`
- **THEN** `pushStatusMap[hotspotId]` 为 `{ status: 'failed', errors: ['channel_name: DNS error'] }`
