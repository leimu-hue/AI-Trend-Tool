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

# 运行全部模块
cargo run -- --config config.toml all

# 或单独运行某个模块
cargo run -- --config config.toml api       # 仅 API 服务
cargo run -- --config config.toml parser    # 仅 RSS 采集
cargo run -- --config config.toml filter    # 仅关键词过滤 + 热点检测
cargo run -- --config config.toml pusher    # 仅 Webhook 推送
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
port = 8080               # 监听端口

[database]
path = "./docs/data/hotspot.db"   # SQLite 数据库路径

[auth]
initial_token = "your-token"      # 可选，初始管理员 Token

[parser]
max_concurrent_fetches = 10
default_user_agent = "HotspotMonitor/1.0"
default_timeout_seconds = 30

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

采用**管道模式（Pipeline）**，三个后台模块独立运行：

```
┌──────────┐     ┌──────────┐     ┌──────────┐
│  Parser  │ ──▶ │  Filter  │ ──▶ │  Pusher  │
│ RSS 采集 │     │ 关键词 + │     │ Webhook  │
│          │     │ 热点检测 │     │  推送    │
└──────────┘     └──────────┘     └──────────┘
```

| 模块 | 职责 | 运行周期 |
|------|------|----------|
| **Parser** | 拉取 RSS Feed，去重写入 `articles` 表 | 按数据源各自间隔 |
| **Filter** | Aho-Corasick 关键词匹配 + 统计突发检测，生成热点事件 | 每 5 分钟 |
| **Pusher** | 轮询待推送记录，POST Webhook，指数退避重试 | 每 10 秒 |

### 技术栈

| 层 | 技术 |
|----|------|
| 语言 | Rust (Edition 2021) |
| Web 框架 | Axum 0.8 + Tower |
| 数据库 | SQLite (sqlx 0.7, WAL 模式) |
| RSS 解析 | feed-rs |
| 关键词匹配 | Aho-Corasick |
| HTTP 客户端 | reqwest 0.12 |
| 前端 | React 19 + Electron 33 + Vite + Ant Design |

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
| `POST` | `/api/v1/tokens` | 创建 Token（返回明文，仅此一次） |
| `GET` | `/api/v1/tokens` | 列出所有 Token |
| `POST` | `/api/v1/tokens/revoke/{id}` | 撤销 Token |

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

1. **关键词匹配** — Aho-Corasick 多模式匹配，扫描未处理文章
2. **小时桶计数** — 按关键词 + 小时窗口聚合文章数
3. **突发判定** — 滑动窗口（默认 24h）计算均值和标准差，当前计数超过 `mean + std_multiplier × stddev` 且达到 `min_hot_count` 阈值时触发
4. **去重** — 同一关键词同一小时内只生成一条热点事件

## 推送重试机制

- 最大重试 3 次
- 退避公式：`retry_after = now + retry_base_seconds × 2^retry_count`
- 乐观锁（`WHERE status = ? AND retry_count < ?`）防止并发重复推送

---

## 项目结构

```
TrendAITool/
├── config.toml              # 后端配置
├── Cargo.toml               # Rust 依赖
├── src/                     # 后端源码
│   ├── main.rs              #   入口、CLI、Token 引导
│   ├── config.rs            #   配置解析
│   ├── error.rs             #   统一错误处理
│   ├── routes.rs            #   路由注册
│   ├── db.rs                #   连接池初始化
│   ├── models/              #   数据模型
│   ├── db/                  #   数据库操作
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
| `data_sources` | RSS 数据源配置 |
| `articles` | 采集文章（`link` 去重） |
| `keywords` | 关键词及敏感度参数 |
| `hot_events` | 热点事件（小时桶统计） |
| `push_channels` | 推送渠道（Webhook URL） |
| `push_records` | 推送状态与重试追踪 |

---

## 开发计划

- [x] 后端项目脚手架（Axum + SQLite + 配置解析）
- [x] 数据库迁移与模型定义
- [x] 认证中间件与 Token 管理 API
- [ ] CRUD API（数据源、关键词、推送渠道）
- [ ] Parser / Filter / Pusher 后台模块
- [ ] 前端管理界面
- [ ] Dashboard 可视化

## License

待定
