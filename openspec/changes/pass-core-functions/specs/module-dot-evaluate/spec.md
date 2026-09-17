# module-dot-evaluate delta spec (modified)

## MODIFIED Requirements

### Requirement: substitute_vars 识别点分函数调用路径

`substitute_vars` SHALL 在 alphanumeric 标识符扫描中将 `.` 视作路径分隔符的一部分，形成完整的点分标识符（如 `math.abs`、`color.alpha`）。当 point 分标识符后接 `(` 时，substitute_vars 将标识符归一化为 `module-fn`（hyphenated）格式传递给 `eval_builtin`；当后接字符不是 `(` 时，将完整标识符（含 `.`）原样输出。

**变更理由 (Reason)**: 原行为使用 `rsplitn(2, '.').next()` 剥除 module 前缀只保留 `fn`，导致 `selector.append` 与 `list.append` 冲突。新行为使用 `module-fn`（hyphenated）作为 builtin dispatch canonical key。

#### Scenario: math.abs 点分调用生成 hyphenated key
- **WHEN** 输入为 `@use "sass:math"; a {b: math.abs(-5px)}`
- **THEN** `eval_builtin` 以 `"math-abs"` 为 name 被调用，结果为 `5px`

#### Scenario: color.alpha
- **WHEN** 输入为 `@use "sass:color"; a {b: color.alpha(red)}`
- **THEN** `eval_builtin` 以 `"color-alpha"` 为 name 被调用，结果为 `1`

#### Scenario: selector.append 以 hyphenated key 路由
- **WHEN** 输入为 `@use "sass:selector"; a {b: selector.append(".a", ".b")}`
- **THEN** `eval_builtin` 以 `"selector-append"` 为 name 被调用，结果为 `.a.b`

#### Scenario: list.append 以 hyphenated key 路由
- **WHEN** 输入含 `list.append("a", "b")`
- **THEN** `eval_builtin` 以 `"list-append"` 为 name 被调用，结果为 `a, b`

#### Scenario: map.get 以 hyphenated key 路由
- **WHEN** 输入含 `map.get($m, k)`
- **THEN** `eval_builtin` 以 `"map-get"` 为 name 被调用

#### Scenario: 无前缀短名单名保留
- **WHEN** 输入为 `a {b: abs(-5px)}`
- **THEN** 仍以 `"abs"` 单短名路由，结果为 `5px`

#### Scenario: CSS 类选择器不触发函数 dispatch
- **WHEN** substitute_vars 处理 selector `.container-fluid`
- **THEN** 输出仍保留 `.container-fluid` 不变

#### Scenario: 属性值中的点分非函数字符串
- **WHEN** 输入为 `a {background: url(foo.bar.png)}`
- **THEN** 输出中 `url(foo.bar.png)` 被正确保留

#### Scenario: 数值中的小数点
- **WHEN** 输入为 `a {width: 1.5px}`
- **THEN** 输出保留 `1.5px` 不变
