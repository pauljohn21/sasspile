## MODIFIED Requirements

### Requirement: 不兼容单位错误检测
当 `calc()` 表达式中出现不兼容单位运算时，系统 SHALL 产生编译错误。

**细化**：对于顶层纯二元 Add/Sub 运算（`calc(1u1 ± 1u2)` 形式，其中 u1、u2 为不同物理量类别的单位），系统 MUST 在简化前检测到不兼容并立即报错，而非走 partial-simplification 路径静默保留。对于嵌套结构（如 `calc(1px * 1s + 1px * 1px)`），当前的 partial-simplification 行为保持不变（视为独立 Mul/Div 单位代数问题）。

#### Scenario: 顶层二元长度+角度报错
- **WHEN** 输入 `a {b: calc(1px + 1deg);}`
- **THEN** 编译失败，报告 "1px and 1deg are incompatible"

#### Scenario: 顶层二元时间+频率报错
- **WHEN** 输入 `a {b: calc(1s + 1hz);}`
- **THEN** 编译失败，报告 "1s and 1hz are incompatible"

#### Scenario: 嵌套 Mul 不兼容不触发顶层报错
- **WHEN** 输入 `a {b: calc(3px * 2 + 1%);}`
- **THEN** 编译成功，Mul 子表达式简化后保留 `calc(6px + 1%)`（partial-simplification 路径不被阻断）

#### Scenario: 兼容单位不受影响
- **WHEN** 输入 `a {b: calc(1px + 1in);}`
- **THEN** 编译成功，长度单位正确转换并简化

#### Scenario: % 单位不被误判为不兼容
- **WHEN** 输入 `a {b: calc(1% + 1px);}`
- **THEN** 编译成功，保留 calc() 包装（% 的兼容性由上下文决定）

## ADDED Requirements

### Requirement: simplify_calc 返回 Result
`simplify_calc` 函数签名从 `fn(&str) -> Value` 改为 `fn(&str) -> Result<Value>`，使得不兼容单位错误能传播到 `eval_value` 并转化为编译失败。

#### Scenario: 错误传播
- **WHEN** `simplify_calc` 在 pre-check 中检测到顶层二元不兼容
- **THEN** 返回 `Err(SassError::Eval("... are incompatible"))`
- **AND** 调用方 `eval_value` 通过 `?` 传播该错误
- **AND** 编译过程终止并报告错误位置

### Requirement: Pre-check 仅在顶层执行
不兼容单位检测 pre-check 仅在 `simplify_calc` 入口处对原始 AST 的顶层节点执行一次，不递归进入子表达式。

#### Scenario: 仅顶层触发
- **WHEN** 输入 `a {b: calc((1px + 1deg) * 2);}`
- **THEN** pre-check 看到顶层是 Mul，不触发报错（partial-simplification 处理嵌套）

#### Scenario: 嵌套不兼容不影响
- **WHEN** 输入 `a {b: calc(2 * (1s + 1hz));}`
- **THEN** pre-check 看到顶层是 Mul(2, Add(1s, 1hz))，不触发（Add 不是顶层）
