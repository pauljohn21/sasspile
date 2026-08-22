## Why

sasspile 当前 sass-spec 通过率 51%（2774/5362），全量扫描发现 1947 次 eval 失败集中在少数几个根因：meta 模块功能缺失（297 次）、参数验证过严（258+192 次）、模块成员变量未导出（54+ 次）、calc(infinity) 边界处理缺失（104 次）、expected_error_but_ok 模式广泛存在（80+ 次）。这些问题大多不需要架构变更，而是功能补全和参数验证逻辑修复，属于低 hanging fruit。

**Phase 1 已完成**（+149 用例，48%→51%）：math 参数验证、selector 命名参数、builtin 模块变量导出、plain CSS 限制检测。

## What Changes

- **Phase 1 — 参数验证 & 边界处理修复**（预计 +400~500 用例）
  - 修复 math 函数参数验证逻辑（`atan2`/`sin`/`cos`/`tan`/`log`/`atan`/`asin`/`acos`/`sqrt`/`clamp` 共 192 次失败）
  - 修复 "Only 1 argument allowed, but N were passed" 参数展开逻辑（258 次失败）
  - 实现 `calc(infinity)`/`calc(-infinity)` 在 `pow` 函数中的边界处理（104 次失败）
  - 修复 `selector-parse`/`selector-extend`/`selector-replace` 参数处理（82 次失败）
  - 修复 plain CSS 中 `sass()` 条件和插值限制（30 次失败）

- **Phase 2 — meta 模块功能实现**（预计 +300 用例）
  - 实现 `meta.load-css` mixin（106 次失败）— 动态加载 CSS 文件
  - 实现 `meta.get-mixin` 函数（74 次失败）— 获取 mixin 引用
  - 实现 `meta.apply` mixin（58 次失败）— 动态调用 mixin
  - 实现 `meta.module-functions`/`meta.module-mixins`/`meta.module-variables` 反射函数（59 次失败）
  - 实现模块成员变量导出（`$ns.var` 访问，54+ 次失败）

- **Phase 3 — expected_error_but_ok 修复**（预计 +80 用例）
  - 表达式语法错误检测（`not`/`and`/`or`/空括号等，8 次）
  - selector 错误检测（无效选择器 append，9 次）
  - map 类型检查和重复键检测（4 次）
  - `@use`/`@forward` conflict 检测（多次）

- **Phase 4 — values + css 深度修复**（预计 +200 用例）
  - `values/numbers`：infinity/nan 序列化格式（20 次失败）
  - `values/lists`：分隔符和括号处理
  - `css/plain`：插值限制、CSS 原生函数透传
  - `css/media`：查询解析

## Capabilities

### New Capabilities
- `meta-module-functions`: `sass:meta` 模块的高级功能——`meta.load-css` mixin、`meta.get-mixin` 函数、`meta.apply` mixin、`meta.module-functions`/`meta.module-mixins`/`meta.module-variables` 反射函数
- `module-member-access`: `@use`/`@forward` 模块的成员变量导出与访问（`$ns.var` 语法），模块成员反射
- `param-validation-fix`: 内置函数参数验证逻辑修正——math 函数参数数量检查、selector 函数参数展开、"Only 1 argument allowed" 修复
- `calc-infinity-handling`: `calc(infinity)`/`calc(-infinity)`/`calc(NaN)` 在数学函数（pow/div/sqrt 等）中的边界处理和序列化
- `error-detection-coverage`: spec 期望的编译错误检测——表达式语法错误、selector 无效输入、map 类型检查、`@use`/`@forward` conflict 检测

### Modified Capabilities
（无现有 spec 需要修改）

## Impact

- **`src/eval/builtin.rs`** — 参数验证逻辑修正、新增 meta 模块函数分派
- **`src/eval/module.rs`** — 新增 `meta.load-css`/`meta.get-mixin`/`meta.apply` 映射、模块成员变量导出
- **`src/eval/builtin/math.rs`** — 参数数量验证修复、infinity 边界处理
- **`src/eval/builtin/selector.rs`** — 参数展开修复
- **`src/eval/builtin/string.rs`** — 参数验证修复
- **`src/eval/builtin/list.rs`** — 参数验证修复
- **`src/eval/mod.rs`** — 模块变量查找逻辑、error 检测增强
- **`src/eval/mixin.rs`** — `meta.apply` mixin 实现、`meta.load-css` mixin 实现
- **`src/parse/expr/mod.rs`** — 表达式语法错误检测增强
- **`src/parse/expr/prefix.rs`** — 模块成员变量解析
- **`src/css/mod.rs`** — plain CSS 插值限制
- **`src/parse/ast/display.rs`** — infinity/nan 序列化
