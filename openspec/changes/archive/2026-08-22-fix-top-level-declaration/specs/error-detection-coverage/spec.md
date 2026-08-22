## ADDED Requirements

### Requirement: 顶层 CSS 声明检测

系统 SHALL 检测出现在文件顶层（非规则体内）的 CSS 声明并报错。系统 SHALL 在以下场景中检测并报错：裸 CSS 声明（`property: value`）出现在顶层、`@include` 在顶层调用 mixin 且 mixin 产生 CSS 声明输出。系统 SHALL 在 plain CSS 模式下跳过此检测。错误信息分别为 `expected "{".` 和 `Declarations may only be used within style rules.`。

#### Scenario: 顶层裸 CSS 声明
- **WHEN** `@import` 导入的文件中包含裸 CSS 声明 `a: b;`（不在规则体内），且非 plain CSS 模式
- **THEN** 系统 SHALL 报 `expected "{".` 错误

#### Scenario: 顶层 @include 产生 CSS 声明
- **WHEN** `@import` 导入的文件中 `@include` 在顶层调用 mixin，且 mixin body 包含 CSS 声明（如 `b: c`），且非 plain CSS 模式
- **THEN** 系统 SHALL 报 `Declarations may only be used within style rules.` 错误

#### Scenario: plain CSS 模式下顶层声明合法
- **WHEN** `.css` 文件中包含裸 CSS 声明 `a: b;` 在顶层
- **THEN** 系统 SHALL 不报错（plain CSS 模式跳过顶层声明检测）

#### Scenario: 规则体内的声明合法
- **WHEN** CSS 声明 `a: b;` 出现在规则体内（如 `foo { a: b; }`）
- **THEN** 系统 SHALL 不报错

#### Scenario: @include 在规则体内调用合法
- **WHEN** `@include` 在规则体内调用 mixin，且 mixin body 包含 CSS 声明
- **THEN** 系统 SHALL 不报错
