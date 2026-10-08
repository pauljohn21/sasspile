# Value System Specification

## Purpose

定义 Sass 运行时值系统 `Value` enum，包含 Number/String/Color/List/Map/Bool/Null 七种类型。实现 `Display`、`PartialEq`、`Clone`，作为 Eval 阶段的运行时值载体。

## Requirements

### Requirement: Value 七种类型
`Value` enum SHALL 包含以下变体：`Number(f64)`（数值）、`String(String)`（字符串）、`Color(u8, u8, u8, u8)`（RGBA 颜色）、`List(Vec<Value>)`（列表）、`Map(Vec<(String, Value)>)`（键值映射）、`Bool(bool)`（布尔）、`Null`（空值）。

#### Scenario: Display Number
- **WHEN** `Value::Number(16.0)` 被格式化
- **THEN** SHALL 输出 "16"

#### Scenario: Display String
- **WHEN** `Value::String("hello")` 被格式化
- **THEN** SHALL 输出 "hello"（不带引号）

#### Scenario: Display Color with alpha=255
- **WHEN** `Value::Color(255, 0, 0, 255)` 被格式化
- **THEN** SHALL 输出 "#ff0000"

#### Scenario: Display Color with alpha<255
- **WHEN** `Value::Color(255, 0, 0, 128)` 被格式化
- **THEN** SHALL 输出 "#ff000080"

#### Scenario: Display Bool
- **WHEN** `Value::Bool(true)` 被格式化
- **THEN** SHALL 输出 "true"

#### Scenario: Display Null
- **WHEN** `Value::Null` 被格式化
- **THEN** SHALL 输出 "null"

#### Scenario: Display List
- **WHEN** `Value::List([Number(1), Number(2), Number(3)])` 被格式化
- **THEN** SHALL 输出 "1, 2, 3"

#### Scenario: Display Map
- **WHEN** `Value::Map([("a", Number(1)), ("b", Number(2))])` 被格式化
- **THEN** SHALL 输出 "(a: 1, b: 2)"

### Requirement: 列表操作
`Value::List` SHALL 支持通过 `Value::List(items)` 创建。列表 SHALL 保留插入顺序。列表 SHALL 通过逗号分隔的 `Display` 输出（Sass 默认列表分隔符）。

#### Scenario: Empty list
- **WHEN** `Value::List(vec![])` 被格式化
- **THEN** SHALL 输出空字符串或 "()"（遵循 Sass 语义）

### Requirement: Map 操作
`Value::Map` SHALL 通过 `Vec<(String, Value)>` 保持插入顺序。Map SHALL 支持通过键查找（返回 `Option<&Value>`）。

#### Scenario: Map key lookup
- **WHEN** Map 为 `[("a", Number(1)), ("b", Number(2))]`，查找 "a"
- **THEN** SHALL 返回 Some(Value::Number(1))

#### Scenario: Map missing key lookup
- **WHEN** Map 为 `[("a", Number(1))]`，查找 "missing"
- **THEN** SHALL 返回 None

### Requirement: 类型语义一致性
SHALL 全体 Value 类型满足 `PartialEq` 用于测试断言。Number 比较 SHALL 使用 `f64` 精确比较（注意 NaN 处理遵循 Sass 行为）。

#### Scenario: Number equality
- **WHEN** `Value::Number(1.0) == Value::Number(1.0)`
- **THEN** SHALL 为 true

### Requirement: Clone 语义
`Value` SHALL 自动 derive `Clone`。深克隆 List/Map 时 SHALL 递归克隆所有子值，确保所有权语义清晰。

#### Scenario: List clone independence
- **WHEN** 克隆 `Value::List` 后修改原列表
- **THEN** 克隆体 SHALL 不受影响
