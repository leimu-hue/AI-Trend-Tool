# Validator Integration

## Purpose

Integrate the `validator` crate for declarative input validation on request structs, replacing repetitive manual validation code with derive macros.

## Requirements

### Requirement: Request 结构体使用 validator derive

系统 SHALL 为所有 Create/Update 请求结构体添加 `#[derive(Validate)]`，使用 `validator` crate 的声明式注解替代手动验证代码。handler 中 SHALL 通过 `req.validate()?` 触发验证。

#### Scenario: Source 创建请求验证
- **WHEN** `CreateSourceRequest` 被反序列化
- **THEN** `name` 字段 SHALL 有 `#[validate(length(min = 1))]` 注解
- **THEN** `url` 字段 SHALL 有 `#[validate(url)]` 注解
- **THEN** handler 调用 `req.validate()?` 后无效请求自动返回 400

#### Scenario: Keyword 创建请求验证
- **WHEN** `CreateKeywordRequest` 被反序列化
- **THEN** `word` 字段 SHALL 有 `#[validate(length(min = 1))]` 注解
- **THEN** `std_multiplier` 和 `min_hot_count` 字段 SHALL 有自定义验证函数

#### Scenario: ValidationErrors 自动转换为 AppError
- **WHEN** `req.validate()` 返回 `Err(ValidationErrors)`
- **THEN** `AppError` 的 `From<ValidationErrors>` 实现 SHALL 将其转换为 HTTP 400 Bad Request
