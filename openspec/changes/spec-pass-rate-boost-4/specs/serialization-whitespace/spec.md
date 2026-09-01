## ADDED Requirements

### Requirement: 同源展平规则间不加空行

当 SCSS 嵌套规则被展平为多个顶层规则时，来自同一父选择器的展平规则之间 SHALL NOT 输出空行分隔。

#### Scenario: 穿插声明和嵌套规则

- **WHEN** 输入 `.a { b: c; .d {e: f} }`
- **THEN** 输出 `.a { b: c; } .a .d { e: f; }`，两规则间无空行

#### Scenario: 声明穿插嵌套规则后再声明

- **WHEN** 输入 `.a { b: c; .d {e: f} g: h; }`
- **THEN** 输出三段 `.a {b:c} .a .d {e:f} .a {g:h}`，规则间无空行

#### Scenario: 不同选择器间仍加空行

- **WHEN** 输入 `.a { color: red; } .b { color: blue; }`
- **THEN** 输出 `.a { color: red; }` 空行 `.b { color: blue; }`，不同选择器间有空行

### Requirement: 展平后规则顺序保持源码顺序

展平嵌套规则时，输出规则顺序 SHALL 匹配 SCSS 源码中声明和嵌套规则的出现顺序。

#### Scenario: 声明在嵌套规则之前

- **WHEN** 输入 `.a { b: c; .d {e: f} }`
- **THEN** 输出顺序为 `.a { b: c; }` 然后 `.a .d { e: f; }`

#### Scenario: 声明在嵌套规则之后

- **WHEN** 输入 `.a { .d {e: f} g: h; }`
- **THEN** 输出顺序为 `.a .d { e: f; }` 然后 `.a { g: h; }`

### Requirement: 注释在声明中的位置处理

注释 SHALL 保留在声明中的正确位置，不因展平而丢失或移位。

#### Scenario: 声明前注释

- **WHEN** 输入 `a { /* comment */ b: c; }`
- **THEN** 输出 `a { /* comment */ b: c; }`

#### Scenario: 声明后值注释

- **WHEN** 输入 `a { b: c /* comment */; }`
- **THEN** 输出 `a { b: c /* comment */; }`
