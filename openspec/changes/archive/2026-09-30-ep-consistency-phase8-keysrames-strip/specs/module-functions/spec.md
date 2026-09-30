## MODIFIED Requirements

### Requirement: module 相关内建函数的规范行为

core_functions/modules 目录下的函数 MUST 符合 Sass 模块系统规范。模块变量 MUST 在嵌套 `@include` 调用中保持可见性，不受 `Env` 作用域进出影响。

#### Scenario: @use 配置覆盖默认变量
- **WHEN** 在模块中定义 `!default` 变量并在 `@use` 时传入 `$var: value`
- **THEN** 变量值 MUST 被正确覆盖

#### Scenario: @use namespace 变量访问
- **WHEN** 在 `@use 'module' as *` 后用全局名访问变量
- **THEN** 变量 MUST 可正确引用

#### Scenario: @forward 传递变量
- **WHEN** 在 `@forward` 后通过链式 use 访问被转发的变量
- **THEN** 变量引用 MUST 可达

#### Scenario: meta.module-list 返回可用模块
- **WHEN** 调用 `meta.module-list()`
- **THEN** MUST 返回当前加载的所有模块名列表

#### Scenario: Module variable visible through nested @include
- **WHEN** module A `@use`s module B with `$var`, and module C `@include`s a mixin from A that references `$var` through nested rule scope
- **THEN** `$var` MUST 在嵌套 `@include` 的任意层级中可解析，输出值与 EP reference 一致
