# calc-simplification Spec

## 需求 1: calc 表达式 AST 类型

### 1.1 类型定义

系统必须定义 `CalcNode` enum 表示 calc 表达式 AST：
- `Number(f64, Option<String>)` — 数字 + 可选单位
- `Constant(CalcConst)` — CSS 常量 `pi`/`e`
- `Var(String, Option<Box<CalcNode>>)` — CSS 变量引用 `var(--c)` 或 `var(--c, fallback)`
- `BinaryOp { op, left, right }` — 二元运算
- `Func { name, args }` — 嵌套 CSS 数学函数（min/max/clamp/round/rem/mod/abs/sign 等）

`CalcOp` enum: `Add`/`Sub`/`Mul`/`Div`
`CalcConst` enum: `Pi`/`E`

### 1.2 所有类型必须 derive `Debug, Clone, PartialEq`

### 1.3 CalcNode 必须实现 `std::fmt::Display`

## 需求 2: calc 表达式解析器

### 2.1 parse_calc_expr 函数

`pub fn parse_calc_expr(input: &str) -> Option<CalcNode>` — 将 calc() 内部表达式解析为 `CalcNode` AST。

### 2.2 解析能力

解析器必须支持：
- 运算符优先级：`*`/`/` 优先于 `+`/`-`
- 括号嵌套改变优先级
- 数字 + 单位（`1px`、`1.5deg`、`1e2`）
- CSS 常量 `pi`、`e`
- CSS 变量 `var(--c)` 和 `var(--c, fallback)`
- CSS 数学函数 `min()`、`max()`、`clamp()`、`round()`、`rem()`、`mod()`、`abs()`、`sign()`、`exp()`、`pow()`、`sqrt()`、`log()`、`sin()`、`cos()`、`tan()`、`asin()`、`acos()`、`atan()`、`atan2()`、`hypot()`
- 无空白操作符：`1px/2px`（需与 `1px / 2px` 等价）

### 2.3 降级策略

无法完全解析的输入返回 `None`，调用方保留原始 `calc()` 字符串。

## 需求 3: 单位兼容性

### 3.1 单位兼容性分组

以下单位的同组内可换算：
- **长度**：`px`、`cm`、`mm`、`in`、`pt`、`pc`、`q`
- **角度**：`deg`、`rad`、`grad`、`turn`
- **时间**：`s`、`ms`
- **频率**：`Hz`、`kHz`
- **分辨率**：`dpi`、`dpcm`、`dppx`
- **无单位**：`""`

`%` 不属于任何组（兼容性取决于上下文，calc 中不做换算）。

### 3.2 units_compatible 函数

`pub fn units_compatible(a: &str, b: &str) -> bool` — 同组返回 `true`，不同组返回 `false`。

### 3.3 convert_unit 函数

`pub fn convert_unit(value: f64, from_unit: &str, to_unit: &str) -> Option<f64>` — 返回换算后的值。不兼容时返回 `None`。

## 需求 4: calc 简化算法

### 4.1 simplify_calc_node 函数

`pub fn simplify_calc_node(node: &CalcNode) -> Result<CalcNode, CalcError>` — 递归简化 AST。

### 4.2 简化规则

1. **递归简化**：先简化子表达式，再处理当前节点
2. **常量折叠**：`Number(a, u) + Number(b, u)` → `Number(a+b, u)`（同单位）
3. **单位转换**：`Number(a, "deg") + Number(b, "rad")` → 先转换为同单位再计算
4. **不兼容单位**：`Number(a, "deg") + Number(b, "s")` → `CalcError::IncompatibleUnits`
5. **乘法规则**：`Number * unitless` → 带单位结果；`unitless * Number` → 带单位结果
6. **除法规则**：`Number / unitless` → 带单位结果；`Number(x, u) / Number(y, u)` → `Number(x/y, None)`（单位抵消）；`Number / 0` → `CalcError::DivisionByZero`
7. **常量替换**：`Constant(Pi)` → `Number(3.141592653589793, None)`
8. **var() 保留**：`Var` 节点不可简化，保留原样
9. **Func 简化**：递归简化参数，如果所有参数都是纯数字则尝试计算

### 4.3 CalcError

```rust
pub enum CalcError {
    IncompatibleUnits(String, String),  // "1deg", "1s"
    DivisionByZero,
    SyntaxError(String),
}
```

## 需求 5: simplify_calc 入口重写

### 5.1 现有 simplify_calc 重写

`simplify_calc(s: &str) -> Value` 函数必须改为使用新 AST：
1. 尝试 `parse_calc_expr` 解析内部表达式
2. 如果解析成功，调用 `simplify_calc_node`
3. 如果简化成功且结果是纯数字 → 返回 `Value::Number`
4. 如果简化成功但含 var/func → 序列化为 `calc(...)` 字符串
5. 如果返回 `CalcError::IncompatibleUnits` → 传播为 `SassError`
6. 如果解析失败 → 降级到现有字符串处理逻辑

### 5.2 保持现有接口兼容

`simplify_calc` 的函数签名不变（接收 `&str` 返回 `Value`），确保调用方不受影响。

## 需求 6: CSS 数学函数简化

### 6.1 round() 简化

`round(value, multiple)` — 当两个参数都是同单位纯数字时计算结果。
`round(10px, 3px)` → `9px`（最近的 3px 倍数）

### 6.2 rem() / mod() 简化

`rem(10px, 3px)` → `1px`（与 `mod` 行为一致）
`mod(10px, 3px)` → `1px`

### 6.3 abs() 简化

`abs(-5px)` → `5px`
`abs(var(--c))` → 保留 `abs(var(--c))`

### 6.4 sign() 简化

`sign(-5)` → `-1`
`sign(0)` → `0`
`sign(5)` → `1`

### 6.5 三角/指数/对数函数简化

`sin(0)` → `0`
`cos(0)` → `1`
`sqrt(4)` → `2`
`pow(2, 3)` → `8`
`exp(0)` → `1`
`log(1)` → `0`

当参数含 var/func 时保留原样。

## 需求 7: 不兼容单位错误检测

### 7.1 错误格式

当 `calc(1deg + 1s)` 检测到不兼容单位时，必须返回错误：

```
Error: 1deg and 1s are incompatible.
  ,
1 | a {b: calc(1deg + 1s)}
  |            ^^^^^^^^^
  '
  input.scss 1:12  root stylesheet
```

### 7.2 错误传播

`CalcError` 必须在 `simplify_calc` 中被捕获并转换为 `SassError`，通过 `?` 传播到求值器。
