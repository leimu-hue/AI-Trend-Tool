# TrendAITool — AI 热点监控系统

基于 Rust + Electron 构建的智能热点监控平台。自动采集 RSS 内容，通过关键词匹配（Aho-Corasick）和统计突发检测（滑动平均 + 标准差）识别趋势热点，经由钉钉/飞书 Webhook 推送告警。

## 目录

- [快速开始](#快速开始)
- [配置说明](#配置说明)
- [架构概览](#架构概览)
- [API 接口](#api-接口)
- [热点检测算法](#热点检测算法)
- [推送重试机制](#推送重试机制)
- [项目结构](#项目结构)
- [开发计划](#开发计划)

---

## 快速开始

### 前置要求

- Rust 工具链 1.75+
- Node.js 18+ & npm
- SQLite 3

### 后端（Rust）

```bash
# 构建
cargo build --release

# 编辑配置（按需修改）
vim config.toml

# 启动（所有模块在单一进程中运行：API + Parser + Filter + Pusher）
# 配置文件路径为第一个位置参数，默认为 "config.toml"
cargo run -- config.toml
```

首次启动时自动执行数据库迁移（`docs/migrations/`），并在 `api_tokens` 表为空时创建初始管理员 Token（日志输出）。

### 前端（Electron + React）

```bash
cd web

# 安装依赖（已配置 .npmrc 使用国内镜像）
npm install

# 若 Electron 二进制下载失败，手动重新安装：
# 设置环境变量后重试
$env:ELECTRON_MIRROR="https://npmmirror.com/mirrors/electron/"
node node_modules/electron/install.js

# 若下载成功但未自动解压，手动解压缓存：
node -e "const e=require('extract-zip'),f=require('fs'),p=require('path');const z=p.join(process.env.LOCALAPPDATA,'electron/Cache');const d=p.join(__dirname,'node_modules/electron/dist');f.readdirSync(z).forEach(h=>e(p.join(z,h,f.readdirSync(p.join(z,h))[0]),{dir:d}).then(()=>{f.writeFileSync(p.join(__dirname,'node_modules/electron/path.txt'),'electron.exe');console.log('Done')}))"

# 启动开发服务器
npm run dev

# 构建生产版本
npm run build
```

> **注意**：`.npmrc` 中的 `electron_mirror` 在 npm 12+ 可能会产生警告，届时可通过环境变量 `ELECTRON_MIRROR` 设置。

---

## 配置说明

配置文件为 TOML 格式（`config.toml`）：

```toml
[server]
host = "0.0.0.0"          # 监听地址
port = 3000               # 监听端口

[database]
path = "./docs/data/hotspot.db"   # SQLite 数据库路径

[auth]
initial_token = "your-token"      # 可选，初始管理员 Token（仅首次启动时生效）

[parser]
max_concurrent_fetches = 10
default_user_agent = "HotspotMonitor/1.0"
default_timeout_seconds = 30
interval_seconds = 30             # 轮询到期数据源的间隔（秒）

[filter]
batch_size = 1000                 # 批量处理文章数
interval_seconds = 300            # 过滤间隔（秒）
history_hours = 24                # 历史窗口（小时）
min_history_hours = 6             # 最少历史数据（小时）

[pusher]
interval_seconds = 10             # 推送轮询间隔（秒）
max_retries = 3
retry_base_seconds = 60           # 退避基础秒数
```

---

## 架构概览

采用**事件驱动管道模式（Event-Driven Pipeline）**，三个后台模块通过 `tokio::mpsc` 事件通道连接，同时保留定时器作为回退触发：

```
┌──────────┐  articles_ready  ┌──────────┐  push_ready  ┌──────────┐
│  Parser  │ ──────────────▶  │  Filter  │ ──────────▶  │  Pusher  │
│ RSS 采集 │   (mpsc event)   │ 关键词 + │  (mpsc event)│ Webhook  │
│          │                  │ 热点检测 │               │  推送    │
└──────────┘                  └──────────┘               └──────────┘
   ▲ 定时器                      ▲ 定时器                   ▲ 定时器
   (30s)                        (300s)                     (10s)
```

| 模块 | 职责 | 触发方式 |
|------|------|----------|
| **Parser** | 拉取 RSS/Atom Feed，去重写入 `articles` 表 | 定时器（`interval_seconds`，默认 30s） |
| **Filter** | Aho-Corasick 关键词匹配 + 统计突发检测，生成热点事件 | 定时器（默认 300s）**或** Parser 事件 |
| **Pusher** | 轮询待推送记录，POST Webhook，线性退避重试 | 定时器（默认 10s）**或** Filter 事件 |

所有模块在单一进程中运行，通过 `CancellationToken` 实现优雅关闭（Ctrl+C）。
另提供手动触发端点：`POST /api/v1/trigger/filter` 和 `POST /api/v1/trigger/pusher`。

### 技术栈

| 层 | 技术 |
|----|------|
| 语言 | Rust (Edition 2021) |
| Web 框架 | Axum 0.8 + Tower |
| 数据库 | SQLite (sqlx 0.7, WAL 模式) |
| RSS 解析 | feed-rs |
| 关键词匹配 | Aho-Corasick |
| HTTP 客户端 | reqwest 0.12 |
| 前端框架 | React 19 + Electron 33 + Vite |
| UI 组件 | Ant Design 5 |
| 样式 | Tailwind CSS v4 |
| 图表 | ECharts 5 |

---

## API 接口

### 认证

除 `/health` 外，所有 `/api/v1/*` 路由需要 Bearer Token 认证：

```http
Authorization: Bearer <your-token>
```

### 端点一览

| 方法 | 路径 | 说明 |
|------|------|------|
| `GET` | `/health` | 健康检查 |
| **Token 管理** | | |
| `POST` | `/api/v1/tokens` | 创建 Token（返回明文，仅此一次） |
| `GET` | `/api/v1/tokens` | 列出所有 Token |
| `POST` | `/api/v1/tokens/revoke/{id}` | 撤销 Token |
| **数据源** | | |
| `GET` | `/api/v1/sources` | 列出数据源 |
| `POST` | `/api/v1/sources` | 创建数据源 |
| `POST` | `/api/v1/sources/{id}/update` | 更新数据源 |
| `POST` | `/api/v1/sources/{id}/delete` | 删除数据源 |
| `POST` | `/api/v1/sources/{id}/fetch` | 手动触发抓取 |
| **关键词** | | |
| `GET` | `/api/v1/keywords` | 列出关键词 |
| `POST` | `/api/v1/keywords` | 创建关键词 |
| `POST` | `/api/v1/keywords/{id}/update` | 更新关键词 |
| `POST` | `/api/v1/keywords/{id}/delete` | 删除关键词 |
| **推送渠道** | | |
| `GET` | `/api/v1/channels` | 列出推送渠道 |
| `POST` | `/api/v1/channels` | 创建推送渠道 |
| `POST` | `/api/v1/channels/{id}/update` | 更新推送渠道 |
| `POST` | `/api/v1/channels/{id}/delete` | 删除推送渠道 |
| **查询** | | |
| `GET` | `/api/v1/articles` | 文章列表（分页 + 过滤） |
| `GET` | `/api/v1/hotspots` | 热点事件列表（分页 + 关键词过滤） |
| `GET` | `/api/v1/hotspots/{id}/push-records` | 热点的推送记录 |
| `GET` | `/api/v1/trend/{keyword_id}` | 关键词小时趋势数据 |
| **手动触发** | | |
| `POST` | `/api/v1/trigger/filter` | 手动执行一次 Filter |
| `POST` | `/api/v1/trigger/pusher` | 手动执行一次 Pusher |

### 响应格式

```json
// 成功 (200 / 201)
{ "data": <value> }

// 错误
{ "error": { "code": "UNAUTHORIZED", "message": "..." } }
```

| 状态码 | 错误码 | 说明 |
|--------|--------|------|
| 400 | `BAD_REQUEST` | 请求参数无效 |
| 401 | `UNAUTHORIZED` | 未认证或 Token 无效/过期 |
| 404 | `NOT_FOUND` | 资源不存在 |
| 409 | `CONFLICT` | 资源冲突 |
| 500 | `INTERNAL_ERROR` / `DATABASE_ERROR` | 内部/数据库错误 |

---

## 热点检测算法

Filter 模块使用统计突发检测：

1. **关键词匹配** — Aho-Corasick 多模式匹配（区分大小写 / 不区分大小写双自动机），扫描未处理文章
2. **命中记录** — 匹配结果写入 `keyword_mentions` 表
3. **小时桶计数** — 按关键词 + 小时窗口（`YYYYMMDDHH`）聚合文章数
4. **突发判定** — 滑动窗口（默认 24h）计算均值和标准差，当前计数超过 `mean + std_multiplier × stddev` 且达到 `min_hot_count` 阈值时触发
5. **去重** — 同一关键词同一小时内通过 `ON CONFLICT(keyword_id, hour_bucket) DO UPDATE` upsert，保证只有一条热点事件

## 推送重试机制

- 最大重试 3 次
- 退避公式：`next_retry_at = now + retry_count × retry_base_seconds`（线性退避）
- 乐观锁（`WHERE status = ? AND retry_count < ?`）防止并发重复推送

---

## 项目结构

```
TrendAITool/
├── config.toml              # 后端配置
├── Cargo.toml               # Rust 依赖
├── src/                     # 后端源码
│   ├── main.rs              #   入口、Token 引导、模块编排
│   ├── config.rs            #   配置解析
│   ├── error.rs             #   统一错误处理
│   ├── routes.rs            #   路由注册 + AppState
│   ├── pipeline.rs          #   事件驱动管道（mpsc 通道 + CancellationToken）
│   ├── db.rs                #   连接池初始化
│   ├── models/              #   数据模型
│   ├── db/                  #   数据库操作（所有 SQL 查询集中于此）
│   ├── handlers/            #   请求处理器
│   ├── middleware/          #   认证中间件
│   └── services/            #   Parser / Filter / Pusher
├── web/                     # 前端（Electron + React）
│   ├── .npmrc               #   npm 镜像配置
│   ├── package.json         #   前端依赖
│   └── src/
│       ├── main/            #   Electron 主进程
│       ├── preload/         #   预加载脚本
│       └── renderer/        #   React 渲染进程
├── docs/
│   ├── migrations/          #   数据库迁移 SQL
│   ├── apis/                #   API 设计文档
│   └── data/                #   SQLite 数据文件
└── openspec/                #   规格文档
```

## 数据库表结构

| 表 | 说明 |
|----|------|
| `api_tokens` | Bearer Token（可撤销、可选过期） |
| `data_sources` | RSS 数据源配置（URL、间隔、JSON 扩展配置） |
| `articles` | 采集文章（`link` 去重，`processed_at` 跟踪过滤状态） |
| `keywords` | 关键词及敏感度参数（`std_multiplier`、`min_hot_count`、`case_sensitive`） |
| `keyword_mentions` | 关键词命中明细（keyword_id + article_id） |
| `hot_events` | 热点事件（小时桶统计，`keyword_id + hour_bucket` 唯一约束支持 upsert） |
| `push_channels` | 推送渠道（Webhook URL） |
| `push_records` | 推送状态与重试追踪（`hot_event_id + channel_id` 唯一约束） |

---

## 开发计划

- [x] 后端项目脚手架（Axum + SQLite + 配置解析）
- [x] 数据库迁移与模型定义
- [x] 认证中间件与 Token 管理 API
- [x] CRUD API（数据源、关键词、推送渠道）
- [x] 查询 API（文章列表、热点列表、趋势数据、推送记录）
- [x] Parser / Filter / Pusher 后台模块
- [x] 事件驱动管道（mpsc 事件通道 + 定时器回退 + 手动触发端点）
- [x] 前端管理界面（Electron + React + Ant Design）
- [x] Dashboard 可视化（ECharts 趋势图表）

## License

待定
