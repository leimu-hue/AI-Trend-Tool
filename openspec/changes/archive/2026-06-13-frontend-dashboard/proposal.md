## Why

仪表盘页目前仅为占位状态（显示"仪表盘内容即将上线"），文章日志页已基本实现但缺少设计文档中要求的 `#` 列、数据源过滤、处理状态过滤和"运行过滤器"按钮。需按照 `docs/plans/08-frontend-dashboard.md` 设计文档完成这两个核心页面，使系统具备可视化监控能力——用户能一眼看到统计概览、热点趋势、活跃热点和最新文章，并能对文章日志进行筛选管理。

## What Changes

- 重写 `Dashboard.tsx`：从占位卡片替换为完整的仪表盘页面，包含统计卡片行、ECharts 趋势图、活跃热点表格、最新文章表格，点击热点行可动态加载趋势数据，提供"手动扫描"和"查看全部"操作入口
- 增强 `Articles.tsx`：新增 `#` 列、数据源下拉过滤、处理状态过滤、"运行过滤器"按钮，调整表格列样式以匹配设计规范
- 扩展 `index.css`：新增 `.stats-row`、`.stat-card`、`.grid-2-wide`、`.severity`、`.badge-warn`、`.trend-chart` 等仪表盘专用 CSS 类，以及 `≤1024px` / `≤768px` 响应式断点规则
- 扩展 `queries.ts`：新增 `getSources`、`getKeywords`、`getTrend`、`getTriggerPusher` API 方法，导出 `Source` 和 `Keyword` 类型

## Capabilities

### New Capabilities

- `dashboard-page`: 仪表盘页面——统计卡片（数据源/关键词/今日文章/活跃热点）、ECharts 暗色主题趋势图（24h 关键词提及折线图）、活跃热点表格（关键词/热度/偏离度/推送状态）、最新文章表格（来源/标题/匹配关键词/时间/状态），支持点击热点行加载趋势、手动触发过滤器扫描

### Modified Capabilities

- `frontend-pagination`: 文章日志页需求变更——新增 `#` 序号列、数据源下拉过滤、处理状态过滤、"运行过滤器"按钮，表格列排序和样式需与设计文档对齐（来源列用 mono 字体、标题列 truncate 截断、匹配关键词列绿色显示）

## Impact

- 前端文件：`src/pages/Dashboard.tsx`（重写）、`src/pages/Articles.tsx`（修改）、`src/styles/index.css`（新增 CSS 类）、`src/api/queries.ts`（扩展 API 方法）
- 依赖：`echarts-for-react`（已在 package.json 中）、`react-router-dom`（已有）、axios client（已有）、Toast 组件（已有）、Empty 组件（已有）
- 后端 API：无需修改，复用已有 `GET /hotspots`、`GET /articles`、`GET /sources`、`GET /keywords`、`GET /trend/:id`、`POST /trigger/filter` 端点
- 路由：无变化，`/dashboard` 和 `/articles` 路由已存在

## Non-goals

- 不修改后端 API 或数据库结构
- 不修改 Electron 主进程或构建配置
- 不新增路由或导航结构变更
- 不实现实时 WebSocket 推送（保持定时刷新 + 手动触发模式）
- 不实现趋势图的交互式缩放/拖拽（保持 ECharts 默认交互）
