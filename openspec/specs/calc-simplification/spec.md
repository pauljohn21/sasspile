# Spec: calc-simplification

## Overview

Sass `calc()` 表达式的求值和简化逻辑，包括 CSS `round()`/`mod()`/`rem()` 函数实现。

## Requirements

### simplify_calc 函数

`src/eval/value/mod.rs` 中的 `simplify_calc` 函数负责简化 `Value::Calc` 字符串：

1. **纯数字简化**：`calc(1px)` → `Value::Number(1, "px")`
2. **CSS 常量替换**：`calc(pi)` → `3.1415926536`，`calc(e)` → `2.7182818285`（大小写不敏感）
3. **括号去除**：`calc((1px))` → `1px`（纯数字括号去除）
4. **科学计数法**：`calc(1e2px)` → `100px`，`calc(1.5e-2px)` → `0.015px`
5. **同单位算术**：`calc(1px + 2px)` → `3px`，`calc(1px - 2px)` → `-1px`
6. **乘除法简化**：`calc(2px * 3)` → `6px`，`calc(6px / 2)` → `3px`
7. **嵌套 min/max**：`calc(max(1px, 2px))` → `2px`，`calc(min(3px, 1px))` → `1px`
8. **clamp 简化**：`clamp(1px, 2.5px, 3px)` → `2.5px`（三个同单位数字）
9. **乘除法括号去除**：`calc(1px + (2% * var(--c)))` → `calc(1px + 2% * var(--c))`
10. **常量表达式替换**：`calc(pi * 2)` → `6.2831853072`，`calc(pi * (1% + 1px))` → `calc(3.1415926536 * (1% + 1px))`

### CSS round() 函数

`src/eval/builtin/math.rs` 中的 `css_round` 函数支持 1-3 参数：

- **1 参数**：`round(3.5)` → `4`（传统 math.round）
- **2 参数**：`round(117, 25)` → `125`（默认 nearest 策略）
- **3 参数**：`round(down, 5px, 25px)` → `0px`（指定策略）

**四种策略**：
- `nearest`：最接近的倍数（默认）
- `up`：向上舍入到倍数
- `down`：向下舍入到倍数
- `to-zero`：向零舍入

**单位转换**：兼容单位自动转换（如 `round(117cm, 25mm)` → `117.5cm`），不兼容单位保留 `round()` 输出。

### CSS mod() 函数

`css_mod` 函数实现 floored modulo：`n - s * floor(n / s)`

- `mod(7, 3)` → `1`
- `mod(-7, 3)` → `2`（结果符号跟随除数）
- 不兼容单位保留 `mod()` 输出

### CSS rem() 函数

`css_rem` 函数实现 truncated modulo：`n - s * trunc(n / s)`

- `rem(7, 3)` → `1`
- `rem(-7, 3)` → `-1`（结果符号跟随被除数）
- 不兼容单位保留 `rem()` 输出

### 单位转换因子

`unit_conversion_factor` 函数支持以下单位组：

| 类别 | 单位 | 基准 |
|------|------|------|
| 长度 | px, in, cm, mm, pt, pc, q | px |
| 角度 | deg, grad, rad, turn | deg |
| 时间 | s, ms | s |
| 频率 | hz, khz | Hz |
| 分辨率 | dpi, dpcm, dppx | dpi |

### Math 函数 Calc 参数透传

以下 math 函数接收 `Value::Calc` 参数时输出 `func(expr)` 格式：

- `abs`/`ceil`/`floor`/`round`/`sqrt`/`sin`/`cos`/`tan`/`asin`/`acos`/`atan`
- `pow`/`log`/`hypot`/`atan2`

### calc 函数名大小写不敏感

`CaLc(1px)` → `1px`。Parser 层面使用 `eq_ignore_ascii_case` 匹配 `calc`/`clamp`/`env`/`var`。
