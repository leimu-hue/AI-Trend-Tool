## Why

后端 `push-state-machine` 变更引入了 Article 四态状态机（`pending/processing/matched/skipped`）和 PushRecord `dead` 终态 + `last_error` 字段。前端当前仍使用二元 `processed_at IS NULL` 判断和 4 种推送状态枚举，与后端新模型不一致，导致：(1) 文章列表无法按 `processing`/`skipped` 状态筛选；(2) 推送记录无法展示 `dead` 终态和失败原因；(3) Dashboard 推送状态展示粒度不足。本次变更适配前端 UI 层，使其与后端双层状态机模型对齐。

## What Changes

- **Article TypeScript 接口更新**: 新增 `status` 字段（`'pending' | 'processing' | 'matched' | 'skipped'`），保留 `processed_at` 向后兼容
- **PushRecord TypeScript 接口更新**: 新增 `last_error` 字段（失败原因），`status` 枚举扩展支持 `'dead'`
- **getArticles 查询参数更新**: 新增 `status` 参数替代 `processed` boolean 筛选，旧参数保留过渡期
- **Articles 页面状态筛选器**: 从 boolean 二选一改为多值枚举下拉（全部/待处理/处理中/已匹配/已跳过）
- **Articles 页面状态 Badge**: 从二元显示改为 4 种颜色 Badge（黄/蓝/绿/灰），含状态圆点
- **Dashboard 推送状态聚合增强**: 跨 channel 聚合推送状态，新增 `dead`/`failed` 区分展示，hover 显示 `last_error`
- **Dashboard 最新文章状态列**: 适配新 4 态 Badge 展示
- **CSS 新增样式**: `badge-info`（蓝色）、`badge-dead`（红色）、`badge-muted`（灰色）及对应 dot 样式

## Capabilities

### New Capabilities

- `frontend-article-status-display`: 前端文章状态展示 — 4 态 Badge（待处理/处理中/已匹配/已跳过）+ 多值枚举筛选器 + 向后兼容 fallback 逻辑

### Modified Capabilities

- `api-client-layer`: Article/PushRecord TypeScript 接口新增字段；getArticles 新增 `status` 参数；保留 `processed` 参数过渡期
- `dashboard-page`: 推送状态聚合逻辑增强（跨 channel 判断 dead/failed/success）；推送状态 Badge 新增 `dead` 终态 + hover 错误信息；最新文章列表状态列适配 4 态 Badge

## Non-goals

- 不新增或修改后端 API 端点（后端 `query-apis` 变更已由 `push-state-machine` 完成）
- 不移除 `processed` 查询参数或 `processed_at` 字段（向后兼容，后续版本清理）
- 不修改 Parser/Filler/Pusher 前端展示（这些模块无前端界面）
- 不提取共享状态 Badge 为独立 npm 包（项目内共享函数即可）

## Impact

- **前端 API 层**: `web/src/renderer/src/api/queries.ts` — Article/PushRecord 接口定义、getArticles 函数签名
- **前端页面**: `web/src/renderer/src/pages/Articles.tsx` — 筛选器 + Badge；`web/src/renderer/src/pages/Dashboard.tsx` — 推送状态聚合 + 文章状态列
- **前端样式**: `web/src/renderer/src/styles/index.css` — 3 个新 Badge class + 2 个新 dot class
- **共享工具**: `web/src/renderer/src/utils/statusBadge.ts`（新建）— `articleStatusBadge()` 工具函数供 Articles 和 Dashboard 复用
- **向后兼容**: 前端优先使用 `status` 字段，fallback 到 `processed_at`；过渡期同时支持 `status` 和 `processed` 查询参数
