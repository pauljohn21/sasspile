## Context

`simplify_calc(s: &str) -> Value`（`src/eval/value/calc.rs` line 85）是 `Value::Calc` 求值的入口。现有流程：

```
Value::Calc(s) → is_pure_calc_expr(inner)? → simplify_calc(s)
                                          → try_ast_simplify(s)     // 优先：AST 解析 + 简化
                                          → simplify_calc_str(s)    // 降级：字符串处理
```

`try_ast_simplify` 解析为 `CalcNode` 后调用 `simplify_calc_node → simplify_recursive → simplify_op`。在 `simplify_op` 中，Add/Sub 遇到不兼容单位时走 partial-simplification：

```rust
Err(_) => {
    return Ok(CalcNode::Op { op, left: Box::new(left), right: Box::new(right) });
}
```

这将 `CalcError::IncompatibleUnits` 静默转为 `Ok(Op)`，最终输出不报错的 CSS。

**关键张力**：calc-css-boost 的 partial-simplification 对嵌套运算（如 `calc(3px * 2 + 1%) → calc(6px + 1%)`）是正确的 — Mul 简化的结果需要保留下来。不能简单地改回 `Err`。

**解决方案**：用 pre-check 区分两种情况：
1. **顶层纯二元不兼容**（`calc(1vmin + 1deg)`: Add 的两个子都是纯 Number，单位不兼容）→ 报错
2. **嵌套结构不兼容**（`calc(3px * 2 + 1%)`: Add 的左子是 Mul Op，右子是 Number）→ 走现有 partial-simplification

## Goals / Non-Goals

**Goals:**
- `calc(1u1 ± 1u2)` 中 u1、u2 属于不同物理量类别（length vs angle、time vs frequency 等）时产生编译错误
- partial-simplification 对嵌套结构保持不变
- 兼容单位的运算（如 `calc(1px + 1in)`）继续正确简化

**Non-Goals:**
- 修改 `simplify_op` 的 partial-simplification 逻辑本身（已验证正确）
- 处理 `Mul`/`Div` 路径的不兼容单位（这是另一个独立问题）
- `values/numbers/` 下的 infinity/NaN 多分子单位问题（单独 change 处理）

## Decisions

### Decision 1: Pre-check 在 simplify_calc 入口执行

**选择**：在 `simplify_calc(s: &str) -> Result<Value>` 中，解析 AST 后、调用 `simplify_calc_node` 之前，检查顶层结构。

**理由：**
- 不污染 `simplify_op` 的 partial-simplification 逻辑
- pre-check 只读 AST，无副作用
- 错误传播路径清晰：`simplify_calc` 返回 `Err(SassError::Unit(...))`

**替代方案**：修改 `simplify_op` 直接返回 Err — 被否决，因为这会破坏嵌套结构的 partial-simplification（错误提前终止整个递归）。

### Decision 2: simplify_calc 改为返回 Result<Value>

**选择**：`simplify_calc(s: &str) -> Result<Value>`（之前返回 `Value`）。

**调用点调整**：`src/eval/value/mod.rs` line 138:
```rust
// 修改前:
Some(inner) if is_pure_calc_expr(inner) => Ok(Self::simplify_calc(s)),
// 修改后:
Some(inner) if is_pure_calc_expr(inner) => Self::simplify_calc(s),
```

**理由**：`eval_value` 本身返回 `Result<Value>`，传播错误只需移除一层 `Ok()` 包装。

### Decision 3: 不兼容单位判断复用 calc_units::units_compatible

**选择**：pre-check 使用 `crate::eval::value::calc_units::units_compatible(u1, u2)`，与 `simplify_add_sub` 使用同一兼容表，保证一致。

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| pre-check 漏过非顶层嵌套不兼容 | 这不是问题 — 这些用例预期行为是保持 calc() 包装，不是报错 |
| 误报：% 与长度组合 | compat table 不含 `%`（`unit_group("%")` 返回 None），`units_compatible` 返回 false；需验证 spec 对 `1% + 1px` 的预期（已知为保留，无错误） |
| `1px + 2px` 等效简化路径受影响 | 不受影响 — pre-check 只拦截 incompat，compat 情况直接放行到 simplify_calc_node |
