# Design: Full Reactive Pipeline

## Context

当前管线结构（断裂点标记为 ⛔）：

```
Source(String)
    │ scan() → Shared::create → .box_it()
    ▼ ⛔ 此处 collect_boxed 为断裂点
Token[] → parse_all_nodes() → Vec<AstNode> → Shared::from_iter → .box_it()
    │
    ▼ ⛔ 此处 collect_boxed 为断裂点
AstStream → collect_boxed() → Vec<AstNode> → Shared::from_iter → flat_map → scan_map → filter_map → .box_it()
    │
    ▼
CssStream → collect_stream() → Vec<String> → join()
    │
    ▼
String (CSS)
```

问题：Parser 入口和 Evaluator 入口各有一个 `collect_boxed`，它们在中间阶段完整收集上游 Observable 输出为 `Vec`，阻止了数据流式传播。

详见 proposal.md。

## Goals / Non-Goals

**Goals:**
- 消除所有中间 `collect_boxed`，管道为单一 Observable 管线
- `box_it()` 仅在 Lexer 返回边界 + Parser 尾端/eval 尾端（Serialize 输出端）各调用一次
- 错误通道从 `Infallible` 统一为 `CompileError`
- `from_string` / `from_path` 签名保持不变

**Non-Goals:**
- 不改变 SCSS 编译语义行为
- 不新增功能
- 不改变 Parser 语法规则算法（仅改变增量驱动方式）

## Decisions

### Decision 1: Parser 采用 Shared::create 而非 scan_map

**选择**: `parse_stream_with_paths` 内部用 `Shared::create(move |subscriber| { ... })` 手动订阅 `token_stream`，逐 token 增量喂入 `parser_feed` 并产出 AstNode，流结束时检查未闭合分隔符。

**理由**:
- `scan_map` 无法在检测到错误时调用 `subscriber.error()`
- 未闭合分隔符检测必须在所有 token 到齐后（Eof）才能进行
- `Shared::create` 是 rxrust 推荐的自定义源创建方式（参见 SKILL.md）
- 上游 unsubscribe 通过 Subscription teardown 正确传播

**代码结构**:
```rust
pub fn parse_stream_with_paths(
    token_stream: TokenStream,
    scope_id: u64,
    include_paths: Vec<PathBuf>,
) -> AstStream {
    Shared::create(move |subscriber| {
        let mut state = ParserState::with_include_paths(include_paths);
        state.scope_id_counter = scope_id;
        token_stream.subscribe_all(
            move |tok| {
                for node in parser_feed(&mut state, tok) {
                    subscriber.next(node);
                }
            },
            move |err| { subscriber.error(err); },
            move || {
                if let Err(e) = check_unclosed_delimiters(&state) {
                    subscriber.error(e);
                } else {
                    subscriber.complete();
                }
            },
        );
        ()  // empty Subscription
    }).box_it()
}
```

**备选 A — scan_map**: `scan_map(ParserState, parser_feed)` + `flat_map(Shared::from_iter)` — 无法处理错误通道。
**备选 B — flat_map + Vec 收集**: 断裂点回归，否决。

### Decision 2: Evaluator 入口 use expand eliminate collect_boxed

**选择**: `eval_stream` 将 `flat_map(emit_events)` 改为 `expand(emit_events)`，消除 `collect_boxed(ast_stream)` 调用，直接链式调用。

**理由**:
- `expand` 是深度优先递归展开（rxrust 标准算子）
- `flat_map` 是 `map + merge_all`（广度优先），对嵌套规则语义不正确
- 消除 eval_stream 入口的 collect_boxed 断裂点

**代码结构**:
```rust
pub fn eval_stream(ast_stream: AstStream, ctx: Arc<EvalContext>) -> CssStream {
    let bus = ctx.bus().clone();
    ast_stream
        .expand(move |node| emit_events(node, None, ctx.clone(), bus.clone()))
        .scan_map(EvalState::root(), fold_frame)
        .filter_map(|opt| opt)
        .box_it()
}
```

### Decision 3: serialize_stream use fold accumulate render at completion

**选择**: 定义 `SerializeState { stmts: Vec<CssStmt>, options: Options }`，管线末端使用 `.fold(SerializeState::new(opts), |mut s, stmt| { s.push(stmt); s }).map(|s| s.render())`。

**理由**:
- `fold` 是有状态累积的标准算子
- `.map(|state| state.render())` 在流结束时产生单个 String
- 原 `serialize()` 纯函数结构保持不变（被 `SerializeState.render()` 调用）

### Decision 4: Type alias CompileError unified

**选择**: 修改 `types.rs` 四个别名的错误类型为 `CompileError`。删除 `Infallible` import。

**理由**:
- `Infallible` 强制所有错误必须在返回值 `Result` 中处理
- `CompileError` 通过 `on_error` 通道传播，下游自动停止
- 标准 rxrust 错误模型

### Decision 5: collect_boxed signature change

**选择**: `collect_boxed<T>(SharedBoxedObservable<'static, T, CompileError>) -> Result<Vec<T>, CompileError>`。订阅时使用双参数 `subscribe(on_next, on_error)` 捕获 `CompileError`。

**理由**: 终端收集器需要向上传播错误，否则 `from_string` 无法返回 `Result::Err`。

## Risks / Trade-offs

### Risk 1: Parser Shared::create 的生命周期复杂度
**风险**: `Shared::create` 闭包中手动管理 `ParserState` 和上游订阅，若 `token_stream` 的 `subscribe_all` 未正确实现 complete 转发，可能导致死锁。
**缓解**: 严格参考 SKILL.md 中 `Shared::create` 的标准写法，`on_complete` 必须调用 `subscriber.complete()` 或 `subscriber.error()`。

### Risk 2: expand 的栈深度
**风险**: `expand(emit_events)` 对深度嵌套规则（如 20 层嵌套）可能栈溢出。
**缓解**: rxrust 的 `expand` 内部使用 Subscription 链而非原生递归，由框架处理背压。SCSS 嵌套通常不超过 10 层，风险可接受。

### Risk 3: 测试兼容性
**风险**: 测试文件可能直接调用 `eval_ast_stream_sync`、`eval_nodes_sync` 等将被删除的函数。
**缓解**: 实现阶段须 grep 所有测试文件中的函数调用，确保无遗漏。如有测试调用被删除的同步函数，要么：(a) 修改测试使用新的响应式管线，或 (b) 保留同步兼容包装函数（但不在 src/ 中，仅在 tests/ 中提供测试辅助函数）。

### Risk 4: Infallible 残留
**风险**: 某些模块（如 `builtin.rs`、`expr.rs`、`prefixer.rs`）可能隐性依赖 `Infallible` 类型推断。
**缓解**: 全局 `grep -r "Infallible" src/` 并逐一确认。

## Migration Plan

实现顺序（每步均可独立编译通过）：

1. **Phase A — 类型系统**: 修改 `types.rs`（别名 + 删除 Infallible import）→ `cargo check`
2. **Phase B — collect_boxed**: 修改 `observable_ext.rs` → `cargo check`
3. **Phase C — Lexer**: 修改 `lexer/mod.rs` 错误通道 → `cargo check`
4. **Phase D — Parser**: 重构 `parser/mod.rs`（parser_feed + Shared::create）→ `cargo check`
5. **Phase E — Eval**: 重构 `eval/mod.rs` + `eval/emit.rs`（去除 collect_boxed + expand）→ `cargo check`
6. **Phase F — Serialize**: 新增 `serialize_stream` → `cargo check`
7. **Phase G — Pipeline**: 重写 `pipeline.rs` → `cargo check`
8. **Phase H — lib.rs**: 清理 exports → `cargo check`
9. **Phase I — Infallible cleanup**: 全局清除 Infallible → `cargo build`
10. **Phase J — Tests**: 修复测试 → `cargo test`

每 Phase 后运行 `cargo check` 确认编译。Phase J 后运行全部测试。

## Open Questions

### Q1: ParserState 是否需新增 check_error 字段？
未闭合分隔符检测在 Parser 内部通过 `check_unclosed_delimiters()` 函数完成（在 Shared::create 的 on_complete 中调用），不需要将错误存储在 ParserState 中。错误直接通过 `subscriber.error()` 发送。

### Q2: Shared::create 闭包是否比 scan_map 性能差？
两者性能特征相似。`Shared::create` 是单次闭包调用，内部逐 token 处理；`scan_map` 在每次 token 到达时调用 FnMut。关键区别：错误处理能力。`Shared::create` 有 subscriber 引用可直接调用 `.error()`，`scan_map` 无。

### Q3: 测试文件中使用 eval_ast_stream_sync 的测试如何处理？
评估所有使用了删除同步函数的测试：
- `eval_ast_stream_sync` 仅在 `pipeline.rs` 和 `eval/mod.rs` 的测试中调用
- 这些调用的消费者应改为使用 `compile_pipeline()` + `collect_boxed`
- 如测试需要单独测试 eval 部分，可直接调用 `eval_stream` + `collect_boxed`

结论：无需保留任何同步包装函数在 src/ 中。
