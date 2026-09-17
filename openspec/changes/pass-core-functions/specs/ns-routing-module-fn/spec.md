# ns-routing-module-fn Specification

## ADDED Requirements

### Requirement: substitute_vars 使用 module-fn（hyphenated）作为 builtin dispatch canonical key

`substitute_vars` SHALL 在识别到 `module.fn(args)` 形式的函数调用时，生成 `module-fn`（hyphenated）格式的 canonical key 传递给 `eval_builtin`，以保持模块上下文并避免跨模块函数名冲突。

#### Scenario: selector.append 路由正确
- **WHEN** 输入为 `@use "sass:selector"; a {b: selector.append(".c", ".d")}`
- **THEN** `eval_builtin` 以 `"selector-append"` 为 name 被调用（非 `"append"`）

#### Scenario: list.append 与 selector.append 不冲突
- **WHEN** 同一上下文中同时调用 `list.append` 与 `selector.append`
- **THEN** 两者分别以 `"list-append"` 和 `"selector-append"` 命中不同的 eval 分支

#### Scenario: 无前缀函数名 fallback 保留
- **WHEN** 输入为 `a {b: abs(-5px)}`（无模块前缀）
- **THEN** 仍以 `"abs"` 单短名路由，行为不变

### Requirement: eval_builtin match 接受连字符规范名

`eval_builtin` SHALL 使用 `module-fn`（连字符分隔）名单作为顶层 dispatch，保留旧短名作为 alias fallback。若两个 key 指向不同实现（如 `selector-append` vs `list-append`），必须独立报出各自结果。

#### Scenario: map.get 路由
- **WHEN** 输入含 `map.get($m, k)`
- **THEN** `eval_builtin` 以 `"map-get"` 命中 builtin_map_get

#### Scenario: math.abs 路由
- **WHEN** 输入含 `math.abs(-5)`
- **THEN** `eval_builtin` 以 `"math-abs"` 命中 builtin_math

### Requirement: 旧短名 alias 必须保留

从 38 个已通过的 sass-spec 案例派生的短名单名（`abs, quote, alpha, red, green, blue, round, ceil, floor, min, max, type-of, inspect, nth, length, append, join, index, get, has-key, keys, values, merge, unquote, percentage, unit, unitless, comparable, variable-exists, global-variable-exists, not, percentage`）SHALL 继续工作。所有 38 个现有通过案例不得回归。

#### Scenario: 全量回归
- **WHEN** 跑 `cargo test --test sass_spec_detail`
- **THEN** core_functions 计数不低于 40/7793（不因重构路由而下降）
