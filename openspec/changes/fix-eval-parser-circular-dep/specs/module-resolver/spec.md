## ADDED Requirements

### Requirement: 模块解析与 eval 层解耦

系统 SHALL 引入 `ModuleResolver` trait，将 `@use`/`@import` 的文件加载、tokenize、parse 职责从 eval 层解耦。eval 层 MUST NOT 直接调用 `crate::parser::parse` 或 `crate::lexer::tokenize`。

#### Scenario: eval 使用 trait 回调解析模块文件

- **WHEN** eval 层遇到 `@use "module"` 或 `@import "module"` 指令
- **THEN** 系统 MUST 通过 `ModuleResolver` trait 回调执行文件加载和解析，而非直接调用 `crate::parser::parse`
- **AND** eval 层 MUST NOT 直接依赖 parser 模块

#### Scenario: compile 入口创建 resolver 实例

- **WHEN** 用户调用 `compile()` 或 `compile_file()` 入口
- **THEN** 系统 MUST 创建一个 `ModuleResolver` 实例并传递给 evaluate 层
- **AND** 该实例 MUST 封装 tokenize → parse → 返回 AST 的完整流程

#### Scenario: 插值求值不再直接调用 parser

- **WHEN** 插值表达式中包含复杂表达式需要重新解析
- **THEN** 系统 MUST 通过 `ModuleResolver` 或预解析的 AST 完成求值
- **AND** `eval_interpolation_expr` 函数 MUST NOT 直接调用 `crate::lexer::tokenize` 或 `crate::parser::Parser`

### Requirement: 模块缓存机制

`ModuleResolver` SHALL 提供模块缓存，避免同一文件被重复加载和解析。

#### Scenario: 同一模块多次 @use 只解析一次

- **WHEN** 多个文件 `@use "common"` 引用同一模块
- **THEN** 系统 MUST 只 tokenize + parse 该模块一次
- **AND** 后续 `@use` MUST 从缓存中获取已解析的 AST

#### Scenario: 循环引用检测

- **WHEN** 模块 A `@use` 模块 B，模块 B 又 `@use` 模块 A
- **THEN** 系统 MUST 发出 `warn!` 级别 tracing 日志并跳过循环引用
- **AND** 系统 MUST NOT 无限递归或 panic
