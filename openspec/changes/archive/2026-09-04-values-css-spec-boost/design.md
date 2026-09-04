## Context

sasspile 当前 sass-spec `values` 通过率 45%（533/1169），`css` 通过率 52%（419/830）。根因分析发现 7 个核心缺陷，集中在 calc 简化器的单位兼容性检查、infinity 单位保留、plain CSS 错误检测三个方面。

当前架构：`calc_simplify.rs` 消费 `CalcNode` AST（move 语义），`simplify_calc_node(node: &CalcNode) -> Result<CalcNode, CalcError>` 递归简化。`plain_css.rs` 的 `check_plain_css_value` 接收 `&Value` 返回 `Result<()>`。`ops.rs` 的 `div`/`modulo` 函数接收 `&Value` 返回 `Result<Value>`。

所有修复必须遵守函数式约束：move 语义 + `self→Self` 链式 + 迭代器链替代 for+push + match 替代 if-else 链 + `?` 传播替代 match Err。

## Goals / Non-Goals

**Goals:**
- 修复 calc 复合单位简化错误（~530 失败）
- 修复 plain CSS 错误检测不完整（~200 失败）
- 修复 infinity/NaN 单位丢失（~50 失败）
- 修复 +0/-0 模运算（~26 失败）
- 修复 slash 除法语义（~15 失败）
- 完善 CSS 数学函数简化（~60 失败）
- 修复选择器规范化差异（~45 失败）

**Non-Goals:**
- 不实现 `.sass` 缩进式语法支持（已跳过）
- 不重构整体架构（DDD 重构是独立变更）
- 不修改颜色系统（颜色测试已跳过）
- 不修改 tracing/OTel 架构

## Decisions

### D1: calc 单位兼容性检查改为 "尝试简化，不兼容则保留"

**决策**：当 `simplify_calc_node` 遇到不兼容单位（如 `1px * 1rad`）时，不返回 `CalcError`，而是保留 `BinaryOp` 节点原样。最终序列化时输出 `calc(1px * 1rad)`。

**理由**：SCSS 规范要求 `calc()` 内部不兼容单位运算保留为 `calc()` 表达式（不简化），而非报错。只有 `calc(1deg + 1s)` 这种加减法才报错；乘除法保留原样。

**替代方案**：返回 `CalcError::IncompatibleUnits` → 被 reject，因为乘除法不兼容单位是合法的（如 `calc(1px * 1rad)`）。

### D2: infinity 单位保留改为 "收集所有单位"

**决策**：`ops.rs` 的 `div` 函数在除零时，将结果从 `Value::Number(inf, unit)` 改为 `Value::Calc("calc(infinity * 1px * 1em)")`，通过 `units_to_calc_string` 函数收集所有分子/分母单位。

**理由**：sass-spec 要求 `math.div(1px * 1em, 0)` 输出 `calc(infinity * 1px * 1em)`，当前只保留第一个单位。

**实现**：新增 `fn format_infinity_with_units(numerator: &[String], denominator: &[String]) -> String`，用迭代器链构建 `infinity * 1px * 1em` 或 `infinity / 1px` 格式。

### D3: +0/-0 模运算 — 词法分析修复

**决策**：在 `lex/mod.rs` 中，`+0` 和 `-0` 应被解析为 `Token::Number(0.0)` 而非 `Token::Plus` + `Token::Number(0)`。

**理由**：`+0 % +1` 中的 `+0` 是一个数值，不是 `+` 运算符后跟 `0`。SCSS 规范中 `+0` 和 `-0` 是合法的数字字面量。

**替代方案**：在 parser 层合并 `Plus` + `Number` → 被 reject，因为词法层就应该识别符号数字。

### D4: slash 除法 — 在非 calc 上下文执行数值除法

**决策**：在 `parse/expr/mod.rs` 的 Pratt 解析器中，`/` 在非 calc 上下文中应被解析为除法运算符（如果两侧都是数字）。结果用 `Value::Number` 表示。

**理由**：SCSS 的 `/` 除法虽然已弃用，但仍然生效。`1/2` 应得到 `0.5`，不是字符串 `1/2`。

**实现**：在 `parse_expr_slash` 中，当 `/` 两侧都是 `Value::Number` 时执行除法，否则保留为 `Slash` 分隔符。

### D5: CSS 数学函数简化 — 在 calc_simplify 中统一处理

**决策**：`CalcNode::Func { name, args }` 的简化逻辑扩展：当 `name` 为 `min`/`max`/`clamp`/`round`/`mod`/`rem` 且所有 `args` 是纯数字时，尝试计算。

**理由**：`min(1px, 2px)` 应简化为 `1px`，`round(10px, 3px)` 应简化为 `9px`。

**实现**：`simplify_calc_node` 中 `Func` 分支用 match 分别处理各函数名，参数兼容性检查复用 `units_compatible`。

### D6: plain CSS 错误检测扩展

**决策**：`plain_css.rs` 的 `check_plain_css_value` 扩展检测范围，覆盖 `calc()` 内部的 `$var`、`#{}`、`&`、命名空间函数调用等。

**理由**：`.css` 文件中 `calc($var)` 应报错 "Sass variables aren't allowed in plain CSS."

**实现**：新增 `check_plain_css_calc_inner(value: &Value) -> Result<()>` 递归检测 calc 内部表达式。用 match 分派 Value 枚举类型，`?` 传播错误。

## Risks / Trade-offs

- [calc 简化保留 BinaryOp 可能增加输出长度] → 仅在不兼容时保留，兼容时正常简化
- [+0/-0 词法修复可能影响其他测试] → 需验证 `compile_test` 和 `stage_test` 不回归
- [slash 除法执行可能改变现有行为] → 仅在两侧都是 Number 时执行，其他情况保留 Slash 分隔符
- [plain CSS 递归检测可能性能下降] → 仅在 plain_css 模式下触发，正常 SCSS 不受影响
