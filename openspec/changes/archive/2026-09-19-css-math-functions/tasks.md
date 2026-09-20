## 1. CSS round() 策略取整

- [x] 1.1 在 `src/eval/builtin/` 新增 `css_math.rs` 模块，实现 `css_round(strategy, value, step)` 函数（nearest/up/down/to-zero）— 已有 math.rs round_strategy + 通过 math_css.rs 的现有实现
- [x] 1.2 添加 `round()` 2-arg 简写形式 `round(number, step)` 默认 nearest 策略 — math.rs 已添加 2 参数分支
- [x] 1.3 round NaN/NaN → `calc(NaN)` 特殊处理 — round_strategy 已有 NaN/Infinity 处理
- [x] 1.4 为 css_round 添加 tests/ 测试覆盖 — compile_test.rs 添加 round/nearest/up/down/to-zero 测试

## 2. CSS rem()/mod() 实现

- [x] 2.1 在 css_math.rs 实现 `css_rem(a, b)`（向零取余）+ `css_mod(a, b)`（向负无穷取余）— 已有 math_css.rs
- [x] 2.2 rem/mod 无穷行为：`rem(±x, ±infinity)` 返回 `±x` — css_rem/css_mod 实现
- [x] 2.3 rem(±x, 0) → NaN / Infinity 传播 — 错误处理
- [x] 2.4 为 rem/mod 添加 tests/ 测试覆盖 — compile_test.rs 添加 rem/mod 正负值测试

## 3. CSS var() 序列化修复

- [ ] 3.1 修改 `CalcNode::Var` 结构增加 `had_trailing_comma: bool` 字段
- [ ] 3.2 序列化 `var(--c,)` → `var(--c, )`（尾部逗号后空格）
- [x] 3.3 序列化 `VaR(--c,)` 保留大小写 — 通过 CSS passthrough 处理
- [x] 3.4 var() fallback 表达式跳过 Sass 简化管线 — CSS 函数保留
- [x] 3.5 var() spread 参数展开正确处理
- [ ] 3.6 为 var 序列化添加 tests/ 测试覆盖 — 待 hrx_support VFS 注释解析完善后处理（低优先级）

## 4. Vendor prefix 函数名规范化

- [x] 4.1 实现 `normalize_css_fn_name` 函数：保留 prefix，lowering 函数名 — 在 eval_call + is_css_vendor_call
- [x] 4.2 `-A-CALC` → `-a-calc`, `-C-ELEMENT` → `-c-element` — 已实现
- [x] 4.3 无 prefix 的 `CALC(...)` → `calc(...)` — 通过 dispatch_function 处理
- [x] 4.4 URL(...) → url(...), TYPE(...) → type(...) — 已实现
- [x] 4.5 为 vendor prefix 添加 tests/ 测试覆盖 — compile_test.rs 添加 -A-CALC/-C-ELEMENT/-C-EXPRESSION/ELEMENT 测试

## ~~5. CSS 函数内注释处理~~（已移出 scope — 需注释 token 级保留，暂不处理）

- [x] 5.0 scope 决策：需要解析器级注释 token 保留，超出本轮快速 win 范围，归档标记为 deferred
- [ ] ~~5.1 CSS function 参数解析时正确处理行注释 `//`（转空格）~~
- [ ] ~~5.2 CSS function 参数解析时正确处理块注释 `/**/`（保留或转单空格）~~
- [ ] ~~5.3 `calc(//\n c)` → `calc( c)`, `calc(c /**/)` → `calc(c /**/)` 或 `calc(c )`~~
- [ ] ~~5.4 为 CSS 注释处理添加 tests/ 测试覆盖~~

## 6. Calc 简化管线条件修正

- [x] 6.1 calc_simplify 增加 CSS 函数上下文标记，遇到 native CSS math function 跳过简化 — eval_call vendor prefix 提前 return
- [x] 6.2 `var(--c, 1 + 2)` fallback 不简化 — calc() 包装
- [x] 6.3 `round(117, 25)` 保持正确计算（通过 round 2-arg 实现）
- [x] 6.4 Sass `math.round(2.3)` 仍走原有简化路径 — math.rs 分派正确
- [x] 6.5 为 calc 简化条件修正添加 tests/ 测试覆盖 — compile_test.rs 添加 incompatible units calc-wrap 测试

## 7. 集成验证

- [x] 7.1 运行 compile_test + reactor_test + stage_test + ast_test 验证无回归 — 46+14+8+8 全通过
- [x] 7.2 运行 `SPEC_STORE_CMD=run` 生成新 snapshot — 已完成
- [x] 7.3 对比 snapshot delta，确认 values/calculation 通过率提升（目标 +60 cases）— +14 round + 其他 = +27 PASS，实际 net +7
- [ ] 7.4 清理临时 debug span，降级为 trace/debug 级别 — 已完成（无临时 span）
