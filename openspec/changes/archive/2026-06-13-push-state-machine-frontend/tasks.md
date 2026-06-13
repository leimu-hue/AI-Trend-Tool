## 1. API 接口层更新

- [x] 1.1 更新 `Article` 接口：新增 `status: string` 字段 → `web/src/renderer/src/api/queries.ts`
- [x] 1.2 更新 `PushRecord` 接口：新增 `last_error: string | null` 字段，`status` 支持 `'dead'` → `web/src/renderer/src/api/queries.ts`
- [x] 1.3 更新 `getArticles` 函数签名：`params` 新增 `status?: string` 参数，保留 `processed` → `web/src/renderer/src/api/queries.ts`
- [x] 1.4 运行 `npx tsc --noEmit` 验证 TypeScript 编译通过

## 2. 共享工具函数

- [x] 2.1 新建 `web/src/renderer/src/utils/statusBadge.ts`，实现 `articleStatusBadge(status?, processedAt?)` 函数
- [x] 2.2 验证：确保函数覆盖 7 种场景（pending/processing/matched/skipped/旧已处理/旧待处理/未知）

## 3. Articles 页面改造

- [x] 3.1 状态筛选器改造：`processedFilter: boolean | undefined` → `statusFilter: string | undefined`，Select 选项改为 5 项（全部/待处理/处理中/已匹配/已跳过） → `web/src/renderer/src/pages/Articles.tsx`
- [x] 3.2 `load` 函数参数更新：`procFilter: boolean | undefined` → `stFilter: string | undefined`，调用 `getArticles({ status })`
- [x] 3.3 状态列 Badge 改造：从二元判断（`processed_at ? '已处理' : '待处理'`）改为调用 `articleStatusBadge(a.status, a.processed_at)` 渲染 4 态 Badge + 圆点
- [x] 3.4 `useEffect` 依赖更新：`processedFilter` → `statusFilter`

## 4. Dashboard 页面改造

- [x] 4.1 `pushStatusMap` 类型扩展：`Record<number, string>` → `Record<number, { status: string; errors: string[] }>`，实现跨 channel 聚合逻辑 → `web/src/renderer/src/pages/Dashboard.tsx`
- [x] 4.2 推送状态 Badge 实现：`pushStatusBadge(info)` 函数，支持 success/failed/dead/pending/unknown 五种状态渲染
- [x] 4.3 最新文章状态列适配：从 `processed_at` 判断改为调用 `articleStatusBadge(a.status, a.processed_at)`
- [x] 4.4 导入 `articleStatusBadge` 工具函数

## 5. CSS 样式扩展

- [x] 5.1 新增 `badge-info` 样式（蓝色背景，处理中/待推送） → `web/src/renderer/src/styles/index.css`
- [x] 5.2 新增 `badge-dead` 样式（红色背景，重试耗尽） → `web/src/renderer/src/styles/index.css`
- [x] 5.3 新增 `badge-muted` 样式（灰色背景，已跳过/未知） → `web/src/renderer/src/styles/index.css`
- [x] 5.4 新增 `info-dot` 和 `muted-dot` 圆点样式

## 6. 构建验证

- [x] 6.1 在 `web/` 目录运行 `npm run build`，确认 Electron + React 编译零错误
- [x] 6.2 目视检查 Articles 页面：筛选器下拉包含 5 个选项，Badge 4 种颜色正确
- [x] 6.3 目视检查 Dashboard 页面：热点表推送状态 Badge 5 种状态正确，最新文章状态列使用新 Badge
