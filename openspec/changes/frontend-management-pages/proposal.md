## Why

前端已有脚手架、路由、API 封装和全局样式，但管理后台缺少实际页面。用户无法在 UI 中管理数据源、关键词、推送渠道和 Token，必须直接操作数据库或通过 API 调试工具，严重影响可用性。需补齐 5 个管理页面以交付完整的运营后台。

## What Changes

- 新增**数据源管理页**：CRUD 表格 + Modal 表单，支持手动触发抓取
- 新增**关键词管理页**：CRUD 表格 + Modal 表单，支持启用/暂停状态切换
- 新增**推送渠道管理页**：CRUD 表格 + Modal 表单，支持测试 Webhook 连接
- 新增**Token 设置页**：生成/复制/吊销 Bearer Token，创建成功时展示一次性明文
- 新增**系统设置页**：只读展示解析器、过滤器、推送器、服务器 4 组配置参数

所有页面使用自定义 CSS 组件（`.panel`、`.btn`、`.badge`、`.modal` 等），匹配原型的暗色主题风格，不依赖组件库。

## Non-goals

- 不涉及后端 API 变更（所有 CRUD API 已在 `source-crud-api`、`keyword-crud-api`、`channel-crud-api`、`token-api` 等 spec 中完成）
- 不修改现有路由结构、布局组件或全局样式
- 不支持数据导出、批量操作、拖拽排序等高级功能
- 不添加实时数据刷新（首次加载即可，手动操作后刷新列表）

## Capabilities

### New Capabilities

- `sources-management-page`: 数据源 CRUD 管理页面，表格展示所有数据源及操作按钮，Modal 表单支持添加/编辑
- `keywords-management-page`: 关键词 CRUD 管理页面，支持大小写敏感、标准差倍数、最小计数等参数配置，行内启用/暂停切换
- `channels-management-page`: 推送渠道 CRUD 管理页面，Webhook URL 脱敏显示，支持测试推送
- `tokens-management-page`: API Token 管理页面，支持生成（一次性明文展示）、复制、吊销操作
- `settings-page`: 系统设置只读展示页面，分 4 组卡片展示解析器/过滤器/推送器/服务器配置

### Modified Capabilities

- `design-token-system`: 移除禁止自定义 CSS 组件类（`.btn`、`.panel`、`.badge`、`.modal` 等）的限制——管理页面使用自定义 CSS 而非 antd 组件
- `shared-components`: 新增基于自定义 CSS 的 Toast 通知 hook（`useToast`），替代 antd 的 notification 上下文，供管理页面使用

## Impact

- 新增 5 个页面文件：`src/pages/Sources.tsx`、`Keywords.tsx`、`Channels.tsx`、`Tokens.tsx`、`Settings.tsx`
- 依赖现有 `src/api/client.ts`（axios 实例）、`src/components/Toast.tsx`（useToast hook）、`src/components/Empty.tsx`（空状态组件）
- 依赖全局样式 `src/styles/global.css` 中已定义的自定义 CSS 组件类名
- 无后端代码变更
