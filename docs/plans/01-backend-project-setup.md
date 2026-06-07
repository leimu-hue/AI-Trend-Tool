# 步骤 01：后端项目脚手架 + 配置 + 统一错误处理

## 前置依赖

无。这是第一个步骤。

## 目标

完成后拥有一个可编译运行的 Rust 项目骨架，包含：
- Cargo.toml 完整依赖声明
- config.toml 配置文件解析
- axum HTTP 服务器启动骨架
- 统一错误处理框架

---

## 模块组织规范

> **⚠️ 禁止使用 `mod.rs` 组织模块。** 本项目采用 Rust 2018 edition 引入的现代模块风格（又称 "non-mod.rs" 或 "folder-less" 风格）。

### 规则

| ❌ 禁止（旧风格） | ✅ 采用（现代风格） |
|---|---|
| `src/models/mod.rs` | `src/models.rs` |
| `src/handlers/mod.rs` | `src/handlers.rs` |
| `src/middleware/mod.rs` | `src/middleware.rs` |
| `src/services/mod.rs` | `src/services.rs` |

### 原理

- **消除歧义**：不再有同名的 `xxx/mod.rs`，每个模块的入口文件路径就是 `xxx.rs`，与模块名一一对应。
- **更好的编辑器体验**：打开多个模块入口文件时，标签页显示 `models.rs`、`handlers.rs` 而非一堆难以区分的 `mod.rs`。
- **Rust 社区推荐**：自 Rust 2018 edition 起，这是官方推荐的模块组织方式，主流项目（tokio、axum、serde 等）均已采用。

### 示例

当 `models` 模块包含子模块时，目录结构如下：

```
src/
├── models.rs            # 模块入口，声明 pub mod token; pub mod source; 等
├── models/
│   ├── token.rs         # crate::models::token
│   ├── source.rs        # crate::models::source
│   └── keyword.rs       # crate::models::keyword
```

而非旧风格的 `src/models/mod.rs`。

---

## 1. 项目初始化

```bash
cargo init --name trend-monitor
```

项目根目录结构：

```
trend-monitor/
├── Cargo.toml
├── config.toml
├── docs/
│   └── migrations/      # sqlx 迁移文件（步骤02创建）
├── src/
│   ├── main.rs          # 入口：解析配置 → 初始化DB → 启动 axum
│   ├── config.rs        # 配置结构体 + TOML 解析
│   ├── error.rs         # 统一错误处理
│   ├── db.rs            # 数据库连接池初始化
│   ├── models.rs        # 数据模型入口（步骤02创建，含 pub mod 声明）
│   ├── models/          # 数据模型子模块文件（步骤02创建）
│   ├── handlers.rs      # API handler 入口（步骤03创建，含 pub mod 声明）
│   ├── handlers/        # API handler 子模块文件（步骤03-05创建）
│   ├── middleware.rs     # 中间件入口（步骤03创建，含 pub mod 声明）
│   ├── middleware/       # 中间件子模块文件（步骤03创建）
│   ├── services.rs      # 后台模块入口（步骤05创建，含 pub mod 声明）
│   ├── services/        # 后台模块子模块文件（步骤05创建）
│   └── routes.rs        # 路由注册
```

> **注意**：每个多文件模块由 **一个同名 `.rs` 入口文件** + **一个同名目录** 组成。入口文件中通过 `pub mod xxx;` 声明子模块。**绝不在目录中放置 `mod.rs`。**

## 2. Cargo.toml 依赖清单

```toml
[package]
name = "trend-monitor"
version = "0.1.0"
edition = "2021"

[dependencies]
# Web 框架
axum = { version = "0.7", features = ["macros"] }
axum-extra = { version = "0.9", features = ["typed-header"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace"] }
tokio = { version = "1", features = ["full"] }

# 数据库
sqlx = { version = "0.7", features = ["runtime-tokio", "sqlite", "chrono"] }

# 序列化
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"

# 时间
chrono = { version = "0.4", features = ["serde"] }

# 日志
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# RSS 解析（步骤05使用，提前声明）
feed-rs = "1"

# 字符串匹配（步骤05使用，提前声明）
aho-corasick = "1"

# HTTP 客户端（webhook推送，步骤05使用）
reqwest = { version = "0.12", features = ["json"] }

# 随机 Token 生成
rand = "0.8"
hex = "0.4"

# CLI 参数解析
clap = { version = "4", features = ["derive"] }
```

## 3. 配置文件 config.toml

### 3.1 config.toml 内容

```toml
[server]
host = "0.0.0.0"
port = 8080

[database]
path = "./docs/data/hotspot.db"

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

### 3.2 配置解析代码 `src/config.rs`

```rust
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
    pub parser: ParserConfig,
    pub filter: FilterConfig,
    pub pusher: PusherConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub path: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AuthConfig {
    pub initial_token: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ParserConfig {
    pub max_concurrent_fetches: usize,
    pub default_user_agent: String,
    pub default_timeout_seconds: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FilterConfig {
    pub batch_size: u32,
    pub interval_seconds: u64,
    pub history_hours: u32,
    pub min_history_hours: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PusherConfig {
    pub interval_seconds: u64,
    pub max_retries: u32,
    pub retry_base_seconds: u64,
}

impl AppConfig {
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&content)?;
        Ok(config)
    }
}
```

## 4. 统一错误处理 `src/error.rs`

所有 API 端点使用统一的错误响应格式：

```json
{
  "error": {
    "code": "NOT_FOUND",
    "message": "Resource not found"
  }
}
```

### 4.1 AppError 枚举定义

```rust
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

pub enum AppError {
    /// 404 - 资源不存在
    NotFound(String),
    /// 400 - 请求参数错误
    BadRequest(String),
    /// 401 - 未认证或 Token 无效
    Unauthorized(String),
    /// 409 - 冲突（如唯一约束违反）
    Conflict(String),
    /// 500 - 内部错误
    Internal(String),
    /// 500 - 数据库错误（自动从 sqlx::Error 转换）
    Database(sqlx::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", msg),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", msg),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, "CONFLICT", msg),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", msg),
            AppError::Database(e) => {
                tracing::error!("Database error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "DATABASE_ERROR",
                    "Internal server error".to_string(),
                )
            }
        };

        let body = json!({
            "error": {
                "code": code,
                "message": message
            }
        });

        (status, Json(body)).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => AppError::NotFound("Resource not found".to_string()),
            _ => AppError::Database(err),
        }
    }
}
```

### 4.2 统一成功响应辅助

```rust
use axum::Json;
use serde::Serialize;

pub struct ApiResponse;

impl ApiResponse {
    /// 200 OK + JSON body
    pub fn ok<T: Serialize>(data: T) -> (StatusCode, Json<serde_json::Value>) {
        (StatusCode::OK, Json(json!({ "data": data })))
    }

    /// 201 Created + JSON body
    pub fn created<T: Serialize>(data: T) -> (StatusCode, Json<serde_json::Value>) {
        (StatusCode::CREATED, Json(json!({ "data": data })))
    }

    /// 204 No Content
    pub fn no_content() -> StatusCode {
        StatusCode::NO_CONTENT
    }
}
```

## 5. main.rs 骨架

```rust
mod config;
mod error;
mod db;
mod routes;

use clap::Parser;
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[derive(Parser)]
#[command(name = "hotspot", about = "AI Trend Monitor")]
struct Cli {
    #[arg(long, default_value = "config.toml")]
    config: String,

    #[arg(default_value = "all")]
    mode: String,  // all | api | parser | filter | pusher
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    let cli = Cli::parse();
    let config = config::AppConfig::load(&cli.config)?;

    // 确保数据目录存在
    let db_dir = std::path::Path::new(&config.database.path).parent().unwrap();
    std::fs::create_dir_all(db_dir)?;

    // 初始化数据库连接池
    let pool = db::init_pool(&config.database.path).await?;

    // 运行迁移
    sqlx::migrate!("./docs/migrations").run(&pool).await?;

    // 构建路由
    let app = routes::create_router(pool.clone(), config.clone());

    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port).parse()?;
    tracing::info!("Server listening on {}", addr);

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
```

## 6. 数据库连接池 `src/db.rs`

```rust
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};

pub async fn init_pool(database_path: &str) -> Result<SqlitePool, sqlx::Error> {
    let db_url = format!("sqlite:{}?mode=rwc", database_path);
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    // 启用 WAL 模式和外键约束
    sqlx::query("PRAGMA journal_mode=WAL")
        .execute(&pool)
        .await?;
    sqlx::query("PRAGMA foreign_keys=ON")
        .execute(&pool)
        .await?;

    Ok(pool)
}
```

## 7. 路由骨架 `src/routes.rs`

```rust
use axum::{
    routing::{get, post, put, delete},
    Router,
};
use sqlx::SqlitePool;
use tower_http::cors::CorsLayer;

use crate::config::AppConfig;

pub fn create_router(pool: SqlitePool, config: AppConfig) -> Router {
    let state = AppState { pool, config };

    Router::new()
        // 健康检查（免认证）
        .route("/health", get(health_check))
        // API v1（需认证，中间件在步骤03添加）
        .nest("/api/v1", api_routes())
        .with_state(state)
        .layer(CorsLayer::permissive())
}

fn api_routes() -> Router<AppState> {
    Router::new()
    // Token API（步骤03实现）
    // Sources API（步骤04实现）
    // Keywords API（步骤04实现）
    // Channels API（步骤04实现）
    // Query API（步骤05实现）
    // System control（步骤05实现）
}

async fn health_check() -> &'static str {
    "ok"
}

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub config: AppConfig,
}
```

---

## 预期文件清单

| 文件路径 | 说明 |
|---------|------|
| `Cargo.toml` | 项目依赖声明 |
| `config.toml` | 运行配置文件 |
| `src/main.rs` | 程序入口 |
| `src/config.rs` | 配置解析 |
| `src/error.rs` | 统一错误处理 |
| `src/db.rs` | 数据库连接池 |
| `src/routes.rs` | 路由注册骨架 |
| `src/models.rs` | 数据模型入口（含 pub mod 声明） |
| `src/models/` | 数据模型子模块文件（步骤02填充） |
| `src/handlers.rs` | API handler 入口（含 pub mod 声明） |
| `src/handlers/` | API handler 子模块文件（步骤03填充） |
| `src/middleware.rs` | 中间件入口（含 pub mod 声明） |
| `src/middleware/` | 中间件子模块文件（步骤03填充） |
| `src/services.rs` | 后台模块入口（含 pub mod 声明） |
| `src/services/` | 后台模块子模块文件（步骤05填充） |
| `docs/migrations/` | 空目录（步骤02填充） |

---

## 验证节点

```bash
# 编译检查（应无错误）
cargo check

# 启动服务器测试健康检查
cargo run -- --config config.toml all

# 另一个终端测试
curl http://localhost:8080/health
# 预期输出: ok
```
