# sasspile sass-spec 全量统计报告

## 一级目录总览

| 目录 | HRX 文件数 | Case 数 | 说明 |
|------|-----------|---------|------|
| variables | 5 | 59 | 变量声明、作用域、!default/!global |
| values | 94 | 3702 | 数字/字符串/列表/map/布尔/null/计算 |
| css | 145 | 3023 | CSS 输出格式、@media/@supports/选择器序列化 |
| operators | 5 | 115 | 算术运算符、比较运算符 |
| expressions | 19 | 732 | 表达式语法、if()、函数调用 |
| directives | 157 | 3657 | @use/@forward/@import/@extend/@if/@for/@each/@mixin |
| core_functions | 874 | 26734 | 内建函数（math/string/list/map/meta/selector/color） |
| parser | 4 | 63 | 解析器语法（缩进、插值、选择器） |
| callable | 3 | 308 | mixin/function 参数和调用 |
| **合计** | **1306** | **38393** | |

## 通过率统计（按一级目录）

| 目录 | Case 数 | 失败数 | 通过数 | 通过率 |
|------|---------|--------|--------|--------|
| variables | 59 | 4 | 55 | 93% |
| values | 3702 | 646 | 3056 | 82% |
| css | 3023 | 490 | 2533 | 83% |
| operators | 115 | 14 | 101 | 87% |
| expressions | 732 | 47 | 685 | 93% |
| directives | 3657 | 390 | 3267 | 89% |
| core_functions | 26734 | 908 | 25826 | 96% |
| parser | 63 | 14 | 49 | 77% |
| callable | 308 | 22 | 286 | 92% |
| **合计** | **38393** | **2535** | **35858** | **93%** |

## 按 HRX 文件失败数（Top 30）

| HRX 文件 | 失败数 |
|----------|--------|
| core_functions/selector/unify/simple/universal.hrx | 29 |
| values/calculation/calc/error/known_incompatible/angle.hrx | 28 |
| core_functions/selector/unify/simple/pseudo.hrx | 27 |
| directives/forward/error/with.hrx | 27 |
| css/supports/whitespace.hrx | 26 |
| core_functions/selector/extend/no_op.hrx | 25 |
| values/calculation/calc/operator.hrx | 24 |
| core_functions/selector/unify/simple/type/and_type.hrx | 23 |
| css/comment.hrx | 23 |
| values/calculation/calc/no_operator.hrx | 23 |
| css/media/logic/error.hrx | 21 |
| css/supports/error.hrx | 21 |
| directives/import/whitespace.hrx | 21 |
| directives/use/extend/scope.hrx | 21 |
| callable/arguments.hrx | 20 |
| core_functions/selector/extend/complex/with_unification.hrx | 20 |
| core_functions/list/join/empty.hrx | 19 |
| core_functions/selector/extend/complex/without_unification.hrx | 19 |
| css/functions/special/prefixed/uppercase.hrx | 19 |
| css/functions/var.hrx | 19 |
| css/plain/error/media.hrx | 19 |
| expressions/if/syntax.hrx | 19 |
| directives/function/name.hrx | 18 |
| directives/import/configuration/separate_file.hrx | 18 |
| directives/use/error/extend.hrx | 18 |
| core_functions/selector/unify/simple/type/and_universal.hrx | 17 |
| css/function.hrx | 17 |
| css/plain/import/conditions.hrx | 17 |
| css/plain/import/whitespace.hrx | 17 |
| core_functions/selector/parse/selector.hrx | 16 |

## 按二级路径失败数

| 路径 | 失败数 |
|------|--------|
| variables/whitespace | 3 |
| variables/semicolon | 1 |
| values/strings | 5 |
| values/numbers | 35 |
| values/mixins | 2 |
| values/maps | 3 |
| values/lists | 15 |
| values/ids | 1 |
| values/identifiers | 2 |
| values/calculation | 583 |
| parser/selector | 6 |
| parser/interpolation | 3 |
| parser/indentation | 5 |
| operators/plus | 3 |
| operators/newlines | 4 |
| operators/modulo | 4 |
| operators/minus | 3 |
| expressions/if | 32 |
| expressions/functions | 5 |
| expressions/comments | 10 |
| directives/warn | 1 |
| directives/use | 99 |
| directives/mixin | 2 |
| directives/import | 114 |
| directives/if | 14 |
| directives/function | 20 |
| directives/forward | 84 |
| directives/for | 8 |
| directives/extend | 32 |
| directives/each | 10 |
| directives/at_root | 6 |
| css/unknown_directive | 15 |
| css/unicode_range | 12 |
| css/supports | 72 |
| css/style_rule | 11 |
| css/selector | 25 |
| css/propset | 12 |
| css/plain | 105 |
| css/percent | 1 |
| css/ms_long_filter_syntax | 1 |
| css/moz_document | 5 |
| css/mixin | 1 |
| css/media | 58 |
| css/keyframes | 6 |
| css/important | 1 |
| css/functions | 86 |
| css/function | 17 |
| css/font-face | 6 |
| css/custom_properties | 32 |
| css/comment | 23 |
| css/charset | 1 |
| core_functions/string | 4 |
| core_functions/selector | 535 |
| core_functions/newlines | 5 |
| core_functions/modules | 13 |
| core_functions/meta | 170 |
| core_functions/math | 82 |
| core_functions/map | 8 |
| core_functions/list | 68 |
| core_functions/global | 18 |
| core_functions/general | 5 |
| callable/parameters | 2 |
| callable/arguments | 20 |

## 文件类型统计

- .scss 失败: 2276
- .sass 失败: 259
- 总失败: 2535
