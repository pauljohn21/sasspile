## ADDED Requirements

### Requirement: @use with 配置注入

系统 SHALL 支持 `@use "module" with ($var: value, ...)` 语法，在子模块求值前将配置变量注入到模块的环境中。

#### Scenario: 单个配置变量注入

- **WHEN** SCSS 源码包含 `@use "module" with ($primary: #336699)`
- **THEN** 系统 MUST 在解析模块文件前，将 `$primary` 设置为 `#336699` 注入到模块的求值环境中
- **AND** 模块内部代码 MUST 能正常引用 `$primary` 变量

#### Scenario: 多个配置变量注入

- **WHEN** SCSS 源码包含 `@use "module" with ($color: red, $size: 16px)`
- **THEN** 系统 MUST 将所有配置变量注入到模块环境中
- **AND** 模块内部的 `!default` 变量 MUST 被配置值覆盖（即配置值优先于 `!default`）

#### Scenario: 配置变量引用已有变量

- **WHEN** SCSS 源码包含 `@use "module" with ($primary: $theme-color)` 且 `$theme-color` 已定义
- **THEN** 系统 MUST 在调用者环境中先求值 `$theme-color`，再将结果注入到模块环境

#### Scenario: 未配置的 !default 变量保持默认值

- **WHEN** 模块内部有 `$gap: 10px !default;` 且 `@use` 未配置 `$gap`
- **THEN** 系统 MUST 保持 `$gap` 的默认值 `10px`

### Requirement: Env 作用域安全

系统 SHALL 使用 `with_child_scope` 闭包模式管理 eval 层的作用域，替代当前的 `std::mem::replace` + `take().unwrap()` 模式。

#### Scenario: 函数调用使用闭包作用域

- **WHEN** eval 层调用用户自定义函数 `call_user_function`
- **THEN** 系统 MUST 使用 `Env::with_child_scope` 创建子作用域
- **AND** 无论函数执行成功还是 panic，父环境 MUST 保持不变

#### Scenario: mixin include 使用闭包作用域

- **WHEN** eval 层执行 `@include` 调用 mixin
- **THEN** 系统 MUST 使用 `Env::with_child_scope` 创建子作用域
- **AND** mixin 执行完毕后，环境 MUST 恢复为调用前的状态
