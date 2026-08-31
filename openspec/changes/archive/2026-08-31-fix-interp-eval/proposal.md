## Why

裸插值 `#{$var}` 出现在 CSS 声明值中时（如 `content: #{$a};`），求值结果不正确——输出字面 `$a` 而非变量的实际值。这导致 sass-spec `directives/use/with/distributed_vars.hrx` 和其他使用裸插值的场景失败。根因是 `eval_interp_str` 只处理嵌套 `#{...}` 模式，不处理纯变量引用 `$var`。

## What Changes

- 修复 `eval_interp_str`（`src/eval/value/display.rs`）：当字符串不含 `#{` 嵌套时，用 `eval_simple_expr` 对整个字符串求值
- 修复 `eval_value` 中 `Value::Interp` 分支（`src/eval/value/mod.rs`）：确保 `Interp("$a")` 走 `eval_simple_expr` 而非逐字符输出

## Capabilities

### New Capabilities
- `interp-eval`: 裸插值 `#{$var}` 在 CSS 声明值中的求值行为规范

### Modified Capabilities

## Impact

- `src/eval/value/display.rs` — `eval_interp_str` 函数修改
- `src/eval/value/mod.rs` — `eval_value` 中 `Value::Interp` 分支可能调整
- `tests/` — 新增裸插值测试用例
- 不影响 `!default` config 验证逻辑（已修复）
