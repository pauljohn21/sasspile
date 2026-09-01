## Why

sasspile 当前 sass-spec 通过率 3068/5362 = 57%，2294 个失败。经诊断分析，失败集中在六大类：空行处理（~300-400）、参数验证过严（~300-400）、内建函数缺失（~200-300）、输出格式差异（~200-300）、plain CSS 错误检测（~120）、模块系统（~100）。这些都是逻辑修复而非架构变更，且全部在 Rust move 语义下可解，不需要 GC 或共享引用。

## What Changes

### Phase 1 — 序列化空行修复（预计 +300~400 PASS）

- 修复 `serialize_expanded` 在展平的同选择器规则间多加空行的问题
  - `.a { b:c; .d {e:f} }` 展平后 `.a {b:c} .a .d {e:f}` 之间不应有空行
  - 当前 `serialize_expanded` 顶层规则间一律加空行，但 sass-spec 期望同源规则不加空行
- 修复声明穿插（interleaved declarations）的输出顺序
  - `.a { b:c; .d {e:f} g:h }` 期望 `.a {b:c} .a .d {e:f} .a {g:h}`（三段无空行）
- 修复注释在声明中的位置处理

### Phase 2 — 参数验证修复（预计 +300~400 PASS）

- 修复 `merge_args` / `merge_math_args`：命名参数不应计为位置参数
  - 消除 "Only 1 argument allowed, but 2 were passed" 误报
- 修复 `if` 函数参数验证（`if requires 3 arguments` 误报）
- 修复 `rgba` 接受 3-4 number 参数
- 修复 `set-nth` 参数验证
- 字符串到数字的隐式转换（`"0"` → `0.0`）

### Phase 3 — 内建函数补全（预计 +200~300 PASS）

- 注册 `string.str-insert` 等模块限定名函数
- 修复 `utils.a` mixin/function 解析（callable 目录 24 fail）
- 修复 `str-index` / `str-slice` 参数类型强制转换
- Calc/字符串拼接运算符支持

### Phase 4 — 输出格式对齐（预计 +200~300 PASS）

- 修复数值精度和格式化（degenerate/infinity 单位等）
- 修复选择器排序差异
- 修复 `@media` / `@supports` 合并规则
- 修复 `has` / `global` / `deep_remove` 等函数的 extra_output / missing_output

### Phase 5 — plain CSS 错误检测（预计 +120 PASS）

- 修复 `expected_error_but_ok` 场景（error/complex, error/compound, error/no_selector 等）
- 修复 plain CSS 中 `@-moz-document`、`url-prefix` 等解析
- 增强错误消息对齐

### Phase 6 — 模块系统修复（预计 +100 PASS）

- 修复 `@use` module loop 检测
- 修复 `@use with` 配置验证
- 修复 `@import` 35 fail 中的冲突检测

## Capabilities

### New Capabilities

- `serialization-whitespace`: 序列化器空行处理规则——展平的同源规则间不加空行，穿插声明顺序正确
- `arg-validation-fix`: 内建函数参数验证修复——命名参数不计位置参数，if/rgba/set-nth 参数数验证
- `builtin-coverage`: 内建函数补全——string.str-insert 注册、utils.a 解析、类型强制转换
- `output-format-alignment`: 输出格式对齐——数值精度、选择器排序、@media 合并
- `plain-css-error`: plain CSS 错误检测增强——expected_error_but_ok 场景修复
- `module-system-fix`: 模块系统修复——module loop 检测、@use with 配置验证

### Modified Capabilities

## Impact

- **src/css/mod.rs** — `serialize_expanded` 空行逻辑、`flatten_nodes` 展平规则
- **src/css/selector.rs** — 选择器排序
- **src/eval/builtin/dispatch.rs** — 函数注册表补全
- **src/eval/builtin/math_helpers.rs** — `merge_math_args` 修复
- **src/eval/builtin/string.rs** — `str-insert` / `str-index` 参数验证
- **src/eval/builtin/list.rs** — `set-nth` 参数验证
- **src/eval/builtin/math.rs** — infinity 参数接受、数值精度
- **src/eval/builtin/selector.rs** — selector 函数参数展开
- **src/eval/builtin/map.rs** — `has` / `deep_remove` 修复
- **src/eval/value/mod.rs** — 类型强制转换、运算符支持
- **src/eval/value/ops.rs** — Calc/字符串拼接
- **src/eval/value/display.rs** — 数值格式化
- **src/eval/module.rs** — module loop 检测
- **src/eval/plain_css.rs** — plain CSS 错误检测
- **src/eval/error_msgs.rs** — 错误消息对齐
- **无 BREAKING 变更**：所有修复都是让输出匹配 sass-spec，不改变已通过测试的行为
