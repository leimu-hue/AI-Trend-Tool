# 触发API规范

<cite>
**本文档引用的文件**
- [openspec/specs/trigger-apis/spec.md](file://openspec/specs/trigger-apis/spec.md)
- [openspec/changes/event-driven-pipeline/specs/trigger-apis/spec.md](file://openspec/changes/event-driven-pipeline/specs/trigger-apis/spec.md)
- [src/services/pusher.rs](file://src/services/pusher.rs)
- [src/models/push_record.rs](file://src/models/push_record.rs)
- [src/db/push_record.rs](file://src/db/push_record.rs)
- [src/routes.rs](file://src/routes.rs)
- [src/main.rs](file://src/main.rs)
- [docs/plans/09-event-driven-pipeline.md](file://docs/plans/09-event-driven-pipeline.md)
- [docs/apis/channel-api.md](file://docs/apis/channel-api.md)
- [docs/apis/keyword-api.md](file://docs/apis/keyword-api.md)
- [docs/apis/source-api.md](file://docs/apis/source-api.md)
- [docs/apis/token-api.md](file://docs/apis/token-api.md)
</cite>

## 目录
1. [简介](#简介)
2. [项目结构](#项目结构)
3. [核心组件](#核心组件)
4. [架构概览](#架构概览)
5. [详细组件分析](#详细组件分析)
6. [依赖关系分析](#依赖关系分析)
7. [性能考虑](#性能考虑)
8. [故障排除指南](#故障排除指南)
9. [结论](#结论)

## 简介

触发API是AI趋势工具项目中的关键组件，负责处理事件驱动的数据推送和通知机制。该系统基于Rust后端服务，结合前端React应用，实现了实时数据更新和用户通知功能。

触发API的核心目标包括：
- 实现事件驱动的数据推送机制
- 提供实时通知功能
- 支持多用户订阅管理
- 确保数据一致性和可靠性

## 项目结构

该项目采用模块化架构设计，主要包含以下核心模块：

```mermaid
graph TB
subgraph "后端服务"
A[src/main.rs] --> B[src/routes.rs]
B --> C[src/services/pusher.rs]
C --> D[src/models/push_record.rs]
D --> E[src/db/push_record.rs]
end
subgraph "前端应用"
F[web/src/renderer/src/api/] --> G[channels.ts]
F --> H[keywords.ts]
F --> I[sources.ts]
F --> J[queries.ts]
F --> K[tokens.ts]
end
subgraph "文档规范"
L[openspec/specs/trigger-apis/spec.md] --> M[事件驱动管道规范]
N[docs/plans/09-event-driven-pipeline.md] --> O[实施计划]
end
A --> F
L --> A
M --> C
```

**图表来源**
- [src/main.rs](file://src/main.rs)
- [src/routes.rs](file://src/routes.rs)
- [src/services/pusher.rs](file://src/services/pusher.rs)
- [openspec/specs/trigger-apis/spec.md](file://openspec/specs/trigger-apis/spec.md)

**章节来源**
- [src/main.rs](file://src/main.rs)
- [src/routes.rs](file://src/routes.rs)
- [openspec/specs/trigger-apis/spec.md](file://openspec/specs/trigger-apis/spec.md)

## 核心组件

### 推送服务 (Pusher Service)

推送服务是触发API的核心组件，负责处理所有推送相关的业务逻辑：

```mermaid
classDiagram
class PusherService {
+push_notification() Result~bool~
+subscribe_user() Result~bool~
+unsubscribe_user() Result~bool~
+broadcast_event() Result~bool~
+validate_subscription() bool
}
class PushRecord {
+id : Uuid
+user_id : Uuid
+channel : String
+created_at : DateTime
+updated_at : DateTime
}
class NotificationEvent {
+event_type : EventType
+payload : Json
+timestamp : DateTime
+target_users : Vec~Uuid~
}
PusherService --> PushRecord : "管理"
PusherService --> NotificationEvent : "处理"
```

**图表来源**
- [src/services/pusher.rs](file://src/services/pusher.rs)
- [src/models/push_record.rs](file://src/models/push_record.rs)

### 数据模型

推送记录模型定义了存储推送状态的数据结构：

| 字段名 | 类型 | 描述 | 约束 |
|--------|------|------|------|
| id | Uuid | 唯一标识符 | 主键 |
| user_id | Uuid | 用户标识符 | 外键 |
| channel | String | 推送频道 | 非空 |
| created_at | DateTime | 创建时间 | 非空 |
| updated_at | DateTime | 更新时间 | 非空 |

**章节来源**
- [src/models/push_record.rs](file://src/models/push_record.rs)
- [src/db/push_record.rs](file://src/db/push_record.rs)

## 架构概览

触发API采用事件驱动架构，实现了完整的推送生命周期管理：

```mermaid
sequenceDiagram
participant Client as 客户端应用
participant API as API网关
participant Pusher as 推送服务
participant DB as 数据库
participant User as 用户客户端
Client->>API : 订阅推送请求
API->>Pusher : validate_subscription()
Pusher->>DB : create_push_record()
DB-->>Pusher : 记录创建成功
Pusher-->>API : 订阅确认
API-->>Client : 订阅响应
Note over Client,User : 事件发生时
Client->>API : 触发事件
API->>Pusher : broadcast_event()
Pusher->>DB : 查询订阅用户
DB-->>Pusher : 用户列表
Pusher->>User : 实时推送通知
User-->>Pusher : 确认接收
Pusher-->>API : 推送完成
API-->>Client : 操作结果
```

**图表来源**
- [src/services/pusher.rs](file://src/services/pusher.rs)
- [src/routes.rs](file://src/routes.rs)
- [src/db/push_record.rs](file://src/db/push_record.rs)

## 详细组件分析

### 订阅管理流程

订阅管理是触发API的基础功能，确保用户能够正确订阅感兴趣的推送频道：

```mermaid
flowchart TD
Start([开始订阅流程]) --> Validate["验证用户身份"]
Validate --> Validate{"身份验证通过?"}
Validate --> |否| Error["返回认证错误"]
Validate --> |是| CheckChannel["检查频道有效性"]
CheckChannel --> ChannelValid{"频道有效?"}
CheckChannel --> |否| ChannelError["返回频道错误"]
ChannelValid --> |否| ChannelError
ChannelValid --> |是| CheckExisting["检查现有订阅"]
CheckExisting --> HasSubscription{"已存在订阅?"}
HasSubscription --> |是| UpdateSubscription["更新订阅信息"]
HasSubscription --> |否| CreateNew["创建新订阅"]
UpdateSubscription --> Save["保存到数据库"]
CreateNew --> Save
Save --> Success["返回成功响应"]
Error --> End([结束])
ChannelError --> End
Success --> End
```

**图表来源**
- [src/services/pusher.rs](file://src/services/pusher.rs)
- [src/db/push_record.rs](file://src/db/push_record.rs)

### 事件广播机制

事件广播是触发API的核心功能，负责将事件实时推送给所有订阅用户：

```mermaid
classDiagram
class EventBroadcaster {
+broadcast(event) Result~Vec~Uuid~~
+filter_subscribers(channel) Vec~Uuid~
+format_payload(event) Json
+send_notifications(users) Result~bool~
}
class SubscriptionManager {
+get_channel_subscribers(channel) Vec~Uuid~
+validate_user_subscription(user_id, channel) bool
+batch_update_subscriptions() Result~bool~
}
class NotificationFormatter {
+format_for_websocket(payload) WebSocketMessage
+format_for_http(payload) HttpResponse
+format_for_email(payload) EmailMessage
}
EventBroadcaster --> SubscriptionManager : "查询订阅"
EventBroadcaster --> NotificationFormatter : "格式化消息"
SubscriptionManager --> PushRecord : "数据库查询"
```

**图表来源**
- [src/services/pusher.rs](file://src/services/pusher.rs)
- [src/models/push_record.rs](file://src/models/push_record.rs)

**章节来源**
- [src/services/pusher.rs](file://src/services/pusher.rs)
- [src/models/push_record.rs](file://src/models/push_record.rs)

### API路由配置

触发API的路由配置定义了所有可用的接口端点：

| 路由路径 | HTTP方法 | 功能描述 | 认证要求 |
|----------|----------|----------|----------|
| `/api/push/subscribe` | POST | 用户订阅推送 | 是 |
| `/api/push/unsubscribe` | POST | 用户取消订阅 | 是 |
| `/api/push/broadcast` | POST | 广播事件给所有用户 | 是 |
| `/api/push/status` | GET | 获取推送状态 | 否 |
| `/api/push/subscriptions` | GET | 获取用户订阅列表 | 是 |

**章节来源**
- [src/routes.rs](file://src/routes.rs)
- [src/main.rs](file://src/main.rs)

## 依赖关系分析

触发API系统涉及多个层次的依赖关系，形成了清晰的分层架构：

```mermaid
graph TB
subgraph "表现层"
A[Web API Routes]
B[WebSocket连接]
C[HTTP客户端]
end
subgraph "业务逻辑层"
D[Pusher Service]
E[Subscription Manager]
F[Event Processor]
end
subgraph "数据访问层"
G[Push Record Repository]
H[User Repository]
I[Channel Repository]
end
subgraph "基础设施层"
J[数据库连接池]
K[Redis缓存]
L[消息队列]
end
A --> D
B --> D
C --> D
D --> E
D --> F
E --> G
F --> G
G --> J
D --> K
D --> L
```

**图表来源**
- [src/routes.rs](file://src/routes.rs)
- [src/services/pusher.rs](file://src/services/pusher.rs)
- [src/db/push_record.rs](file://src/db/push_record.rs)

**章节来源**
- [src/routes.rs](file://src/routes.rs)
- [src/services/pusher.rs](file://src/services/pusher.rs)

## 性能考虑

触发API在设计时充分考虑了性能优化，采用了多种策略来确保系统的高效运行：

### 缓存策略
- Redis缓存常用查询结果
- 内存中维护活跃用户会话
- 频繁访问的订阅信息缓存

### 连接管理
- 连接池复用数据库连接
- WebSocket连接池管理
- 异步处理减少阻塞

### 批量操作
- 批量推送减少网络开销
- 批量订阅更新优化性能
- 事务性操作保证一致性

## 故障排除指南

### 常见问题及解决方案

**订阅失败**
- 检查用户认证状态
- 验证频道名称格式
- 确认数据库连接正常

**推送延迟**
- 检查Redis连接状态
- 监控数据库性能
- 查看消息队列积压情况

**连接断开**
- 检查WebSocket配置
- 验证防火墙设置
- 监控服务器资源使用

**章节来源**
- [src/services/pusher.rs](file://src/services/pusher.rs)
- [src/error.rs](file://src/error.rs)

## 结论

触发API规范为AI趋势工具项目提供了一个完整、可靠的事件驱动推送系统。通过模块化的架构设计、完善的错误处理机制和性能优化策略，该系统能够满足高并发场景下的实时推送需求。

关键优势包括：
- 清晰的分层架构便于维护和扩展
- 完善的订阅管理机制确保用户体验
- 高性能的事件广播系统支持大规模用户
- 详细的文档规范指导开发和部署

未来可以考虑的功能增强包括：
- 更精细的权限控制机制
- 支持更多推送渠道（邮件、短信等）
- 增强的监控和日志功能
- 自动化的负载均衡和容错机制