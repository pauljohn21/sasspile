## Why

sasspile 的 `values/calculation` 目录有 345 个失败 case（通过率 64.9%），是单一最大失败来源。根因是 Sass 内置 `math.*` 函数与 CSS 原生数学函数（`round()`, `clamp()`, `rem()`, `mod()`, `min()`, `max()`, `var()`）的行为混淆——CSS 函数应保持原样输出、不在 Sass 层简化，且需要独立的 dispatch 路径。此专项直接对标 345 个 case 中约 ~80 个可修复项，目标将通过率从 65% 推至 71%+。

## What Changes

- **新增 CSS 原生数学函数独立分派路径**：`round()`, `clamp()`, `rem()`, `mod()`, `min()`, `max()` 在 CSS 上下文中不再进入 Sass 简化管线
- **实现 CSS `round()` 策略取整**：支持 `nearest`/`up`/`down`/`to-zero` 四种策略 + step 参数
- **实现 CSS `rem()` 和 `mod()`**：模运算（rem 向零取余，mod 向负无穷取余）+ 无穷/NaN 特殊行为
- **修复 `clamp()` 单位一致性**：混合单位报错、ClAmP 大小写保留
- **修复 `var()` 序列化**：尾部逗号空格 (`var(--c,)` → `var(--c, )`)、fallback 表达式保留不简化
- **修复 vendor prefix 函数名大小写**：`-A-CALC` → `-a-calc`, `-C-ELEMENT` → `-c-element` 规范化
- **修复 CSS function 内注释处理**：`calc(//\n c)` 应序列化为 `calc( c)`

## Capabilities

### New Capabilities

- `css-math-dispatch`: CSS 原生 math 函数（round/clamp/rem/mod/min/max）的独立分派，绕过 Sass 简化管线
- `css-round-strategy`: CSS `round()` 四种策略（nearest/up/down/to-zero）+ step 取整算法
- `css-var-serialization`: `var()` 序列化修正（尾部逗号空格、fallback 懒求值、大小写保留）

### Modified Capabilities

- `calc-simplification`: 修改 calc 简化触发条件——识别 CSS 原生函数并跳过简化
- `css-function-serial`: vendor prefix 函数名（如 `-a-calc`, `-c-element`）小写规范化

## Impact

- **代码**: `src/css/` 增加 css_math.rs 模块, `src/eval/builtin/calc_ast.rs` 修改分派逻辑, `src/serialize/display.rs` 修改 var() 序列化
- **测试**: `tests/` 可能增加 css_math_test.rs
- **性能**: 无显著影响
- **兼容性**: 仅影响 CSS 原生 math 函数行为，不影响 Sass math 模块
