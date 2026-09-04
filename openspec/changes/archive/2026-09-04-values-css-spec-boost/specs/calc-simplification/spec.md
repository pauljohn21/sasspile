## MODIFIED Requirements

### Requirement: 需求 4: calc 简化算法

### 4.1 simplify_calc_node 函数

`pub fn simplify_calc_node(node: &CalcNode) -> Result<CalcNode, CalcError>` — 递归简化 AST。函数签名为不可变借用（`&CalcNode`），返回新 `CalcNode`，符合函数式不可变值语义。

### 4.2 简化规则

1. **递归简化**：先简化子表达式，再处理当前节点
2. **常量折叠**：`Number(a, u) + Number(b, u)` → `Number(a+b, u)`（同单位）
3. **单位转换**：`Number(a, "deg") + Number(b, "rad")` → 先转换为同单位再计算
4. **不兼容单位加减**：`Number(a, "deg") + Number(b, "s")` → `CalcError::IncompatibleUnits`
5. **乘法规则**：`Number * unitless` → 带单位结果；`unitless * Number` → 带单位结果
6. **乘法不兼容单位保留**：`Number(a, "px") * Number(b, "rad")` → 保留 `BinaryOp { Mul, Number(a, "px"), Number(b, "rad") }` 不简化，序列化输出 `calc(1px * 1rad)`
7. **除法规则**：`Number / unitless` → 带单位结果；`Number(x, u) / Number(y, u)` → `Number(x/y, None)`（单位抵消）；`Number / 0` → `CalcError::DivisionByZero`
8. **除法不兼容单位保留**：`Number(a, "px") / Number(b, "s")` → 保留 `BinaryOp { Div, ... }` 不简化，序列化输出 `calc(1px / 1s)`
9. **常量替换**：`Constant(Pi)` → `Number(3.141592653589793, None)`
10. **var() 保留**：`Var` 节点不可简化，保留原样
11. **Func 简化**：递归简化参数，如果所有参数都是纯数字且同单位则尝试计算

### 4.3 CalcError

```rust
pub enum CalcError {
    IncompatibleUnits(String, String),  // "1deg", "1s" — 仅加减法
    DivisionByZero,
    SyntaxError(String),
}
```

**关键变更**：乘除法不兼容单位不再返回 `CalcError`，而是保留 `BinaryOp` 节点原样。只有加减法不兼容单位才返回 `CalcError::IncompatibleUnits`。
