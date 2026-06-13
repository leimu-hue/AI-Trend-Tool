## Context

当前 `Dashboard.tsx` 为占位状态，`Articles.tsx` 已实现基础分页列表但缺少设计文档要求的列和过滤功能。前端项目已有完整的 API 客户端层（`queries.ts` 提供 `getHotspots`、`getArticles`、`triggerFilter`）、自定义 CSS 设计系统（`index.css` 中定义 `.panel`、`.badge`、`.btn` 等组件类）、Toast 通知系统、Empty 组件和响应式侧边栏布局。

后端 API 已完备：`GET /hotspots`、`GET /articles`、`GET /sources`、`GET /keywords`、`GET /trend/:id`、`POST /trigger/filter`，无需修改。

设计文档 `docs/plans/08-frontend-dashboard.md` 定义了完整的 UI 结构、组件样式和响应式断点规则。

## Goals / Non-Goals

**Goals:**
- 实现仪表盘页面的完整 UI：统计卡片行、ECharts 趋势图、活跃热点表格、最新文章表格
- 增强文章日志页面：新增 `#` 列、数据源过滤、处理状态过滤、"运行过滤器"按钮
- 新增仪表盘专用 CSS 类（`.stats-row`、`.stat-card`、`.grid-2-wide`、`.severity`、`.badge-warn`、`.trend-chart`）及响应式断点规则
- 扩展 `queries.ts` API 层以支持仪表盘所需的全部数据获取

**Non-Goals:**
- 不修改后端 API
- 不修改路由配置（`/dashboard` 和 `/articles` 已存在）
- 不修改 Electron 主进程
- 不实现实时数据推送（WebSocket/SSE）
- 不修改其他已有页面（Sources、Keywords、Channels、Tokens、Settings）

## Decisions

### 决策 1：使用 ECharts 而非手写 SVG

**选择**：使用 `echarts-for-react`（已在 `package.json` 中声明为依赖）绘制趋势图。

**替代方案**：手写内联 SVG（如原型 `docs/Live-Artifact/index.html` 所示）。

**理由**：ECharts 提供开箱即用的 tooltip、暗色主题适配、响应式 resize、平滑曲线等能力，避免大量 SVG 手写代码。原型 SVG 为静态示例，实际数据需要动态绑定——ECharts 的 `setOption` 机制天然适合。`echarts-for-react` 已在项目依赖中，无需新增包。

**注意**：原型使用 `echarts.graphic.LinearGradient`，这需要导入 `echarts` 模块本身用于渐变 API。React 组件中可使用 `import * as echarts from 'echarts'` 或直接使用 `new echarts.graphic.LinearGradient(...)`。

### 决策 2：趋势图交互模式——点击热点行加载

**选择**：点击活跃热点表格中的某一行，触发对应关键词的 24h 趋势数据加载，更新趋势图。

**理由**：设计文档定义了 `handleHotspotClick` 行为。首次加载时默认显示第一个热点的趋势（如有），否则显示空状态。这样用户可主动探索不同关键词的趋势，避免一次性加载所有数据。

### 决策 3：统计卡片数据来源——并行请求

**选择**：`useEffect` 中使用 `Promise.all` 并行请求 4 个接口：`/hotspots`、`/articles?per_page=5`、`/sources`、`/keywords`。

**理由**：统计卡片需要 4 个独立数据源（数据源数、关键词数、今日文章数、活跃热点数）。并行请求减少首屏等待时间。4 个请求无依赖关系，适合并行。

**替代方案**：后端提供 `/dashboard/stats` 聚合接口。这可以减少请求数，但需要修改后端，且 `08-frontend-dashboard.md` 设计文档未定义此接口，保持前端聚合更简单。

### 决策 4：CSS 类的组织方式——扩展现有 `index.css`

**选择**：在 `src/styles/index.css` 中新增仪表盘相关的 CSS 类和响应式断点规则，与已有的 `.panel`、`.badge`、`.btn` 等类保持一致风格。

**理由**：项目已使用 Tailwind CSS v4 的 `@theme` 定义设计 Token + 手写 CSS 组件类的混合模式。仪表盘组件类（`.stats-row`、`.stat-card` 等）是应用级别的组件样式，放在 `index.css` 中与现有模式一致。不创建单独的 CSS 文件以避免样式加载顺序问题。

### 决策 5：Articles 页面增强方案——扩展现有组件

**选择**：在现有 `Articles.tsx` 基础上增量添加 `#` 列、`sourceFilter`、`processedFilter` 状态和"运行过滤器"按钮。

**理由**：Articles 页面已有完整的分页逻辑、数据获取和表格渲染。最小化修改风险，保持代码结构不变。新增的过滤参数通过 URL query params 传递给 `getArticles`，API 层已支持 `source_id` 和 `processed` 参数。

### 决策 6：响应式布局——CSS 媒体查询

**选择**：使用纯 CSS 媒体查询（`@media (max-width: 1024px)` 和 `@media (max-width: 768px)`）实现响应式布局，不引入 JS 断点检测。

**理由**：项目的侧边栏响应式已使用纯 CSS 方案（`index.css` 中无相关媒体查询，但设计文档定义了断点规则）。仪表盘的统计卡片列数变化（4→2→1）和双栏布局切换（2列→1列）纯属视觉适配，不需要 JS 逻辑。与项目现有模式一致。

## Risks / Trade-offs

- **[风险] ECharts 暗色主题的 CSS 变量引用**：ECharts 的 `axisLabel.color` 等配置接受字符串，但 `'var(--muted)'` 不是有效的 CSS 颜色值——ECharts 运行在 Canvas 上，不解析 CSS 变量。→ **缓解**：在 ECharts option 中使用硬编码的暗色主题颜色值（如 `'#868584'` 对应 `--muted`），与设计 Token 保持一致但直接使用 hex 值。

- **[风险] 首次加载时趋势图为空**：如果用户尚未添加关键词或没有热点数据，趋势图区域无内容。→ **缓解**：趋势图面板始终渲染，无数据时 ECharts 显示空坐标轴或"暂无趋势数据"提示。

- **[风险] `Promise.all` 中任一接口失败导致整个仪表盘无数据**：4 个并行请求共享同一个 Promise.all，一个失败全部 reject。→ **缓解**：使用 `Promise.allSettled` 替代，每个请求独立处理失败，部分数据失败不影响其他卡片显示。这是 axios 拦截器已在全局处理错误提示（toast），页面只需处理数据缺失的 UI 回退。

- **[取舍] 趋势图无动画/缩放交互**：设计文档未定义交互式缩放或数据下钻功能，ECharts 保持默认的 tooltip 交互。→ 如需增强交互，可在后续迭代中通过 ECharts 的 `dataZoom` 和 `brush` 组件扩展，本次不实现。

## Open Questions

- 无。设计文档 `docs/plans/08-frontend-dashboard.md` 对 UI 结构、组件层级、API 调用方式和样式规范有明确定义，所有决策点已覆盖。
