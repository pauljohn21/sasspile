# sasspile 代码索引

> 查找函数/类型/概念在哪个文件？查这张表。
>
> **动态查询**（调用者/被调用者/影响分析/源码查看）请优先用 CodeGraph：
> ```bash
> codegraph sync                    # 同步索引（每次 git 提交后必跑）
> codegraph node eval_node           # 查看符号源码 + 调用链路
> codegraph callers apply_extends   # 谁调了这个函数？
> codegraph impact eval_value       # 修改影响范围分析
> codegraph explore "color conversion"  # 探索某领域
> ```
> **每次 git 提交后必须运行 `codegraph sync` 同步索引。**
> 详见 [`AGENTS.md`](../AGENTS.md#codegraph-代码导航) CodeGraph 章节。

## 文件 → 职责速查

| 文件 | 行数 | 职责 |
|------|------|------|
| **lib.rs** | 334 | 公共 API（compile/compile_expanded/compile_compressed/compile_file/compile_file_with_load_paths）+ init_tracing |
| **main.rs** | 60 | CLI 入口（支持文件路径参数 + stdin 回退 + .css 文件 plain CSS 模式） |
| **error.rs** | 95 | SassError 定义（全英文错误消息） |
| **lex/token.rs** | 170 | Token 枚举定义 + Display impl（含重新转义） |
| **lex/mod.rs** | 523 | Lexer + Iterator impl（scan_* 方法）+ scan_escape_ident 返回 Result |
| **parse/ast/mod.rs** | 335 | AST 类型定义（Node, Value, MixinRefData, BinOp, Param, Arg, VarFlags, ConfigVar, Separator 等）+ MixinRefData PartialEq impl |
| **parse/ast/color_types.rs** | 180 | ColorFormat 枚举 + Color 结构体 + hsl_to_rgb_percent/format_pct_val/format_hue/format_pct/format_alpha 辅助函数 |
| **parse/ast/display.rs** | 374 | Display for Value（ColorFormat 分派序列化，含 CSS Color 4 现代空间 + 负 infinity 处理 + Slash/SlashLiteral 分隔符区分） |
| **parse/ast_impl.rs** | 288 | Node::to_scss() |
| **parse/mod.rs** | 92 | Parser 结构 + parse() 入口 + 基础操作（peek/advance/skip_w/expect） |
| **parse/nodes.rs** | 668 | parse_node/parse_rule/parse_decl/parse_variable/parse_body + parse_params/parse_args + is_namespace_var/parse_namespace_var + parse_config (ConfigVar) |
| **parse/at_rules.rs** | 513 | 所有 @ 规则解析（@if/@for/@each/@while/@mixin/@include/@function/@use/@forward/@import/@extend/@at-root/@warn/@debug/@error）+ @import 多值/修饰符解析 |
| **parse/expr/mod.rs** | 293 | Pratt 表达式解析入口 + parse_decl_value/parse_value_with_slash/parse_expr_slash + slash_followed_by_arith_op + parse_number/parse_hash_color（SlashLiteral 用于声明值中字面量 `/`） |
| **parse/expr/prefix.rs** | 465 | Pratt 前缀解析 + parse_prefix/peek_binding_power/parse_value_start |
| **eval/mod.rs** | 282 | Env（move 语义 self→Self + HashMap 字段） + ModuleExports + MixinDef + FunctionDef + Evaluator + evaluate/eval_nodes/eval_node（Env move） + evaluate_with_env(pub(crate)) + plain CSS 检查入口 |
| **eval/meta_ops.rs** | 329 | meta.apply / meta.load-css mixin + meta.get-mixin / meta.module-functions / meta.module-mixins / meta.module-variables 反射函数 |
| **eval/rule.rs** | 155 | eval_rule（move 语义） + combine_selectors（规则体变量作用域隔离，传播命名空间/!global 变量） |
| **eval/value/mod.rs** | 482 | eval_value + eval_binop + add/sub/mul/div/modulo/compare + values_eq + inspect_value + eval_interp_str + units_compatible + 命名空间变量赋值 + if() 命名参数支持 |
| **eval/value/ops.rs** | 223 | 值运算实现（add/sub/mul/div/modulo/compare 细节 + infinity 带单位 calc 表达式） |
| **eval/value/display.rs** | 253 | inspect_value + 值显示格式化（Slash/SlashLiteral 分隔符区分） |
| **eval/control_flow.rs** | 205 | eval_if/eval_for/eval_each/eval_while |
| **eval/mixin.rs** | 287 | eval_include（move） + exec_mixin（move） + bind_params（&Env） + call_function（&Env） + call_user_function（&Env） + eval_at_root（move） + eval_at_rule（move） + is_truthy |
| **eval/extend.rs** | 76 | apply_extends（跨模块 @extend 传播 + 选择器合并 + 占位符替换） |
| **eval/at_params.rs** | 240 | @media/@supports 参数插值和表达式求值 |
| **eval/import.rs** | 66 | eval_import（move 语义）— sass: 模块加载 + .css/http 透传 + 文件加载回退 |
| **eval/module_dispatch.rs** | 352 | 内建函数注册结构体（MathBuiltins/StringBuiltins/...）+ `#[derive(BuiltinRegistry)]` 属性声明 + 宏生成的单一数据源注册 |
| **eval/plain_css.rs** | 238 | check_plain_css_value + check_plain_css_node + check_plain_css_selector + check_plain_css_call（含 sass() 禁止检测 + is_css_function/is_known_builtin 区分） |
| **eval/module.rs** | 348 | load_module（module_cache 缓存 + pending_config 注入） + load_import（forwarded→local 合并） + call_module_function + eval_use + eval_forward |
| **eval/file_resolver.rs** | 198 | resolve_file + try_resolve_dir + check_resolve_ambiguity（partial/extension/index/import-only 四种冲突检测） |
| **eval/module_helpers.rs** | 174 | bind_exports（BindMode Use/Forward + show/hide 过滤 + values_eq + Display 后备） + merge_module_cache + builtin_module_exports + BindMode + FilterConfig + merge_with_local_precedence |
| **eval/color.rs** | 665 | hsl_to_rgb/hwb_to_rgb/rgb_to_hsl + builtin_rgba（SlashLiteral 兼容）/builtin_darken/builtin_lighten/builtin_mix + simple_random |
| **eval/builtin.rs** | 409 | call_builtin 分派入口（优先调用宏生成的 dispatch_builtin_module）+ rgba/rgb/darken/lighten/mix 手工分派 + meta 函数（get-mixin/module-functions/module-mixins/module-variables/mixin-exists/type-of） |
| **eval/builtin/math.rs** | 412 | abs/ceil/floor/round/min/max/percentage/div/pow/sqrt/sin/cos/tan/atan2/asin/acos/atan/hypot/log/random/clamp/unit/is-unitless/compatible/comparable + validate_single_number 参数验证 + div 除零 calc(infinity) 表达式 |
| **eval/builtin/math_helpers.rs** | 89 | merge_math_args 命名参数合并 + math_param_names 参数名映射 + validate_single_number 参数验证辅助 |
| **eval/builtin/color.rs** | 520 | invert/grayscale/color-channel/hwb/complement/hsl/hsla/adjust-hue/saturate/desaturate/transparentize/opacify/alpha/red/green/blue/hue/saturation/lightness + adjust-color/change-color/scale-color（旧版 RGB/HSL/HWB） + is-powerless/is-in-gamut/is-legacy + is_channel_powerless + flatten_space_list（SlashLiteral 兼容） |
| **eval/builtin/color_adjust.rs** | 550 | color.adjust/change/scale 现代色彩空间实现（Oklch/Lab/Lch/Oklab/DisplayP3/sRGB 等）— 直接在 ColorFormat 中修改通道值，保留原始格式输出 |
| **eval/builtin/color_conv.rs** | 461 | f64 精度色彩空间转换算法（sRGB↔XYZ/Lab/Oklab/Oklch/DisplayP3）— W3C 参考实现有理数分数矩阵 + 扩展传递函数（支持负值） |
| **eval/builtin/color_conv_ops.rs** | 261 | 颜色空间转换工具函数：is_same_space/convert_space/format_to_srgb_f64/make_color + HSL/HWB f64 精度转换 |
| **eval/builtin/color_gamut.rs** | 293 | color.to-gamut 实现（clip 直接截断 + local-minde 在 Oklch 空间二分搜索减小 chroma） |
| **eval/builtin/color_parse.rs** | 178 | CSS Color 4 颜色函数解析：lab/lch/oklab/oklch/color() — 从 Sass 值参数解析为 Value::Color + split_alpha（Slash/SlashLiteral 兼容） + flatten_space_list（SlashLiteral 兼容） |
| **eval/builtin/color_space.rs** | 390 | color.channel/to-space/space/same 函数 + get_channel_value（各空间通道值提取） |
| **eval/builtin/list.rs** | 306 | length/nth/append/join/index/separator/set-nth/is-bracketed/list-slash/zip（SlashLiteral 兼容处理） |
| **eval/builtin/map.rs** | 302 | map-get/keys/values/has-key/merge/remove/set/deep-remove + value_to_map/nested_map_merge/nested_map_set |
| **eval/builtin/string.rs** | 376 | str-length/to-upper-case/to-lower-case/unquote/quote/str-slice/str-index/str-insert/str-split/unique-id + 参数验证（$string: X is not a string） |
| **eval/builtin/selector.rs** | 110 | selector-append/nest/is-super/parse/simple-selectors/unify/extend + merge_selector_args 命名参数合并 |
| **css/mod.rs** | 359 | Serializer（CSS 树 → 字符串，选择器净化 + 组合器验证 + @规则合并 + @import 间不加空行） |
| **css/node.rs** | 93 | CssNode 枚举（Rule/Declaration/AtRule/AtRoot/Comment/Raw/Return） |
| **css/selector.rs** | 366 | sanitize_selector + normalize_attr_selectors + has_bogus_combinators + 占位符处理 |
| **stage/*.rs** | 15-86 | 管线阶段类型（Source: from_file+base_path+load_paths / Lexed: 透传路径 / Parsed: evaluate()构建Env / Evaluated / Serialized） |

## 函数 → 文件定位

### 求值器（Evaluator）

| 函数 | 文件 |
|------|------|
| `evaluate` / `evaluate_with_env` | `eval/mod.rs` |
| `eval_nodes` / `eval_node` | `eval/mod.rs` |
| `eval_rule` / `combine_selectors` | `eval/rule.rs` |
| `eval_value` / `eval_binop` / `units_compatible` | `eval/value/mod.rs` |
| `add` / `sub` / `mul` / `div` / `modulo` / `compare` | `eval/value/ops.rs` |
| `values_eq` / `inspect_value` / `is_truthy` | `eval/value/mod.rs` |
| `eval_interp_str` / `eval_simple_expr` | `eval/value/mod.rs` |
| `eval_if` / `eval_for` / `eval_each` / `eval_while` | `eval/control_flow.rs` |
| `eval_meta_apply` / `eval_meta_load_css` / `meta_get_mixin` / `meta_module_functions` / `meta_module_mixins` / `meta_module_variables` | `eval/meta_ops.rs` |
| `exec_mixin` / `eval_include` / `bind_params` / `call_function` | `eval/mixin.rs` |
| `call_user_function` / `eval_at_root` / `eval_at_rule` | `eval/mixin.rs` |
| `apply_extends` | `eval/extend.rs` |
| `resolve_file` / `try_resolve_dir` / `check_resolve_ambiguity` | `eval/file_resolver.rs` |
| `load_module` / `load_import` / `call_module_function` | `eval/module.rs` |
| `eval_use` / `eval_forward` | `eval/module.rs` |
| `bind_exports` / `merge_module_cache` / `builtin_module_exports` / `BindMode` / `FilterConfig` | `eval/module_helpers.rs` |
| `module_builtin_name` / `is_known_builtin` / `dispatch_builtin_module` | `eval/module_dispatch.rs`（宏自动生成） |
| `MathBuiltins` / `StringBuiltins` / `MapBuiltins` / `ListBuiltins` / `ColorBuiltins` / `MetaBuiltins` / `SelectorBuiltins` | `eval/module_dispatch.rs` |
| `BuiltinRegistry` derive 宏 | `sasspile-macros/src/lib.rs` |
| `eval_import` | `eval/import.rs` |
| `check_plain_css_value` / `check_plain_css_node` / `check_plain_css_selector` / `check_plain_css_call` | `eval/plain_css.rs` |
| `call_builtin` | `eval/builtin.rs` |
| `merge_math_args` / `math_param_names` / `validate_single_number` | `eval/builtin/math_helpers.rs` |
| math::call (abs/ceil/floor/round/div/pow/clamp/...) | `eval/builtin/math.rs` |
| `call_map_builtin` / `value_to_map` / `nested_map_merge` / `nested_map_set` | `eval/builtin/map.rs` |
| `call_string_builtin` / `str_slice` / `str_insert` / `str_split` | `eval/builtin/string.rs` |
| `hsl_to_rgb` / `hwb_to_rgb` / `rgb_to_hsl` / `simple_random` | `eval/color.rs` |
| `builtin_rgba` / `builtin_darken` / `builtin_lighten` / `builtin_mix` | `eval/color.rs` |
| `hsl_to_rgb_percent` / `format_pct_val` / `format_hue` / `format_pct` / `format_alpha` | `parse/ast/mod.rs` |
| `escape_quoted_string` / `escape_css_ident` / `escape_css_chars` | `parse/ast/mod.rs` |
| `is_channel_powerless` | `eval/builtin/color.rs` |
| `to_gamut` / `clip_gamut` / `local_minde` | `eval/builtin/color_gamut.rs` |
| `srgb_to_linear` / `linear_to_srgb` / `srgb_to_xyz` / `xyz_to_lab` / `xyz_to_oklab` / `lab_to_lch` / `oklab_to_oklch` / `bradford_d50_to_d65` | `eval/builtin/color_conv.rs` |
| `channel` / `to_space` / `space` / `same` | `eval/builtin/color_space.rs` |
| `is_same_space` / `convert_space` / `format_to_srgb_f64` / `make_color` / `hsl_to_srgb_f64` / `hwb_to_srgb_f64` | `eval/builtin/color_conv_ops.rs` |
| `parse_color_fn` / `parse_lab` / `parse_lch` / `parse_oklab` / `parse_oklch` / `parse_color_space` | `eval/builtin/color_parse.rs` |
| `adjust` / `change` / `scale`（现代色彩空间） | `eval/builtin/color_adjust.rs` |
| `clone_with` | `parse/ast/mod.rs` (ColorFormat 方法) |
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
| `ColorFormat` (Auto/Rgb/RgbPercent/Hsl/Hwb/Lab/Lch/Oklab/Oklch/DisplayP3/Srgb/...) | `parse/ast/mod.rs` |
| `BinOp` / `BinOpKind` | `parse/ast/mod.rs` |
| **Separator** (Comma/Space/Slash/SlashLiteral/Undecided) | `parse/ast/mod.rs` |
| `Ast` | `parse/ast/mod.rs` |
| `Param` / `Arg` / `VarFlags` / `ConfigVar` | `parse/ast/mod.rs` |
| `Token` | `lex/token.rs` |
| `Lexer` | `lex/mod.rs` |
| `Parser` | `parse/mod.rs` |
| `Env` | `eval/mod.rs` |
| `ModuleExports` (含 extends + loaded_modules) | `eval/mod.rs` |
| `MixinRefData` | `parse/ast/mod.rs` |
| `Value::MixinRef` | `parse/ast/mod.rs` |
| `Evaluator` | `eval/mod.rs` |
| `CssNode` (含 `Return(Value)`) | `css/node.rs` |
| `SassError` | `error.rs` |

## 概念 → 文件定位

| 概念 | 相关文件 |
|------|----------|
| SCSS 编译入口 | `lib.rs` → `compile_expanded()` / `compile_file_with_load_paths()` |
| 变量作用域 | `eval/mod.rs` → `Env` (vars/mixins/functions/namespaces/load_paths/global_writes) + `eval/rule.rs` → 规则体变量隔离 |
| @use 模块系统 | `eval/module.rs` → `eval_use` / `load_module`（含缓存 + extends 传播）/ `call_module_function`（注入模块 vars） + `builtin_module_exports`（注册 sass:math 变量） |
| @extend 继承 | `eval/extend.rs` → `apply_extends`（跨模块传播 + 选择器合并） |
| @forward with() 配置 | `eval/module.rs` → `eval_forward`（ConfigVar + is_default 语义 + apply_config + bind_exports） |
| @import 多值/修饰符 | `parse/at_rules.rs` → `parse_import` + `eval/mod.rs` → `eval_import` |
| 命名空间变量赋值 | `parse/nodes.rs` → `is_namespace_var` + `eval/value/mod.rs` → `eval_variable` |
| @mixin/@include | `eval/mixin.rs` → `eval_include` / `bind_params` |
| @return 控制流 | `eval/mixin.rs` → `call_user_function` (捕获 CssNode::Return) |
| 控制流 (@if/@for/@each/@while) | `eval/control_flow.rs` |
| @media/@supports 参数求值 | `eval/at_params.rs` → 参数插值和表达式求值 |
| 颜色转换 | `eval/color.rs` → `hsl_to_rgb` / `rgb_to_hsl` / `hwb_to_rgb` + `parse/ast/mod.rs` → `hsl_to_rgb_percent` + `eval/builtin/color_conv.rs` → sRGB↔XYZ/Lab/Oklab 矩阵转换 |
| CSS Color 4 色彩空间 | `eval/builtin/color_space.rs` → `channel`/`to_space`/`space`/`same` + `eval/builtin/color_conv_ops.rs` → `convert_space`/`format_to_srgb_f64`/`make_color` + `eval/builtin/color_parse.rs` → `parse_color_fn` + `eval/builtin/color_adjust.rs` → 现代空间 adjust/change/scale + `eval/builtin/color_gamut.rs` → `to_gamut` (clip + local-minde) + `eval/builtin/color_conv.rs` → W3C 有理数分数矩阵 |
| 颜色序列化 | `parse/ast/display.rs` → `Display for Value`（ColorFormat 分派，含 CSS Color 4 现代空间） |
| 颜色格式追踪 | `parse/ast/mod.rs` → `ColorFormat` 枚举（Auto/Rgb/RgbPercent/Hsl/Hwb/Lab/Lch/Oklab/Oklch/DisplayP3/Srgb/...） |
| 选择器净化 | `css/selector.rs` → `sanitize_selector` / `normalize_attr_selectors` / `has_bogus_combinators` |
| 内建函数注册 | `eval/module_dispatch.rs` → `#[derive(BuiltinRegistry)]` 宏自动生成单一数据源（`sasspile-macros` crate） |
| 数学函数分派 | `eval/builtin/math.rs` → `call()` + `merge_math_args()` 命名参数合并（由 `MathBuiltins` 结构体通过宏注册） |
| 错误消息格式 | `error.rs` → 全英文错误消息（无前缀）；math/string 函数内联验证 |
| CSS 序列化 | `css/mod.rs` → Serializer |
| Tracing span | `eval/mod.rs` (eval_nodes/eval_node) + 各子模块 + `eval/rule.rs` (eval_rule) |
| Tracing events | `eval/color.rs` (sasspile::color) + `eval/extend.rs` (sasspile::extend) + `eval/value/mod.rs` (sasspile::binop) |
| CSS diff 工具 | `tests/common/mod.rs` |
| HRX 解析（VFS + `===` 分组） | `hrx_auditor` crate（`../scss-rust`）→ `tests/sass_spec_full.rs` + `tests/cf_diag.rs` 调用 |
| spec 跳过列表 | `tests/spec_manifest.rs` → `SKIP_DIRS`（跳过 libsass/non_conformant/core_functions/color/values/colors） |
| 颜色测试跳过 | `#[ignore]` 标记的 5 个颜色测试函数（cf_color/cf_diag/minimize/sass_spec_full），需 `--ignored` 手动触发 |
| 最小化工具 | `tests/minimize.rs` |
| AST → SCSS 序列化 | `parse/ast_impl.rs` → `Node::to_scss()` |
| OpenSpec specs | `openspec/specs/` — 5 个 capability spec（calc-infinity-handling / error-detection-coverage / meta-module-functions / module-member-access / param-validation-fix） |
| OpenSpec 归档 | `openspec/changes/archive/` — 已完成变更的归档存储 |
