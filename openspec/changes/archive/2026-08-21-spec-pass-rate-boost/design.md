## Context

sasspile 当前 sass-spec 通过率 48%（2626/5624），全量扫描发现 1947 次 eval 失败。分析显示失败集中在 5 个根因类别，其中 3 类是功能缺失（meta 模块、模块成员访问、error 检测），2 类是逻辑 bug（参数验证、infinity 边界）。

现有架构：
```
Source → Lexer → Parser → Evaluator → Serializer → CSS
         (lex/)   (parse/)  (eval/)     (css/)
```

- `eval/builtin.rs`：内建函数分派入口，按 match arms 路由到各子模块
- `eval/module.rs`：模块限定函数调用映射（`math.abs` → `abs` 等）
- `eval/builtin/math.rs`：math 函数实现（412 行）
- `eval/builtin/selector.rs`：selector 函数实现
- `eval/mixin.rs`：mixin 定义和调用（285 行）

## Goals / Non-Goals

**Goals:**
- Phase 1：修复参数验证和边界处理，预期 +400~500 用例 → 通过率达 ~57%
- Phase 2：实现 meta 模块高级功能，预期 +300 用例 → 通过率达 ~62%
- Phase 3：修复 expected_error_but_ok 模式，预期 +80 用例 → 通过率达 ~64%
- Phase 4：values + css 深度修复，预期 +200 用例 → 通过率达 ~68%
- 每个 Phase 完成后核心测试保持 202/202 全通过
- 所有修改遵循单文件 ≤ 500 行限制

**Non-Goals:**
- 不修改颜色系统（颜色 spec 测试已跳过，需 color-f64-architecture-upgrade 后续处理）
- 不重写 VFS 或对照工具（spec-compliance-drive 的范畴）
- 不优化编译性能
- 不追求 100% 通过率

## Decisions

### D1: math 函数参数验证 — 命名参数展开而非位置检查

**问题**：`atan2 requires 2 number arguments`（72 次）、`sin/cos/tan/log/atan requires 1 argument`（各 20 次）。

**根因**：`call_module_function` 将模块限定调用（如 `math.atan2($y, $x)`）的参数传递给 `call_builtin`，但 math 函数的参数验证只检查 `pos_args.len()`，未考虑命名参数通过 `kw_args` 传递的情况。

**决策**：在 `math::call` 中合并 `pos_args` 和 `kw_args` 后再验证参数数量。与 `map` 函数的 `merge_map_args` 模式一致。

**替代方案**：在 `call_module_function` 层面展开命名参数 → 拒绝，因为不同函数对命名参数的处理不同，应在各函数模块内处理。

### D2: "Only 1 argument allowed" — CSS 函数透传修复

**问题**：`Only 1 argument allowed, but N were passed`（258 次）。

**根因**：`is_css_function` 判断后，CSS 原生函数被构造为 `Value::String(format!("{name}({arg_str})"))`，但 `arg_str` 使用 `join(", ")` 拼接所有参数。部分 CSS 函数（如 `clamp`、`min`、`max`）在 `call_builtin` 中有专门的 match arm，当传入多个参数时走到了参数验证分支而非 CSS 透传分支。

**决策**：`clamp`/`min`/`max` 在 CSS 上下文中（非 math 模块调用）应透传为 CSS 函数。在 `call_builtin` 中增加判断：当参数包含非 Number 类型（如 `Calc`、`String`）时，走 CSS 透传路径。

### D3: calc(infinity) 边界处理 — 特殊值识别

**问题**：`$base: calc(infinity) is not a number`（28+28+24+24=104 次）。

**根因**：`pow` 函数检查 `$base` 是否为 `Value::Number`，但 `calc(infinity)` 是 `Value::Calc` 类型。sass-spec 期望 `pow(calc(infinity), 1)` 返回 `calc(infinity)`。

**决策**：在 `pow`/`div`/`sqrt` 等函数中增加 `Value::Calc` 分支，识别 `calc(infinity)`/`calc(-infinity)`/`calc(NaN)` 字符串并返回对应的 Calc 值。不将 Calc 转为 Number，而是让数学函数直接处理 Calc 特殊值。

### D4: meta.load-css — mixin 实现而非函数

**问题**：`Undefined mixin: meta.load-css`（106 次）。

**决策**：`meta.load-css` 是 mixin 而非函数。在 `eval/mixin.rs` 中新增 `load_css` mixin 实现：
- 接收 `$module` 或 `$url` 参数
- 调用 `load_module` 加载指定模块
- 将模块的 CSS 输出注入当前上下文
- 支持 `$with` 配置参数

在 `eval/mod.rs` 的 mixin 调用路径中增加 `meta.load-css` 的识别。

### D5: meta.get-mixin + meta.apply — mixin 引用传递

**问题**：`Undefined function: meta.get-mixin`（74 次）、`Undefined mixin: meta.apply`（58 次）。

**决策**：
- `meta.get-mixin($name, $module: null)` 返回 `Value::MixinRef`（新增值类型），包含 mixin 名称和来源模块
- `meta.apply($mixin, $args...)` 接收 `MixinRef` 并调用对应 mixin
- 需要在 `Value` 枚举中新增 `MixinRef` 变体
- `meta.apply` 作为 mixin 而非函数实现（因为它接收 `@content`）

**替代方案**：用 `Value::String` 存储 mixin 名称 → 拒绝，无法区分字符串和 mixin 引用。

### D6: 模块成员变量导出 — Env 扩展

**问题**：`Undefined variable: $ns.name`（54+ 次）。

**根因**：`eval/mod.rs` 的 `lookup_var` 只在 `env.vars` 中查找，未检查 `env.namespaces` 中的模块变量。

**决策**：在 `lookup_var` 中增加命名空间查找：当变量名包含 `.` 时，拆分为 `ns` 和 `name`，在 `env.get_namespace(ns)` 的 `vars` 中查找。与 `call_module_function` 的命名空间查找逻辑一致。

### D7: meta.module-functions/mixins/variables — 反射 API

**问题**：`Undefined function: meta.module-functions`（10+14+35=59 次）。

**决策**：实现三个反射函数：
- `meta.module-functions($module)` → 返回 map: {name: function-ref}
- `meta.module-mixins($module)` → 返回 map: {name: mixin-ref}
- `meta.module-variables($module)` → 返回 map: {name: value}

需要 `Value::Module` 类型或从 `env.namespaces` 获取模块导出。参数 `$module` 是通过 `meta.get-mixin` 或 `@use` 获得的模块引用。

### D8: expected_error_but_ok — 逐子目录修复

**决策**：不统一处理，而是按子目录逐个修复：
- `expressions`：在 parser 中增加 `not`/`and`/`or` 后无有效表达式的错误检测
- `selector`：在 `selector-append` 中增加无效选择器类型检测
- `map`：在 `map-deep-merge`/`map-remove` 中增加参数类型检查
- `@use`/`@forward`：在模块加载时检测同名 conflict

### D9: infinity/nan 序列化 — CSS Values Level 4

**问题**：`values/numbers` 中 20 次失败，infinity/nan 输出格式不正确。

**决策**：序列化 `infinity` 为 `infinity`（无单位），`-infinity` 为 `-infinity`，`nan` 为 `NaN`。与 CSS Values Level 4 规范一致。在 `parse/ast/display.rs` 的 Number Display 实现中增加特殊值分支。

## Risks / Trade-offs

- **[Value::MixinRef 新增变体]** → 需要在所有 match arms 中处理新类型，可能遗漏 → 在 `Value` 的 `Display`、`PartialEq`、`is_truthy` 中添加默认处理
- **[meta.load-css 递归加载]** → 可能导致无限循环 → 增加加载深度限制（已有 `depth` 字段）
- **[参数验证放宽]** → 可能导致本应报错的用例通过 → 通过 `expected_error_but_ok` 测试用例验证修复不会过度放宽
- **[Calc 字符串解析]** → `calc(infinity)` 的字符串匹配可能不够健壮 → 使用 starts_with/ends_with 而非精确匹配
- **[单文件行数限制]** → `eval/builtin.rs`（471行）和 `eval/mixin.rs`（285行）可能超限 → 新增功能拆分到新文件（如 `eval/meta_ops.rs`）
