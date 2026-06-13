# Dashboard Page

## Purpose

仪表盘页面提供系统核心指标的可视化概览：统计卡片、关键词趋势图、活跃热点表格和最新文章表格。

## Requirements

### Requirement: 统计卡片行显示

仪表盘页面 SHALL 在顶部显示 4 个统计卡片（数据源、关键词、今日文章、活跃热点），使用 `.stats-row` 网格布局和 `.stat-card` 卡片样式。

#### Scenario: 正常加载显示所有统计卡片

- **WHEN** 仪表盘页面加载完成且所有 API 请求成功
- **THEN** 页面 SHALL 显示 4 个 `.stat-card`：数据源（含总数和启用数详情）、关键词（含总数和启用数详情）、今日文章（含总数和环比趋势）、活跃热点（含总数）
- **THEN** 统计数值 SHALL 使用 `.stat-value` 样式（大号字体突出显示）
- **THEN** 今日文章趋势正值 SHALL 显示绿色 `.stat-sub.up` 样式
- **THEN** 活跃热点数 > 0 时 SHALL 显示黄色 `.stat-sub.warn` 样式

#### Scenario: 部分 API 请求失败

- **WHEN** 4 个并行请求中某个请求失败
- **THEN** 对应统计卡片 SHALL 显示 "—" 作为占位值
- **THEN** 其他卡片 SHALL 正常显示已获取的数据

#### Scenario: 响应式布局——平板

- **WHEN** 视口宽度 ≤ 1024px
- **THEN** `.stats-row` SHALL 显示为 2 列网格

#### Scenario: 响应式布局——手机

- **WHEN** 视口宽度 ≤ 768px
- **THEN** `.stats-row` SHALL 显示为 1 列网格

### Requirement: ECharts 关键词趋势图

仪表盘页面 SHALL 使用 ECharts 渲染 24 小时关键词提及次数趋势折线图，采用暗色主题配色，包含绿色渐变填充区域和平滑曲线。

#### Scenario: 点击热点行加载趋势数据

- **WHEN** 用户点击活跃热点表格中的某一行
- **THEN** 系统 SHALL 请求 `GET /api/v1/trend/{keyword_id}?hours=24`
- **THEN** 趋势图 SHALL 更新为该关键词的 24 小时数据点折线图
- **THEN** 面板标题右侧的 badge SHALL 更新为当前关键词名称

#### Scenario: 无热点数据时趋势图显示空状态

- **WHEN** 热点列表为空或用户尚未点击任何热点行
- **THEN** 趋势图 SHALL 显示空坐标轴和"暂无趋势数据"占位提示

#### Scenario: 趋势图暗色主题配置

- **WHEN** 趋势图渲染
- **THEN** 图表背景 SHALL 为透明（`backgroundColor: 'transparent'`）
- **THEN** X 轴标签 SHALL 使用 `#868584` 颜色（对应 `--muted`）
- **THEN** Y 轴分割线 SHALL 使用 `rgba(255,255,255,0.03)` 颜色
- **THEN** 折线 SHALL 使用 `#16a34a` 颜色（对应 `--success`），宽度 2px
- **THEN** 区域渐变 SHALL 从 `rgba(22,163,74,0.25)` 渐变到 `rgba(22,163,74,0)`

### Requirement: 活跃热点表格

仪表盘页面 SHALL 在双栏布局的右侧面板显示活跃热点表格，包含关键词、热度（次/小时）、偏离度（σ 值）、推送状态列。

#### Scenario: 显示热点列表

- **WHEN** 仪表盘加载完成且存在热点事件
- **THEN** 表格 SHALL 显示每个热点的关键词名称、当前计数（格式 `{count} 次/小时`）、偏离度（格式 `{deviation}σ`）
- **THEN** 热度列 SHALL 使用 mono 字体
- **THEN** 偏离度 SHALL 使用 `.severity` 样式类，根据 σ 值显示不同颜色（≥3.0 critical、≥2.0 high、其他 medium）

#### Scenario: 显示推送状态

- **WHEN** 热点已推送
- **THEN** 状态列 SHALL 显示绿色 `.badge-success` 徽章「已推送」
- **WHEN** 热点待推送
- **THEN** 状态列 SHALL 显示黄色 `.badge-warn` 徽章「待推送」

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
- **THEN** 表格 SHALL 显示最近 5 篇文章，每行包含：来源名（mono 字体 11px）、标题（truncate 截断，max-width: 320px，可点击打开原文链接）、匹配关键词（mono 字体绿色）、发布时间（mono 字体 11px）、处理状态（badge-success 已处理 / badge-warn 待处理）

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

### Requirement: 仪表盘加载状态

仪表盘页面 SHALL 在数据加载期间显示加载中状态，加载完成后显示内容。

#### Scenario: 加载中状态

- **WHEN** 仪表盘页面首次渲染且 API 请求未完成
- **THEN** 页面 SHALL 显示居中的"加载中..."文本（颜色 `var(--muted)`）

#### Scenario: 加载完成

- **WHEN** 所有 API 请求完成（无论成功或失败）
- **THEN** 加载状态 SHALL 解除，页面 SHALL 渲染完整布局（统计卡片 + 双栏 + 最新文章）
