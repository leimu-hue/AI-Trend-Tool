# source-article-count

## Purpose

Return article count for each data source via LEFT JOIN + COUNT + GROUP BY query, so the frontend can display how many articles each source has fetched.

## Requirements

### Requirement: 数据源列表包含文章计数

系统 SHALL 提供 `list_sources_with_count` 查询函数，通过 LEFT JOIN articles 表计算每个数据源的文章数量，返回 `SourceWithCount` 结构体。

#### Scenario: 返回带计数的数据源列表

- **WHEN** `list_sources_with_count` 被调用
- **THEN** 返回的每个数据源 SHALL 包含 `article_count` 字段
- **THEN** 没有文章的数据源 `article_count` SHALL 为 0
- **THEN** 有 5 篇文章的数据源 `article_count` SHALL 为 5

#### Scenario: 结果按创建时间降序排列

- **WHEN** 多个数据源存在
- **THEN** 返回列表 SHALL 按 `created_at DESC` 排序
