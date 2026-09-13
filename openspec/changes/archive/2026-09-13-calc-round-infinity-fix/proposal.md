## Why

当前 sass-spec calculation 域有 401 个失败（占总失败数 33%），其中最大的单点根因有两个：(1) `round()` 函数仅支持 1 参数形式但 CSS 规范要求支持 2-3 参数的 `round(strategy, number, step?)` 形式，导致 103 个 spec 被错误拒绝；(2) 三角函数（sin/cos/tan/asin/acos/atan）在参数为 infinity/-infinity/NaN（字符串形式传入）时被类型校验层误拒为"not a number"，共 25 个 spec 失败。

这两个问题根因独立、修复范围明确，是当前最小投入最大回报的突破口。

## What Changes

- **round() 函数扩展**：在 `src/eval/builtin/math.rs` 中为 `round` 添加 2-3 参数形式的 CSS `round(strategy, number, step?)` 策略取整（up/down/nearest/to-zero）
- **infinity/NaN 数字接受**：修改 `validate_single_number`（`src/eval/builtin/math_helpers.rs`）使三角函数能接受字符串形式传入的 `"infinity"`、`"-infinity"`、`"nan"` 为合法数字

## Capabilities

### New Capabilities
- `css-round-strategy`: CSS round 函数的 strategy+step 取整协议（up/down/nearest/to-zero），支持可选 step 参数

### Modified Capabilities
- `trig-special-values`: sin/cos/tan/asin/acos/atan 接受 infinity/-infinity/NaN 作为合法输入

## Impact

- `src/eval/builtin/math.rs` (round 分支扩展)
- `src/eval/builtin/math_helpers.rs` (validate_single_number)
- `src/eval/builtin/math_trig.rs` (trig_func / inverse_trig_func 调用路径)
- 不破坏现有 `round(x)` 1 参数行为（向后兼容）
- 预估通过率：64.6% → 65.7%（+128 cases）
