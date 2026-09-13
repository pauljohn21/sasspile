## 1. 基础设施：simplify_calc 签名改造

- [x] 1.1 修改 `src/eval/value/calc.rs` 中 `simplify_calc` 函数签名从 `pub(crate) fn simplify_calc(s: &str) -> Value` 改为 `-> Result<Value>`
- [x] 1.2 修改 `src/eval/value/mod.rs` line 138 的调用点：`Ok(Self::simplify_calc(s))` → `Self::simplify_calc(s)`（移除多余 `Ok` 包装）
- [x] 1.3 确认 `try_ast_simplify` 内部 `.ok()?` 逻辑不变（Err 仍转为 None 触发字符串降级路径）

## 2. Pre-check 实现：顶层二元不兼容检测

- [x] 2.1 在 `calc.rs` 添加辅助函数 `check_top_level_incompat(node: &CalcNode) -> Option<String>`：匹配顶层 `Op { op: Add|Sub, left: Number(_, Some(u1)), right: Number(_, Some(u2)) }` 且 `unit_group(u1).is_some() && unit_group(u2).is_some() && unit_group(u1) != unit_group(u2)` → 返回错误消息
- [x] 2.2 `check_top_level_incompat` 必须排除 `%` 单位（`unit_group("%")` 返回 None，自然排除）
- [x] 2.3 在 `simplify_calc` 中、`try_ast_simplify` 之前调用 pre-check：若 Some(msg) → `return Err(SassError::Eval(msg))`

## 3. 单位组辅助函数

- [x] 3.1 在 `calc_units.rs` 提升 `unit_group` 为 `pub(crate)` （calc.rs 通过 `super::calc_units` 路径访问）
- [x] 3.2 将 `UnitGroup` 枚举从私有提升为 `pub(crate)`

## 4. 错误消息格式

- [x] 4.1 错误消息格式为 `"{value1}{unit1} and {value2}{unit2} are incompatible."` — 匹配 Sass 输出风格
- [x] 4.2 确认 `tracing::event!` 记录调试信息（span 字段包含 incompat 详情）

## 5. 回归测试 + 验证

- [x] 5.1 运行 `cargo test --test compile_test --test reactor_test --test stage_test --test ast_test --test common_test --test interp_test --test bs_spec --test ep_full` 确认 112/112 核心测试通过（所有 ok）
- [x] 5.2 运行 `SPEC_STORE_CMD=run cargo test --test spec_store -- --nocapture` 更新 SQLite snapshot
- [x] 5.3 运行 `SPEC_STORE_CMD=stats cargo test --test spec_store -- --nocapture` 统计 `values/calculation` 通过率提升
- [x] 5.4 验证 `values/calculation/calc/error/known_incompatible` 路径：0/218 → **177/218** PASS（+177）
- [x] 5.5 验证无回归：抽查 `calc(3px * 2 + 1%)`、`calc(1px + 1%)`、`calc(1px + 1in)` 行为不变，`calc(1px + 1deg)` 正确报错

## 6. 文档更新

- [x] 6.1 更新 `AGENTS.md` sass-spec 基线数字
- [ ] 6.2 在 openspec/changes/calc-known-incompat/ 添加完成记录
