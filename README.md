# TrendAITool — AI 热点监控系统

基于 Rust 构建的智能热点监控平台。自动采集 RSS 内容，通过关键词匹配（Aho-Corasick 算法）和统计突发检测（滑动平均 + 标准差）识别趋势热点，经由钉钉/飞书 Webhook 推送告警。

## 架构概览

采用**管道模式（Pipeline）**，三个后台模块独立运行：

```
┌──────────┐     ┌──────────┐     ┌──────────┐
│  Parser  │ ──▶ │  Filter  │ ──▶ │  Pusher  │
│ RSS 采集 │     │ 关键词+  │     │ Webhook  │
│          │     │ 热点检测  │     │  推送    │
└──────────┘     └──────────┘     └──────────┘
```

| 模块 | 职责 | 运行周期 |
|------|------|----------|
| **Parser** | 按配置周期拉取 RSS Feed，去重写入 `articles` 表 | 按数据源各自间隔 |
| **Filter** | 每 5 分钟运行。Aho-Corasick 关键词匹配，小时级桶计数，统计突发检测，生成 `hot_events` 和待推送记录 | 每 5 分钟 |
| **Pusher** | 每 10 秒轮询 `push_records`（status=pending），POST Webhook，指数退避重试（最多 3 次），乐观锁防重复 | 每 10 秒 |

所有模块可单独或组合运行：`hotspot all|api|parser|filter|pusher`

## 技术栈

- **语言**: Rust (Edition 2021)
- **Web 框架**: [Axum 0.8](https://github.com/tokio-rs/axum) + Tower
- **数据库**: SQLite (sqlx 0.7)，WAL 模式 + 外键约束
- **RSS 解析**: [feed-rs](https://github.com/feed-rs/feed-rs)
- **关键词匹配**: [Aho-Corasick](https://github.com/BurntSushi/aho-corasick)
- **HTTP 客户端**: reqwest 0.12（Webhook 推送）
- **序列化**: serde / serde_json / toml
- **时间**: chrono 0.4
- **日志**: tracing
- **CLI**: clap 4

## 快速开始

### 前置要求

- Rust 工具链（1.75+）
- SQLite 3

### 构建与运行

```bash
# 克隆项目
git clone <repo-url>
cd TrendAITool

# 构建
cargo build --release

# 编辑配置（按需修改）
vim config.toml

# 运行全部模块
cargo run -- --config config.toml all

# 仅运行 API 服务
cargo run -- --config config.toml api

# 仅运行采集模块
cargo run -- parser

# 仅运行过滤模块
cargo run -- filter

# 仅运行推送模块
cargo run -- pusher
```

### 数据库初始化

首次启动时自动执行迁移，创建所有表结构。迁移文件位于 `docs/migrations/` 目录。

### 初始 Token

首次启动时，若 `api_tokens` 表为空，系统自动创建初始管理员 Token：

- **配置了** `auth.initial_token` — 直接使用配置值
- **未配置** — 自动生成 64 位随机 hex 字符串，通过日志输出（`tracing::warn!`）

```
============================================
  INITIAL TOKEN (save this!): a1b2c3d4...
============================================
```

## 配置说明

配置文件为 TOML 格式，默认路径 `config.toml`：

```toml
[server]
host = "0.0.0.0"        # 监听地址
port = 8080             # 监听端口

[database]
path = "./docs/data/hotspot.db"   # SQLite 数据库路径

[auth]
initial_token = "your-preconfigured-token"   # 可选，初始管理员 Token

[parser]
max_concurrent_fetches = 10       # 最大并发采集数
default_user_agent = "HotspotMonitor/1.0"
default_timeout_seconds = 30      # RSS 拉取超时

[filter]
batch_size = 1000                 # 批量处理文章数
interval_seconds = 300            # 过滤间隔（秒）
history_hours = 24                # 历史窗口（小时）
min_history_hours = 6             # 最少历史数据（小时）

[pusher]
interval_seconds = 10             # 推送轮询间隔（秒）
max_retries = 3                   # 最大重试次数
retry_base_seconds = 60           # 退避基础秒数
```

## API 接口

### 认证

除 `/health` 外，所有 `/api/v1/*` 路由需要 Bearer Token 认证：

```http
Authorization: Bearer <your-token>
```

认证中间件验证流程：提取 Token → 数据库校验（非撤销）→ 过期检查 → 后台更新 `last_used_at` → 注入请求上下文。

### Token 管理

| 方法 | 路径 | 说明 |
|------|------|------|
| `POST` | `/api/v1/tokens` | 创建 Token（返回明文，仅此一次） |
| `GET` | `/api/v1/tokens` | 列出所有 Token（不返回明文） |
| `POST` | `/api/v1/tokens/revoke/{id}` | 撤销 Token（软删除） |

**创建 Token**：

```bash
curl -X POST http://localhost:8080/api/v1/tokens \
  -H "Authorization: Bearer <admin-token>" \
  -H "Content-Type: application/json" \
  -d '{"name": "My Token", "expires_at": "2026-12-31T23:59:59"}'
```

**列出 Token**：

```bash
curl http://localhost:8080/api/v1/tokens \
  -H "Authorization: Bearer <admin-token>"
```

**撤销 Token**：

```bash
curl -X POST http://localhost:8080/api/v1/tokens/revoke/2 \
  -H "Authorization: Bearer <admin-token>"
```

### 健康检查

```bash
curl http://localhost:8080/health
# {"status": "ok"}
```

### 统一错误响应

所有错误响应遵循统一格式：

```json
{
  "error": {
    "code": "UNAUTHORIZED",
    "message": "Invalid or revoked token"
  }
}
```

| HTTP 状态码 | 错误码 | 说明 |
|------------|--------|------|
| 400 | `BAD_REQUEST` | 请求参数无效 |
| 401 | `UNAUTHORIZED` | 未认证或 Token 无效/过期 |
| 404 | `NOT_FOUND` | 资源不存在 |
| 409 | `CONFLICT` | 资源冲突 |
| 500 | `INTERNAL_ERROR` | 内部错误 |
| 500 | `DATABASE_ERROR` | 数据库错误 |

### 统一成功响应

```json
// 200 / 201
{ "data": <value> }

// 204 — 无响应体
```

## 数据库表结构

| 表 | 说明 |
|----|------|
| `api_tokens` | Bearer Token（可撤销、可选过期时间） |
| `data_sources` | RSS 数据源配置（URL、拉取间隔） |
| `articles` | 采集的文章（`link` 去重，`processed_at` 追踪过滤状态） |
| `keywords` | 关键词及敏感度参数（`std_multiplier`、`min_hot_count`） |
| `hot_events` | 检测到的热点事件（小时桶统计） |
| `push_channels` | 推送渠道配置（Webhook URL） |
| `push_records` | 每热点每渠道推送状态与重试追踪 |

## 项目结构

```
TrendAITool/
├── config.toml                  # 配置文件
├── Cargo.toml                   # Rust 依赖与元数据
├── docs/
│   ├── migrations/              # 数据库迁移 SQL
│   ├── apis/                    # API 设计文档
│   ├── plans/                   # 实施计划文档
│   └── data/                    # SQLite 数据文件
├── src/
│   ├── main.rs                  # 入口点、CLI、Token 引导
│   ├── config.rs                # 配置解析（TOML → 结构体）
│   ├── error.rs                 # 统一错误处理（AppError + ApiResponse）
│   ├── routes.rs                # 路由注册与中间件层
│   ├── db.rs                    # 连接池初始化（SQLite WAL）
│   ├── models/
│   │   ├── token.rs             # ApiToken / ApiTokenInfo / CreateTokenRequest
│   │   ├── article.rs           # 文章模型
│   │   ├── keyword.rs           # 关键词模型
│   │   ├── hot_event.rs         # 热点事件模型
│   │   ├── push_record.rs       # 推送记录模型
│   │   ├── source.rs            # 数据源模型
│   │   └── channel.rs           # 推送渠道模型
│   ├── db/
│   │   ├── token.rs             # Token 数据库操作
│   │   ├── article.rs           # 文章数据库操作
│   │   ├── keyword.rs           # 关键词数据库操作
│   │   ├── hot_event.rs         # 热点事件数据库操作
│   │   ├── push_record.rs       # 推送记录数据库操作
│   │   ├── source.rs            # 数据源数据库操作
│   │   └── channel.rs           # 推送渠道数据库操作
│   ├── handlers/
│   │   └── token.rs             # Token CRUD 处理器
│   ├── middleware/
│   │   └── auth.rs              # Bearer Token 认证中间件
│   └── services/                # 业务逻辑层（Parser/Filter/Pusher）
└── openspec/                    # 规格文档（spec-driven 工作流）
    ├── specs/                   # 主规格文档
    └── changes/archive/         # 已归档的变更
```

## 开发计划

当前实现阶段：

- [x] 后端项目脚手架（Axum + SQLite + 配置解析）
- [x] 数据库迁移与模型定义
- [x] 认证中间件与 Token 管理 API
- [ ] CRUD API（数据源、关键词、推送渠道管理）
- [ ] Parser 模块（RSS 采集）
- [ ] Filter 模块（关键词匹配 + 热点检测）
- [ ] Pusher 模块（Webhook 推送）
- [ ] 前端管理界面（React）
- [ ] Dashboard 可视化

## 热点检测算法

Filter 模块使用统计突发检测：

1. **关键词匹配** — Aho-Corasick 多模式匹配，扫描未处理文章
2. **小时桶计数** — 按关键词 + 小时窗口聚合文章数
3. **突发判定** — 滑动窗口（默认 24 小时）计算均值和标准差，当前计数超过 `mean + (std_multiplier × stddev)` 且达到 `min_hot_count` 阈值时触发热点
4. **去重** — 同一关键词在同一个小时内只生成一条热点事件

## 推送重试机制

Pusher 模块使用指数退避：

- 最大重试 3 次
- 退避公式：`retry_after = now + retry_base_seconds × 2^retry_count`
- 乐观锁（`WHERE status = ? AND retry_count < ?`）防止并发重复推送

## License

待定
