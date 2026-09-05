## ADDED Requirements

### Requirement: meta.feature-exists()
MUST 实现 `meta.feature-exists($feature)` 返回指定 feature 是否被支持（布尔值）。

#### Scenario: 已知 feature
- **WHEN** 调用 `meta.feature-exists(global-variable-shadowing)`
- **THEN** 返回 true（sass 核心支持）

#### Scenario: 未知 feature
- **WHEN** 调用 `meta.feature-exists(exists-fn)`
- **THEN** 返回 true（@function 已定义）

#### Scenario: 不支持的 feature
- **WHEN** 调用 `meta.feature-exists(unknown-feature)`
- **THEN** 返回 false

### Requirement: meta.content-exists()
MUST 实现 `meta.content-exists()` 检测当前 mixin 是否接收了 @content 块。

#### Scenario: 有 content
- **WHEN** mixin 被 `@include` 时传递了 `@content`
- **THEN** 返回 true

#### Scenario: 无 content
- **WHEN** mixin 被 `@include` 时未传递 `@content`
- **THEN** 返回 false

#### Scenario: 非 mixin 上下文
- **WHEN** 在普通函数中调用 `meta.content-exists()`
- **THEN** 报错（content-exists 只在 mixin 内有效）

### Requirement: meta.global-variable-exists() / meta.variable-exists()
MUST 正确区分全局 vs 局部变量作用域查找。

#### Scenario: 全局变量存在
- **WHEN** 调用 `meta.global-variable-exists($name)` 且变量在全局定义
- **THEN** 返回 true

#### Scenario: 局部变量不触发全局
- **WHEN** 变量仅在局部作用域定义
- **THEN** `global-variable-exists` 返回 false / `variable-exists` 返回 true

### Requirement: meta.apply()
MUST 实现 `meta.call($function, $args...)` 动态函数调用。

#### Scenario: 调用已定义函数
- **WHEN** 通过 `meta.call` 调用 `@function add($a, $b)`
- **THEN** 返回正确计算结果
