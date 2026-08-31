# Proposal: Nested Output Format + Quick Fixes

## Problem

sass-spec 通过率 54.6%（2929/5362），2433 个失败。两大根因：

1. **嵌套输出格式**（~1633 个失败）— Evaluator 的 `RuleBuilder` 在求值阶段展平嵌套选择器（`a { b { c: d } }` → `a b { c: d }`），但 sass-spec expanded 模式期望保留嵌套结构
2. **快速修复类**（~300+ 个失败）— 参数验证过严、类型转换缺失、运算符不支持

## Solution

### Phase 1: Quick Fixes（预期 +50~80 PASS）

- 修复 `merge_args` 对多参数/命名参数的接受逻辑
- 修复 `"0" is not a number` — 字符串到数字的隐式转换
- 修复 `Unsupported + operation` — Calc/字符串拼接
- 修复 `set-nth` / `if` 等函数参数验证

### Phase 2: Nested Output Format（预期 +100~300 PASS）

重构 `RuleBuilder` 保留嵌套结构：

- `RuleBuilder::push` 不再合并选择器，而是保留子规则在 `children` 中
- `flatten_nodes` 只在 compressed 模式调用
- `serialize_expanded` 已支持嵌套 children 输出
- `&` 引用和 `@media` 嵌套正确处理

## Scope

- `src/eval/rule.rs` — RuleBuilder 重构
- `src/css/mod.rs` — 序列化器按模式分流
- `src/eval/value/ops.rs` — 运算符修复
- `src/eval/builtin/*.rs` — 参数验证修复

## Key Risks

- 嵌套输出重构影响所有测试，需要逐目录验证
- `@media` 嵌套和 `@at-root` 行为可能改变
- `&` 引用选择器在嵌套模式下的行为差异
