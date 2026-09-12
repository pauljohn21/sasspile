## ADDED Requirements

### Requirement: 嵌套 calc() 展开
Sass 编译器 SHALL 对 `calc(calc(...))` 形式的嵌套 calc 进行展开，移除外层 calc。

#### Scenario: 简单嵌套 calc
- **WHEN** 表达式为 `calc(calc(1px + 1%))`
- **THEN** 输出 SHALL 为 `calc(1px + 1%)`

### Requirement: 纯数字除法常量折叠
Sass 编译器 SHALL 对 `calc()` 中的纯数字除法运算执行常量折叠。

#### Scenario: 像素除法
- **WHEN** 表达式为 `calc(3px / 2 + 1%)`
- **THEN** 输出 SHALL 为 `calc(1.5px + 1%)`

### Requirement: 一元负号规范化
Sass 编译器 SHALL 对 `calc()` 中连续的一元负号进行规范化。

#### Scenario: 双负号
- **WHEN** 表达式为 `calc(1% - -1px)`
- **THEN** 输出 SHALL 为 `calc(1% + 1px)`
