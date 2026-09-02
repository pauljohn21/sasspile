## Context

sasspile 架构审核发现三类函数式违规：
1. 两个源文件超过 500 行限制（`eval/builtin.rs` 524 行、`parse/expr/prefix.rs` 503 行）
2. `combine_selectors()` 使用嵌套 `for+push` 而非迭代器链
3. `call_user_function()` 和 `bind_params()` 中的 `env.clone()` 可优化为 move 语义

当前测试基线：compile_test 43 + stage_test 10 + ast_test 8 + common_test 5 + bs_spec 15 + ep_full 121 = 202/202。

## Goals / Non-Goals

**Goals:**
- 所有 `src/` 文件 ≤ 500 行
- `combine_selectors()` 使用迭代器链而非 `for+push`
- `call_user_function()` 和 `bind_params()` 消除 `env.clone()`
- 202/202 测试全通过，零回归

**Non-Goals:**
- 不修改任何业务逻辑（仅重构）
- 不修改 `css/mod.rs` 的 `&mut String` 缓冲区写入（性能优化例外）
- 不修改 `eval_while` 的命令式 `loop`（while 语义合理例外）
- 不修改 `apply_extends` 内部 `for`（extend 可变选择器累积的合理例外）

## Decisions

### D1: builtin.rs 拆分策略 — 提取手工分派函数

**决策**：将 `call_builtin` 中的手工 match 分支（rgba/rgb/darken/lighten/mix/if/inspect/type-of 等）提取到新文件 `src/eval/builtin/manual_dispatch.rs`。

**理由**：`dispatch_builtin_module` 已通过派生宏处理大部分模块分派，手工 match 仅保留特殊函数。将这些函数移到独立文件，`builtin.rs` 仅保留入口和模块声明。

**替代方案**：拆分为更多子模块（如 `rgba.rs`、`meta_manual.rs`），但手工分派函数总量不大（约 200 行），一个文件足够。

### D2: prefix.rs 拆分策略 — 提取字面量解析

**决策**：将 `parse_prefix` 中的字面量解析分支（Number/String/Hash/Color/Dollar/Ident 等）提取到 `src/parse/expr/literals.rs`，`prefix.rs` 保留控制流（Minus/Not/Percent/LParen/Call）和二元运算解析。

**理由**：字面量解析是独立的语法分支，与运算符解析无耦合。拆分后两个文件各约 250 行。

**替代方案**：按 token 类型拆分（number.rs、string.rs），但每个分支仅 10-20 行，过度拆分。

### D3: combine_selectors 重构 — 迭代器笛卡尔积

**决策**：将嵌套 `for p in parents / for c in children` 改为 `parents.iter().flat_map(|p| children.iter().map(move |c| ...)).collect()`。

**替代方案**：引入 `itertools::iproduct`，但增加外部依赖。手写 `flat_map + map` 零依赖。

### D4: call_user_function 重构 — move + exit_scope

**决策**：`call_user_function` 的签名从 `&Env` 改为 `Env`（move），内部用 `env` 直接操作（bind 参数），求值后通过 `exit_scope` 恢复外层作用域（与 `eval_rule` 一致），避免 `env.clone()`。

**理由**：`eval_rule` 已成功使用此模式。函数体作用域与规则体作用域语义一致——局部变量不传播，但命名空间变量和 !global 变量需要传播。

### D5: bind_params 重构 — 返回 (Env, ...)

**决策**：`bind_params` 签名从 `&Env -> Result<Env>` 改为 `Env -> Result<Env>`，内部直接用 `self.bind()` 链式调用，消除 `env.clone()`。

**理由**：`bind_params` 的调用者（`exec_mixin`）已经 move 了 env，`bind_params` 只需继续链式 bind。

## Risks / Trade-offs

- [D4 风险] `call_user_function` 签名变更影响调用者 — Mitigation: 调用者仅 `call_function`（mixin.rs），同步修改
- [D5 风险] `bind_params` 签名变更影响 `call_module_function` — Mitigation: `call_module_function` 已有 `env.clone()`，改为传入 clone 后的 env 或调整调用模式
- [D3 风险] `combine_selectors` 迭代器重构可能改变选择器顺序 — Mitigation: `flat_map` 保持外层优先序，与嵌套 for 语义一致
- [整体风险] 拆分文件可能遗漏 `pub` 可见性 — Mitigation: 编译器检查 + 全量测试验证
