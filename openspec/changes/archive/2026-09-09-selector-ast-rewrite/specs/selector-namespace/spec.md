## ADDED Requirements

### Requirement: Namespace 枚举定义
系统 SHALL 提供 `Namespace` 枚举，包含四个变体：`None`（无前缀类型如 `c`）、`Empty`（空命名空间 `|c`）、`Any`（任意命名空间 `*|c`）、`Explicit(String)`（显式命名空间 `ns|type`）。

#### Scenario: 枚举变体完整性
- **WHEN** 解析任意 CSS 选择器类型标记
- **THEN** 系统 SHALL 能将其映射到 `Namespace` 四个变体之一

### Requirement: SimpleSelector::Type 结构体化
`SimpleSelector::Type`  SHALL 从 `Type(String)` 改造为 `Type { namespace: Namespace, name: String }`，其中 `name` 为 `"*"` 时表示命名空间内通用选择器（如 `c|*`）。

#### Scenario: 类型选择器结构化存储
- **WHEN** 解析 `svg|circle` 类型选择器
- **THEN** 系统存储为 `Type { namespace: Namespace::Explicit("svg"), name: "circle" }`

#### Scenario: 命名空间通用选择器
- **WHEN** 解析 `svg|*` 选择器
- **THEN** 系统存储为 `Type { namespace: Namespace::Explicit("svg"), name: "*" }`

### Requirement: 命名空间感知解析
`take_type_with_ns` SHALL 解析 `ns|type`、`|type`、`*|type`、`ns|*`、`*|*` 五种命名空间语法，输出结构化 `Namespace` + `name`。

#### Scenario: 显式命名空间解析
- **WHEN** 解析 `svg|circle`
- **THEN** 输出 `Namespace::Explicit("svg")` + name `"circle"`

#### Scenario: 空命名空间解析
- **WHEN** 解析 `|circle`
- **THEN** 输出 `Namespace::Empty` + name `"circle"`

#### Scenario: 任意命名空间解析
- **WHEN** 解析 `*|circle`
- **THEN** 输出 `Namespace::Any` + name `"circle"`

#### Scenario: 无前缀类型解析
- **WHEN** 解析 `circle`
- **THEN** 输出 `Namespace::None` + name `"circle"`

### Requirement: 命名空间序列化
`Display for SimpleSelector::Type` SHALL 将结构化 Type 序列化为 `ns|name` 格式。`Namespace::None` 无前缀，`Namespace::Empty` 输出 `|`，`Namespace::Any` 输出 `*|`，`Namespace::Explicit(ns)` 输出 `ns|`。

#### Scenario: 显式命名空间序列化
- **WHEN** 序列化 `Type { namespace: Namespace::Explicit("svg"), name: "circle" }`
- **THEN** 输出 `"svg|circle"`

#### Scenario: 空命名空间序列化
- **WHEN** 序列化 `Type { namespace: Namespace::Empty, name: "circle" }`
- **THEN** 输出 `"|circle"`

#### Scenario: 无前缀序列化
- **WHEN** 序列化 `Type { namespace: Namespace::None, name: "circle" }`
- **THEN** 输出 `"circle"`
