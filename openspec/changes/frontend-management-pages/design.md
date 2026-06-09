## Context

5 个管理页面的文件已存在（`Sources.tsx`、`Keywords.tsx`、`Channels.tsx`、`Tokens.tsx`、`Settings.tsx`），但均为 Ant Design `Card` 占位符。API 客户端（`client.ts`）和类型化 API 模块（`sources.ts`、`keywords.ts`、`channels.ts`、`tokens.ts`）已完成。

设计要求：用自定义 CSS 组件（`.panel`、`.btn`、`.badge`、`.modal` 等）替换所有 Ant Design 组件，匹配 `docs/Live-Artifact/index.html` 原型的暗色主题风格。

当前 `styles/index.css` 仅有设计 Token（颜色、字体、圆角、滚动条），缺少组件级 CSS 类。Toast 通知系统也未实现。

## Goals / Non-Goals

**Goals:**
- 将 5 个页面从 Ant Design 占位符替换为完整的自定义 CSS 管理界面
- 定义组件级 CSS 类（`.panel`、`.btn`、`.badge`、`.modal`、`.table-wrap`、`.field`、`.settings-grid` 等）
- 实现 `useToast` hook 用于操作反馈（创建/删除/错误通知）
- 所有 CRUD 操作对接已有 API 模块
- Settings 页支持后端 API 缺失时的静默回退（使用默认值）

**Non-Goals:**
- 不修改后端 API 或数据模型
- 不新增路由或布局变更
- 不实现数据导出、批量操作、拖拽排序
- 不替换 Empty/Loading/ErrorBoundary 等已有共享组件（它们可在后续独立清理）
- Settings 页为只读，不实现配置编辑

## Decisions

### 1. 自定义 CSS 组件类 → 统一在 `styles/index.css` 追加

**理由**：设计 Token 已在此文件定义，组件类与 Token 紧密耦合（引用 `var(--fg)`、`var(--surface)` 等）。单独文件会增加维护成本和加载顺序问题。

**考虑的替代方案**：CSS Modules 或 CSS-in-JS — 但原型使用全局类名（`.panel`、`.btn`），保持一致性更优先。

### 2. Toast 通知 → 自定义 context-based hook，不依赖 antd

实现 `components/Toast.tsx`：`ToastProvider` 包裹应用根节点，`useToast()` hook 返回 `{ show, success, error }` 方法。通知渲染为固定定位的底部居中气泡，自动 3 秒消失。

**理由**：设计文档要求 "不使用 Ant Design"。当前 `lib/notification.ts` 依赖 antd 静态方法，需替换。Context 方式可被页面级 hook 自然使用。

### 3. Modal 弹窗 → 自定义 CSS overlay 模式，每个页面内联实现

使用 `.modal-overlay` + `.modal` CSS 类，点击 overlay 关闭。表单状态用 `useState` 管理，提交时调 API。

**理由**：5 个页面的 Modal 表单字段各不相同，抽象泛型 Modal 组件会过度设计。每个页面维护自己的 Modal 状态更简单直观。

### 4. 不再使用 Ant Design 组件

页面中移除所有 `import { Card } from 'antd'` 及其他 antd 导入，改用自定义 CSS 类。

**例外**：Empty、Loading、ErrorBoundary 等共享组件暂保持 antd 依赖，不在本次变更范围。

### 5. Settings 页数据源 → 优先调 `/settings` API，失败回退默认值

后端可能未实现 `/api/v1/settings` 端点。前端先尝试 `GET /settings`，失败时使用硬编码默认值（与 `config.toml` 默认值一致）。

## Risks / Trade-offs

- **[无 Toast 动画]**：自定义 Toast 使用 CSS transition，不如 antd 的丰富动画 → 可接受，功能优先
- **[Modal 无焦点陷阱/键盘导航]**：自定义 Modal 不实现无障碍焦点管理 → 桌面管理工具，影响有限，后续可增量改进
- **[Settings 页默认值不同步]**：后端 `/settings` 不可用时回退硬编码默认值，可能与实际运行配置不一致 → 标注 "默认配置" 提示
- **[Table 无排序/筛选]**：自定义 table 无内置排序 → 管理数据量小（通常 <100 行），无需服务端分页

## Open Questions

1. Settings 页的 `/api/v1/settings` 端点是否已实现？（若未实现，使用硬编码回退）
2. Channels 页的「测试」按钮：后端是否有 `POST /channels/test/{id}` 端点？（API 模块已预留 `channelApi.test()`，需确认后端状态）
3. Sources 页的「抓取」按钮：后端 `POST /sources/fetch/{id}` 是否已实现？（API 模块已预留 `sourceApi.fetch()`）
