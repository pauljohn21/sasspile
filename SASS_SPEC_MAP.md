# Sass 功能 ↔ sass-spec 对照表

> 生成时间：2026-08-18（最后更新）
> 项目版本：v0.9.3
> 数据来源：sass 官方手册（sass-lang.com/documentation）+ sass-spec HRX 文件
> 用途：每个 Sass 语言功能对应的 sass-spec 测试文件索引

---

## 一、Syntax（语法）

### 1.1 Parsing a Stylesheet

手册: https://sass-lang.com/documentation/syntax/parsing/

| spec 路径 | 文件 |
|-----------|------|
| `parser/` | indentation.hrx, interpolation.hrx, operator_precedence.hrx, selector.hrx |

### 1.2 Structure of a Stylesheet

手册: https://sass-lang.com/documentation/syntax/structure/

| spec 路径 | 文件 |
|-----------|------|
| `css/` 根级 | blockless_directive_without_semicolon.hrx, charset.hrx, comment.hrx, directive_with_lots_of_whitespace.hrx, empty_block_directive.hrx, escape.hrx, font-face.hrx, function.hrx, function_name_identifiers.hrx, important.hrx, keyframes.hrx, mixin.hrx, ms_long_filter_syntax.hrx, percent.hrx, propset.hrx, style_rule.hrx, url.hrx |

### 1.3 Comments

手册: https://sass-lang.com/documentation/syntax/comments/

| spec 路径 | 文件 |
|-----------|------|
| `css/plain/` | boolean_operations.hrx, custom_properties.hrx, error/expression/calculation.hrx, error/expression/function.hrx, error/expression/if.hrx, error/expression/interpolation.hrx, error/expression/list.hrx, error/expression/map.hrx, error/expression/operation.hrx, error/expression/parent_selector.hrx, error/expression/parentheses.hrx, error/expression/variable.hrx, error/media.hrx, error/statement/at_rule.hrx, error/statement/silent_comment.hrx, error/statement/style_rule.hrx, extend.hrx, function.hrx, functions.hrx, hacks.hrx, if.hrx, media.hrx, null.hrx, single_equals.hrx, slash.hrx, style_rule/nesting/combinator.hrx, style_rule/nesting/media.hrx, style_rule/nesting/multiple_complex.hrx, style_rule/nesting/one_level.hrx, style_rule/nesting/parent.hrx, style_rule/nesting/supports.hrx, style_rule/nesting/through_import.hrx, style_rule/nesting/through_load_css.hrx, style_rule/nesting/two_levels.hrx, style_rule/nesting/unknown.hrx, style_rule/nesting/with_declaration.hrx, style_rule/top_level_parent.hrx |
| `css/plain/calculation` | calculation.hrx |
| `css/plain/import` | conditions.hrx, css_before_index.hrx, in_css.hrx, partial_conflict.hrx, sass_takes_precedence.hrx, scss_takes_precedence.hrx, whitespace.hrx |

### 1.4 Special Functions

手册: https://sass-lang.com/documentation/syntax/special-functions/

| spec 路径 | 文件 |
|-----------|------|
| `css/functions/` | error.hrx, newlines.hrx, not_special.hrx, special/comment.hrx, special/error.hrx, special/prefixed/lowercase.hrx, special/prefixed/uppercase.hrx, special/unprefixed.hrx, special_variable.hrx, var.hrx |

---

## 二、Style Rules（样式规则）

### 2.1 Property Declarations

手册: https://sass-lang.com/documentation/style-rules/declarations/

| spec 路径 | 文件 |
|-----------|------|
| `css/custom_properties/` | empty.hrx, error.hrx, exclamation.hrx, indentation.hrx, name_interpolation.hrx, nesting_characters.hrx, script.hrx, simple.hrx, strings.hrx, syntax.hrx, trailing_comment.hrx, trailing_whitespace.hrx, value_interpolation.hrx, without_semicolon.hrx |

### 2.2 Parent Selector

手册: https://sass-lang.com/documentation/style-rules/parent-selector/

| spec 路径 | 文件 |
|-----------|------|
| `css/selector/` | parent.hrx |

### 2.3 Placeholder Selectors

手册: https://sass-lang.com/documentation/style-rules/placeholder-selectors/

| spec 路径 | 文件 |
|-----------|------|
| `css/selector/` | placeholder.hrx |

### 2.4 Selectors（完整选择器语法）

| spec 路径 | 文件 |
|-----------|------|
| `css/selector/` | attribute.hrx, combinator/adjacent.hrx, combinator/has.hrx, combinator/is.hrx, combinator/leading.hrx, combinator/middle.hrx, combinator/newline.hrx, combinator/only_one_bogus.hrx, combinator/selector_pseudo.hrx, combinator/trailing.hrx, escaping.hrx, inline_comments.hrx, pseudoselector.hrx, reference_combinator.hrx, slotted.hrx |

---

## 三、Variables（变量）

手册: https://sass-lang.com/documentation/variables/

| spec 路径 | 文件 |
|-----------|------|
| `variables/` | comments.hrx, double_flag.hrx, semi_global.hrx, semicolon.hrx, whitespace.hrx |

---

## 四、Interpolation（插值）

手册: https://sass-lang.com/documentation/interpolation/

| spec 路径 | 文件 |
|-----------|------|
| `css/unknown_directive/` | comment.hrx, error.hrx, name_interpolation.hrx, plain.hrx, semicolon.hrx, value_interpolation.hrx, whitespace.hrx |
| `parser/` | interpolation.hrx |
| `css/custom_properties/` | name_interpolation.hrx, value_interpolation.hrx |

---

## 五、At-Rules（@规则）

### 5.1 `@use`

手册: https://sass-lang.com/documentation/at-rules/use/

| spec 路径 | 文件 |
|-----------|------|
| `directives/use/` 根级 | comment.hrx, escaped.hrx, load.hrx, whitespace.hrx |
| `directives/use/css/` | import.hrx |
| `directives/use/css/order/` | use_and_import.hrx, use_only.hrx |
| `directives/use/extend/` | diamond.hrx, extended.hrx, midstream_extend_within_pseudoselector.hrx, optional_and_mandatory.hrx, scope.hrx, upstream.hrx |
| `directives/use/member/` | global.hrx, namespaced.hrx, nested_global_variable.hrx, use_to_import.hrx |
| `directives/use/with/` | core_module.hrx, dash_insensitive.hrx, distributed_vars.hrx, doesnt_run_default.hrx, from_variable.hrx, multi_load.hrx, multiple.hrx, null.hrx, single.hrx, some_unconfigured.hrx, through_forward.hrx, through_import.hrx, trailing_comma.hrx, used_in_input.hrx, variable_exists.hrx |
| `directives/use/error/` | extend.hrx, load.hrx, member/before_use.hrx, member/conflict.hrx, member/inaccessible.hrx, member/missing.hrx, syntax/after.hrx, syntax/as_invalid.hrx, syntax/as_nothing.hrx, syntax/empty.hrx, syntax/member.hrx, syntax/url.hrx, syntax/with.hrx, syntax/within.hrx, with/conflict.hrx, with/core_module.hrx, with/invalid_expression.hrx, with/missing_distributed_vars.hrx, with/multiconfiguration.hrx, with/namespace.hrx, with/nested.hrx, with/not_default.hrx, with/private.hrx, with/repeated_variable.hrx, with/through_forward.hrx, with/through_forward_twice.hrx, with/undefined.hrx |

### 5.2 `@forward`

手册: https://sass-lang.com/documentation/at-rules/forward/

| spec 路径 | 文件 |
|-----------|------|
| `directives/forward/` 根级 | comment.hrx, css.hrx, escaped.hrx, extend.hrx, whitespace.hrx |
| `directives/forward/member/` | as.hrx, bare.hrx, newlines.hrx, shadowed.hrx, visibility.hrx |
| `directives/forward/member/import/` | forward_to_import.hrx, import_to_forward/nested.hrx, import_to_forward/override.hrx, import_to_forward/top_level.hrx, import_to_forward/transitive.hrx, import_to_forward/use_to.hrx, import_to_forward/with.hrx, precedence.hrx |
| `directives/forward/with/` | core_module.hrx, dash_insensitive.hrx, default.hrx, doesnt_run_default.hrx, facade_contains_multiple_configured_forwards.hrx, from_variable.hrx, multi_load.hrx, multiple.hrx, null.hrx, single.hrx, some_unconfigured.hrx, through_forward.hrx, through_import.hrx, trailing_comma.hrx, used_in_input.hrx, variable_exists.hrx |
| `directives/forward/error/` | extend.hrx, load.hrx, member/conflict.hrx, member/import_to_forward.hrx, member/inaccessible.hrx, syntax.hrx, with.hrx |

### 5.3 `@import`

手册: https://sass-lang.com/documentation/at-rules/import/

| spec 路径 | 文件 |
|-----------|------|
| `directives/import/` 根级 | comment.hrx, css.hrx, escaped.hrx, implicit_dependencies.hrx, load.hrx, nested.hrx, top_level_parent.hrx, whitespace.hrx |
| `directives/import/configuration/` | import_twice.hrx, indirect.hrx, midstream_definition.hrx, nested.hrx, prefixed_as.hrx, same_file.hrx, separate_file.hrx, unrelated_variable.hrx |
| `directives/import/error/` | conflict.hrx, member.hrx, not_found.hrx, top_level_declaration.hrx |

### 5.4 `@mixin` / `@include`

手册: https://sass-lang.com/documentation/at-rules/mixin/

| spec 路径 | 文件 |
|-----------|------|
| `directives/mixin/` | comment.hrx, custom_ident_include.hrx, double_underscore_name.hrx, sass.hrx, whitespace.hrx |
| `callable/` | arguments.hrx, parameters.hrx, whitespace.hrx |

### 5.5 `@function` / `@return`

手册: https://sass-lang.com/documentation/at-rules/function/

| spec 路径 | 文件 |
|-----------|------|
| `directives/function/` | comment.hrx, escaped.hrx, name.hrx, whitespace.hrx |
| `directives/` | return.hrx |

### 5.6 `@extend`

手册: https://sass-lang.com/documentation/at-rules/extend/

| spec 路径 | 文件 |
|-----------|------|
| `directives/extend/` | after_target.hrx, bogus.hrx, comment.hrx, error.hrx, pseudo.hrx, trims_super_selector_without_combinator.hrx, whitespace.hrx |

### 5.7 `@at-root`

手册: https://sass-lang.com/documentation/at-rules/at-root/

| spec 路径 | 文件 |
|-----------|------|
| `directives/at_root/` | comment.hrx, keyframes.hrx, load_css.hrx, nested_import.hrx, property_only.hrx, sass.hrx, whitespace.hrx |

### 5.8 `@if` / `@else`

手册: https://sass-lang.com/documentation/at-rules/control/if/

| spec 路径 | 文件 |
|-----------|------|
| `directives/if/` | comment.hrx, error/syntax.hrx, escaped.hrx, sass.hrx, whitespace.hrx |

### 5.9 `@for`

手册: https://sass-lang.com/documentation/at-rules/control/for/

| spec 路径 | 文件 |
|-----------|------|
| `directives/for/` | comment.hrx, for.hrx, whitespace.hrx |

### 5.10 `@each`

手册: https://sass-lang.com/documentation/at-rules/control/each/

| spec 路径 | 文件 |
|-----------|------|
| `directives/` | each.hrx |

### 5.11 `@while`

手册: https://sass-lang.com/documentation/at-rules/control/while/

| spec 路径 | 文件 |
|-----------|------|
| `directives/` | while.hrx |

### 5.12 `@error`

手册: https://sass-lang.com/documentation/at-rules/error/

| spec 路径 | 文件 |
|-----------|------|
| `directives/` | error.hrx |

### 5.13 `@warn`

手册: https://sass-lang.com/documentation/at-rules/warn/

| spec 路径 | 文件 |
|-----------|------|
| `directives/` | warn.hrx |

### 5.14 `@debug`

手册: https://sass-lang.com/documentation/at-rules/debug/

| spec 路径 | 文件 |
|-----------|------|
| `directives/` | debug.hrx |

### 5.15 From CSS — `@media`

手册: https://sass-lang.com/documentation/at-rules/css/

| spec 路径 | 文件 |
|-----------|------|
| `css/media/` | bubbling.hrx, comment.hrx, indentation.hrx, logic/and.hrx, logic/and_not.hrx, logic/error.hrx, logic/nested.hrx, logic/not.hrx, logic/or.hrx, range/error.hrx, range/from_interpolation.hrx, range/static.hrx, range/with_expressions.hrx, type.hrx, whitespace.hrx |

### 5.16 From CSS — `@supports`

手册: https://sass-lang.com/documentation/at-rules/css/

| spec 路径 | 文件 |
|-----------|------|
| `css/supports/` | comment.hrx, error.hrx, nesting.hrx, syntax/anything.hrx, syntax/calculations.hrx, syntax/declaration.hrx, syntax/function.hrx, syntax/lone_interpolation.hrx, syntax/operator.hrx, whitespace.hrx |

### 5.17 From CSS — `@-moz-document`

手册: https://sass-lang.com/documentation/breaking-changes/moz-document/

| spec 路径 | 文件 |
|-----------|------|
| `css/moz_document/` | comment.hrx, empty_prefix.hrx, functions/interpolated.hrx, functions/static.hrx, multi_function.hrx, whitespace.hrx |

### 5.18 From CSS — `@unicode-range`

| spec 路径 | 文件 |
|-----------|------|
| `css/unicode_range/` | error.hrx, question_mark.hrx, range.hrx, simple.hrx |

### 5.19 From CSS — 未知指令

| spec 路径 | 文件 |
|-----------|------|
| `css/unknown_directive/` | comment.hrx, error.hrx, name_interpolation.hrx, plain.hrx, semicolon.hrx, value_interpolation.hrx, whitespace.hrx |

---

## 六、Values（值类型）

### 6.1 Numbers

手册: https://sass-lang.com/documentation/values/numbers/

| spec 路径 | 文件 |
|-----------|------|
| `values/numbers/` | bounds.hrx, degenerate.hrx, divide/slash_free/argument.hrx, divide/slash_free/return.hrx, divide/slash_free/value.hrx, divide/slash_free/variable.hrx, divide/slash_separated.hrx, error.hrx, modulo/floats.hrx, modulo/ints.hrx, modulo/zeros.hrx, precision.hrx, units/multiple.hrx, very_large.hrx |

### 6.2 Strings

手册: https://sass-lang.com/documentation/values/strings/

| spec 路径 | 文件 |
|-----------|------|
| `values/identifiers/` | escape/normalize.hrx, escape/script.hrx, if.hrx |
| `values/` 根级 | strings.hrx, ids.hrx |

### 6.3 Colors

手册: https://sass-lang.com/documentation/values/colors/

| spec 路径 | 文件 |
|-----------|------|
| `values/colors/` | alpha_hex/initial_digit.hrx, alpha_hex/initial_letter.hrx, equality.hrx |

### 6.4 Lists

手册: https://sass-lang.com/documentation/values/lists/

| spec 路径 | 文件 |
|-----------|------|
| `values/lists/` | brackets.hrx, equality.hrx, sass.hrx, slash.hrx |

### 6.5 Maps

手册: https://sass-lang.com/documentation/values/maps/

| spec 路径 | 文件 |
|-----------|------|
| `values/maps/` | duplicate-keys.hrx, errors.hrx, invalid-key.hrx, key_equality.hrx, length.hrx, map-values.hrx, sass.hrx |

### 6.6 `true` / `false`

手册: https://sass-lang.com/documentation/values/booleans/

| spec 路径 | 文件 |
|-----------|------|
| `expressions/` | comments.hrx, functions.hrx, if/css.hrx, if/else.hrx, if/raw.hrx, if/sass.hrx, if/short_circuit.hrx, if/syntax.hrx, syntax.hrx |
| `expressions/if/error/` | and.hrx, empty.hrx, invalid_function_name.hrx, missing.hrx, missing_whitepsace.hrx, not.hrx, or.hrx, paren.hrx, raw.hrx, semicolon.hrx |

### 6.7 `null`

手册: https://sass-lang.com/documentation/values/null/

| spec 路径 | 文件 |
|-----------|------|
| `css/plain/` | null.hrx |

### 6.8 Calculations

手册: https://sass-lang.com/documentation/values/calculations/

| spec 路径 | 文件 |
|-----------|------|
| `values/calculation/` 根级 | abs.hrx, acos.hrx, asin.hrx, atan.hrx, atan2.hrx, calc-size.hrx, clamp.hrx, cos.hrx, exp.hrx, hypot.hrx, log.hrx, max.hrx, min.hrx, mod.hrx, pow.hrx, rem.hrx, sign.hrx, sin.hrx, sqrt.hrx, tan.hrx |
| `values/calculation/calc/` | constant.hrx, no_operator.hrx, operator.hrx, parens.hrx, simplify.hrx, space.hrx |
| `values/calculation/calc/error/` | complex_units.hrx, operator.hrx, space.hrx, syntax.hrx, value.hrx |
| `values/calculation/calc/error/known_incompatible/` | angle.hrx, complex.hrx, frequency.hrx, length/ch.hrx, length/cm.hrx, length/em.hrx, length/ex.hrx, length/in.hrx, length/mm.hrx, length/pc.hrx, length/pt.hrx, length/px.hrx, length/q.hrx, length/rem.hrx, length/vh.hrx, length/vmax.hrx, length/vmin.hrx, length/vw.hrx, minus.hrx, time.hrx, unknown_and_none.hrx |
| `values/calculation/round/` | error.hrx, one_argument.hrx, strategy/down.hrx, strategy/nearest.hrx, strategy/to-zero.hrx, strategy/up.hrx, three_arguments.hrx, two_arguments.hrx |

### 6.9 Functions & Mixins

手册: https://sass-lang.com/documentation/values/functions/

| spec 路径 | 文件 |
|-----------|------|
| `values/` 根级 | mixins.hrx |

---

## 七、Operators（运算符）

### 7.1 Equality

手册: https://sass-lang.com/documentation/operators/equality/

| spec 路径 | 文件 |
|-----------|------|
| `values/lists/` | equality.hrx |
| `values/colors/` | equality.hrx |
| `values/maps/` | key_equality.hrx |

### 7.2 Relational

手册: https://sass-lang.com/documentation/operators/relational/

| spec 路径 | 文件 |
|-----------|------|
| `operators/` | minus.hrx, newlines.hrx, plus.hrx |

### 7.3 Numeric

手册: https://sass-lang.com/documentation/operators/numeric/

| spec 路径 | 文件 |
|-----------|------|
| `operators/` | minus.hrx, modulo.hrx, newlines.hrx, plus.hrx, slash.hrx |

### 7.4 String

手册: https://sass-lang.com/documentation/operators/string/

| spec 路径 | 文件 |
|-----------|------|
| `operators/` | plus.hrx |

### 7.5 Boolean

手册: https://sass-lang.com/documentation/operators/boolean/

| spec 路径 | 文件 |
|-----------|------|
| `expressions/` | comments.hrx, functions.hrx, syntax.hrx |
| `css/plain/` | boolean_operations.hrx |

---

## 八、Built-In Modules（内建模块）

### 8.1 `sass:color`

手册: https://sass-lang.com/documentation/modules/color/

| spec 路径 | 文件 |
|-----------|------|
| `core_functions/color/` 根级 | alpha.hrx, blackness.hrx, blue.hrx, complement.hrx, darken.hrx, desaturate.hrx, fade_in.hrx, fade_out.hrx, grayscale.hrx, green.hrx, hsla.hrx, hue.hrx, ie_hex_str.hrx, is_in_gamut.hrx, is_legacy.hrx, is_missing.hrx, lighten.hrx, lightness.hrx, red.hrx, rgba.hrx, same.hrx, saturate.hrx, saturation.hrx, space.hrx, whiteness.hrx |
| `core_functions/color/adjust/` | a98_rgb.hrx, display_p3.hrx, display_p3_linear.hrx, error/args.hrx, error/incompatible_channel.hrx, error/missing.hrx, error/mixed_formats.hrx, error/space.hrx, error/type.hrx, error/units/a98_rgb.hrx, error/units/display_p3.hrx, error/units/display_p3_linear.hrx, error/units/hwb.hrx, error/units/lab.hrx, error/units/lch.hrx, error/units/oklab.hrx, error/units/oklch.hrx, error/units/prophoto_rgb.hrx, error/units/rec2020.hrx, error/units/srgb.hrx, error/units/srgb_linear.hrx, error/units/xyz.hrx, error/units/xyz_d50.hrx, global.hrx, hsl.hrx, hwb.hrx, lab.hrx, lch.hrx, no_channels.hrx, oklab.hrx, oklch.hrx, prophoto_rgb.hrx, rec2020.hrx, rgb.hrx, space.hrx, srgb.hrx, srgb_linear.hrx, units.hrx, xyz_d50.hrx, xyz_d65.hrx |
| `core_functions/color/adjust_color/` | error/missing_globals.hrx |
| `core_functions/color/adjust_hue/` | above_max.hrx, alpha.hrx, error.hrx, fraction.hrx, max.hrx, middle.hrx, min.hrx, named.hrx, negative.hrx, units.hrx |
| `core_functions/color/change/` | a98_rgb.hrx, display_p3.hrx, display_p3_linear.hrx, error/args.hrx, error/bounds.hrx, error/incompatible_channel.hrx, error/mixed_formats.hrx, error/space.hrx, error/type.hrx, error/units/a98_rgb.hrx, error/units/display_p3.hrx, error/units/display_p3_linear.hrx, error/units/hwb.hrx, error/units/lab.hrx, error/units/lch.hrx, error/units/oklab.hrx, error/units/oklch.hrx, error/units/prophoto_rgb.hrx, error/units/rec2020.hrx, error/units/srgb.hrx, error/units/srgb_linear.hrx, error/units/xyz.hrx, error/units/xyz_d50.hrx, global.hrx, hsl.hrx, hwb.hrx, lab.hrx, lch.hrx, no_space.hrx, oklab.hrx, oklch.hrx, prophoto_rgb.hrx, rec2020.hrx, rgb.hrx, space.hrx, srgb.hrx, srgb_linear.hrx, xyz.hrx, xyz_d50.hrx |
| `core_functions/color/channel/` | a98-rgb.hrx, display-p3.hrx, display_p3_linear.hrx, error.hrx, hsl.hrx, hwb.hrx, lab.hrx, lch.hrx, missing.hrx, named.hrx, oklab.hrx, oklch.hrx, positional.hrx, prophoto-rgb.hrx, rec2020.hrx, rgb.hrx, srgb-linear.hrx, srgb.hrx, xyz-d50.hrx, xyz.hrx |
| `core_functions/color/color/` | alpha.hrx, degenerate.hrx, error.hrx, no_alpha.hrx, relative_color.hrx, spaces/a98_rgb.hrx, spaces/display_p3.hrx, spaces/display_p3_linear.hrx, spaces/prophoto_rgb.hrx, spaces/rec2020.hrx, spaces/srgb.hrx, spaces/srgb_linear.hrx, spaces/xyz.hrx, spaces/xyz_d50.hrx, special_functions.hrx |
| `core_functions/color/hsl/` | error/five_args.hrx, error/four_args.hrx, error/one_arg.hrx, error/three_args.hrx, error/two_args.hrx, error/zero_args.hrx, four_args/alpha.hrx, four_args/in_gamut.hrx, four_args/out_of_gamut.hrx, four_args/special_functions.hrx, multi_argument_var.hrx, one_arg/alpha.hrx, one_arg/no_alpha.hrx, one_arg/relative_color.hrx, one_arg/special_functions/alpha.hrx, one_arg/special_functions/no_alpha.hrx, one_arg/special_functions/slash_list.hrx, three_args/bounds.hrx, three_args/named.hrx, three_args/out_of_gamut.hrx, three_args/special_functions.hrx, three_args/units.hrx, three_args/w3c/black_to_white_through.hrx, three_args/w3c/blue_to_red.hrx, three_args/w3c/gray_to.hrx, three_args/w3c/green_to_blue.hrx, three_args/w3c/hue.hrx, three_args/w3c/red_to_green.hrx |
| `core_functions/color/hwb/` | error/five_args.hrx, error/four_args.hrx, error/one_arg.hrx, error/three_args.hrx, error/two_args.hrx, error/zero_args.hrx, four_args.hrx, global.hrx, one_arg.hrx, three_args/bounds.hrx, three_args/named.hrx, three_args/units.hrx, three_args/w3c/blue_magentas.hrx, three_args/w3c/blues.hrx, three_args/w3c/cyan_blues.hrx, three_args/w3c/cyans.hrx, three_args/w3c/green_cyans.hrx, three_args/w3c/greens.hrx, three_args/w3c/magenta_reds.hrx, three_args/w3c/magentas.hrx, three_args/w3c/oranges.hrx, three_args/w3c/reds.hrx, three_args/w3c/yellow_greens.hrx, three_args/w3c/yellows.hrx |
| `core_functions/color/invert/` | alpha.hrx, error.hrx, global.hrx, legacy.hrx, modern.hrx, named.hrx, number.hrx |
| `core_functions/color/is_powerless/` | error.hrx, hsl.hrx, hwb.hrx, lab.hrx, lch.hrx, named.hrx, oklab.hrx, oklch.hrx, space.hrx |
| `core_functions/color/lab/` | alpha.hrx, error.hrx, no_alpha.hrx, relative_color.hrx, special_functions/alpha.hrx, special_functions/no_alpha.hrx, special_functions/slash_list.hrx |
| `core_functions/color/lch/` | alpha.hrx, error.hrx, no_alpha.hrx, special_functions.hrx |
| `core_functions/color/mix/` | alpha.hrx, both_weights.hrx, error.hrx, explicit_method.hrx, explicit_weight.hrx, hue_interpolation.hrx, missing.hrx, mixed_spaces.hrx, named.hrx, predefined.hrx, units.hrx, unweighted.hrx |
| `core_functions/color/oklab/` | alpha.hrx, error.hrx, no_alpha.hrx, special_functions.hrx |
| `core_functions/color/oklch/` | alpha.hrx, error.hrx, no_alpha.hrx, special_functions.hrx |
| `core_functions/color/rgb/` | error/five_args.hrx, error/four_args.hrx, error/one_arg.hrx, error/three_args.hrx, error/two_args.hrx, error/zero_args.hrx, four_args/alpha.hrx, four_args/clamped.hrx, four_args/in_gamut.hrx, four_args/special_functions.hrx, multi_argument_var.hrx, one_arg/alpha.hrx, one_arg/no_alpha.hrx, one_arg/relative_color.hrx, one_arg/special_functions/alpha.hrx, one_arg/special_functions/no_alpha.hrx, one_arg/special_functions/slash_list.hrx, three_args/percents.hrx, three_args/special_functions.hrx, three_args/unitless.hrx, two_args.hrx |
| `core_functions/color/scale/` | a98_rgb.hrx, display_p3.hrx, display_p3_linear.hrx, error/args.hrx, error/bounds.hrx, error/incompatible_channel.hrx, error/missing.hrx, error/mixed_formats.hrx, error/polar.hrx, error/space.hrx, error/type.hrx, error/units/a98_rgb.hrx, error/units/display_p3.hrx, error/units/display_p3_linear.hrx, error/units/hsl.hrx, error/units/hwb.hrx, error/units/lab.hrx, error/units/lch.hrx, error/units/oklab.hrx, error/units/oklch.hrx, error/units/prophoto_rgb.hrx, error/units/rec2020.hrx, error/units/rgb.hrx, error/units/srgb.hrx, error/units/srgb_linear.hrx, error/units/xyz_d50.hrx, error/units/xyz_d65.hrx, global.hrx, hsl.hrx, hwb.hrx, lab.hrx, lch.hrx, no_channels.hrx, no_space.hrx, oklab.hrx, out_of_gamut.hrx, prophoto_rgb.hrx, rec2020.hrx, rgb.hrx, space.hrx, srgb.hrx, srgb_linear.hrx, xyz_d50.hrx, xyz_d65.hrx |
| `core_functions/color/to_gamut/` | a98_rgb.hrx, display_p3.hrx, display_p3_linear.hrx, error.hrx, hsl.hrx, hwb.hrx, lab.hrx, lch.hrx, named.hrx, oklab.hrx, oklch.hrx, positional.hrx, prophoto_rgb.hrx, rec2020.hrx, rgb.hrx, space.hrx, srgb.hrx, srgb_linear.hrx, xyz.hrx, xyz_d50.hrx |
| `core_functions/color/to_space/` | a98_rgb/*.hrx (16), display_p3/*.hrx (16), display_p3_linear/*.hrx (16), error.hrx, hsl/*.hrx (16), hwb/*.hrx (16), lab/*.hrx (16), lch/*.hrx (16), oklab/*.hrx (16), oklch/*.hrx (16), prophoto_rgb/*.hrx (16), rec2020/*.hrx (16), rgb/*.hrx (16), srgb/*.hrx (16), srgb_linear/*.hrx (16), xyz/*.hrx (16), xyz_d50/*.hrx (16) |

### 8.2 `sass:list`

手册: https://sass-lang.com/documentation/modules/list/

| spec 路径 | 文件 |
|-----------|------|
| `core_functions/list/` 根级 | append.hrx, index.hrx, is_bracketed.hrx, length.hrx, nth.hrx, separator.hrx, set_nth.hrx, slash.hrx, utils.hrx, zip.hrx |
| `core_functions/list/join/` | empty.hrx, error.hrx, multi.hrx, single.hrx |

### 8.3 `sass:map`

手册: https://sass-lang.com/documentation/modules/map/

| spec 路径 | 文件 |
|-----------|------|
| `core_functions/map/` | deep_merge.hrx, deep_remove.hrx, get.hrx, has_key.hrx, keys.hrx, merge.hrx, remove.hrx, set.hrx, values.hrx |

### 8.4 `sass:math`

手册: https://sass-lang.com/documentation/modules/math/

| spec 路径 | 文件 |
|-----------|------|
| `core_functions/math/` 根级 | abs.hrx, acos.hrx, asin.hrx, atan.hrx, ceil.hrx, clamp.hrx, comparable.hrx, cos.hrx, div.hrx, floor.hrx, hypot.hrx, log.hrx, max.hrx, min.hrx, percentage.hrx, random.hrx, round.hrx, sin.hrx, sqrt.hrx, tan.hrx, unit.hrx, unitless.hrx, variables.hrx |
| `core_functions/math/atan2/` | arguments.hrx, y_infinite.hrx, y_non_zero_finite.hrx, y_zero.hrx |
| `core_functions/math/pow/` | arguments.hrx, base_greater_than_zero.hrx, base_less_than_zero.hrx, base_negative_infinity.hrx, base_negative_zero.hrx, base_positive_infinity.hrx, base_positive_zero.hrx |

### 8.5 `sass:meta`

手册: https://sass-lang.com/documentation/modules/meta/

| spec 路径 | 文件 |
|-----------|------|
| `core_functions/meta/` 根级 | accepts_content.hrx, apply.hrx, calc_args.hrx, calc_name.hrx, call.hrx, content_exists.hrx, feature_exists.hrx, function_exists.hrx, global_variable_exists.hrx, keywords.hrx, mixin_exists.hrx, module_functions.hrx, module_mixins.hrx, module_variables.hrx, type_of.hrx, variable_exists.hrx |
| `core_functions/meta/get_function/` | different_module.hrx, equality.hrx, error.hrx, meta.hrx, same_module.hrx, scope.hrx |
| `core_functions/meta/get_mixin/` | content.hrx, different_module.hrx, equality.hrx, error.hrx, same_module.hrx, scope.hrx |
| `core_functions/meta/inspect/` | boolean.hrx, color.hrx, error.hrx, function.hrx, inspect.hrx, list/bracketed.hrx, list/comma.hrx, list/empty.hrx, list/nested.hrx, list/single.hrx, list/space.hrx, map.hrx, mixin.hrx, null.hrx, number.hrx, string.hrx |
| `core_functions/meta/load_css/` | extend.hrx, plain_css.hrx, twice.hrx |
| `core_functions/meta/load_css/error/` | content.hrx, from_other.hrx, load.hrx, member.hrx, too_few_args.hrx, too_many_args.hrx, type.hrx, with.hrx |
| `core_functions/meta/load_css/with/` | core_module.hrx, dash_insensitive.hrx, doesnt_run_default.hrx, empty.hrx, multi_load.hrx, multiple.hrx, null.hrx, single.hrx, some_unconfigured.hrx, through_forward.hrx, through_import.hrx, variable_exists.hrx |

### 8.6 `sass:selector`

手册: https://sass-lang.com/documentation/modules/selector/

| spec 路径 | 文件 |
|-----------|------|
| `core_functions/selector/` 根级 | append.hrx, replace.hrx |
| `core_functions/selector/extend/` | complex/combinator_only.hrx, complex/with_unification.hrx, complex/without_unification.hrx, error.hrx, format.hrx, list.hrx, named.hrx, no_op.hrx, simple/attribute.hrx, simple/class.hrx, simple/id.hrx, simple/placeholder.hrx, simple/pseudo/arg.hrx, simple/pseudo/no_arg.hrx, simple/pseudo/selector/idempotent/any.hrx, simple/pseudo/selector/idempotent/current.hrx, simple/pseudo/selector/idempotent/is.hrx, simple/pseudo/selector/idempotent/matches.hrx, simple/pseudo/selector/idempotent/not.hrx, simple/pseudo/selector/idempotent/nth_child.hrx, simple/pseudo/selector/idempotent/nth_last_child.hrx, simple/pseudo/selector/idempotent/prefixed.hrx, simple/pseudo/selector/idempotent/where.hrx, simple/pseudo/selector/match.hrx, simple/pseudo/selector/non_idempotent.hrx, simple/type.hrx, simple/universal.hrx |
| `core_functions/selector/is_superselector/` | complex/adjacent_sibling.hrx, complex/bogus.hrx, complex/child.hrx, complex/descendant.hrx, complex/sibling.hrx, compound.hrx, error.hrx, input.hrx, list.hrx, named.hrx, simple/attribute.hrx, simple/class.hrx, simple/id.hrx, simple/placeholder.hrx, simple/pseudo/arg.hrx, simple/pseudo/no_arg.hrx, simple/pseudo/selector_arg/any.hrx, simple/pseudo/selector_arg/current.hrx, simple/pseudo/selector_arg/has.hrx, simple/pseudo/selector_arg/host.hrx, simple/pseudo/selector_arg/host_context.hrx, simple/pseudo/selector_arg/is.hrx, simple/pseudo/selector_arg/matches.hrx, simple/pseudo/selector_arg/not.hrx, simple/pseudo/selector_arg/nth_child.hrx, simple/pseudo/selector_arg/nth_last_child.hrx, simple/pseudo/selector_arg/slotted.hrx, simple/pseudo/selector_arg/where.hrx, simple/type.hrx, simple/universal.hrx |
| `core_functions/selector/nest/` | combinator.hrx, error.hrx, format.hrx, list.hrx, many_args.hrx, one_arg.hrx, parent.hrx |
| `core_functions/selector/parse/` | error.hrx, named.hrx, selector.hrx, structure.hrx |
| `core_functions/selector/unify/` | chooses_superselector.hrx, complex/combinators/child.hrx, complex/combinators/initial.hrx, complex/combinators/multiple.hrx, complex/combinators/next_sibling.hrx, complex/combinators/sibling.hrx, complex/distinct.hrx, complex/identical.hrx, complex/lcs.hrx, complex/overlap.hrx, complex/rootish.hrx, complex/superselector.hrx, compound.hrx, error.hrx, format.hrx, simple/attribute.hrx, simple/class.hrx, simple/different_types.hrx, simple/id.hrx, simple/placeholder.hrx, simple/pseudo.hrx, simple/type/and_type.hrx, simple/type/and_universal.hrx, simple/universal.hrx |

### 8.7 `sass:string`

手册: https://sass-lang.com/documentation/modules/string/

| spec 路径 | 文件 |
|-----------|------|
| `core_functions/string/` 根级 | index.hrx, insert.hrx, length.hrx, quote.hrx, split.hrx, to_lower_case.hrx, to_upper_case.hrx, unique_id.hrx, unquote.hrx |
| `core_functions/string/slice/` | combining_character.hrx, double_width_character.hrx, empty.hrx, end.hrx, error.hrx, named.hrx, start.hrx, unquoted.hrx |

### 8.8 模块系统

| spec 路径 | 文件 |
|-----------|------|
| `core_functions/modules/color/` | adjust.hrx, alpha.hrx, blue.hrx, change.hrx, complement.hrx, css_overloads.hrx, error.hrx, green.hrx, hue.hrx, ie_hex_str.hrx, invert.hrx, lightness.hrx, mix.hrx, red.hrx, saturation.hrx, scale.hrx |

### 8.9 Legacy 全局函数

| spec 路径 | 文件 |
|-----------|------|
| `core_functions/global/color/` | alpha.hrx, blue.hrx, change.hrx, complement.hrx, darken.hrx, desaturate.hrx, error.hrx, fade-in.hrx, fade-out.hrx, grayscale.hrx, green.hrx, hue.hrx, ie_hex_str.hrx, invert.hrx, lighten.hrx, lightness.hrx, mix.hrx, opacify.hrx, opacity.hrx, red.hrx, saturate.hrx, saturation.hrx, scale.hrx, transparentize.hrx |
| `core_functions/global/` 根级 | list.hrx, map.hrx, math.hrx, meta.hrx, selector.hrx, string.hrx |

### 8.10 其他

| spec 路径 | 文件 |
|-----------|------|
| `core_functions/` 根级 | general.hrx, newlines.hrx |
