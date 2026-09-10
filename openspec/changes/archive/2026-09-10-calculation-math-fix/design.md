## Context

`sasspile` 的 calc() 特殊常量（infinity、NaN、-infinity）处理存在系统性缺陷。诊断 `values/calculation` 553 个失败后，发现 454 个 DIFF + 99 个 ERR 集中在 5 个根因。

当前问题：
1. `extract_unitless()` 仅接受有限浮点数拒绝 `infinity`/`NaN`
2. calc 输出阶段未对 `1/0`、`-1/0`、`0/0` 做特殊常量简化
3. calc-size() 实现未解析运算符，导致 `80pxsize` 等错误输出
4. `type-of(calc(infinity))` 返回 `calculation` 而非 `number`

## Goals / Non-Goals

**Goals:**
- `extract_unitless` 接受 `infinity`/`-infinity`/`NaN` 关键词（作为一元特殊值解析）
- calc 输出规范化：大小写统一（`infinity`、`NaN`）、除法特殊值简化
- `calc-size()` 参数保留正确结构
- `atan`/`asin`/`acos` 遇 Sass 变量时保留函数形式

**Non-Goals:**
- 不完全重写 calc 简化引擎（涉及太长，留后续变更）
- 修改 `values/numbers` 的单位消约逻辑（独立变更）
- 修复 `values/calculation` 中非 math 相关的 case（如 slash-separated）

## Decisions

### 决策 1: extract_unitless 扩展方式

**选择**: 在 `extract_unitless` 函数中新增特殊常量分支，将字符串 `"infinity"`/`"-infinity"`/`"NaN"` 映射为 `f64::INFINITY`/`f64::NEG_INFINITY`/`f64::NAN`。

**替代方案**: 在调用 `extract_unitless` 前做关键词替换 — 选择不采用，因为会引入字符串→Value 转换的额外开销。

**影响**: `math_trig.rs` 中 `unitless_unary_func`/`inverse_trig_func`/`call_pow`/`call_log` 所有间接调用 `extract_unitless` 的函数自动受益。

### 决策 2: calc 输出特殊常量简化

**选择**: 在 calc 输出的 `format_number` 辅助函数中，对 `f64::INFINITY`/`f64::NEG_INFINITY`/`NAN` 做字符串映射，并在 CalcNode 序列化阶段简化 `infinity * N` → `infinity`。

**替代方案**: 在 parser 阶段就将 `1/0` 解析为特殊节点 — 改动太大。

### 决策 3: calc-size 保留

**选择**: 将 `calc-size` 加入 CSS 原生函数列表（`is_css_function`），使其参数完全保留；后续在层级 3 再实现完整的 calc-size 求值。

**替代方案**: 现在实现完整 calc-size 求值 — 超出本次变更范围。

## Risks / Trade-offs

- **[Risk]** `extract_unitless` 扩展后，原本期望报错的 case 可能不再报错 → 验证：仅接受明确的特殊常量关键词，不影响常规数字解析
- **[Risk]** infinity 传播规则可能与其他运算交互 → 验证：f64::INFINITY 的自然传播已符合 IEEE 754，Sass 规范与之一致
- **[Trade-off]** `calc-size` 暂只做保留不计算 → 少数 calc-size 的简化 case 仍会失败，留后续变更解决
