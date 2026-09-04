# Implementation Tasks

## Phase 1: 选择器 AST 类型 + 解析器

- [x] 1.1 创建 `src/css/selector_ast.rs` — 定义 `Selector`/`ComplexSelector`/`CompoundSelector`/`SimpleSelector`/`Combinator` 类型（derive Debug, Clone, PartialEq）
- [x] 1.2 实现 `parse_selector(input: &str) -> Selector` — 选择器字符串解析为 AST
  - 逗号分隔列表、组合器（空格/>/+/~）、类型/通配符/类/ID/属性/伪类/伪元素/占位符/命名空间
  - 降级策略：无法解析时返回包装原始字符串的降级 AST
- [x] 1.3 实现 `std::fmt::Display for Selector` — AST 序列化为规范 CSS 字符串
- [x] 1.4 添加 `#[instrument]` tracing span（parse_selector、level = debug）
- [x] 1.5 注册模块到 `src/css/mod.rs`

## Phase 2: 选择器算法 — unify

- [x] 2.1 创建 `src/css/selector_ops.rs`
- [x] 2.2 实现 `unify_compound(a, &b) -> Option<CompoundSelector>`
  - Type 冲突 → None；Id 冲突 → None；PseudoElement 冲突 → None
  - Universal + Type → Type；Class/PseudoClass/Attribute → 并集去重
  - 结果排序：Type → Universal → Id → Class → Attribute → PseudoClass → PseudoElement
- [x] 2.3 实现 `unify_complex(a, &b) -> Option<ComplexSelector>` — 从右端匹配复合选择器
- [x] 2.4 实现 `unify(a, &b) -> Option<Selector>` — 笛卡尔积 + 过滤 None
- [x] 2.5 添加 tracing span

## Phase 3: 选择器算法 — is_superselector

- [x] 3.1 实现 `is_super_compound(super_c, &sub_c) -> bool` — 简单选择器子集 + PseudoElement 例外
- [x] 3.2 实现 `is_super_complex(super_c, &sub_c) -> bool` — 组合器子序列匹配（LCS）
- [x] 3.3 实现 `is_superselector(super_sel, &sub_sel) -> bool` — 顶层入口
- [x] 3.4 添加 tracing span

## Phase 4: 选择器算法 — extend/replace

- [x] 4.1 实现 `extend_selector(selector, &extendee, &extender) -> Selector`
- [x] 4.2 实现 `replace_selector(selector, &original, &replacement) -> Selector`
- [x] 4.3 添加 tracing span

## Phase 5: 选择器内建函数重写

- [x] 5.1 重写 `selector-unify` — `parse_selector` + `unify` + `Display`
- [x] 5.2 重写 `selector-extend` — `parse_selector` + `extend_selector` + `Display`
- [x] 5.3 重写 `selector-replace` — `parse_selector` + `replace_selector` + `Display`
- [x] 5.4 重写 `is-superselector` — `parse_selector` + `is_superselector`
- [x] 5.5 验证 `selector-append`/`selector-nest`/`selector-parse`/`selector-simple-selectors` 无回归

## Phase 6: @extend 指令重写

- [x] 6.1 重写 `apply_extends` — 使用 `extend_selector` 替代字符串 `replace`
- [x] 6.2 保持模块 scope 检查逻辑（`module_selectors`）不变
- [x] 6.3 保持占位符移除逻辑
- [x] 6.4 添加 tracing span（level = info）

## Phase 7: 选择器测试

- [x] 7.1 创建 `tests/selector_ast_test.rs` — AST 解析 + 序列化 round-trip
- [x] 7.2 创建 `tests/selector_unify_test.rs` — unify 算法（对照 sass-spec 预期值）
- [x] 7.3 创建 `tests/selector_super_test.rs` — is_superselector 算法
- [x] 7.4 创建 `tests/selector_extend_test.rs` — extend/replace 算法
- [x] 7.5 运行核心测试确认无回归（202/202）
- [x] 7.6 运行 sass-spec `core_functions/selector/` + `directives/extend/` 验证提升

## Phase 8: calc 表达式 AST 类型 + 解析器

- [x] 8.1 创建 `src/eval/value/calc_units.rs` — 单位兼容性表 + `units_compatible` + `convert_unit`
- [x] 8.2 创建 `src/eval/value/calc_ast.rs` — 定义 `CalcNode`/`CalcOp`/`CalcConst`/`CalcError` 类型
- [x] 8.3 实现 `parse_calc_expr(input: &str) -> Option<CalcNode>` — 运算符优先级 + 括号 + CSS 函数 + var()
- [x] 8.4 实现 `std::fmt::Display for CalcNode` — 序列化为 CSS calc 表达式
- [x] 8.5 添加 tracing span
- [x] 8.6 注册模块到 `src/eval/value/mod.rs`

## Phase 9: calc 简化算法

- [x] 9.1 实现 `simplify_calc_node(node: &CalcNode) -> Result<CalcNode, CalcError>`
  - 递归简化子表达式
  - 常量折叠（同单位加减法）
  - 兼容单位转换（deg+rad 等）
  - 不兼容单位错误检测
  - 乘除法规则
  - 常量替换（pi/e → 数字）
  - var()/Func 保留
- [x] 9.2 实现 CSS 数学函数简化（min/max/clamp/round/rem/mod/abs/sign/exp/pow/sqrt/log/sin/cos/tan 等）
- [x] 9.3 添加 tracing span

## Phase 10: calc 入口重写

- [x] 10.1 重写 `simplify_calc(s: &str) -> Value` — 调用新 AST
  - parse → simplify → 纯数字返回 Number / 含 var/func 返回 Calc / 错误传播
  - 降级策略：parse 失败时回退到现有字符串处理
- [x] 10.2 处理不兼容单位错误传播（CalcError → SassError）
- [x] 10.3 验证现有 calc 测试无回归

## Phase 11: calc 测试

- [x] 11.1 创建 `tests/calc_ast_test.rs` — AST 解析 + 序列化 round-trip
- [x] 11.2 创建 `tests/calc_units_test.rs` — 单位兼容性 + 转换
- [x] 11.3 创建 `tests/calc_simplify_test.rs` — 简化规则（对照 sass-spec 预期值）
- [x] 11.4 运行核心测试确认无回归
- [x] 11.5 运行 sass-spec `values/calculation/` 全量验证提升

## Phase 12: 最终验证

- [x] 12.1 全量核心测试：`cargo test --test compile_test --test stage_test --test ast_test --test common_test --test interp_test --test bs_spec --test ep_full --test default_config_test`
- [x] 12.2 sass-spec 全量统计：`SHOW_FAILS=1 RUST_LOG="sass_spec_full=info,sasspile=warn" cargo test --test sass_spec_full -- --nocapture`
- [x] 12.3 更新 sass-spec-failures.md 统计报告
- [x] 12.4 更新 AGENTS.md 基线数字
- [x] 12.5 `codegraph sync`
