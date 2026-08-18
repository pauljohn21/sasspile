## Why

sasspile 的 eval 层在 `@use`、`@import` 和插值求值时直接调用 `crate::parser::parse` 和 `crate::lexer::tokenize`，形成了 eval → parser 的循环依赖。这违反了 Clean Architecture 的 `dep-acyclic-dependencies` 规则，导致：

1. 模块无法独立测试——测试 eval 必须连带 parser
2. 每次插值都重新 tokenize + parse，性能损失
3. `@use` 的 `with ($var: value)` 配置被完全忽略（`_config` 参数未使用），导致 Bootstrap 等真实项目编译失败

同时，`Env` 使用 `std::mem::replace` + `take().unwrap()` 管理作用域，panic 时会破坏环境链不变量。

## What Changes

- 引入 `ModuleResolver` trait，将文件加载/解析职责从 eval 层提取到独立模块
- `eval_use_rule` 和 `eval_import_rule` 通过 trait 回调调用 parser，而非直接 `crate::parser::parse`
- 实现 `@use` 的 `with (...)` 配置注入——在子模块求值前设置配置变量
- 引入 `ModuleCache` 求值结果缓存——保证每个模块只求值一次（Sass spec 要求），后续 `@use` 复用第一次的求值结果
- 实现 `@forward` 指令——将目标模块的公开成员转发给当前模块的使用者，支持 `show`/`hide` 过滤
- `Env` 改用 `with_child_scope` 闭包模式，保证作用域不变量
- 插值求值 `eval_interpolation_expr` 中的重新 tokenize+parse 路径改为预解析 AST 传递

## Non-goals

- 不重写整个编译管道架构
- 不实现 `@forward ... as` 前缀重命名（后续迭代）
- 不修改 `@extend` 的字符串匹配问题（单独变更处理）
- 不修改 `color.rs` 行数超限问题（单独变更处理）

## Capabilities

### New Capabilities

- `module-resolver`: 模块解析能力——将 `@use`/`@import` 的文件加载、tokenize、parse 职责从 eval 层解耦，通过 trait 注入，消除循环依赖
- `use-config`: `@use with` 配置注入能力——支持 `@use "module" with ($var: value)` 语法，在子模块求值前注入配置变量
- `module-cache`: 模块求值结果缓存能力——保证每个模块只求值一次，后续 `@use` 复用第一次的求值结果（vars/funcs/mixins/CSS输出），符合 Sass spec 语义
- `forward-rule`: `@forward` 指令能力——将目标模块的公开成员转发给当前模块的使用者，支持 `show`/`hide` 成员过滤

### Modified Capabilities

（无现有 spec 需要修改）

## Impact

- **`src/eval/mod.rs`** — `eval_use_rule` 重构为通过 trait 调用 + ModuleCache 检查；新增 `eval_forward_rule` 实现
- **`src/eval/atrule.rs`** — `eval_import_rule` 重构为通过 trait 调用
- **`src/eval/interp.rs`** — `eval_interpolation_expr` 消除直接 tokenize+parse 调用
- **`src/env.rs`** — `Env` 新增 `with_child_scope` 方法，替换 `mem::replace` 模式
- **`src/eval/expr.rs`** — `call_user_function` 使用新作用域方法
- **`src/lib.rs`** — `compile`/`compile_file` 创建 `ModuleResolver` 实例传入 eval
- **新增 `src/resolver/mod.rs`** — `ModuleResolver` trait + `FileResolver` 默认实现
- **新增 `src/eval/module_cache.rs`** — `ModuleCache` + `EvaluatedModule` 结构体
