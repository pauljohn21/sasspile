## Context

`sasspile` 当前 sass-spec 通过率 63% (7647/12131)。`values/calculation` 区域（991 cases）仅 39% 通过，596 个失败可归为三个互不依赖的根因。本设计文档描述三个修复的技术方案。

### 当前状态

```
失败分布（values/calculation 内）:
├── 未分派函数（exp/sign/hypot/atan2/log）  ~50 cases
├── calc() 简化不完整                        ~15 cases  
└── max/min 单位错误                          ~10 cases
```

### 核心约束

- 纯 Rust，无 GC，所有权语义
- 全局 CSS 函数 → `Value::Call(name, args)` → `call_function` → `call_builtin` → `dispatch_builtin_module`
- 已知函数注册表由 `dispatch.rs` 的 const 数组宏生成（MATH_NAMES 等）
- calc 简化器消费 `CalcNode::Number`/`CalcNode::BinOp`，输出简化后的 `CalcNode`

## Goals / Non-Goals

**Goals:**
- 补全 `MATH_NAMES` 注册缺失，使 `exp`/`sign`/`hypot`/`atan2`/`log` 全局可调用
- 扩展 calc 简化器能力（乘除/符号/常量吸收/数学常量）
- 修复 `min`/`max` unitless 和 unknown-unit 边界
- sass-spec +63 cases

**Non-Goals:**
- 不实现新的 calc 嵌套简化（如 `calc(calc(...))` 深层展开）
- 不改变任何现有公开 API
- 不实现其他区域的失败修复（如 `values/numbers`、`css/plain` 等）

## Decisions

### Decision 1: MATH_NAMES 补全方式

**选项 A**：在 `dispatch.rs` 的 const 数组中添加缺失条目  
**选项 B**：使用 `#[builtin(alias = "...")]` proc-macro 属性  

**选择 A**。原因：最小改动、显式可控、无需修改宏实现。`dispatch.rs` 已是手工维护的 const 数组（即使存在 proc-macro 版本，当前编译使用的是 handmade const）。

**具体条目补充**：

```rust
("math.exp", "exp"),
("math.sign", "sign"),
("math.hypot", "hypot"),
("math.atan2", "atan2"),
("math.log", "log"),
```

### Decision 2: calc 简化器扩展方式

**选项 A**：在 `simplify_binary` 中增加 Mul/Div 分支处理常量折叠  
**选项 B**：新增独立的 `simplify_func_constants` 函数处理所有常量规则  

**选择 A**。原因：扩展现有 `simplify_binary` 逻辑最自然，乘除与加减对称；符号反转（`a - -b` → `a + b`）在 Sub 分支中已有检测逻辑，只需补充符号取反；特殊常量（infinity/NaN）在运算结果计算时通过 `f64` 数学规则自然吸收。

**新增逻辑**：
- `Mul`: 左右均为 `CalcOp::Number` → 直接计算数值乘积；一边为数字 1 → 返回另一边
- `Div`: 左边为 Number、右边为 Number（且非零）→ 直接计算数值商
- `Sub`: 右子为 `UnaryOp::Neg(x)` → 转为 `Add(x)`（符号反转）
- 任何运算产生 `infinity`/`-infinity`/`NaN` → `f64` 运算自然产出（如 `f64::INFINITY * 2.0 = infinity`）

**数学常量处理**（`e`）：
- `calc(e * 2)`：当 calc 内只有一个 `CalcNode::Number(Mul(Number(std::f64::consts::E), Number(2)))` 时，数值部分直接得 `5.4365636569...`
- 当前问题：`e` 被解析为标识符而非数字常量，简化器无法折叠
- **新增规则**：在 `CalcNode::Number` 构造阶段（解析时），识别 `e` 和 `pi` 标识符，将其映射为对应 `f64` 常量

### Decision 3: min/max 单位转换修复

**根因**：当前 `min`/`max` 实现使用 `first_unit` 作为转换目标。两个 bug：
1. `max(1px, 2.5, 0.9px)` — 最值 `2.5` 是 unitless，但结果被标记为 `px`
2. `max(1d, 2, 3e)` — 未知 unit `d`/`e` 时回退透传应能求数值大小

**修复策略**：

```rust
// 先找最大值（比较数值大小）
let mut result = first_val;
let mut result_unit = first_unit.clone();
for arg in args.iter().skip(1) {
    // 比较数值（同单位直接比，兼容单位转换后比）
    // 如果当前 arg > result，更新 result 和 result_unit
    // 如果 arg 是 unitless 且 result 是有单位，需要统一到同一基准比较
    // 当 unitless 成为最值时，result_unit = None
}
```

**unknown unit 处理**：对于 `1d`、`3e` 这样的未知单位（无法转换），视为"未知兼容但可比较数值"，即仅在该组所有值均为 unknown unit 时比较数值大小，返回最大者（仍保留 unknown unit 字符串）。

## Risks / Trade-offs

- **R1**: 添加 `("math.exp", "exp")` 可能影响 Sass `math.exp` 的已有调用路径 → 调用路径是 `math.exp` → `call_module_function` → 解析模块限定名 → 查找 MATH_NAMES → 分派到 `math::call` → 匹配 `"exp"`。新增全局 `"exp"` 映射不影响模块路径查找。**缓解**：验证 `math.exp` 和 `exp` 都能正确分派。

- **R2**: `calc(e * 2)` 中 `e` 映射到 `std::f64::consts::E` 需确认精度（spec 期望 `2.7182818285`，而 Rust 的 `E = 2.718281828459045`，截断到 10 位小数 = `2.7182818285` ✓）。**缓解**：实际编译验证。

- **R3**: min/max 修复中 zero-value 判断（`+0.0` vs `-0.0`）和 NaN 传播需确保 sass-spec 行为一致。**缓解**：f64 默认遵循 IEEE 754，NaN 比较返回 false 会将 result 保留为前一个值，需测试验证。

## Migration Plan

所有改动均为内部求值增强，无 schema/API 变更。无需 migration：
1. 修改代码
2. `SPEC_STORE_CMD=run cargo test --test spec_store` 验证
3. `SPEC_STORE_CMD=stats` 确认通过数增加

## Open Questions

- `atan2` 函数当前在 `math_trig.rs` 中的分派名为 `"atan2"`，但 `MATH_NAMES` 中没有对应条目。补全后是否会影响 unit 检查（`atan2/units/compatible` 和 `atan2/units/unknown` 用例仍会失败因为 atan2 本质是角度计算？）→ 需验证。
- `"exp"` 与 Sass 传统冲突：Sass 不要求 `exp` 用 `math.exp` 调用，CSS Color Level 4 和 `calc-size` 上下文使用 `exp` 全局名 → 确认无矛盾。
