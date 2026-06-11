## 1. Validator crate 集成

- [ ] 1.1 修改 `Cargo.toml` — 添加 `validator = { version = "0.19", features = ["derive"] }`
- [ ] 1.2 修改 `src/models/source.rs` — `CreateSourceRequest` 添加 `#[derive(Validate)]`
- [ ] 1.3 修改 `src/models/keyword.rs` — `CreateKeywordRequest` 添加 `#[derive(Validate)]`
- [ ] 1.4 修改 `src/models/token.rs` — request 结构体添加 `#[derive(Validate)]`
- [ ] 1.5 修改 `src/error.rs` — 添加 `From<validator::ValidationErrors> for AppError`
- [ ] 1.6 修改 `src/handlers/source.rs` — handler 添加 `req.validate()?`，移除手动验证
- [ ] 1.7 修改 `src/handlers/keyword.rs` — 同理
- [ ] 1.8 验证 — `cargo build`

## 2. 日志级别动态配置

- [ ] 2.1 修改 `src/main.rs` — 替换 `.with_env_filter("info")` 为 `EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"))`
- [ ] 2.2 验证 — `cargo build`

## 3. 单元测试

- [ ] 3.1 修改 `src/config.rs` — 底部添加 `#[cfg(test)]` 模块（有效/无效配置测试）
- [ ] 3.2 修改 `src/error.rs` — 底部添加 `#[cfg(test)]` 模块（404/500 状态码测试）
- [ ] 3.3 修改 `src/services/filter.rs` — 提取 `compute_stats` 纯函数，底部添加测试（空/单元素/正常数据）
- [ ] 3.4 验证 — `cargo test`

## 4. 前端 Auth 页面组件替换

- [ ] 4.1 修改 `web/src/renderer/src/pages/Auth.tsx` — antd 组件替换为自定义 div/原生 input/原生 button
- [ ] 4.2 验证 — `cd web && npm run build`

## 5. 前端分页控件

- [ ] 5.1 修改 `web/src/renderer/src/pages/Articles.tsx` — 添加 page/total 状态、分页按钮 UI
- [ ] 5.2 验证 — `cd web && npm run build`

## 6. Parser JoinSet 跟踪

- [ ] 6.1 修改 `src/services/parser.rs` — 使用 `JoinSet` 收集 spawn 的子任务
- [ ] 6.2 修改 `src/services/parser.rs` — cancel 信号后 `join_next().await` 等待所有任务
- [ ] 6.3 验证 — `cargo build`

## 7. DB 连接池 & 路径安全

- [ ] 7.1 修改 `src/db.rs` — `max_connections` 设为 `max_concurrent_fetches + 5`（最少 15）
- [ ] 7.2 修改 `src/main.rs` — DB 路径 `parent().unwrap()` 改为 `ok_or("...")?`
- [ ] 7.3 验证 — `cargo build`

## 8. hours 参数安全 clamp

- [ ] 8.1 修改 `src/handlers/query.rs` — trend handler 中 `hours as i32` 改为 `hours.clamp(1, 8760) as i32`
- [ ] 8.2 验证 — `cargo build`

## 9. 前端安全 & 代码卫生

- [ ] 9.1 修改 `web/src/renderer/src/components/ErrorBoundary.tsx` — UI 显示通用消息，详细错误仅 console.error
- [ ] 9.2 修改 `web/src/renderer/src/pages/Tokens.tsx` — `execCommand('copy')` 替换为 `window.electronAPI.clipboard.writeText()`
- [ ] 9.3 删除 `web/src/renderer/src/hooks/useApi.ts`
- [ ] 9.4 删除 `web/src/renderer/src/components/Loading.tsx`
- [ ] 9.5 修改 `web/src/preload/index.ts` — 移除 `clipboard.readText` 暴露
- [ ] 9.6 修改 `web/src/main/index.ts` — 移除 `ipcMain.handle('clipboard:read', ...)`
- [ ] 9.7 修改 `web/src/renderer/src/lib/notification.ts` — `useNotificationBridge` 添加 cleanup 逻辑
- [ ] 9.8 验证 — `cd web && npm run build`

## 10. 最终验证

- [ ] 10.1 后端编译 + 全部测试 — `cargo build && cargo test`
- [ ] 10.2 前端编译 — `cd web && npm run build`
- [ ] 10.3 验证 validator 错误信息 — 创建无效 source/keyword，确认 400 响应
- [ ] 10.4 验证日志级别 — `RUST_LOG=debug cargo run` 确认 debug 日志输出
- [ ] 10.5 验证分页控件 — 启动前端，确认 Articles 页面有分页 UI
