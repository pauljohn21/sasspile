## Context

sass-spec `values/calculation/` 域中有 401 个失败，最大的两个单点根因：
1. `round()` 当前仅允许 1 参数（`math.rs:70-93`），但 CSS 规范要求 `round(strategy, number, step?)` 2-3 参数形式（103 failures）
2. 三角函数参数为字符串形式 `"infinity"/"-infinity"/"nan"` 时被 `validate_single_number` 拒绝（`math_helpers.rs:67-82`），虽然后续 `extract_unitless` 已经能处理这些字符串（25 failures）

两个修复相互独立，无交叉影响。

## Goals / Non-Goals

**Goals:**
- round() 支持 CSS 策略取整的 2-3 参数形式
- 三角函数接受特殊浮点值（infinity/NaN）作为合法输入
- 完全不破坏现有 round(x) 1 参数行为

**Non-Goals:**
- 不修改 calc/error 类 96 个 DIFF（本变更聚焦 ERR 类修复）
- 不修改 % 单位兼容性（仅 7 个，次要问题）
- 不涉及 CSS calc-simplify 逻辑

## Decisions

### Decision 1: round 策略分派实现方式
**方案 A（选定）**: 在 `math.rs` 的 round match 分支内直接处理策略分派，复用现有 `validate_single_number` 提取数字后接策略匹配逻辑
- 优点：修改集中在一个文件中，不引入辅助函数
- 缺点：round 分支变长

**方案 B**: 提取到独立 `math_round.rs` 模块
- 优点：模块化
- 缺点：引入新文件，而变更逻辑 < 50 行

**选 A** 理由：round 策略分派约 30 行逻辑，且只被一个调用点使用，不需要额外的模块抽象。

### Decision 2: infinity/NaN 验证修复位置
**方案 A（选定）**: 修改 `validate_single_number` 使其接受字符串特殊值
- 优点：一处修改覆盖所有调用者（abs/ceil/floor/round/sin/cos/tan/asin/acos/atan/exp/sign）
- 风险：可能让某些不应接受字符串特殊值的函数也接受

**方案 B**: 在 `trig_func` / `inverse_trig_func` 内预先转换字符串特殊值为 Number
- 优点：不改动共享的 `validate_single_number`
- 缺点：逻辑碎片化

**选 A** 理由：字符串 `"infinity"`/`"nan"` 在 Sass 语义中就是合法数字，让 `validate_single_number` 接受它们符合语言规范。且 abs/ceil/floor 等函数也应接受这些值（如 `abs(infinity)` = infinity）。

### Decision 3: round 策略语义
CSS Values Level 4 规范：
| 策略 | 语义 | Rust 等价 |
|------|------|-----------|
| `up` | 向 +∞ 取整 | `f64::ceil()` |
| `down` | 向 -∞ 取整 | `f64::floor()` |
| `nearest` | 四舍五入（ties to +∞） | `f64::round()` |
| `to-zero` | 向零截断 | `f64::trunc()` |

带 step 的 round: `result = step * round(strategy, number / step)`

## Risks / Trade-offs

- **[Risk]** `validate_single_number` 放宽后可能被所有调用者继承 → **Mitigation**: 仅在 `math.rs` round 分支和 trig 路径中实际处理字符串特殊值，其他函数 (`abs/ceil/floor`) 的 `Value::String` match 分支仍会 reject 非数字字符串
- **[Risk]** 修改 math.rs 需注意不超出 500 行限制 → **Mitigation**: 当前 math.rs 已经较长，如超出限制则将 round 策略辅助函数提取到独立文件
