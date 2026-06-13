## ADDED Requirements

### Requirement: keyword_mentions 唯一索引

`keyword_mentions` 表 SHALL 具有 `(keyword_id, article_id)` 复合唯一索引，防止同一关键词对同一文章产生重复提及记录。

#### Scenario: 插入重复提及被忽略

- **WHEN** 尝试插入已存在的 `(keyword_id, article_id)` 对
- **THEN** `INSERT OR IGNORE` SHALL 静默跳过该行
- **THEN** 不产生错误

#### Scenario: 迁移清理已有重复

- **WHEN** 迁移 `20260610000002_mentions_unique_index.sql` 执行
- **THEN** 已有的重复行 SHALL 被清理（保留最早记录）
- **THEN** 唯一索引 SHALL 创建成功

#### Scenario: 不同关键词同一文章正常插入

- **WHEN** 同一文章匹配两个不同关键词
- **THEN** 两个 `(keyword_id_1, article_id)` 和 `(keyword_id_2, article_id)` SHALL 均插入成功
