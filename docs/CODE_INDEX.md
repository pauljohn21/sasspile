# sasspile 代码索引

> 查找函数/类型/概念在哪个文件？查这张表。

## 文件 → 职责速查

| 文件 | 行数 | 职责 |
|------|------|------|
| **lib.rs** | 316 | 公共 API（compile/compile_expanded）+ init_tracing |
| **main.rs** | 36 | CLI 入口 |
| **error.rs** | 77 | SassError 定义 |
| **lex/token.rs** | 131 | Token 枚举定义 |
| **lex/mod.rs** | 459 | Lexer + Iterator impl（scan_* 方法） |
| **parse/ast.rs** | 327 | AST 类型定义（Node, Value, Color, BinOp, Param, Arg, VarFlags 等） |
| **parse/ast_impl.rs** | 281 | Display for Value + Node::to_scss() |
| **parse/mod.rs** | 75 | Parser 结构 + parse() 入口 + 基础操作（peek/advance/skip_ws/expect） |
| **parse/nodes.rs** | 484 | parse_node/parse_rule/parse_decl/parse_variable/parse_body + parse_params/parse_args |
| **parse/at_rules.rs** | 463 | 所有 @ 规则解析（@if/@for/@each/@while/@mixin/@include/@function/@use/@forward/@import/@extend/@at-root/@warn/@debug/@error） |
| **parse/expr.rs** | 471 | Pratt 表达式解析 + parse_number/parse_hash_color |
| **eval/mod.rs** | 402 | Env + ModuleExports + MixinDef + Evaluator + evaluate/eval_nodes/eval_node |
| **eval/rule.rs** | 138 | eval_rule + combine_selectors |
| **eval/value.rs** | 461 | eval_value + eval_binop + add/sub/mul/div/modulo/compare + values_eq + inspect_value + eval_interp_str |
| **eval/control_flow.rs** | 148 | eval_if/eval_for/eval_each/eval_while |
| **eval/mixin.rs** | 168 | eval_include + bind_params + call_function + call_user_function + eval_at_root + eval_at_rule + is_truthy |
| **eval/extend.rs** | 78 | apply_extends |
| **eval/module.rs** | 177 | resolve_file + load_module + call_module_function |
| **eval/color.rs** | 268 | hsl_to_rgb/hwb_to_rgb/rgb_to_hsl + builtin_rgba/builtin_darken/builtin_lighten/builtin_mix + simple_random |
| **eval/builtin.rs** | 464 | call_builtin 分派入口（match 骨架 + math/string/map/meta inline arms） |
| **eval/builtin/list.rs** | 195 | length/nth/append/join/index/separator/set-nth/is-bracketed/list-slash/zip |
| **eval/builtin/color.rs** | 206 | invert/grayscale/color-channel/hwb/complement/hsl/hsla/adjust-hue/saturate/desaturate/transparentize/opacify/alpha/red/green/blue/hue/saturation/lightness |
| **eval/builtin/selector.rs** | 82 | selector-append/nest/is-super/parse/simple-selectors/unify/extend |
| **css/mod.rs** | 347 | Serializer（CSS 树 → 字符串） |
| **css/node.rs** | 74 | CssNode 枚举（Rule/Declaration/AtRule/AtRoot/Comment） |
| **stage/*.rs** | 14-89 | 管线阶段类型（Source/Lexed/Parsed/Evaluated/Serialized） |

## 函数 → 文件定位

### 求值器（Evaluator）

| 函数 | 文件 |
|------|------|
| `evaluate` / `evaluate_with_path` | `eval/mod.rs` |
| `eval_nodes` / `eval_node` | `eval/mod.rs` |
| `eval_rule` / `combine_selectors` | `eval/rule.rs` |
| `eval_value` / `eval_binop` | `eval/value.rs` |
| `add` / `sub` / `mul` / `div` / `modulo` / `compare` | `eval/value.rs` |
| `values_eq` / `inspect_value` / `is_truthy` | `eval/value.rs` |
| `eval_interp_str` / `eval_simple_expr` | `eval/value.rs` |
| `eval_if` / `eval_for` / `eval_each` / `eval_while` | `eval/control_flow.rs` |
| `eval_include` / `bind_params` / `call_function` | `eval/mixin.rs` |
| `call_user_function` / `eval_at_root` / `eval_at_rule` | `eval/mixin.rs` |
| `apply_extends` | `eval/extend.rs` |
| `resolve_file` / `load_module` / `call_module_function` | `eval/module.rs` |
| `call_builtin` | `eval/builtin.rs` |
| `hsl_to_rgb` / `hwb_to_rgb` / `rgb_to_hsl` / `simple_random` | `eval/color.rs` |
| `builtin_rgba` / `builtin_darken` / `builtin_lighten` / `builtin_mix` | `eval/color.rs` |

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
| `parse_expr` / `is_value_start` / `parse_prefix` / `peek_binding_power` | `parse/expr.rs` |
| `parse_number` / `parse_hash_color` / `hex2` / `hex1` | `parse/expr.rs` |

### 词法分析器（Lexer）

| 函数 | 文件 |
|------|------|
| `new` / `next` (Iterator impl) | `lex/mod.rs` |
| `scan_ident` / `scan_number` / `scan_string` | `lex/mod.rs` |
| `scan_interp` / `scan_at` / `scan_dollar` / `scan_hash` | `lex/mod.rs` |
| `scan_line_comment` / `scan_block_comment` | `lex/mod.rs` |

## 类型 → 文件定位

| 类型 | 定义位置 |
|------|----------|
| `Node` (AST 节点枚举) | `parse/ast.rs` |
| `Value` (值枚举) | `parse/ast.rs` |
| `Color` | `parse/ast.rs` |
| `BinOp` / `BinOpKind` | `parse/ast.rs` |
| `Separator` | `parse/ast.rs` |
| `Ast` | `parse/ast.rs` |
| `Param` / `Arg` / `VarFlags` | `parse/ast.rs` |
| `Token` | `lex/token.rs` |
| `Lexer` | `lex/mod.rs` |
| `Parser` | `parse/mod.rs` |
| `Env` | `eval/mod.rs` |
| `ModuleExports` | `eval/mod.rs` |
| `MixinDef` / `FunctionDef` | `eval/mod.rs` |
| `Evaluator` | `eval/mod.rs` |
| `CssNode` | `css/node.rs` |
| `SassError` | `error.rs` |

## 概念 → 文件定位

| 概念 | 相关文件 |
|------|----------|
| SCSS 编译入口 | `lib.rs` → `compile_expanded()` |
| 变量作用域 | `eval/mod.rs` → `Env` (vars/mixins/functions/namespaces) |
| @use 模块系统 | `eval/module.rs` → `load_module` / `call_module_function` |
| @extend 继承 | `eval/extend.rs` → `apply_extends` |
| @mixin/@include | `eval/mixin.rs` → `eval_include` / `bind_params` |
| 控制流 (@if/@for/@each/@while) | `eval/control_flow.rs` |
| 颜色转换 | `eval/color.rs` → `hsl_to_rgb` / `rgb_to_hsl` / `hwb_to_rgb` |
| 内建函数注册 | `eval/builtin.rs` → `call_builtin` match 分派 |
| CSS 序列化 | `css/mod.rs` → Serializer |
| Tracing span | `eval/mod.rs` (eval_nodes/eval_node) + 各子模块 |
| Tracing events | `eval/color.rs` (sasspile::color) + `eval/extend.rs` (sasspile::extend) + `eval/value.rs` (sasspile::binop) |
| CSS diff 工具 | `tests/common/mod.rs` |
| 最小化工具 | `tests/minimize.rs` |
| AST → SCSS 序列化 | `parse/ast_impl.rs` → `Node::to_scss()` |
