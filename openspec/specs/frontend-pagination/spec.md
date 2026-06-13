# Frontend Pagination

## Purpose

Provide pagination UI controls for list pages (Articles, Hotspots, etc.) using project-consistent button styles.

## Requirements

### Requirement: Articles 列表分页控件

前端 Articles 页面 SHALL 提供分页 UI 控件，使用项目统一的按钮样式（`btn btn-ghost btn-sm`），显示当前页/总页数和上一页/下一页按钮。

#### Scenario: 显示分页控件
- **WHEN** Articles 页面加载完成且总文章数超过 per_page
- **THEN** 页面底部 SHALL 显示分页控件，包含"上一页"和"下一页"按钮
- **THEN** 控件 SHALL 显示当前页码和总页数

#### Scenario: 点击翻页
- **WHEN** 用户点击"下一页"按钮
- **THEN** 页面 SHALL 重新请求 `GET /api/v1/articles?page=2&per_page=20`
- **THEN** 列表 SHALL 更新为第 2 页数据

#### Scenario: 首页时禁用上一页
- **WHEN** 当前为第 1 页
- **THEN** "上一页"按钮 SHALL 为禁用状态

#### Scenario: 末页时禁用下一页
- **WHEN** 当前为最后一页
- **THEN** "下一页"按钮 SHALL 为禁用状态
