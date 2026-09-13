## 1. MATH_NAMES 注册表补全

- [ ] 1.1 在 `src/eval/builtin/dispatch.rs` 的 `MATH_NAMES` 数组中添加 `("math.exp", "exp")`、`("math.sign", "sign")` 条目
- [ ] 1.2 在 `MATH_NAMES` 中添加 `("math.hypot", "hypot")`、`("math.atan2", "atan2")`、`("math.log", "log")` 条目
- [ ] 1.3 运行 `cargo test --test compile_test` 确认无回归
- [ ] 1.4 运行 `SPEC_STORE_CMD=run cargo test --test spec_store` 确认新增通过数

## 2. calc 简化器扩展

- [ ] 2.1 在 `src/eval/value/calc_simplify.rs` 的 `simplify_binary` 函数中增加 `CalcOp::Mul` 分支：两操作数均为 `CalcNode::Number` 时直接计算乘积
- [ ] 2.2 在 `simplify_binary` 中增加 `CalcOp::Div` 分支：两操作数均为 Number 且除数非零时直接计算商
- [ ] 2.3 在 `simplify_binary` 的 `CalcOp::Sub` 分支中检测右子为 `UnaryOp::Neg` 的模式，转换为 `Add`
- [ ] 2.4 在 calc 解析阶段（`literals.rs` 或 calc 解析器）将标识符 `e` 映射为 `CalcNode::Number(std::f64::consts::E, None)`
- [ ] 2.5 验证 infinity/NaN 在 calc 运算中通过 `f64` 默认语义自然吸收（`infinity * n` → `infinity`）
- [ ] 2.6 运行 `cargo test --test compile_test` + `SPEC_STORE_CMD=run` 验证

## 3. min/max 单位转换修复

- [ ] 3.1 修改 `src/eval/builtin/math.rs` 中 `min`/`max` 实现的单位处理逻辑：当所有参数为兼容单位时以最大数值参数的单位输出；当最大值为 unitless 时输出 unitless
- [ ] 3.2 添加 unknown unit 处理：当所有参数单位相同且不可转换时，仅比较数值大小并返回最大者的原始值
- [ ] 3.3 运行 `cargo test --test compile_test` + `SPEC_STORE_CMD=run` 验证

## 4. 集成验证

- [ ] 4.1 运行 `SPEC_STORE_CMD=stats cargo test --test spec_store` 确认通过率提升（目标 7647 → 7710+）
- [ ] 4.2 运行核心测试全量 `cargo test --test compile_test --test reactor_test --test stage_test --test ast_test --test common_test --test interp_test` 确认无回归
- [ ] 4.3 运行 `cargo test --test ep_full` 确认无回归
- [ ] 4.4 更新 AGENTS.md 中的 sass-spec 基线数字
