## ADDED Requirements

### Requirement: compute_stats 纯函数

系统 SHALL 提取 `compute_stats(counts: &[i32]) -> (f64, f64)` 纯函数，计算数组的均值和标准差。该函数 SHALL 有单元测试覆盖。

#### Scenario: 空数组

- **WHEN** `compute_stats(&[])` 被调用
- **THEN** 返回值 SHALL 为 `(0.0, 0.0)`

#### Scenario: 单元素

- **WHEN** `compute_stats(&[5])` 被调用
- **THEN** 均值 SHALL 为 5.0
- **THEN** 标准差 SHALL 为 0.0

#### Scenario: 多元素正常分布

- **WHEN** `compute_stats(&[2, 4, 4, 4, 5, 5, 7, 9])` 被调用
- **THEN** 均值 SHALL 约等于 5.0
- **THEN** 标准差 SHALL 约等于 2.0
