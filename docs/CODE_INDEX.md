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
| **lib.rs** | 405 | 公共 API（compile/compile_expanded/compile_compressed/compile_file/compile_file_with_load_paths）+ init_tracing |
| **main.rs** | 49 | CLI 入口 |
| **error.rs** | 95 | SassError 定义 |
| **lex/token.rs** | 170 | Token 枚举定义 + Display impl（含重新转义） |
| **lex/mod.rs** | 499 | Lexer + Iterator impl（scan_* 方法）+ scan_escape_ident 返回 Result |
| **parse/ast/mod.rs** | 420 | AST 类型定义（Node, Value, Color, BinOp, Separator, Param, Arg, VarFlags 等）|
| **parse/ast/display.rs** | 348 | Display trait 实现 + escape_quoted_string/escape_css_ident/escape_css_chars + round_alpha |
| **parse/ast_impl.rs** | 289 | Node::to_scss() 实现 |
| **parse/mod.rs** | 102 | Parser 结构 + parse() 入口 + 基础操作（peek/advance/skip_ws/expect）+ paren_depth |
| **parse/nodes.rs** | 594 | parse_node/parse_rule/parse_decl/parse_variable/parse_body + parse_params/parse_args |
| **parse/at_rules.rs** | 536 | 所有 @ 规则解析（@if/@for/@each/@while/@mixin/@include/@function/@use/@forward/@import/@extend/@at-root/@warn/@debug/@error） |
| **parse/expr/mod.rs** | 328 | Pratt 表达式解析 + has_other_operator_at_top_level |
| **parse/expr/prefix.rs** | 512 | parse_number/parse_hash_color + hex2/hex1 |
| **eval/mod.rs** | 526 | Env + ModuleExports + MixinDef + FunctionDef + Evaluator + evaluate/eval_nodes/eval_node |
| **eval/rule.rs** | 169 | eval_rule + combine_selectors |
| **eval/value/mod.rs** | 449 | eval_value + eval_binop + values_eq + eval_interp_str + eval_simple_expr + units_compatible |
| **eval/value/ops.rs** | 290 | add/sub/mul/div/modulo/compare 算术/比较运算 |
| **eval/value/display.rs** | 186 | inspect_value + 值格式化输出 |
| **eval/control_flow.rs** | 150 | eval_if/eval_for/eval_each/eval_while |
| **eval/mixin.rs** | 264 | eval_include + bind_params + call_function + call_user_function + eval_at_root + eval_at_rule + is_truthy |
| **eval/extend.rs** | 76 | apply_extends |
| **eval/module.rs** | 302 | resolve_file（含 load_paths） + load_module + call_module_function（含 is-powerless/is-in-gamut/is-legacy/to-space/to-gamut 映射） |
| **eval/color.rs** | 621 | hsl_to_rgb/hwb_to_rgb/rgb_to_hsl + builtin_rgba/builtin_darken/builtin_lighten/builtin_mix + simple_random |
| **eval/builtin.rs** | 497 | call_builtin 分派入口（match 骨架 → 子模块分派）+ is_known_builtin + is_css_function |
| **eval/builtin/color.rs** | 553 | invert/grayscale/color-channel/hwb/complement/hsl/hsla/adjust-hue/saturate/desaturate/transparentize/opacify/alpha/red/green/blue/hue/saturation/lightness + is-powerless/is-in-gamut/is-legacy + is_channel_powerless |
| **eval/builtin/list.rs** | 282 | length/nth/append/join/index/separator/set-nth/is-bracketed/list-slash/zip |
| **eval/builtin/map.rs** | 303 | map-get/keys/values/has-key/merge/remove/set/deep-remove + value_to_map/nested_map_merge/nested_map_set |
| **eval/builtin/string.rs** | 281 | str-length/to-upper-case/to-lower-case/unquote/quote/str-slice/str-index/str-insert/str-split/unique-id |
| **eval/builtin/selector.rs** | 156 | selector-append/nest/is-super/parse/simple-selectors/unify/extend |
| **eval/memory_limit.rs** | 91 | 内存限制保护（RSS 检查 + 链式释放） |
| **eval/selector/mod.rs** | 13 | 选择器模块入口（parse_selectors/unify_compound/compound_matches） |
| **eval/selector/algorithms.rs** | 426 | 选择器算法（parse_complex/parse_compound/extend_complex/unify_compound/compound_matches/compound_to_string） |
| **eval/selector/parse.rs** | 317 | 选择器解析（parse_selector_list/parse_complex_selector/parse_compound_selector + 命名空间支持） |
| **css/mod.rs** | 350 | Serializer（CSS 树 → 字符串，@规则合并 + charset 处理） |
| **css/node.rs** | 93 | CssNode 枚举（Rule/Declaration/AtRule/AtRoot/Comment/Raw/Return） |
| **css/selector.rs** | 366 | 选择器净化（sanitize_selector + 组合器验证 + 属性选择器规范化） |
| **stage/mod.rs** | 14 | 管线阶段模块入口 |
| **stage/source.rs** | 60 | Source 阶段类型 |
| **stage/lexed.rs** | 41 | Lexed 阶段类型 |
| **stage/parsed.rs** | 43 | Parsed 阶段类型 |
| **stage/evaluated.rs** | 48 | Evaluated 阶段类型 |
| **stage/serialized.rs** | 69 | Serialized 阶段类型 |

## 函数 → 文件定位

### 求值器（Evaluator）

| 函数 | 文件 |
|------|------|
| `evaluate` / `evaluate_with_path` / `evaluate_with_path_and_load_paths` | `eval/mod.rs` |
| `eval_nodes` / `eval_node` | `eval/mod.rs` |
| `eval_rule` / `combine_selectors` | `eval/rule.rs` |
| `eval_value` / `eval_binop` / `units_compatible` | `eval/value/mod.rs` |
| `add` / `sub` / `mul` / `div` / `modulo` / `compare` | `eval/value/ops.rs` |
| `values_eq` / `is_truthy` | `eval/value/mod.rs` |
| `inspect_value` | `eval/value/display.rs` |
| `eval_interp_str` / `eval_simple_expr` | `eval/value/mod.rs` |
| `eval_if` / `eval_for` / `eval_each` / `eval_while` | `eval/control_flow.rs` |
| `eval_include` / `bind_params` / `call_function` | `eval/mixin.rs` |
| `call_user_function` / `eval_at_root` / `eval_at_rule` | `eval/mixin.rs` |
| `apply_extends` | `eval/extend.rs` |
| `resolve_file` / `load_module` / `call_module_function` | `eval/module.rs` |
| `call_builtin` | `eval/builtin.rs` |
| `call_map_builtin` / `value_to_map` / `nested_map_merge` / `nested_map_set` | `eval/builtin/map.rs` |
| `call_string_builtin` / `str_slice` / `str_insert` / `str_split` | `eval/builtin/string.rs` |
| `hsl_to_rgb` / `hwb_to_rgb` / `rgb_to_hsl` / `simple_random` | `eval/color.rs` |
| `builtin_rgba` / `builtin_darken` / `builtin_lighten` / `builtin_mix` | `eval/color.rs` |
| `escape_quoted_string` / `escape_css_ident` / `escape_css_chars` | `parse/ast/display.rs` |
| `is_channel_powerless` | `eval/builtin/color.rs` |
| `parse_selectors` / `unify_compound` / `compound_matches` | `eval/selector/mod.rs` |
| `parse_selector_list` / `parse_complex_selector` / `parse_compound_selector` | `eval/selector/parse.rs` |
| `extend_complex` / `compound_to_string` | `eval/selector/algorithms.rs` |
| `sanitize_selector` / `has_bogus_combinators` | `css/selector.rs` |

### 解析器（Parser）

| 函数 | 文件 |
|------|------|
| `parse` / `parse_node` / `is_rule` | `parse/nodes.rs` |
| `parse_rule` / `parse_rule_or_decl` / `parse_selector` | `parse/nodes.rs` |
| `parse_decl` / `parse_property` / `check_important` | `parse/nodes.rs` |
| `parse_variable` / `parse_var_flags` / `parse_body` | `parse/nodes.rs` |
| `parse_params` / `parse_args` / `parse_config` / `parse_member_list` | `parse/nodes.rs` |
| `parse_at_rule` / `parse_if` / `parse_for` / `parse_each` | `parse/at_rules.rs` |
| `parse_while` / `parse_mixin_def` / `parse_include` / `parse_function_def` | `parse/at_rules.rs` |
| `parse_return` / `parse_use` / `parse_forward` / `parse_import` | `parse/at_rules.rs` |
| `parse_extend` / `parse_at_root` / `parse_warn` / `parse_debug` / `parse_error` | `parse/at_rules.rs` |
| `parse_generic_at_rule` / `parse_at_params` | `parse/at_rules.rs` |
| `parse_expr` / `is_value_start` / `parse_prefix` / `peek_binding_power` | `parse/expr/mod.rs` |
| `has_other_operator_at_top_level` | `parse/expr/mod.rs` |
| `parse_number` / `parse_hash_color` / `hex2` / `hex1` | `parse/expr/prefix.rs` |

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
| `Value` (值枚举，含 Raw + Interp + Paren + Spread) | `parse/ast/mod.rs` |
| `Color` | `parse/ast/mod.rs` |
| `ColorFormat` | `parse/ast/mod.rs` |
| `BinOp` / `BinOpKind` | `parse/ast/mod.rs` |
| `Separator` (含 Slash/SlashDiv/Comma/Space/Undecided) | `parse/ast/mod.rs` |
| `Ast` | `parse/ast/mod.rs` |
| `Param` / `Arg` / `VarFlags` | `parse/ast/mod.rs` |
| `Token` | `lex/token.rs` |
| `Lexer` | `lex/mod.rs` |
| `Parser` | `parse/mod.rs` |
| `Env` | `eval/mod.rs` |
| `ModuleExports` | `eval/mod.rs` |
| `MixinDef` / `FunctionDef` | `eval/mod.rs` |
| `Evaluator` | `eval/mod.rs` |
| `CssNode` (含 `Return(Value)`) | `css/node.rs` |
| `SassError` | `error.rs` |

## 概念 → 文件定位

| 概念 | 相关文件 |
|------|----------|
| SCSS 编译入口 | `lib.rs` → `compile_expanded()` / `compile_file_with_load_paths()` |
| 变量作用域 | `eval/mod.rs` → `Env` (vars/mixins/functions/namespaces/load_paths) |
| @use 模块系统 | `eval/module.rs` → `load_module` / `call_module_function` |
| @extend 继承 | `eval/extend.rs` → `apply_extends` |
| @mixin/@include | `eval/mixin.rs` → `eval_include` / `bind_params` |
| @return 控制流 | `eval/mixin.rs` → `call_user_function` (捕获 CssNode::Return) |
| 控制流 (@if/@for/@each/@while) | `eval/control_flow.rs` |
| 颜色转换 | `eval/color.rs` → `hsl_to_rgb` / `rgb_to_hsl` / `hwb_to_rgb` |
| 内建函数注册 | `eval/builtin.rs` → `call_builtin` match 分派 |
| CSS 序列化 | `css/mod.rs` → Serializer |
| 选择器净化 | `css/selector.rs` → sanitize_selector + 组合器验证 |
| 选择器解析/统一 | `eval/selector/` → parse.rs + algorithms.rs |
| 内存限制保护 | `eval/memory_limit.rs` → RSS 检查 + 链式释放 |
| Tracing span | `eval/mod.rs` (eval_nodes/eval_node) + 各子模块 |
| Tracing events | `eval/color.rs` (sasspile::color) + `eval/extend.rs` (sasspile::extend) + `eval/value.rs` (sasspile::binop) |
| CSS diff 工具 | `tests/common/mod.rs` |
| 最小化工具 | `tests/minimize.rs` |
| AST → SCSS 序列化 | `parse/ast_impl.rs` → `Node::to_scss()` |
