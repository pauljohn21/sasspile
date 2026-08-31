## Why

sasspile 当前 sass-spec 通过率 54%（2918/5362），2444 次失败中 1689 次为编译错误、755 次为输出不匹配。错误集中在参数验证过严（~300 次）、plain CSS 限制检测（~120 次）、selector 函数参数展开（~76 次）、中文错误消息（~22 次）、运算符对特殊值不支持（~50 次）等问题上。这些大多是逻辑修复而非架构变更，属于低 hanging fruit。

## What Changes

- **Phase 1 — 参数验证与错误消息修复**（预计 +300~400 用例）
  - 修复 `merge_args` 逻辑：命名参数不应计入位置参数计数，消除 "Only 1 argument allowed, but 2/3 were passed" 误报（286 次）
  - 修复 `merge_math_args` 同类问题：命名参数 `$number: 1` 不应被当作多余位置参数
  - 将所有中文错误消息改为英文（如 `"1 不是 map"` → `"1 is not a map"`，22 次）
  - 修复 selector 函数参数展开（selector-parse/extend/replace 参数合并，76 次）
  - 修复 `is_unitless` / `is-unitless` snake_case vs kebab-case 名称映射（16 次）
  - 修复 `infinity` / `-infinity` 作为参数传入 math 函数时的类型检查（18 次）

- **Phase 2 — plain CSS 错误检测增强**（预计 +120 用例）
  - 增强错误检测覆盖：`sass() conditions`、Interpolation 限制、Operators 限制的 `expected_error_but_ok` 场景
  - 修复误报：`This at-rule isn't allowed in plain CSS` 在合法场景下不应触发
  - 增强遗漏：`Top-level leading combinators`、`Parent selectors can't have suffixes` 检测

- **Phase 3 — 运算符与模块修复**（预计 +100 用例）
  - 增强 `+`/`-` 运算符对 `calc()`/`get-mixin()` 等特殊值的支持
  - 修复模块循环检测（`Module loop: this module is already being loaded`，23 次）
  - 修复 callable spec 的 `utils.a` 模块函数/mixin 解析（27 次）

- **Phase 4 — 输出序列化对齐**（预计 +200~300 用例）
  - 修复选择器排序差异
  - 修复 @media/@supports 合并规则
  - 修复数值精度和格式化
  - 修复空白处理

## Capabilities

### New Capabilities

- `arg-merge-fix`: 内建函数参数合并逻辑修复——命名参数不应计入位置参数计数，消除 "Only N argument allowed" 误报
- `plain-css-error-coverage`: plain CSS 模式错误检测增强——覆盖 sass() conditions、Interpolation、Operators 限制的 expected_error 场景
- `operator-special-values`: 运算符对特殊值（calc/get-mixin/var）的处理支持
- `error-message-i18n`: 错误消息国际化——将所有中文错误消息改为英文，对齐 sass-spec 期望

### Modified Capabilities

- `math-param-validation`: 修改参数验证逻辑——infinity/-infinity 应被接受为合法参数，snake_case 名称映射修复
- `error-detection-coverage`: 扩展错误检测覆盖 plain CSS 限制场景和模块循环检测

## Impact

- **src/eval/builtin/math_helpers.rs** — `merge_math_args` 修复命名参数合并逻辑
- **src/eval/builtin/string.rs** — 参数验证修复（str-length/to-upper-case 等的命名参数处理）
- **src/eval/builtin/list.rs** — 参数验证修复（list-separator/nth 等的命名参数处理）
- **src/eval/builtin/math.rs** — infinity 参数接受、is_unitless 名称映射
- **src/eval/builtin/selector.rs** — selector-parse/extend/replace 参数展开
- **src/eval/value/mod.rs** — 运算符对特殊值的处理
- **src/eval/mod.rs** — 模块循环检测修复
- **src/eval/module.rs** — utils.a 模块解析
- **src/css/mod.rs** — plain CSS 错误检测增强
- **src/eval/error_msgs.rs** — 中文错误消息改英文
- **tests/sass_spec_full.rs** — 解除部分 skip 标记
- **无 BREAKING 变更**：所有修复都是让输出匹配 sass-spec，不改变已通过测试的行为
