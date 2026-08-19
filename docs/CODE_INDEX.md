# sasspile 代码索引

> 查找函数/类型/概念在哪个文件？查这张表。
>
> **动态查询**（调用者/被调用者/影响分析/源码查看）请用 CodeGraph：
> ```bash
> codegraph node eval_node           # 查看符号源码 + 调用链路
> codegraph callers apply_extends   # 谁调了这个函数？
> codegraph impact eval_value       # 修改影响范围分析
> codegraph explore "color conversion"  # 探索某领域
> ```
> 详见 [`AGENTS.md`](../AGENTS.md#codegraph-代码导航) CodeGraph 章节。

## 文件 → 职责速查

| 文件 | 行数 | 职责 |
|------|------|------|
| **lib.rs** | 334 | 公共 API（compile/compile_expanded/compile_compressed/compile_file/compile_file_with_load_paths）+ init_tracing |
| **main.rs** | 32 | CLI 入口 |
| **error.rs** | 95 | SassError 定义（全英文错误消息） |
| **lex/token.rs** | 170 | Token 枚举定义 + Display impl（含重新转义） |
| **lex/mod.rs** | 499 | Lexer + Iterator impl（scan_* 方法）+ scan_escape_ident 返回 Result |
| **parse/ast/mod.rs** | 457 | AST 类型定义（Node, Value, Color, BinOp, Param, Arg, VarFlags, ConfigVar, ColorFormat 等）+ escape_quoted_string/escape_css_ident/escape_css_chars + hsl_to_rgb_percent/format_pct_val/format_hue/format_pct/format_alpha |
| **parse/ast/display.rs** | 253 | Display for Value（ColorFormat 分派序列化） |
| **parse/ast_impl.rs** | 281 | Node::to_scss() |
| **parse/mod.rs** | 92 | Parser 结构 + parse() 入口 + 基础操作（peek/advance/skip_w/expect） |
| **parse/nodes.rs** | 668 | parse_node/parse_rule/parse_decl/parse_variable/parse_body + parse_params/parse_args + is_namespace_var/parse_namespace_var + parse_config (ConfigVar) |
| **parse/at_rules.rs** | 513 | 所有 @ 规则解析（@if/@for/@each/@while/@mixin/@include/@function/@use/@forward/@import/@extend/@at-root/@warn/@debug/@error）+ @import 多值/修饰符解析 |
| **parse/expr/mod.rs** | 177 | Pratt 表达式解析入口 + parse_number/parse_hash_color |
| **parse/expr/prefix.rs** | 465 | Pratt 前缀解析 + parse_prefix/peek_binding_power/parse_value_start |
| **eval/mod.rs** | 589 | Env + ModuleExports + MixinDef + FunctionDef + Evaluator + evaluate/eval_nodes/eval_node + global_writes 字段 + loaded_modules 缓存 + extends 传播 + @import 多值/修饰符处理 |
| **eval/rule.rs** | 197 | eval_rule + combine_selectors（规则体变量作用域隔离，传播命名空间/!global/@import 变量） |
| **eval/value/mod.rs** | 464 | eval_value + eval_binop + add/sub/mul/div/modulo/compare + values_eq + inspect_value + eval_interp_str + units_compatible + 命名空间变量赋值 |
| **eval/value/ops.rs** | 209 | 值运算实现（add/sub/mul/div/modulo/compare 细节） |
| **eval/value/display.rs** | 244 | inspect_value + 值显示格式化 |
| **eval/control_flow.rs** | 149 | eval_if/eval_for/eval_each/eval_while |
| **eval/mixin.rs** | 272 | eval_include + bind_params + call_function + call_user_function + eval_at_root + eval_at_rule + is_truthy |
| **eval/extend.rs** | 76 | apply_extends（跨模块 @extend 传播 + 选择器合并 + 占位符替换） |
| **eval/at_params.rs** | 240 | @media/@supports 参数插值和表达式求值 |
| **eval/module.rs** | 305 | resolve_file（含 load_paths） + load_module（含模块缓存 loaded_modules） + load_import + call_module_function（注入模块 vars 到函数环境 + 命名空间变量赋值） |
| **eval/color.rs** | 617 | hsl_to_rgb/hwb_to_rgb/rgb_to_hsl + builtin_rgba/builtin_darken/builtin_lighten/builtin_mix + simple_random |
| **eval/builtin.rs** | 461 | call_builtin 分派入口（match 骨架 → 子模块分派）+ is_known_builtin + is_css_function + meta 函数（inspect/calc-args/calc-name 等） |
| **eval/builtin/math.rs** | 395 | abs/ceil/floor/round/min/max/percentage/div/pow/sqrt/sin/cos/tan/atan2/asin/acos/atan/hypot/log/random/clamp/unit/is-unitless/compatible/comparable + merge_math_args 命名参数合并 + 参数验证（Missing argument / Only N arguments / $number: X is not a number） |
| **eval/builtin/color.rs** | 623 | invert/grayscale/color-channel/hwb/complement/hsl/hsla/adjust-hue/saturate/desaturate/transparentize/opacify/alpha/red/green/blue/hue/saturation/lightness + adjust-color/change-color/scale-color + is-powerless/is-in-gamut/is-legacy + is_channel_powerless |
| **eval/builtin/list.rs** | 303 | length/nth/append/join/index/separator/set-nth/is-bracketed/list-slash/zip |
| **eval/builtin/map.rs** | 302 | map-get/keys/values/has-key/merge/remove/set/deep-remove + value_to_map/nested_map_merge/nested_map_set |
| **eval/builtin/string.rs** | 376 | str-length/to-upper-case/to-lower-case/unquote/quote/str-slice/str-index/str-insert/str-split/unique-id + 参数验证（$string: X is not a string） |
| **eval/builtin/selector.rs** | 110 | selector-append/nest/is-super/parse/simple-selectors/unify/extend |
| **css/mod.rs** | 359 | Serializer（CSS 树 → 字符串，选择器净化 + 组合器验证 + @规则合并 + @import 间不加空行） |
| **css/node.rs** | 93 | CssNode 枚举（Rule/Declaration/AtRule/AtRoot/Comment/Raw/Return） |
| **css/selector.rs** | 366 | sanitize_selector + normalize_attr_selectors + has_bogus_combinators + 占位符处理 |
| **stage/*.rs** | 14-89 | 管线阶段类型（Source/Lexed/Parsed/Evaluated/Serialized） |

## 函数 → 文件定位

### 求值器（Evaluator）

| 函数 | 文件 |
|------|------|
| `evaluate` / `evaluate_with_path` / `evaluate_with_path_and_load_paths` | `eval/mod.rs` |
| `eval_nodes` / `eval_node` | `eval/mod.rs` |
| `eval_rule` / `combine_selectors` | `eval/rule.rs` |
| `eval_value` / `eval_binop` / `units_compatible` | `eval/value/mod.rs` |
| `add` / `sub` / `mul` / `div` / `modulo` / `compare` | `eval/value/ops.rs` |
| `values_eq` / `inspect_value` / `is_truthy` | `eval/value/mod.rs` |
| `eval_interp_str` / `eval_simple_expr` | `eval/value/mod.rs` |
| `eval_if` / `eval_for` / `eval_each` / `eval_while` | `eval/control_flow.rs` |
| `eval_include` / `bind_params` / `call_function` | `eval/mixin.rs` |
| `call_user_function` / `eval_at_root` / `eval_at_rule` | `eval/mixin.rs` |
| `apply_extends` | `eval/extend.rs` |
| `resolve_file` / `load_module` / `load_import` / `call_module_function` | `eval/module.rs` |
| `call_builtin` | `eval/builtin.rs` |
| `merge_math_args` / `math_param_names` | `eval/builtin/math.rs` |
| math::call (abs/ceil/floor/round/div/pow/clamp/...) | `eval/builtin/math.rs` |
| `call_map_builtin` / `value_to_map` / `nested_map_merge` / `nested_map_set` | `eval/builtin/map.rs` |
| `call_string_builtin` / `str_slice` / `str_insert` / `str_split` | `eval/builtin/string.rs` |
| `hsl_to_rgb` / `hwb_to_rgb` / `rgb_to_hsl` / `simple_random` | `eval/color.rs` |
| `builtin_rgba` / `builtin_darken` / `builtin_lighten` / `builtin_mix` | `eval/color.rs` |
| `hsl_to_rgb_percent` / `format_pct_val` / `format_hue` / `format_pct` / `format_alpha` | `parse/ast/mod.rs` |
| `escape_quoted_string` / `escape_css_ident` / `escape_css_chars` | `parse/ast/mod.rs` |
| `is_channel_powerless` | `eval/builtin/color.rs` |
| `sanitize_selector` / `normalize_attr_selectors` / `has_bogus_combinators` | `css/selector.rs` |

### 解析器（Parser）

| 函数 | 文件 |
|------|------|
| `parse` / `parse_node` / `is_rule` | `parse/nodes.rs` |
| `parse_rule` / `parse_rule_or_decl` / `parse_selector` | `parse/nodes.rs` |
| `parse_decl` / `parse_property` / `check_important` | `parse/nodes.rs` |
| `parse_variable` / `parse_var_flags` / `parse_body` | `parse/nodes.rs` |
| `is_namespace_var` / `parse_namespace_var` / `parse_config` | `parse/nodes.rs` |
| `parse_params` / `parse_args` / `parse_config` / `parse_member_list` | `parse/nodes.rs` |
| `parse_at_rule` / `parse_if` / `parse_for` / `parse_each` | `parse/at_rules.rs` |
| `parse_while` / `parse_mixin_def` / `parse_include` / `parse_function_def` | `parse/at_rules.rs` |
| `parse_return` / `parse_use` / `parse_forward` / `parse_import` | `parse/at_rules.rs` |
| `parse_extend` / `parse_at_root` / `parse_warn` / `parse_debug` / `parse_error` | `parse/at_rules.rs` |
| `parse_generic_at_rule` / `parse_at_params` | `parse/at_rules.rs` |
| `parse_expr` / `is_value_start` / `peek_binding_power` | `parse/expr/mod.rs` |
| `parse_prefix` / `parse_number` / `parse_hash_color` / `hex2` / `hex1` | `parse/expr/prefix.rs` |

### 词法分析器（Lexer）

| 函数 | 文件 |
|------|------|
| `new` / `next` (Iterator impl) | `lex/mod.rs` |
| `scan_ident` / `scan_number` / `scan_string` | `lex/mod.rs` |
| `scan_interp` / `scan_at` / `scan_dollar` / `scan_hash` | `lex/mod.rs` |
| `scan_escape_ident` | `lex/mod.rs` |
| `scan_line_comment` / `scan_block_comment` | `lex/mod.rs` |

## 类型 → 文件定位

| 类型 | 定义位置 |
|------|----------|
| `Node` (AST 节点枚举) | `parse/ast/mod.rs` |
| `Value` (值枚举) | `parse/ast/mod.rs` |
| `Color` | `parse/ast/mod.rs` |
| `ColorFormat` (Auto/Rgb/RgbPercent/Hsl/Hwb) | `parse/ast/mod.rs` |
| `BinOp` / `BinOpKind` | `parse/ast/mod.rs` |
| `Separator` | `parse/ast/mod.rs` |
| `Ast` | `parse/ast/mod.rs` |
| `Param` / `Arg` / `VarFlags` / `ConfigVar` | `parse/ast/mod.rs` |
| `Token` | `lex/token.rs` |
| `Lexer` | `lex/mod.rs` |
| `Parser` | `parse/mod.rs` |
| `Env` | `eval/mod.rs` |
| `ModuleExports` (含 extends + loaded_modules) | `eval/mod.rs` |
| `MixinDef` / `FunctionDef` | `eval/mod.rs` |
| `Evaluator` | `eval/mod.rs` |
| `CssNode` (含 `Return(Value)`) | `css/node.rs` |
| `SassError` | `error.rs` |

## 概念 → 文件定位

| 概念 | 相关文件 |
|------|----------|
| SCSS 编译入口 | `lib.rs` → `compile_expanded()` / `compile_file_with_load_paths()` |
| 变量作用域 | `eval/mod.rs` → `Env` (vars/mixins/functions/namespaces/load_paths/global_writes) + `eval/rule.rs` → 规则体变量隔离 |
| @use 模块系统 | `eval/module.rs` → `load_module`（含缓存 + extends 传播）/ `call_module_function`（注入模块 vars） |
| @extend 继承 | `eval/extend.rs` → `apply_extends`（跨模块传播 + 选择器合并） |
| @forward with() 配置 | `eval/mod.rs` → Node::Forward 分支（ConfigVar + is_default 语义） |
| @import 多值/修饰符 | `parse/at_rules.rs` → `parse_import` + `eval/mod.rs` → Node::Import 分支 |
| 命名空间变量赋值 | `parse/nodes.rs` → `is_namespace_var` + `eval/value/mod.rs` → `eval_variable` |
| @mixin/@include | `eval/mixin.rs` → `eval_include` / `bind_params` |
| @return 控制流 | `eval/mixin.rs` → `call_user_function` (捕获 CssNode::Return) |
| 控制流 (@if/@for/@each/@while) | `eval/control_flow.rs` |
| @media/@supports 参数求值 | `eval/at_params.rs` → 参数插值和表达式求值 |
| 颜色转换 | `eval/color.rs` → `hsl_to_rgb` / `rgb_to_hsl` / `hwb_to_rgb` + `parse/ast/mod.rs` → `hsl_to_rgb_percent` |
| 颜色序列化 | `parse/ast/display.rs` → `Display for Value`（ColorFormat 分派） |
| 颜色格式追踪 | `parse/ast/mod.rs` → `ColorFormat` 枚举（Auto/Rgb/RgbPercent/Hsl/Hwb） |
| 选择器净化 | `css/selector.rs` → `sanitize_selector` / `normalize_attr_selectors` / `has_bogus_combinators` |
| 内建函数注册 | `eval/builtin.rs` → `call_builtin` match 分派 |
| 数学函数分派 | `eval/builtin/math.rs` → `call()` + `merge_math_args()` 命名参数合并 |
| 错误消息格式 | `error.rs` → 全英文错误消息（无前缀）；math/string 函数内联验证 |
| CSS 序列化 | `css/mod.rs` → Serializer |
| Tracing span | `eval/mod.rs` (eval_nodes/eval_node) + 各子模块 + `eval/rule.rs` (eval_rule) |
| Tracing events | `eval/color.rs` (sasspile::color) + `eval/extend.rs` (sasspile::extend) + `eval/value/mod.rs` (sasspile::binop) |
| CSS diff 工具 | `tests/common/mod.rs` |
| 最小化工具 | `tests/minimize.rs` |
| AST → SCSS 序列化 | `parse/ast_impl.rs` → `Node::to_scss()` |
