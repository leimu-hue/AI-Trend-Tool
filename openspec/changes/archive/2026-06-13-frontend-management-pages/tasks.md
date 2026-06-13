## 1. CSS 基础设施

- [x] 1.1 在 `web/src/renderer/src/styles/index.css` 追加自定义 CSS 组件类：`.panel` / `.panel-header` / `.panel-title`、`.table-wrap` / `table`、`.btn` / `.btn-primary` / `.btn-ghost` / `.btn-sm` / `.btn-danger`、`.badge` / `.badge-success` / `.badge-danger` / `.badge-neutral`、`.modal-overlay` / `.modal` / `.modal-actions`、`.field` / `.field-help`、`.settings-grid` / `.settings-group` / `.setting-row` / `.setting-label` / `.setting-value`

## 2. Toast 通知组件

- [x] 2.1 创建 `web/src/renderer/src/components/Toast.tsx`：实现 `ToastProvider`（context + state 管理）和 `useToast()` hook（返回 `{ success, error, info }` 方法），通知渲染为底部居中堆叠气泡，auto-dismiss 3 秒
- [x] 2.2 在 `web/src/renderer/src/main.tsx` 中注册 `<ToastProvider>` 包裹路由器
- [x] 2.3 验证：浏览器 console 调用 toast 方法确认气泡显示和自动消失

## 3. 系统设置页

- [x] 3.1 重写 `web/src/renderer/src/pages/Settings.tsx`：移除 antd Card，使用 `.settings-grid` + `.settings-group` 自定义布局，4 组配置卡片，`GET /settings` 获取数据，失败回退硬编码默认值并显示"默认配置"提示
- [x] 3.2 验证：`npm run dev` 访问 `/settings`，确认 4 组卡片正确展示，暗色主题匹配

## 4. Token 设置页

- [x] 4.1 重写 `web/src/renderer/src/pages/Tokens.tsx`：移除 antd Card，实现表格（名称、最后使用、过期时间、状态 badge、操作按钮）、生成 Modal（名称 + 过期时间字段）、成功展示界面（明文 + 复制按钮 + 安全警告）、吊销确认 + `useToast` 反馈
- [x] 4.2 验证：访问 `/tokens`，测试生成 Token→复制→吊销完整流程，确认已吊销 Token 显示「—」

## 5. 关键词管理页

- [x] 5.1 重写 `web/src/renderer/src/pages/Keywords.tsx`：移除 antd Card，实现表格（关键词 mono、大小写、标准差倍数、最小计数、24h 命中、状态 badge、操作按钮）、添加/编辑 Modal（关键词 + 大小写敏感 + 标准差倍数 + 最小计数字段）、行内启用/暂停切换、删除确认
- [x] 5.2 验证：访问 `/keywords`，测试添加→编辑→暂停→启用→删除完整流程

## 6. 推送渠道管理页

- [x] 6.1 重写 `web/src/renderer/src/pages/Channels.tsx`：移除 antd Card，实现表格（名称、类型 badge、Webhook URL 脱敏显示、推送次数、上次推送、状态 badge、操作按钮）、添加/编辑 Modal（名称 + 类型 + Webhook URL 字段，config JSON 组装）、测试按钮、删除确认
- [x] 6.2 验证：访问 `/channels`，测试添加 Webhook 渠道→编辑→测试→删除流程，确认 URL 脱敏正确

## 7. 数据源管理页

- [x] 7.1 重写 `web/src/renderer/src/pages/Sources.tsx`：移除 antd Card，实现表格（名称、类型 badge、URL 截断 mono、间隔、文章数、上次抓取、状态 badge、操作按钮）、添加/编辑 Modal（名称 + 类型 + URL + 间隔字段）、手动抓取按钮、行内删除按钮 + Confirm 确认弹窗
- [x] 7.2 验证：访问 `/sources`，测试添加 RSS 数据源→手动抓取→编辑→删除完整流程

## 8. 最终验证

- [x] 8.1 运行 `cd web && npm run build` 确认 TypeScript 编译和 Vite 构建无错误
- [x] 8.2 运行 `cd web && npm run dev`，依次访问 5 个页面确认：暗色主题一致、所有 CRUD 操作正常、Toast 反馈正确、无 antd 组件残留
