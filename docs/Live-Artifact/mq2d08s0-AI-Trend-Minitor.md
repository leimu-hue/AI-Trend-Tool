# AI热点监控项目 - 完整方案设计

## 1. 项目概述

### 1.1 目标
构建一个可扩展的热点监控系统，用户通过 Web UI 管理数据源（当前支持 RSS）和关键词，系统自动拉取内容、匹配关键词、检测突发热点，并通过配置的渠道（如钉钉 webhook）推送告警。同时提供 REST API，支持 Token 认证，便于后续集成 CLI 工具或 AI Skills。

### 1.2 核心特性
- **多源扩展**：数据源类型可扩展（RSS 为首发实现）
- **关键词监控**：用户自定义关键词，支持敏感度参数（标准差倍数、最小计数）
- **热点检测**：基于时间序列的突发检测（移动平均 + 标准差）
- **管道解耦**：解析器、过滤器、推送器独立运行，可水平扩展
- **多渠道推送**：支持 webhook（钉钉、飞书等），记录推送历史，防止重复
- **API 认证**：用户可生成/吊销 Token，支持前端、CLI、AI Skills 安全访问
- **Web UI**：React 前端，管理数据源、关键词、查看热点仪表盘

## 2. 整体架构

```text
┌───────────────────────────────────────────────────────────┐
│                        React UI / CLI / AI Skills                        │
└─────────────┬──────────────────────────────┬─────────────┘
              │ REST API                      │ REST API
              ▼                               ▼
┌─────────────────────────┐     ┌─────────────────────────┐
│        API 服务         │     │        SQLite            │
│  (axum + 后台任务)      │◄───►│  data_sources,          │
└─────────────┬───────────┘     │  keywords, channels,     │
              │                 │  articles, ...           │
              │ 触发/调度        └─────────────────────────┘
              ▼
┌───────────────────────────────────────────────────────────┐
│                   解析器模块（可扩展）                     │
│  - 根据 type 字段选择对应解析器（RssParser, ...）         │
│  - 每个数据源独立定时拉取                                 │
│  - 写入 articles 表                                       │
└─────────────┬─────────────────────────────────────────────┘
              │
              ▼
┌───────────────────────────────────────────────────────────┐
│                    过滤器模块                              │
│  - 轮询未处理文章 (processed_at IS NULL)                  │
│  - 关键词匹配（Aho‑Corasick）                             │
│  - 热点检测（滑动窗口 + 标准差）                          │
│  - 生成 hot_events 和 push_records (status=pending)       │
│  - 更新 processed_at                                      │
└─────────────┬─────────────────────────────────────────────┘
              │
              ▼
┌───────────────────────────────────────────────────────────┐
│                    推送器模块                              │
│  - 轮询 push_records (status=pending 或重试)              │
│  - 调用对应渠道 webhook                                   │
│  - 更新状态为 success/failed                              │
└───────────────────────────────────────────────────────────┘
```

## 3. 数据模型（SQLite）

### 3.1 API Token 表 `api_tokens`
用于用户生成 token，支持前端、CLI、AI Skills 访问 API。

| 字段 | 类型 | 说明 |
|------|------|------|
| id | INTEGER PRIMARY KEY | |
| name | TEXT | Token 名称/用途（例如“前端UI”、“CLI工具”、“GitHub Skill”） |
| token | TEXT UNIQUE | 实际 token 字符串（建议随机生成 64 字符） |
| last_used_at | DATETIME | 最后使用时间 |
| created_at | DATETIME | |
| expires_at | DATETIME | 可选，NULL 表示永不过期 |
| revoked | BOOLEAN | 是否吊销，默认 0 |

### 3.2 数据源配置表 `data_sources`

| 字段 | 类型 | 说明 |
|------|------|------|
| id | INTEGER PRIMARY KEY | |
| type | TEXT | 源类型：`rss` (当前)，未来可扩展 `atom`, `json_feed` 等 |
| name | TEXT | 用户自定义名称 |
| url | TEXT | 数据源地址 |
| config | TEXT (JSON) | 扩展配置，如 `{"timeout": 30, "user_agent": "..."}` |
| enabled | BOOLEAN | 默认 1 |
| interval_seconds | INTEGER | 拉取间隔，默认 300 |
| last_fetched_at | DATETIME | |
| created_at | DATETIME | |
| updated_at | DATETIME | |

### 3.3 文章表 `articles`

| 字段 | 类型 | 说明 |
|------|------|------|
| id | INTEGER PRIMARY KEY | |
| source_id | INTEGER | 外键 → data_sources.id |
| link | TEXT UNIQUE | 文章唯一 URL |
| title | TEXT | |
| summary | TEXT | |
| content | TEXT | 保留全文扩展 |
| published_at | DATETIME | 原始发布时间 |
| fetched_at | DATETIME | 抓取时间 |
| processed_at | DATETIME | 过滤器处理时间（NULL=未处理） |

### 3.4 关键词表 `keywords`

| 字段 | 类型 | 说明 |
|------|------|------|
| id | INTEGER PRIMARY KEY | |
| word | TEXT | |
| case_sensitive | BOOLEAN | 默认 0 |
| enabled | BOOLEAN | 默认 1 |
| std_multiplier | REAL | 标准差倍数，默认 2.0 |
| min_hot_count | INTEGER | 最小触发计数，默认 3 |
| created_at | DATETIME | |

### 3.5 关键词命中明细表 `keyword_mentions`（可选）

| 字段 | 类型 | 说明 |
|------|------|------|
| id | INTEGER PRIMARY KEY | |
| keyword_id | INTEGER | |
| article_id | INTEGER | |
| matched_at | DATETIME | |

### 3.6 热点事件表 `hot_events`

| 字段 | 类型 | 说明 |
|------|------|------|
| id | INTEGER PRIMARY KEY | |
| keyword_id | INTEGER | |
| hour_bucket | TEXT | YYYYMMDDHH |
| count | INTEGER | |
| mean_historical | REAL | 过去24个完整小时均值 |
| stddev_historical | REAL | 过去24个完整小时标准差 |
| created_at | DATETIME | |

### 3.7 推送渠道表 `push_channels`

| 字段 | 类型 | 说明 |
|------|------|------|
| id | INTEGER PRIMARY KEY | |
| name | TEXT | |
| channel_type | TEXT | 目前仅 `webhook` |
| config | TEXT (JSON) | `{"url": "https://..."}` |
| enabled | BOOLEAN | 默认 1 |

### 3.8 推送记录表 `push_records`

| 字段 | 类型 | 说明 |
|------|------|------|
| id | INTEGER PRIMARY KEY | |
| hot_event_id | INTEGER | 外键 → hot_events.id |
| channel_id | INTEGER | 外键 → push_channels.id |
| status | TEXT | `pending`, `success`, `failed` |
| retry_count | INTEGER | 默认 0 |
| next_retry_at | DATETIME | 下次重试时间（NULL=立即） |
| created_at | DATETIME | |
| updated_at | DATETIME | |
| UNIQUE(hot_event_id, channel_id) | | 防重复 |

## 4. 模块详细设计

### 4.1 API 服务与 Token 认证

#### 4.1.1 Token 管理 API
| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/v1/tokens` | 生成新 Token：`{name, expires_at?}` → `{token}` |
| GET | `/api/v1/tokens` | 列出所有 Token（不含 token 明文，只显示名称、创建时间等） |
| DELETE | `/api/v1/tokens/{id}` | 吊销指定 Token |

#### 4.1.2 认证中间件
- **Header**：`Authorization: Bearer <token>`
- **验证逻辑**：检查 `api_tokens` 表中是否存在该 token，且 `revoked=0` 且 `(expires_at IS NULL OR expires_at > NOW())`。验证通过后，将 token 信息（如 id、name）存入请求扩展中，供后续日志审计。
- **无需认证的端点**：`/health` (可选)

#### 4.1.3 前端 Token 存储与使用
- 首次加载时，前端弹出输入框让用户填写已生成的 Token（用户需事先通过命令行或初始 token 生成一个管理 token，可提供引导）。
- Token 保存到 `localStorage`，后续请求自动携带。
- 提供“设置”页面，允许用户生成新 Token、吊销旧 Token，以及更换当前使用的 Token。

#### 4.1.4 初始 Token 引导
系统首次启动时，若 `api_tokens` 表为空，则在日志中输出一个初始 Token（一次性），用户使用该 Token 通过 API 生成更多 Token。也可通过配置文件预设初始 Token。

### 4.2 解析器模块（Parser）

- **职责**：根据 `data_sources.type` 调用对应解析实现，写入 `articles`。
- **当前实现**：`RssParser`（使用 `feed-rs`）。
- **调度**：后台循环扫描 `data_sources`，对每个启用的源，检查是否需要拉取（`NOW() - last_fetched_at >= interval_seconds`）。每个源独立异步任务。
- **扩展新类型**：实现 `Parser` trait，在解析器工厂中注册。

### 4.3 过滤器模块（Filter）

- **运行频率**：每 5 分钟（可配置）。
- **处理流程**：
  1. 获取 `processed_at IS NULL` 的文章（批量 1000 条）。
  2. 加载所有启用的关键词，编译 Aho‑Corasick 自动机。
  3. 对每篇文章的 `title + summary` 匹配，生成命中记录（写入 `keyword_mentions` 表可选）。
  4. 按关键词累加当前小时桶计数（桶 = 文章 `fetched_at` 的 UTC 小时）。
  5. 对每个有命中的关键词：
     - 获取当前小时计数和过去 24 个完整小时的历史计数。
     - 计算均值和标准差，若超过阈值且满足最小计数，则创建 `hot_events` 记录。
  6. 对新热点，查询所有启用的推送渠道，为每个渠道插入 `push_records`（`status='pending'`）。
  7. 更新已处理文章的 `processed_at = NOW()`。

### 4.4 推送器模块（Pusher）

- **运行频率**：每 10 秒。
- **处理流程**：
  1. 查询待推送记录（`status='pending'` 或重试条件满足）。
  2. 对每条记录，根据 `channel_type` 发送（目前 webhook）。
  3. 成功 → 更新 `status='success'`；失败 → 更新 `retry_count` 和 `next_retry_at`。
  4. 使用乐观锁避免重复发送。

## 5. REST API 完整列表

所有 API（除 `/health`）需 Bearer Token 认证。

### 5.1 Token 管理
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/tokens` | 创建新 Token |
| GET | `/tokens` | 列出 Token 元数据 |
| DELETE | `/tokens/{id}` | 吊销 Token |

### 5.2 数据源管理
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/sources` | 列表 |
| POST | `/sources` | 添加（type, name, url, interval_seconds, config） |
| PUT | `/sources/{id}` | 更新 |
| DELETE | `/sources/{id}` | 删除 |
| POST | `/sources/{id}/fetch` | 手动触发抓取 |

### 5.3 关键词管理
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/keywords` | 列表 |
| POST | `/keywords` | 添加（word, case_sensitive, std_multiplier, min_hot_count） |
| PUT | `/keywords/{id}` | 更新 |
| DELETE | `/keywords/{id}` | 删除 |

### 5.4 推送渠道管理
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/channels` | 列表 |
| POST | `/channels` | 添加（name, channel_type, config） |
| PUT | `/channels/{id}` | 更新 |
| DELETE | `/channels/{id}` | 删除 |

### 5.5 数据查询
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/articles` | 文章列表（分页，过滤） |
| GET | `/hotspots` | 热点事件列表 |
| GET | `/hotspots/{id}/push-records` | 某热点的推送记录 |
| GET | `/trend/{keyword_id}` | 近 N 小时计数曲线 |

### 5.6 系统控制
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/trigger/filter` | 手动运行过滤器 |
| POST | `/trigger/pusher` | 手动运行推送器 |
| GET | `/health` | 健康检查（可免认证） |

## 6. UI 设计要点（React）

- **Token 设置页**：展示当前使用的 Token，允许生成新 Token、切换 Token、吊销不需要的 Token。
- **数据源管理页**：CRUD 表格，类型选择（当前 RSS），填写 URL、名称、间隔。
- **关键词管理页**：CRUD 表格，可调参数。
- **推送渠道页**：管理 webhook URL。
- **仪表盘**：热点事件列表 + 趋势图（ECharts）。
- **文章日志页**：查看原始文章和处理状态。

## 7. 部署与配置

### 7.1 配置文件 `config.toml`

```toml
[server]
host = "0.0.0.0"
port = 8080

[database]
path = "./data/hotspot.db"

[auth]
# 初始 Token（仅当 api_tokens 表为空时自动创建）
initial_token = "optional-initial-token"

[parser]
max_concurrent_fetches = 10
default_user_agent = "HotspotMonitor/1.0"
default_timeout_seconds = 30

[filter]
batch_size = 1000
interval_seconds = 300
history_hours = 24
min_history_hours = 6

[pusher]
interval_seconds = 10
max_retries = 3
retry_base_seconds = 60
```

### 7.2 运行方式

```bash
# 启动所有模块（开发）
hotspot --config config.toml all

# 或分别启动
hotspot api          # 仅 API 服务
hotspot parser       # 仅解析器
hotspot filter       # 仅过滤器
hotspot pusher       # 仅推送器
```

### 7.3 数据库迁移

使用 `sqlx migrate` 管理 schema。

## 8. 扩展性说明

- **新数据源类型**：实现 `Parser` trait，在 `data_sources.type` 中使用新值，解析器工厂自动注册。
- **新推送渠道**：扩展 `push_channels.channel_type`，在推送器中增加对应发送函数。
- **多用户**：后续可在所有表中增加 `user_id`，并在 API 中引入用户认证（取代单一 token 系统），当前 token 管理可作为管理员级别的访问控制。
- **AI Skills 集成**：外部 AI 可通过生成的 Token 调用 API，获取热点数据或触发操作。

## 9. 总结

本方案完整涵盖了从数据源管理、关键词监控、热点检测到多渠道推送的全流程，并设计了用户可管理的 Token 认证体系，支持前端、CLI 和 AI Skills 安全访问。模块解耦，易于扩展，适合作为项目开发的最终蓝图。