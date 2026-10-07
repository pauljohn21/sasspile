# Tasks

## Phase 1: Lexer 内容补全

- [ ] 1.1 扩展 lexer/state.rs — 增加字符串转义处理（`\"` `\\` `\n`）、silent 注释 `//`（抑制）、preserved 注释 `/*! */`（输出 Token）、数字单位（`16px` → `Number(16.0, Some("px"))`）、双字符操作符 `==` `!=` `<=` `>=`
- [ ] 1.2 更新 AstNode::VariableDecl — 增加 `flags: VarFlags { is_default, is_global }`，替换原 `scope_id: u64` 字段
- [ ] 1.3 更新 Token — 增加 `AtImport` `AtAtRoot` `AtContent` `AtError` `Atkeyframes` `AtFontFace` `AtPage` `AtCharset` 变体
- [ ] 1.4 验证: cargo test --test lexer_test 通过

## Phase 2: Parser 内容补全

- [ ] 2.1 增加 Pratt 表达式解析器 — 优先级: or<and<cmp<add/sub<mul/div<unary<primary，替换现有 parse_value() 为表达式入口
- [ ] 2.2 扩展 @rule 解析 — `!default` `!global` 标志、`@use` `@forward` `@import`（仅注册无 CSS）、`@at-root`、`@content`、`@error`、`@keyframes`
- [ ] 2.3 解析器选择器插值 — `#{}` 在 selector 中正确分割 token 产出 Interpolation 节点
- [ ] 2.4 验证: cargo test --test parser_test 通过

## Phase 3: Evaluator 内容补全

- [x] 3.1 增加 call_builtin() 分派 — `fn(name: &str, args: &[Value]) -> Option<Value>`，覆盖 Bootstrap 所需全部内建函数
- [x] 3.2 实现 eval/builtin.rs — math、string、list、map、color、meta 内建函数
- [x] 3.3 修正 function @return — FunctionCall eval 执行 body 中 Return 节点
- [x] 3.4 增加选择器组合 — `&` 引用替换 parent selector + 无 `&` 时 descendant 组合
- [x] 3.5 增加 @at-root 求值 — hoist 子规则到根级（作为特殊 CssStmt 标记）
- [ ] 3.6 增加递归深度保护 — 防止 Bootstrap mixin 嵌套导致栈 overflow

## Phase 4: Serializer 内容补全

- [ ] 4.1 增加 Nested 输出风格 — 与 Expanded 类似但有特定缩进规则
- [ ] 4.2 增加 @media 相邻同 query 合并 — 两相邻 `@media (min-width: 768px)` 合并为一个
- [x] 4.3 增加 @at-root hoist — AtRoot 节点提升到文档根级
- [ ] 4.4 增加重要注释保留 — `/*! */` 在 Compressed 模式下保留

## Phase 5: Bootstrap 验证

- [x] 5.1 编译 `bootstrap/scss/bootstrap.scss` — Expanded 风格，产出 ~60KB CSS
- [ ] 5.2 迭代修复 — 输出质量提升（变量解析、mixin 展开完善）
- [ ] 5.3 验证产物包含 .btn/.container/.modal/.navbar 选择器

## Phase 6: 集成

- [x] 6.1 端到端集成测试 — tests/integration_test.rs 覆盖核心场景
- [x] 6.2 全部测试通过 — cargo test 182 tests 无失败
- [ ] 6.3 clippy 清理 — 预先存在的 warning 待处理
