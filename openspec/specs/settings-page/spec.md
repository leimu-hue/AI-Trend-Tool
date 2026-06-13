# Settings Page

## Purpose

系统设置页 — 只读展示后端配置参数，分 4 组卡片（解析器、过滤器、推送器、服务器），使用自定义 CSS 组件匹配原型暗色主题。

## Requirements

### Requirement: Settings grouped card display
系统 SHALL 以分组卡片形式展示系统配置参数，分为解析器配置、过滤器配置、推送器配置和服务器配置 4 组。

#### Scenario: Four setting groups rendered
- **WHEN** 页面加载
- **THEN** 使用 `.settings-grid` 2 列网格布局显示 4 个 `.settings-group` 卡片，每组有标题和键值对列表

#### Scenario: Parser config group
- **WHEN** 页面渲染解析器配置组
- **THEN** 显示：最大并发抓取数、默认超时时间（秒）、默认抓取间隔（秒）

#### Scenario: Filter config group
- **WHEN** 页面渲染过滤器配置组
- **THEN** 显示：批处理大小、运行间隔（秒）、历史窗口（小时）、最小历史数据（小时）

#### Scenario: Pusher config group
- **WHEN** 页面渲染推送器配置组
- **THEN** 显示：轮询间隔（秒）、最大重试次数、重试基础间隔（秒）

#### Scenario: Server config group
- **WHEN** 页面渲染服务器配置组
- **THEN** 显示：监听地址、端口

### Requirement: Settings data source with fallback
系统 SHALL 优先从 `GET /api/v1/settings` 获取配置，失败时使用硬编码默认值。

#### Scenario: API returns settings successfully
- **WHEN** 后端实现 `/api/v1/settings` 端点并返回配置 JSON
- **THEN** 页面展示后端实际配置值

#### Scenario: API unavailable — fallback defaults
- **WHEN** `GET /api/v1/settings` 请求失败（404 或网络错误）
- **THEN** 页面静默回退使用默认配置值，配置卡片标题下方显示"默认配置"muted 提示

#### Scenario: Loading state
- **WHEN** 数据正在请求中
- **THEN** 显示"加载中..."文字

### Requirement: Custom CSS styling without Ant Design
页面 SHALL 使用自定义 CSS 类（`.settings-grid`、`.settings-group`、`.setting-row`、`.setting-label`、`.setting-value`），不导入 antd 组件。

#### Scenario: Settings grid responsive layout
- **WHEN** 视口宽度 > 768px
- **THEN** 使用 2 列网格布局

#### Scenario: Settings grid single column on narrow viewport
- **WHEN** 视口宽度 ≤ 768px
- **THEN** 切换为 1 列布局
