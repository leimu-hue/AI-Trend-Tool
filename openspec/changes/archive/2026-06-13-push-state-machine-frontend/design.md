## Context

后端 `push-state-machine` 变更引入 Article 四态状态机（`pending/processing/matched/skipped`）和 PushRecord `dead` 终态。后端 API 响应已包含新字段（`status`、`last_error`），但前端仍使用旧接口定义和二元状态判断。前端需要与后端数据模型对齐，同时保持向后兼容以平滑过渡。

## Goals / Non-Goals

**Goals:**
- 前端 TypeScript 接口与后端新字段对齐（`Article.status`、`PushRecord.last_error`）
- Articles 页面支持 4 态筛选和彩色 Badge 展示
- Dashboard 推送状态聚合增强，区分 `dead`/`failed`/`success`/`pending`
- Dashboard 最新文章状态列适配 4 态 Badge
- 向后兼容：`processed_at` 缺失时仍可正常工作

**Non-Goals:**
- 不添加新 API 端点
- 不修改 Electron 主进程
- 不修改前端路由结构
- 不移除 `processed` 查询参数（保留过渡期）

## Decisions

### 1. 状态 Badge 工具函数放置

**选择**: 新建 `web/src/renderer/src/utils/statusBadge.ts` 作为独立工具文件。

**备选**: 内联在 Articles.tsx 并由 Dashboard.tsx 导入。

**理由**: 工具函数被 Articles 和 Dashboard 两个页面使用，独立文件避免循环依赖和重复代码。与现有 `utils/` 目录结构一致（当前有 `theme.ts` 等工具文件）。

### 2. 向后兼容策略

**选择**: 前端优先读取 `status` 字段，缺失时 fallback 到 `processed_at`。

```
status === 'matched' → "已匹配"
status 缺失 + processed_at !== null → "已处理"（旧数据）
status 缺失 + processed_at === null → "待处理"（旧数据）
```

**理由**: 部署时后端先升级，前端稍后更新。中间窗口期旧前端可能被打开，但本次变更只考虑新前端+后端双字段并存的场景。`status` 字段全部后端数据都有值（migration 填充），前端优先使用即可。

### 3. 筛选器参数发送策略

**选择**: Articles 页面同时支持 `status` 和 `processed` 参数，优先使用 `status`。当用户选择旧选项（待处理/已处理）时，仍使用新参数映射。

| 用户选择 | 发送参数 |
|---------|---------|
| 全部状态 | 无 status 参数 |
| 待处理 | `status=pending` |
| 处理中 | `status=processing` |
| 已匹配 | `status=matched` |
| 已跳过 | `status=skipped` |

**备选**: 保留旧 `processed` 参数供旧选项使用。

**理由**: 后端 `query-apis` 已新增 `status` 参数，同时保留 `processed` 参数过渡期。直接使用 `status` 参数即可，无需同时发送两个参数。

### 4. Dashboard 推送状态聚合

**选择**: 跨 channel 聚合逻辑：
- 全部 channel 均为 `success` → 显示「已推送」(green)
- 存在 `dead` 记录 → 显示「已放弃」(red)，hover 显示所有 `last_error`
- 存在 `failed` 但无 `dead` → 显示「推送失败」(yellow)，hover 显示错误
- 全部 `pending` → 显示「待推送」(blue)

**理由**: 一个热点事件可能对应多个推送 channel。聚合展示优先暴露最严重状态，引导用户关注需人工介入的记录。

### 5. CSS 样式扩展

**选择**: 在现有 `styles/index.css` 末尾追加新样式，不引入新 CSS 文件。

**理由**: 变更量小（3 个 badge class + 2 个 dot class），单独文件增加维护负担。与现有 `badge-success`/`badge-warn` 放置在同一文件便于查找。

## Risks / Trade-offs

**[风险] 后端 `status` 字段可能为空（中间窗口期旧数据）** → 缓解: `articleStatusBadge` 实现 fallback 逻辑，先检查 `status` 是否存在，缺失时退回到 `processed_at` 判断。

**[风险] `pushStatusMap` 类型从 `Record<number, string>` 改为 `Record<number, { status: string; errors: string[] }>` 可能影响其他引用** → 缓解: `pushStatusMap` 仅在 Dashboard.tsx 中使用，作用域明确。

**[权衡] `articleStatusBadge` 作为独立工具文件 vs React hook** → 选择纯函数而非 hook：该函数无状态/无副作用，纯函数形式更轻量，测试成本更低。
