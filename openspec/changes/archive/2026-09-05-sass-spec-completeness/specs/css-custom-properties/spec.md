## ADDED Requirements

### Requirement: CSS 自定义属性解析
MUST 解析 CSS 自定义属性声明（`--name: value;`）及其值中的插值。

#### Scenario: 基本自定义属性
- **WHEN** 输入包含 `:root { --color: red; }`
- **THEN** 输出保留 `--color: red;`

#### Scenario: 自定义属性带插值
- **WHEN** 输入包含 `--size: #{$size}px;`
- **THEN** 插值被求值后输出

### Requirement: CSS 变量引用
MUST 支持 `var(--name)` 函数引用自定义属性。

#### Scenario: 基本 var 引用
- **WHEN** 输入包含 `color: var(--color);`
- **THEN** 输出保留 var() 调用（不求值，运行时行为）

#### Scenario: var 带 fallback
- **WHEN** 输入包含 `color: var(--color, red);`
- **THEN** 输出保留 fallback 值
