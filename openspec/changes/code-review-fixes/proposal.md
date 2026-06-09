## Why

代码审查发现 13 个问题，跨 security（明文 token）、data integrity（外键断裂）、performance（N+1 查询、串行推送）和 reliability（硬编码配置、无优雅关闭）四个维度。这些问题在低负载下不暴露，但规模增长后会引发数据不一致、推送丢失和性能瓶颈。现在修复是因为当前数据量小，迁移成本最低。

## What Changes

- **P0 Bug 修复**：push_records retry_count 从硬编码改为从配置读取；hot_events UPSERT 用 ON CONFLICT 替代 DELETE+INSERT 避免外键断裂
- **P1 安全加固**：api_tokens 表新增 token_hash 列，用 SHA-256 替代明文存储
- **P1 性能优化**：RssParser 复用 reqwest::Client；所有 mutation handler 增加输入校验
- **P2 性能优化**：keyword_mentions 批量插入；Pusher 并发推送（最多 8 路）；Filter 历史统计批量查询
- **P2 可靠性**：优雅关闭等待后台任务完成；health check 增加数据库探活；配置加载后校验字段合法性
- **P3 代码质量**：list_articles/count_articles 消除 4 路 match 分支重复；revoke_token 先检查后操作

## Capabilities

### New Capabilities

- `config-validation`: 配置加载后校验字段合法性（port > 0、interval > 0 等），非法配置直接拒绝启动
- `input-validation`: 所有 Create/Update handler 入口处增加输入校验（空字符串、无效 URL、非法 JSON 等）

### Modified Capabilities

- `database-schema`: api_tokens 新增 token_hash 列 + 唯一索引；hot_events 新增 (keyword_id, hour_bucket) UNIQUE 约束
- `token-api`: create_token 和 token 查询改为基于 SHA-256 哈希存储和匹配
- `filter-module`: hot_events 写入从 DELETE+INSERT 改为 ON CONFLICT UPSERT；keyword_mentions 改为批量插入；历史统计改为单次批量查询
- `pusher-module`: list_retry_due_records 接受外部 max_retries 参数；推送改为并发执行
- `parser-module`: RssParser 持有复用的 reqwest::Client 实例
- `source-crud-api`: create_source / update_source 增加 name/url 输入校验
- `keyword-crud-api`: create_keyword / update_keyword 增加 word/std_multiplier/min_hot_count 输入校验
- `channel-crud-api`: create_channel / update_channel 增加 name/config JSON 校验

## Impact

- **数据库迁移**: 2 个新迁移文件（hot_events UNIQUE 约束、api_tokens token_hash 列）
- **依赖变更**: Cargo.toml 新增 `sha2`、`futures`
- **API 行为**: 输入校验失败将返回 400 Bad Request（此前静默接受无效数据）
- **路由结构**: `/health` 端点需要访问 AppState，路由结构微调
- **破坏性**: 无。token_hash 迁移保留原 token 列向后兼容

## Non-goals

- 不引入新的 validate/deserialize 库，手动校验
- 不修改前端代码
- 不重构 filter/pusher 的整体调度循环，仅优化内部热点路径
- 不在本次变更中删除 token 明文列
