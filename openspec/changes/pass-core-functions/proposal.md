## Why

`sass-spec/core_functions` 通过率仅 0.5%（38/7193），已成为 sass-spec 进度的瓶颈。Diagnostic 显示 fail 案例并非"复杂 Rust 架构问题"，而是集中在：(1) 一个 1 行可修的 panic，(2) 命名空间路由缺陷（`selector.append` 被派发到 `list.append`），(3) ~60 个 builtin 函数未在 `eval_builtin` match 名单中，(4) Parser 对 sass-spec 使用的部分 SCSS 语法（list/map 字面量、嵌套函数调用、keyword args）支持不完整。

修好后 core_functions 预期通过率从 0.5% 提升到 40-50%+（基于已有 alias 和 dispatch 机制的杠杆效应）。这不只是 sass-spec 数字 —— 这些 builtin（尤其是 `color.*、math.*`、`list.*`）是 Bootstrap/Element Plus 真实 SCSS 使用的函数子集。

## What Changes

- **Fix 1 行 panic**: `evaluate_dst/mod.rs` 函数调用参数扫描的 `i += 1` 添加越界保护
- **命名空间映射**: `module.fn(args)` 剥前缀时保留模块上下文。现有短名单名（`abs`, `quote`, `append`）存在冲突（`selector.append` vs `list.append`），改为统一用 `module-fn`（`selector-append`, `list-append`, `map-get`）规范名路由
- **扩充 builtin 注册表**: 补全 sass-spec 所需的 ~60 个函数，按模块批量实现：
  - `math`: `random, sin, cos, tan, asin, acos, atan, atan2, pow, sqrt, sqrt, log, hypot, clamp`
  - `color`: `adjust, change, scale, opacify, fade-in, transparentize, fade-out, ie-hex-str`
  - `string`: `str-insert, str-slice, str-split, str-to-lower-case, str-to-upper-case, str-unique-id, str-replace`
  - `list`: `set-nth, is-bracketed, zip`
  - `map`: `remove, deep-merge, deep-remove`
  - `selector`: `extend, parse, replace, unify, is-superselector, simple-selectors`
  - `meta`: `module-variables, module-functions, keywords, calc-args, calc-name`
  - `general`: 若干全局函数
- **Parser 增强**: 支持 list/map 字面量解析、嵌套函数调用、keyword argument
- **eval_builtin match 重构**: 用 `module-fn`（含连字符）规范名做顶层 dispatch，保留旧短名作为 fallback alias

## Capabilities

### New Capabilities

- `ns-routing-module-fn`: `module.fn(args)` 使用 `module-fn`（hyphenated）作为 canonical 名路由到 builtin
- `color-adjust-change-scale`: `color.adjust, color.change, color.scale, color.opacify, color.transparentize` 实现
- `math-advanced`: `math.random, math.sin, math.cos, math.tan, math.pow, math.sqrt, math.log, math.hypot, math.clamp, math.atan2` 实现
- `string-advanced`: `string.insert, string.slice, string.split, string.to-upper-case, string.to-lower-case, string.unique-id, string.replace` 实现
- `list-advanced`: `list.set-nth, list.zip, list.is-bracketed` 实现
- `map-advanced`: `map.remove, map.deep-merge, map.deep-remove` 实现
- `selector-advanced`: `selector.extend, selector.parse, selector.replace, selector.unify, selector.is-superselector, selector.simple-selectors` 实现
- `meta-advanced`: `meta.module-variables, meta.module-functions, meta.keywords, meta.calc-args, meta.calc-name` 实现

### Modified Capabilities

- `module-dot-evaluate`: **要求变更** — `substitute_vars` 必须生成 `module-fn`（hyphenated）而非仅 `fn`（stripped）作为 builtin dispatch key

## Impact

- **影响文件**: `src/evaluate_dst/mod.rs` (builtin 路由 + panic fix + namespace), `src/evaluate_dst/builtins.rs` (新函数实现), `src/parse_dst/directives.rs` (可能 list/map 解析), `src/evaluate_dst/eval_ctx.rs` (可能 helper)
- **测试影响**: `core_functions` HRX spec 预期 +4000~5000 cases；已有 62 个 cargo test 不能回归
- **依赖**: 不引入新依赖，全部在 `std` + 现有 `tracing` 内实现
- **风险**: 旧短名 alias（`abs`, `quote` 等）必须保留以维持 38 个已有 pass
