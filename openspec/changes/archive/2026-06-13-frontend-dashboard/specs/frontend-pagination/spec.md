## MODIFIED Requirements

### Requirement: Articles 列表分页控件

前端 Articles 页面 SHALL 提供分页 UI 控件，使用项目统一的按钮样式（`btn btn-ghost btn-sm`），显示总记录数、当前页/总页数和上一页/下一页按钮。

#### Scenario: 显示分页控件

- **WHEN** Articles 页面加载完成
- **THEN** 页面底部 SHALL 显示分页控件，包含"上一页"和"下一页"按钮
- **THEN** 控件 SHALL 显示总记录数（格式 `共 {total} 条`）、当前页码和总页数（格式 `第 {page}/{totalPages} 页`）

#### Scenario: 点击翻页

- **WHEN** 用户点击"下一页"按钮
- **THEN** 页面 SHALL 重新请求 `GET /api/v1/articles?page=2&per_page=20`（携带当前过滤参数）
- **THEN** 列表 SHALL 更新为第 2 页数据

#### Scenario: 首页时禁用上一页

- **WHEN** 当前为第 1 页
- **THEN** "上一页"按钮 SHALL 为禁用状态

#### Scenario: 末页时禁用下一页

- **WHEN** 当前为最后一页
- **THEN** "下一页"按钮 SHALL 为禁用状态

#### Scenario: 过滤条件变化时重置页码

- **WHEN** 用户变更数据源过滤或处理状态过滤
- **THEN** 页码 SHALL 重置为第 1 页
- **THEN** 系统 SHALL 重新请求第 1 页数据

## ADDED Requirements

### Requirement: Articles 表格列定义

Articles 页面 SHALL 显示以下列表格：`#` 序号列、来源、标题、匹配关键词、发布时间、处理状态。

#### Scenario: 完整列渲染

- **WHEN** Articles 页面加载完成
- **THEN** 表头 SHALL 依次显示：#、来源、标题、匹配关键词、发布时间、处理状态
- **THEN** `#` 列 SHALL 显示文章 ID，使用 mono 字体 11px，颜色 `var(--meta)`
- **THEN** 来源列 SHALL 使用 mono 字体 11px
- **THEN** 标题列 SHALL 为可点击外链，使用 truncate 截断（max-width: 340px），用 `var(--accent)` 颜色
- **THEN** 匹配关键词列 SHALL 使用 mono 字体 11px，颜色 `var(--success)`
- **THEN** 发布时间列 SHALL 使用 mono 字体 11px
- **THEN** 处理状态列 SHALL 显示带圆点指示器的 badge（badge-success 已处理 / badge-warn 待处理）

#### Scenario: 无匹配关键词

- **WHEN** 文章的 `matched_keywords` 字段为 null 或空字符串
- **THEN** 匹配关键词列 SHALL 显示「—」（灰色占位符）

### Requirement: Articles 数据源过滤

Articles 页面 SHALL 提供数据源下拉过滤控件，允许用户按数据源筛选文章。

#### Scenario: 按数据源过滤

- **WHEN** 用户从数据源下拉列表中选择某个数据源
- **THEN** 系统 SHALL 请求 `GET /api/v1/articles?source_id={id}&page=1&per_page=20`
- **THEN** 列表 SHALL 仅显示该数据源的文章
- **THEN** 页码 SHALL 重置为第 1 页

#### Scenario: 清除数据源过滤

- **WHEN** 用户选择"全部数据源"选项
- **THEN** `source_id` 参数 SHALL 从请求中移除
- **THEN** 列表 SHALL 显示所有数据源的文章

#### Scenario: 下拉列表数据来源

- **WHEN** Articles 页面加载
- **THEN** 下拉列表 SHALL 调用 `GET /api/v1/sources` 获取数据源列表
- **THEN** 下拉列表第一项 SHALL 为"全部数据源"（清除过滤）

### Requirement: Articles 处理状态过滤

Articles 页面 SHALL 提供处理状态过滤控件，允许用户按"全部"/"已处理"/"待处理"筛选文章。

#### Scenario: 过滤已处理文章

- **WHEN** 用户选择"已处理"过滤
- **THEN** 系统 SHALL 请求 `GET /api/v1/articles?processed=true&page=1&per_page=20`
- **THEN** 列表 SHALL 仅显示 `processed_at IS NOT NULL` 的文章

#### Scenario: 过滤待处理文章

- **WHEN** 用户选择"待处理"过滤
- **THEN** 系统 SHALL 请求 `GET /api/v1/articles?processed=false&page=1&per_page=20`
- **THEN** 列表 SHALL 仅显示 `processed_at IS NULL` 的文章

#### Scenario: 显示全部（不做处理状态过滤）

- **WHEN** 用户选择"全部"过滤或清除处理状态过滤
- **THEN** `processed` 参数 SHALL 从请求中移除
- **THEN** 列表 SHALL 显示所有文章（不分处理状态）

### Requirement: Articles "运行过滤器"按钮

Articles 页面 SHALL 在面板头部提供"运行过滤器"按钮，允许用户手动触发后端过滤模块。

#### Scenario: 点击运行过滤器

- **WHEN** 用户点击"运行过滤器"按钮
- **THEN** 系统 SHALL 调用 `POST /api/v1/trigger/filter`
- **THEN** Toast 通知 SHALL 显示"过滤器已触发，正在处理..."
- **WHEN** 请求成功后
- **THEN** 文章列表 SHALL 自动刷新当前页数据

#### Scenario: 触发失败时的错误处理

- **WHEN** `POST /api/v1/trigger/filter` 请求失败
- **THEN** axios 拦截器 SHALL 自动显示错误 Toast（无需页面额外处理）
