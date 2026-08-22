## ADDED Requirements

### Requirement: 单一数据源注册

系统 SHALL 提供一个派生宏 `#[derive(BuiltinRegistry)]`，将所有内建函数的名称映射关系集中在一处结构体声明中。每个模块一个结构体，字段名代表全局函数名，`#[aliases = [...]]` 属性声明模块限定名和别名。

宏 MUST 从结构体声明自动生成以下三组代码：
1. `module_builtin_name(name: &str) -> &str` — 将模块限定名映射到内建名
2. `is_known_builtin(name: &str) -> bool` — 检查函数名是否为已知内建函数
3. `dispatch_builtin_module(name, pos_args, kw_args) -> Option<Result<Value>>` — 按模块路由到子模块 `call` 函数

#### Scenario: 添加新函数只需修改结构体

- **WHEN** 开发者在 `MathBuiltins` 结构体中添加字段 `log: ()` 和属性 `#[aliases = ["math.log"]]`
- **THEN** `module_builtin_name("math.log")` 返回 `"log"`
- **AND** `is_known_builtin("log")` 和 `is_known_builtin("math.log")` 都返回 `true`
- **AND** `dispatch_builtin_module("log", ...)` 和 `dispatch_builtin_module("math.log", ...)` 都能正确分派到 `math::call`

#### Scenario: 一对多别名映射

- **WHEN** `ColorBuiltins` 的 `adjust_color` 字段声明 `#[aliases = ["color.adjust", "color.adjust-color"]]`
- **THEN** `module_builtin_name("color.adjust")` 返回 `"adjust-color"`
- **AND** `module_builtin_name("color.adjust-color")` 返回 `"adjust-color"`
- **AND** `is_known_builtin("adjust-color")` / `is_known_builtin("color.adjust")` / `is_known_builtin("color.adjust-color")` 都返回 `true`

### Requirement: 字段名到函数名的 kebab-case 转换

派生宏 SHALL 将 Rust `snake_case` 字段名自动转换为 SCSS `kebab-case` 函数名。例如字段 `is_unitless` 转换为 `"is-unitless"`，字段 `str_length` 转换为 `"str-length"`。转换 MUST 在编译期完成，零运行时开销。

#### Scenario: snake_case 到 kebab-case

- **WHEN** 结构体定义字段 `is_unitless: ()`
- **THEN** 宏生成的 `is_known_builtin("is-unitless")` 返回 `true`
- **AND** 全局函数名 `"is-unitless"` 被纳入分派 match arm

### Requirement: 模块限定名映射

`module_builtin_name` 函数 SHALL 接受模块限定名（如 `"math.abs"`）并返回对应的内建函数名（如 `"abs"`）。未匹配的名称 MUST 原样返回。该函数 MUST 只从 `#[aliases]` 属性中提取模块限定名，不含字段名转换后的全局名。

#### Scenario: 已注册的模块限定名

- **WHEN** 调用 `module_builtin_name("math.abs")`
- **THEN** 返回 `"abs"`

#### Scenario: 未注册的名称

- **WHEN** 调用 `module_builtin_name("custom.func")`
- **THEN** 返回 `"custom.func"`（原样返回）

### Requirement: 已知函数检查

`is_known_builtin` 函数 SHALL 检查给定函数名是否为 sasspile 的内建函数。所有通过结构体字段和 `#[aliases]` 注册的函数名（包括 kebab-case 转换后的全局名 + 所有别名）MUST 被识别为已知函数。

#### Scenario: 全局函数名

- **WHEN** 调用 `is_known_builtin("abs")`
- **THEN** 返回 `true`

#### Scenario: 模块限定名

- **WHEN** 调用 `is_known_builtin("math.abs")`
- **THEN** 返回 `true`

#### Scenario: 未知函数

- **WHEN** 调用 `is_known_builtin("nonexistent")`
- **THEN** 返回 `false`

### Requirement: 模块级分派

`dispatch_builtin_module` 函数 SHALL 将函数名按模块路由到对应的子模块 `call` 函数。返回 `Some(Ok(value))` 表示已分派成功，返回 `Some(Err(...))` 表示分派目标已匹配但执行出错，返回 `None` 表示未匹配（调用方继续手工分派）。每个模块的所有函数名（全局名 + 模块限定名 + 别名）MUST 路由到同一个子模块的 `call` 函数。

#### Scenario: math 模块分派

- **WHEN** 调用 `dispatch_builtin_module("abs", [Number(-5)], {})`
- **THEN** 返回 `Some(Ok(...))`，结果来自 `math::call`
- **WHEN** 调用 `dispatch_builtin_module("math.abs", [Number(-5)], {})`
- **THEN** 返回 `Some(Ok(...))`，结果来自 `math::call`，与全局名调用一致

#### Scenario: color 模块分派

- **WHEN** 调用 `dispatch_builtin_module("adjust-color", ...)` 或 `dispatch_builtin_module("color.adjust", ...)`
- **THEN** 两者都调用 `color::call` 并返回相同结果

#### Scenario: 未匹配返回 None

- **WHEN** 调用 `dispatch_builtin_module("type-of", ...)`
- **THEN** 返回 `None`（meta 内联函数不纳入宏）

### Requirement: 零回归

宏重构后所有现有测试 MUST 零回归通过，包括：compile_test (43)、stage_test (10)、ast_test (8)、common_test (5)、bs_spec (15)、ep_full (121)。

#### Scenario: 全量测试通过

- **WHEN** 运行 `cargo test --test compile_test` 和 `cargo test --test ep_full`
- **THEN** 所有测试通过，通过数与重构前一致

### Requirement: 不宏化的函数保留手工处理

以下函数的 `call_builtin` 分派逻辑 SHALL 保留手工编写，不纳入派生宏：`sass`、`type-of`、`inspect`、`if`、`content-exists`、`feature-exists`、`mixin-exists`、`function-exists`、`global-variable-exists`、`variable-exists`、`get-function`、`call`、`get-mixin`、`module-functions`、`module-mixins`、`module-variables`、`accepts-content`、`keywords`、`calc-args`、`calc-name`、`calc`、`env`、`var`、`rgba`、`rgb`、`darken`、`lighten`、`mix`、CSS 透传 fallback。`call_builtin` MUST 在 `dispatch_builtin_module` 返回 `None` 后继续手工 match。

#### Scenario: meta 内联函数不受宏影响

- **WHEN** 调用 `call_builtin("type-of", [Number(42)], {})`
- **THEN** `dispatch_builtin_module` 返回 `None`，由 `call_builtin` 中的手工 match arm 处理，返回 `String("number")`

#### Scenario: color 特殊函数不受宏影响

- **WHEN** 调用 `call_builtin("rgba", ...)`
- **THEN** `dispatch_builtin_module` 返回 `None`（rgba 未注册到结构体），由 `call_builtin` 中的手工 arm `Self::builtin_rgba` 处理
