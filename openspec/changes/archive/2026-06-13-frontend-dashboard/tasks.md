## 1. API 层扩展

- [x] 1.1 在 `web/src/renderer/src/api/queries.ts` 中新增 `Source`、`Keyword` 类型接口和 `getSources`、`getKeywords`、`getTrend`、`triggerPusher` API 方法
- [x] 1.2 验证 TypeScript 编译通过：`cd web && npx tsc --noEmit`

## 2. 仪表盘 CSS 类

- [x] 2.1 在 `web/src/renderer/src/styles/index.css` 中新增 `.stats-row`（4 列网格）、`.stat-card`（统计卡片）、`.stat-label`、`.stat-value`、`.stat-sub`、`.stat-sub.up`、`.stat-sub.warn` 样式
- [x] 2.2 在 `index.css` 中新增 `.grid-2-wide`（2:1 比例双栏布局）、`.severity.critical`、`.severity.high`、`.severity.medium`、`.badge-warn`、`.badge-dot`、`.dot-show`、`.success-dot`、`.warn-dot`、`.trend-chart` 样式
- [x] 2.3 在 `index.css` 中新增 `@media (max-width: 1024px)` 断点规则（`.stats-row` 变 2 列、`.grid-2-wide` 变 1 列）
- [x] 2.4 在 `index.css` 中新增 `@media (max-width: 768px)` 断点规则（`.stats-row` 变 1 列、`.sidebar` 隐藏逻辑、`.topbar .menu-btn` 显示）
- [x] 2.5 验证 CSS 构建无错误：`cd web && npm run build`

## 3. 仪表盘页面实现

- [x] 3.1 重写 `web/src/renderer/src/pages/Dashboard.tsx`：实现 `useEffect` 中使用 `Promise.allSettled` 并行获取 `/hotspots?per_page=50`、`/articles?per_page=5`、`/sources`、`/keywords` 数据
- [x] 3.2 实现统计卡片行：从 API 响应中提取 `sources` 总数/启用数、`keywords` 总数/启用数、今日文章数、活跃热点数，渲染 4 个 `.stat-card`
- [x] 3.3 实现 ECharts 趋势图：使用 `echarts-for-react`，暗色主题 option（透明背景、`#16a34a` 折线、`linearGradient` 渐变填充），点击热点行时调用 `GET /trend/:id?hours=24` 更新数据
- [x] 3.4 实现活跃热点表格：关键词、热度（`{count} 次/小时`）、偏离度（`.severity` 类，根据 σ 值显示 critical/high/medium）、推送状态（`.badge-success` 已推送 / `.badge-warn` 待推送）
- [x] 3.5 实现最新文章表格：来源、标题（可点击外链，truncate 截断 max-width: 320px）、匹配关键词、发布时间、处理状态，面板头部「查看全部 →」按钮跳转 `/articles`
- [x] 3.6 实现加载状态和无数据状态：加载中显示居中文本，无数据时使用 `<Empty>` 组件
- [x] 3.7 验证仪表盘功能：启动 `cd web && npm run dev`，访问 `/dashboard`，检查 4 个统计卡片、趋势图、热点表格、最新文章显示正确，点击热点行趋势图更新，点击「手动扫描」触发过滤器，点击「查看全部」跳转 `/articles`

## 4. 文章日志页增强

- [x] 4.1 修改 `web/src/renderer/src/pages/Articles.tsx`：新增 `#` 列（文章 ID，mono 字体 11px，颜色 `var(--meta)`），调整列顺序为 # → 来源 → 标题 → 匹配关键词 → 发布时间 → 处理状态
- [x] 4.2 实现数据源过滤：新增 `sourceFilter` 状态，`useEffect` 中调用 `getSources()` 获取数据源列表，渲染 `<select>` 下拉控件（第一项为「全部数据源」），选择后重置页码并重新请求
- [x] 4.3 实现处理状态过滤：新增 `processedFilter` 状态（`undefined` = 全部、`true` = 已处理、`false` = 待处理），渲染过滤按钮组或下拉控件
- [x] 4.4 实现「运行过滤器」按钮：面板头部右侧新增按钮，点击调用 `client.post('/trigger/filter')`，成功后显示 Toast 并刷新列表
- [x] 4.5 调整表格单元格样式：标题列 truncate max-width: 340px，匹配关键词列绿色 mono 字体，状态列使用带圆点的 badge（`.badge-dot` + `.badge-success`/`.badge-warn`），空关键词显示「—」
- [x] 4.6 更新分页控件：显示「共 {total} 条 · 第 {page}/{totalPages} 页」格式
- [x] 4.7 验证文章日志功能：访问 `/articles`，检查 # 列、数据源过滤、处理状态过滤、运行过滤器按钮功能正常，分页正确

## 5. 编译与端到端验证

- [x] 5.1 前端编译检查：`cd web && npm run build`，确保无 TypeScript 编译错误
- [x] 5.2 端到端测试：启动后端 `cargo run -- config.toml all` + 前端 `npm run dev`，执行完整操作流程（认证 → 仪表盘 → 添加数据源/关键词 → 触发过滤 → 查看热点/趋势 → 文章日志过滤）
