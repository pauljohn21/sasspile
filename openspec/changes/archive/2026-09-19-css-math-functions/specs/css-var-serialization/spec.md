# css-var-serialization Specification

## Purpose
定义 CSS `var()` 函数的序列化规范——保持 fallback 表达式不简化、保留大小写、正确处理尾部逗号空格。

## ADDED Requirements

### Requirement: var() fallback 懒求值
系统 SHALL 保留 `var()` 的 fallback 表达式原样，不进行 Sass 简化。

#### Scenario: fallback 表达式保留
- **WHEN** 表达式 `var(--c, 1 + 2)` 进过 eval
- **THEN** 序列化输出为 `var(--c, 1 + 2)` 而非 `var(--c, 3)`

#### Scenario: fallback 变量保留
- **WHEN** 表达式 `var(--c, $undefined)` 中 `$undefined` 未定义
- **THEN** 序列化输出保留变量引用而非报错

### Requirement: var() 尾部逗号空格
CSS `var(--c,)` 序列化时 SHALL 在逗号后保留一个空格。

#### Scenario: 空 fallback 带逗号
- **WHEN** 表达式 `var(--c,)` 序列化
- **THEN** 输出为 `var(--c, )`（逗号 + 空格）

#### Scenario: 右侧多余空格规范化
- **WHEN** 表达式 `var(--c , )` 序列化
- **THEN** 输出为 `var(--c, )`（移除 --c 后多余空格）

### Requirement: var() 大小写保留
CSS `var()` 函数调用的大小写 SHALL 在序列化时保留原始形式。

#### Scenario: VaR 大小写保留
- **WHEN** 表达式 `VaR(--c,)` 序列化
- **THEN** 输出为 `VaR(--c, )`（保留 VaR 大小写）

#### Scenario: 全小写保留
- **WHEN** 表达式 `var(--c,)` 序列化
- **THEN** 输出为 `var(--c, )`（全小写保留）

### Requirement: var() spread 参数
当 var() 通过参数列表 spread 调用时，系统 SHALL 正确展开。

#### Scenario: 2-arg spread
- **WHEN** `$args: --c, d;` 然后 `var($args...)`
- **THEN** 输出为 `var(--c, d)`

#### Scenario: function 定义覆盖
- **WHEN** 用户定义 `@function var($name, $arg) { @return null }` 然后调用 `var($name: --c,)`
- **THEN** 调用用户定义的 `var` 函数（Sass 函数优先）
