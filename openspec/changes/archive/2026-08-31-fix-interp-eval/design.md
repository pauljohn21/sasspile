## Context

sasspile 的 `eval_interp_str`（`src/eval/value/display.rs:187`）负责求值 `#{...}` 插值内容。Lexer 扫描 `#{$a}` 时，提取 `#{}` 内部内容 `$a` 作为 `Token::Interp("$a")`。Parser 将其构建为 `Value::Interp("$a")`。Evaluator 在 `eval_value` 第 430 行调用 `eval_interp_str`。

`eval_interp_str` 当前逻辑：遍历字符找 `#{` 嵌套，只对 `#{...}` 内的表达式调用 `eval_simple_expr`。当字符串是纯变量引用 `$a`（不含嵌套 `#{`），逐字符输出 `$a`，不调用 `eval_simple_expr`。

## Goals / Non-Goals

**Goals:**
- `#{$var}` 裸插值在 CSS 声明值中正确求值为变量值
- 不破坏字符串内插值 `"#{$var}"` 的现有行为
- 不破坏 `#{complex(expr)}` 嵌套插值行为

**Non-Goals:**
- 不修改 Lexer 的 `#{}` 扫描逻辑
- 不修改 Parser 的 `Value::Interp` 构建逻辑
- 不涉及 `!default` config 验证（已修复）

## Decisions

### 决策 1：在 `eval_interp_str` 中对非嵌套内容 fallback 到 `eval_simple_expr`

**选择**：在 `eval_interp_str` 遍历结束后，如果结果字符串仍包含 `$`（变量引用），对整个字符串调 `eval_simple_expr`。

**替代方案**：在 `eval_value` 的 `Value::Interp` 分支直接调 `eval_simple_expr` 而非 `eval_interp_str`。但这会破坏 `#{a}#{b}` 多段拼接和 `prefix#{expr}suffix` 场景。

**理由**：`eval_interp_str` 的设计是处理「可能含嵌套插值的字符串」，fallback 到 `eval_simple_expr` 是最局部的修复，不影响多段拼接。

### 决策 2：优先用 `eval_simple_expr` 求值整个字符串

**选择**：在 `eval_interp_str` 开头，先用 `eval_simple_expr(s, env)` 尝试求值整个字符串。如果成功，返回结果；如果失败（非纯表达式），回退到逐字符扫描模式。

**理由**：纯变量引用 `$a`、数字 `42`、函数调用 `rgba(...)` 都会被 `eval_simple_expr` 正确处理。只有混合文本（如 `prefix#{expr}suffix`）才需要逐字符扫描。先尝试整体求值，失败再分段。

## Risks / Trade-offs

- [纯文本误求值] → `eval_simple_expr` 对非表达式的纯文本（如 `hello`）会尝试 Lexer + Parser 解析，可能误解析。Mitigation：`eval_simple_expr` 失败时回退逐字符输出，纯文本 `hello` 会被 parse 为 `Value::String("hello")` 正常处理。
- [性能] → 每次插值都先尝试 `eval_simple_expr`。Mitigation：插值场景不多，且 `eval_simple_expr` 对简单字符串的 Lexer+Parser 开销可忽略。
