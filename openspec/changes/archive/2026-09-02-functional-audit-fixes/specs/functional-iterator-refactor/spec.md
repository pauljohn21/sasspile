## ADDED Requirements

### Requirement: combine_selectors MUST use iterator chain instead of for+push

`Evaluator::combine_selectors` MUST 使用迭代器链（`flat_map` + `map` + `collect`）而非嵌套 `for` 循环 + `Vec::push`。

#### Scenario: combine_selectors uses flat_map
- **WHEN** 检查 `src/eval/rule.rs` 中 `combine_selectors` 函数
- **THEN** MUST 使用 `parents.iter().flat_map(|p| children.iter().map(|c| ...)).collect()` 模式，不得使用 `for p in &parents { for c in &children { result.push(...) } }`

#### Scenario: Selector order preserved
- **WHEN** 调用 `combine_selectors("a, b", "c, d")`
- **THEN** 返回 `"a c, a d, b c, b d"` — 外层（parent）优先序 MUST 与重构前一致

#### Scenario: & replacement still works
- **WHEN** 子选择器包含 `&`（如 `combine_selectors(".btn", "&:hover")`）
- **THEN** MUST 返回 `".btn:hover"` — `&` 替换逻辑 MUST 保持不变

#### Scenario: Empty parent handling
- **WHEN** 调用 `combine_selectors("", ".foo")`
- **THEN** MUST 返回 `".foo"` — 空 parent 直接使用 child

#### Scenario: All tests pass after refactor
- **WHEN** 运行完整测试套件
- **THEN** 202/202 测试 MUST 全通过
