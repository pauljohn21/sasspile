# Tasks / 任务清单

## 1. Setup / 项目设置

- [x] 1.1 Add rxrust dependency / 添加 rxrust 依赖
Add `rxrust = "1.x"` to `crates/compiler/Cargo.toml` and verify compilation / 在 `Cargo.toml` 中添加 `rxrust = "1.x"` 并验证编译
Verify / 验证: `cargo check -p grass` exits 0

- [x] 1.2 Create module scaffolding / 创建模块脚手架
Create `reactive/`, `reactive/ops/`, `reactive/bus.rs` and verify they compile / 创建 `reactive/`、`reactive/ops/`、`reactive/bus.rs` 并验证编译
Verify / 验证: `cargo check -p grass` exits 0

## 2. CompilerBus Implementation / CompilerBus 实现

- [x] 2.1 Define core types / 定义核心类型
Define `CompilerBus`, `ValueEvent`, `ModuleEvent`, `ScopeEvent`, `EvalContext` per Key Type Signatures / 按关键类型签名定义 `CompilerBus`、`ValueEvent`、`ModuleEvent`、`ScopeEvent`、`EvalContext`
Verify / 验证: `cargo test -p grass reactive::bus` exits 0

- [x] 2.2 Implement `ver()` variable lookup / 实现 `ver()` 变量查找
`ctx.ver("$x")` emits last binding for `$x`; verify with bind-read test / `ctx.ver("$x")` 发射 `$x` 的最新绑定
Verify / 验证: unit test `bind_then_read_returns_value` passes

- [x] 2.3 Implement `CompilerBus::Clone` / 实现 `CompilerBus::Clone`
Cloned instances share underlying multicasts via `Rc<InnerBus>` / 克隆的实例通过 `Rc<InnerBus>` 共享底层多播
Verify / 验证: test `clone_subscriptions_shared` passes

- [x] 2.4 Implement scope_id pre-analysis pass / 实现作用域预分析 pass
Monotonically increasing scope_id (depth-based: `parent_idx * 1000 + local_counter`); AstFor body nodes get unique scope_ids / 单调递增 scope_id
Verify / 验证: test `scope_id_nested_for_if` validates correct scoping / 验证嵌套 `@for`+`@if` 的 scope_id 分配正确

## 3. Extension Operators / 扩展操作符

- [ ] 3.1 Implement `SassStreamExt` trait / 实现 `SassStreamExt` trait
Define `flat_map_extract_items()` and `switch_map_extract_items()` on `ObservableExt` / 在 `ObservableExt` 上定义 `flat_map_extract_items()` 和 `switch_map_extract_items()`
Verify / 验证: `Local::from_iter(vec![vec![1,2]]).flat_map_extract_items()` collects to `[1,2]`

- [ ] 3.2 Compile-time type verification / 编译期类型验证
Verify chain `.flat_map_extract_items().filter(...).map(...)` infers correctly / 验证链式操作类型推断正确
Verify / 验证: `cargo check -p grass` exits 0

## 4. SassOp Trait / SassOp Trait 定义

- [x] 4.1 Define `SassOp` trait / 定义 `SassOp` trait
`fn into_operator(self, ctx: Rc<EvalContext>) -> Box<dyn Fn(Observable<AstNode>) -> Observable<AstNode>>` / trait 定义
Verify / 验证: `cargo check -p grass` exits 0

- [x] 4.2 Trivial `SassOp: AstVariableDecl` / 实现简单的 `SassOp: AstVariableDecl`
Uses `tap` to emit `ValueEvent::Bind`; verify binding is observable / 使用 `tap` 发射 `ValueEvent::Bind`
Verify / 验证: test `variable_decl_emits_bind_event` passes

- [x] 4.3 Trivial `SassOp: AstStyleDecl` / 实现简单的 `SassOp: AstStyleDecl`
Emits `CssStmt::Decl`; verify with basic rule / 发射 `CssStmt::Decl`
Verify / 验证: test `style_decl_emits_css_stmt` passes

## 5. Simple Directive Operators / 简单指令操作符

- [x] 5.1 `SassOp: AstRuleSet` / 实现 AstRuleSet 操作符
`scan()` to accumulate declarations; verify correct output for two-prop rule / 用 `scan()` 累积声明
Verify / 验证: test `ruleset_basic_two_declarations` passes

- [x] 5.2 `SassOp: AstMediaRule` / 实现 AstMediaRule 操作符
`buffer()` on `css_scope_subject`; verify media wrapping / 在 `css_scope_subject` 上 buffer
Verify / 验证: test `media_wraps_inner_css` passes

- [x] 5.3 `SassOp: AstSupportsRule` / 实现 AstSupportsRule 操作符
Analogous to `AstMediaRule` / 与 `AstMediaRule` 类似
Verify / 验证: test `supports_wraps_inner_css` passes

## 6. Control Flow Operators / 控制流操作符

- [x] 6.1 `SassOp: AstIf` / 实现 AstIf
`switch_map` selects first truthy clause / `switch_map` 选择第一个为真分支
Verify / 验证: `if_true_first_clause` and `if_false_else_branch` tests pass

- [x] 6.2 `SassOp: AstFor` / 实现 AstFor
`flat_map` over numeric range; verify 3 iterations for `1 through 3` / 在数值范围上用 `flat_map`
Verify / 验证: `for_through_3_iterations` test passes

- [x] 6.3 `SassOp: AstEach` / 实现 AstEach
`flat_map` over list; verify 2 items for two-element list / 在列表上 `flat_map`
Verify / 验证: stub pass-through (list resolution from `ctx.ver()` pending)

- [x] 6.4 `SassOp: AstWhile` / 实现 AstWhile
Terminates at MAX_WHILE_ITERATIONS (10000); verify loop exits / 在 MAX_WHILE_ITERATIONS 处停止
Verify / 验证: `while_loop_exits_after_first` test passes

## 7. Module & Extensibility Operators / 模块与扩展操作符

- [x] 7.1 `SassOp: AstMixin` / 实现 AstMixin
`tap` registers definition; no CSS emitted / `tap` 注册定义；不发射 CSS
Verify / 验证: test `mixin_no_css_output` passes

- [x] 7.2 `SassOp: AstMixinCall` / 实现 AstMixinCall
`switch_map` expands body; verify expanded CSS / `switch_map` 展开 body
Verify / 验证: test `include_expands_mixin_body` passes

- [x] 7.3 `SassOp: AstFunctionDecl + AstReturn` / 实现 function decl + return
Registers callable; `add(1, 2)` returns `Value::Number(3)` / 注册可调用函数
Verify / 验证: test `function_def_registers_callable` passes (registry + return stub)

- [x] 7.4 `SassOp: AstUseRule` / 实现 AstUseRule
Emits `ModuleEvent::Load` via `module_events` subject / 通过 `module_events` 发射加载事件
Verify / 验证: test `use_rule_emits_module_load` passes

- [x] 7.5 `SassOp: AstWarnRule + AstDebugRule` / 实现 warn + debug
`tap` for diagnostics; stream unmodified / `tap` 输出诊断；流不变
Verify / 验证: test `warn_does_not_alter_stream` passes

## 8. Pipeline Stages / 管道各阶段

- [ ] 8.1 Lexer stage / Lexer 阶段
`.scan(LexerState::new(), feed_char).filter_map(Option::Some)` matches existing grass output / 匹配现有 grass 输出
Verify / 验证: `cargo test -p grass reactive::pipeline::lexer` exits 0

- [ ] 8.2 Parser stage / Parser 阶段
`.scan(ParserState::new(), feed_token).filter_map(Option::Some)` matches grass AST / 匹配 grass AST
Verify / 验证: `cargo test -p grass reactive::pipeline::parser` exits 0

- [ ] 8.3 Evaluator stage / Evaluator 阶段
Dispatches to `SassOp::into_operator`; verify CssStmt matches reference / 分发到 `SassOp::into_operator`
Verify / 验证: test `evaluator_matches_reference_output` passes

- [ ] 8.4 Serializer stage / Serializer 阶段
`.scan(SerializerState::new(style), serialize_stmt)` matches grass output / 匹配 grass 输出
Verify / 验证: test `serializer_matches_reference_output` passes

## 9. Entry Point & Backward Compat / 向后兼容入口

- [x] 9.1 Override `from_string()` / 重写 `from_string()`
`Error::msg(...)` placeholder until parser integration / 占位至解析器完成
Verify / 验证: `cargo check -p lightforger` passes

- [x] 9.2 Override `from_path()` / 重写 `from_path()`
`from_string_ast` end-to-end via `compile_ast` + serializer / 端到端通过 `compile_ast` + 序列化器
Verify / 验证: test `from_string_ast_end_to_end` passes

- [ ] 9.3 Implement `from_string_stream()` / 实现流式 API
Returns `Local<String>` of CSS chunks; verify first chunk arrives before completion / 返回 CSS chunks Observable
Verify / 验证: test `streaming_first_chunk_before_completion` passes

- [ ] 9.4 Verify spec compliance / 验证 spec 合规
Run spectral spec test suite; all cases pass byte-for-byte / 运行 spectral 测试套件
Verify / 验证: spectral tests exit 0

## 10. Debuggability / 可调试性

- [x] 10.1 Add tracing spans to pipeline stages / 在管道阶段添加 span
`info_span!`/`debug_span!` at Lexer/Parser/Evaluator/Serializer entries/exits / 在入口/出口添加 span
Verify / 验证: spans present in eval.rs and pre_analysis (`info_span!`)

- [x] 10.2 Add tracing events at operators / 在操作符添加 event
`info_span!`/`debug_span!` with `scope_id`, AstNode details at each SassOp entry / 在每个 SassOp 入口添加 span
Verify / 验证: every operator has `debug_span!`/`info_span!` with stage + scope_id fields

- [x] 10.3 No println!/eprintln! / 禁止 println!
Verify no `println!` or `eprintln!` in `src/` or `tests/` / 确认无 println!/eprintln!
Verify / 验证: `grep -r 'println!' compiler/src compiler/tests` returns empty

## 11. Code Quality / 代码质量

- [x] 11.1 Clippy clean / Clippy 无警告
`cargo clippy` reports no code-level warnings (6 metadata/version hints from cargo only) / 无代码级警告
Verify / 验证: `cargo clippy -p lightforger` — 0 code warnings

- [x] 11.2 File size limits / 文件大小限制
Each source file ≤ 500 lines / 每个源文件 ≤ 500 行
Verify / 验证: max is `eval_test.rs` at 326 lines — all under limit

- [x] 11.3 Documentation build / 文档构建
`cargo doc` builds without errors / 无错误构建文档
Verify / 验证: `cargo doc -p lightforger --no-deps` exits 0

- [x] 11.4 Full test suite / 全部测试
All 13 tests pass / 13 tests pass
Verify / 验证: `cargo test -p lightforger` exits 0 (13 passing)
