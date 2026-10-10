# Spec Delta

本文档是全链路响应式重构的**执行级规格说明**。实现者 SHALL 严格遵循本文档中的规定和禁止事项，任何偏离 SHALL 视为违反本 spec。

This document is an EXECUTION-LEVEL spec for the full reactive pipeline refactor. Implementers SHALL strictly follow the mandates and prohibitions below. ANY deviation is a spec violation.

## 反模式禁止清单 / Anti-Pattern Prohibitions

实现者在任何阶段均 SHALL NOT：

1. **在 pipe intermediate 位置调用 `collect_boxed` / `collect::<Vec<_>>`** — `collect` 仅能在最终消费端（`from_string`/`from_path` 的阻塞收集器）使用
2. **在 `flat_map`/`map`/`scan/filter` 之后调用 `.box_it()`** — `box_it()` 只能在每个 stage 最终返回边界调用一次
3. **使用 `for x in vec { result.push(f(x))` 命令式模式** — 不得用命令式循环处理 Observable 管道发射的数据
4. **使用 `Arc<Mutex<Vec>>` + `subscribe(push)` 做 GC 收集** — 不得绕过 rxrust 的 `collect` 终止算子
5. **保留 `Infallible` 类型参数** — 所有类型别名和函数签名中的错误类型 SHALL 为 `CompileError`
6. **在 `eval_stream` 内 `let nodes: Vec<_> = collect_boxed(ast_stream)`** — 这是断裂点，必须消除
7. **在 `parse_stream` 内 `let tokens: Vec<_> = collect_boxed(token_stream)`** — 这是断裂点，必须消除
8. **在 serialize 阶段 `let stmts: Vec<_> = collect_boxed(css_stream)` 然后再 `serialize(&stmts, ...)`** — 必须改为管线末端 `fold` 累积

---

## MODIFIED Requirements

### Requirement: Pipeline SHALL connect all compilation stages via Observable streams without intermediate collect / 管道在无中间 collect 的情况下连接所有编译阶段

系统 SHALL 将 Lexer → Parser → Evaluator → Serializer 连接成单一 Observable 管线。每个阶段消费上游的产出 Observable，并产出新的 Observable 给下一阶段消费。**管道中间 SHALL 不存在任何 `collect` 或 `collect_boxed` 调用**——所有数据通过 Observable 算子惰性流动，仅在最终消费端（`from_string`/`from_path`）一次性收集为最终 String。

The system SHALL connect Lexer → Parser → Evaluator → Serializer as a single Observable pipeline. **There SHALL NOT be any `collect` or `collect_boxed` calls between stages** — all data flows lazily through Observable operators, collected only at the final consumer.

#### Scenario: Zero intermediate collect in pipeline / 管道中间零 collect
- **WHEN / 当** 检查管道中间代码（Lexer↔Parser、Parser↔Evaluator、Evaluator↔Serializer 之间的连接）时
- **THEN / 那么** 不出现 `collect_boxed`、`collect::<Vec<_>>()`、或任何将 Observable 完整收集为 Vec 后再重新 from_iter 的模式

#### Scenario: Pipeline produces output incrementally / 管道渐进式输出
- **WHEN / 当** 编译一条包含 100 个顶层规则（top-level rules）的样式表时
- **THEN / 那么** 第一个 CSS 块在所有 100 个规则被解析完成之前就已经被发出（emit）

---

### Requirement: Parser SHALL use scan_map(ParserState) for incremental parsing / Parser SHALL 使用 scan_map 增量解析

系统 SHALL 提供 Parser，接受 `Observable<Token, CompileError>` 并产出 `Observable<AstNode, CompileError>`。Parser SHALL 使用 `scan_map(ParserState)` + `flat_map(Shared::from_iter)` 响应式管线：`scan_map` 增量接收单个 Token 并维护 ParserState（包含内部 token 缓冲区），每轮调用 `try_step()` 尝试解析完整语句；当语句完成时 `scan_map` 输出 `Vec<AstNode>`（可能为空），`flat_map` 将非空 Vec 展平为单个节点流。Parser SHALL 在流结束时检查未闭合分隔符。

The system SHALL use `scan_map(ParserState)` + `flat_map(Shared::from_iter)`: `scan_map` receives one Token incrementally while maintaining ParserState (with internal token buffer); each step calls `try_step()` to attempt parsing a complete statement; when a statement completes, `scan_map` outputs `Vec<AstNode>` (possibly empty); `flat_map` flattens non-empty Vecs into individual nodes.

#### Scenario: Parser scan_map emits per completed statement / scan_map 按完成语句 emit
- **WHEN / 当** token 序列代表 `"a { color: red; }"` 时
- **WHEN / 当** 最后一个 token（`}`）被推入 ParserState
- **THEN / 那么** `scan_map` 输出 `vec![AstNode::Rule { selector: "a", inner: [...] }]`
- **THEN / 那么** `flat_map` 将其展平为单个 `AstNode::Rule` 发射到下游

#### Scenario: Parser buffers tokens without emitting / Parser 缓冲 token 不 emit
- **WHEN / 当** 仅推入 `"a {"` token（语句尚未完成）
- **THEN / 那么** `scan_map` 输出空 `vec![]`，下游不发射任何值

#### Scenario: Parser detects unclosed delimiters at stream end / Parser 在流结束时检测未闭合分隔符
- **WHEN / 当** token 序列代表 `"a { color: red;"`（缺少 `}`）时
- **THEN / 那么** Parser 在流结束检查时发现未闭合 `{`，通过 Observable 错误通道发射 `CompileError::Parse`

---

### Requirement: Evaluator SHALL consume AstStream directly with expand / Evaluator 直接使用 expand 消费 AstStream

系统 SHALL 提供 Evaluator，接受 `Observable<AstNode, CompileError>` 并产出 `Observable<CssStmt, CompileError>`。Evaluator 入口 SHALL **直接消费上游 AstStream**，不通过 `collect_boxed` 中间收集。Evaluator 使用 `expand(emit_events)`（而非 `flat_map`）递归展开 AST 树为 `Observable<EvalEvent>`，随后通过 `scan_map(EvalState::root(), fold_frame)` 有状态折叠累积 frame 栈，最终 `filter_map` 提取已完成的 CssStmt。

The system SHALL consume `AstStream` directly without intermediate `collect_boxed`. The evaluator uses `expand(emit_events)` (NOT `flat_map`) for recursive AST tree expansion, then `scan_map(EvalState::root(), fold_frame)` for stateful frame accumulation, and finally `filter_map` to extract completed CssStmt.

#### Scenario: eval_stream has no intermediate collect / eval_stream 无中间 collect
- **WHEN / 当** 阅读 `eval_stream` 函数体
- **THEN / 那么** 不出现 `collect_boxed`、`collect::<Vec<_>>()`、或 `from_iter(vec)` 模式
- **THEN / 那么** 函数体为 `ast_stream.expand(...).scan_map(...).filter_map(...).box_it()`

#### Scenario: Evaluator uses expand for tree recursion / 使用 expand 做树递归
- **WHEN / 当** 输入为嵌套规则（3 层深度）时
- **THEN / 那么** `expand(emit_events)` 递归展开所有层级，scan_map 正确累积 frame 栈，最终产出正确的嵌套 CSS 规则

---

### Requirement: Serializer SHALL be a reactive stage using fold / Serializer  SHALL 使用 fold 的响应式阶段

系统 SHALL 提供 Serializer，接受 `Observable<CssStmt, CompileError>` 并产出 `Observable<String, CompileError>`。Serializer 使用 `fold(SerializeState)` 将所有 CssStmt 累积到 SerializeState 中，流结束时调用 `state.render(opts)` 产出最终 CSS 字符串块。SerializeState SHALL 处理输出样式（Expanded / Compressed / Nested）。

The system SHALL provide a Serializer using `fold(SerializeState)` to accumulate all CssStmt, then `state.render(opts)` at stream completion to produce the final CSS string chunk.

#### Scenario: Serializer produces compressed CSS / 压缩模式
- **WHEN / 当** 在 Compressed 样式下 CssStmt 流结束时
- **THEN / 那么** Serializer 最终发射的单个 String 为 `"a{color:red}"`

#### Scenario: Serializer produces expanded CSS / 展开模式
- **WHEN / 当** 相同输入使用 Expanded 样式时
- **THEN / 那么** Serializer 最终发射的单个 String 为 `"a {\n  color: red;\n}\n"`

#### Scenario: Serializer does not collect into Vec / Serializer 不 collect 为 Vec
- **WHEN / 当** 检查 `serialize_stream` 函数体
- **THEN / 那么** 不出现 `collect_boxed(css_stream)` 后传 `&[CssStmt]` 给纯函数的模式

---

### Requirement: Error propagation SHALL use CompileError channel / 错误传播 SHALL 使用 CompileError 通道

系统 SHALL 将解析错误、求值错误和运行时错误通过 Observable 的 `CompileError` 错误通道传播。所有类型别名的错误类型 SHALL 为 `CompileError`（替代 `Infallible`）。`collect_boxed` 终端 SHALL 正确捕获错误并返回 `Result<Vec<T>, CompileError>`。

The system SHALL propagate all errors through the Observable's `CompileError` channel. All type alias error types SHALL be `CompileError` (NOT `Infallible`). The terminal `collect_boxed` SHALL properly capture errors and return `Result<Vec<T>, CompileError>`.

#### Scenario: CompileError type across all stages / 所有阶段使用 CompileError
- **WHEN / 当** 检查类型别名 `TokenStream`、`AstStream`、`CssStream`、`OutputStream` 时
- **THEN / 那么** 所有别名的错误类型参数均为 `CompileError`

#### Scenario: collect_boxed returns Result / collect_boxed 返回 Result
- **WHEN / 当** `collect_boxed` 消费的 Observable 发射了 `on_error`
- **THEN / 那么** `collect_boxed` 返回 `Err(CompileError)`，终端调用者（`from_string`）将其传播给最终用户

---

## ADDED Requirements

### Requirement: Pipeline type aliases SHALL use CompileError / 管道类型别名 SHALL 使用 CompileError

`src/types.rs` SHALL 定义以下类型别名（精确代码）：

```rust
pub type TokenStream = SharedBoxedObservable<'static, Token, CompileError>;
pub type AstStream   = SharedBoxedObservable<'static, AstNode, CompileError>;
pub type CssStream   = SharedBoxedObservable<'static, CssStmt, CompileError>;
pub type OutputStream = SharedBoxedObservable<'static, String, CompileError>;
```

文件顶部的 `use std::convert::Infallible;` SHALL 被删除（不再需要）。

The file SHALL delete `use std::convert::Infallible;`.

#### Scenario: Type aliases match exactly / 类型别名精确匹配
- **WHEN / 当** 检查 `src/types.rs` 中的 `pub type` 定义
- **THEN / 那么** 四个别名的错误类型参数均为 `CompileError`

#### Scenario: No Infallible import / 无 Infallible import
- **WHEN / 当** 检查 `src/types.rs` 的 import 区域
- **THEN / 那么** 不出现 `use std::convert::Infallible;`

---

### Requirement: Lexer SHALL not box_it and use CompileError / Lexer SHALL 不使用 box_it 且错误类型为 CompileError

`src/lexer/mod.rs` 的 `scan` 函数 SHALL 返回不再 box_it 的类型。精确签名变更：

变更前：
```rust
pub fn scan(source: &str) -> TokenStream {
    // ... Shared::create(...).box_it()
}
```

变更后：
```rust
pub fn scan(source: &str) -> ScanSource {
    let src = source.to_string();
    Shared::create(move |subscriber| {
        let mut st = LexerState::new();
        for ch in src.chars() {
            let toks = st.feed(ch);
            for t in toks {
                if !matches!(t, Token::Whitespace) {
                    subscriber.next(t);
                }
            }
        }
        let mut final_toks = Vec::new();
        st.flush_buf_checked(&mut final_toks);
        if st.take_pending_slash() {
            final_toks.push(Token::Slash);
        }
        for t in final_toks {
            if !matches!(t, Token::Whitespace) {
                subscriber.next(t);
            }
        }
        subscriber.next(Token::Eof);
        subscriber.complete();
    })
}
```

其中 `ScanSource` 定义为 `type ScanSource = Shared<Create<Token, impl FnMut(&mut dyn Observer<Token, CompileError>) -> Subscription, CompileError>>;`

但由于 impl Trait 在 type alias 中的限制，替代方案为在不同方案间选择：

**方案 A**: 使用 `Shared<Create<Token, Box<dyn FnMut(...) -> Subscription>, CompileError>>` — 需要 heap allocation
**方案 B**: 继续返回 `TokenStream = SharedBoxedObservable<'static, Token, CompileError>` 但 `box_it()` 在 Parser 边界处理

**推荐**: 方案 B（保持 Lexer 接口稳定），Lexer `scan()` 仍返回 `TokenStream`（即 boxed），box_it 仍在 Lexer 边界调用。关键是**不在算子链中间 box_it**，而不是完全消除 box_it。

Lexer 的错误类型从 `Infallible` 改为 `CompileError`（在 `Shared::create` 闭包中通过 `subscriber.error(CompileError::Parse {...})` 发射错误）。

#### Scenario: Lexer box_it at boundary / Lexer 在边界 box_it
- **WHEN / 当** 检查 `src/lexer/mod.rs` 的 `scan` 函数
- **THEN / 那么** `box_it()` 仅在 `Shared::create(...).box_it()` 一处出现
- **THEN / 那么** 错误类型参数包含 `CompileError`

---

### Requirement: Parser Entry SHALL use scan_map / Parser 入口 SHALL 使用 scan_map

`src/parser/mod.rs` 的 `parse_stream_with_paths` 函数 SHALL 重构为：

```rust
pub fn parse_stream_with_paths(
    token_stream: TokenStream,
    scope_id: u64,
    include_paths: Vec<std::path::PathBuf>,
) -> AstStream {
    let mut state = ParserState::with_include_paths(include_paths);
    state.scope_id_counter = scope_id;
    token_stream
        .scan_map(state, parser_feed)
        .flat_map(Shared::from_iter)
        .box_it()
}

fn parser_feed(state: &mut ParserState, tok: Token) -> Vec<AstNode> {
    state.push_token(tok);
    let mut emitted = Vec::new();
    while let Some(node) = try_step(state) {
        emitted.push(node);
    }
    emitted
}
```

**关键**：`parser_feed` 闭包是 `scan_map` 的 FnMut，接收单个 Token 并可能产出 0 或多个 AstNode。`try_step(state)` 是新增函数，内部循环检测缓冲区中的 token 是否构成完整语句。

#### Scenario: No collect_boxed in parse_stream / parse_stream 中无 collect_boxed
- **WHEN / 当** 检查 `parse_stream_with_paths` 函数体
- **THEN / 那么** 不包含 `crate::collect_boxed(token_stream)` 调用
- **THEN / 那么** 不包含 `let tokens: Vec<_> = ...`

#### Scenario: scan_map + flat_map pipeline / scan_map + flat_map 管线
- **WHEN / 当** 用 grep 搜索 `scan_map` 
- **THEN / 那么** 在 `src/parser/mod.rs` 中找到 `.scan_map(state, parser_feed)` 调用
- **THEN / 那么** 下接 `.flat_map(Shared::from_iter)` 调用

---

### Requirement: Parser try_step for incremental completion / Parser try_step 增量完成检测

`src/parser/mod.rs`  SHALL 包含 `try_step(state: &mut ParserState) -> Option<AstNode>` 函数，实现增量完成检测：

1. 检测缓冲区头部 token 类型（根据首个 token 决定解析路径）
2. 尝试从当前缓冲区解析完整一条语句
3. 若缓冲区 token 不足以完成解析（如仅有 `"a {"` 没有 `}`），返回 `None`
4. 若成功解析完整语句，消费对应 token，返回 `Some(AstNode)`
5. 若解析失败，跳过错误 token 并返回 `None`（让后续 token 继续尝试）

`try_step` SHALL 复用现有 `parse_*` 函数（`parse_rule`, `parse_style_decl`, `parse_at_rule` 等），但因这些函数是为修改了 cmd 状态设计的，需确保 `ParserState::peek()`/`next_token()` 操作的是动态缓冲区。

**注意**：现有的 `parse_all_nodes` 函数将被 `scan_map(parser_feed)` 管线替代。但现有的 `parse_*` 子函数（`parse_rule`, `parse_at_rule`, `parse_style_decl` 等）应保留并可能被 `try_step` 内部调用。

**未闭合分隔符检测**：ParserState 的 `brace_stack` 字段已在跟踪 `{`/`}`，当 scan_map 管线完成（上游 Eof 到来后），需要 `do_unclosed_check(state)` 检查 `brace_stack` 是否为空；若非空，通过 `CompileError::Parse` 发射错误。

由于 `scan_map` 是纯算子，无法在管线结束时注入额外事件——解决方案：在 Eof token 到来时（parser_feed 接收 `Token::Eof`），进行检测并存储错误标志，后续由 `scan_map` 的 on_complete 处理；或者使用 `tap` 算子在 terminal 捕获错误。

**实际方案**：将未闭合分隔符检测逻辑移至 `parser_feed` 处理 `Token::Eof` 时：
```rust
fn parser_feed(state: &mut ParserState, tok: Token) -> Vec<AstNode> {
    if matches!(tok, Token::Eof) {
        if let Err(e) = check_unclosed_delimiters(state) {
            // 无法在 scan_map 中 emit error event...
        }
        return vec![];
    }
    state.push_token(tok);
    // ... 正常解析
}
```

由于 rxrust 的 scan_map 映射闭包无法直接发送错误到 observable 错误通道，一种方案是：使用 `Shared::create` 包装整个 scan 过程，手动调用 `subscriber.error()`。或者将错误用 Option/Result 包装为 Output item，在下游用 `filter_map` 处理。

**最终方案**：使用 `scan_map` + `map` 将错误转为 `Result<Vec<AstNode>, CompileError>`，下游用自定义算子处理；或者更简单地在 scan_map 中 panic/send error via shared state。

鉴于 rxrust 算子链中传递错误的复杂性，**推荐**将 unclosed delimiter 错误存储在 ParserState 的 `errors: Vec<ParseError>` 字段中，在 `parser_feed` 的 Eof 分支检测并 pop 错误，然后由管线末端的 `tap` 算子检查状态并调用 `subscriber.error()`——但这超出了 scan_map 的能力范围。

**最简洁方案**：ParserState 增加 `unclosed_error: Option<CompileError>` 字段，在 Eof 分支填充，然后使用 `.last()` 终止算子检查此状态——但这破坏了响应式原则。

**最终推荐**：保留简单的 `check_unclosed_delimiters` 函数在 parser_feed 的 Eof 分支直接执行检查，如果检测到未闭合，存入 ParserState.errors（非阻塞）。然后在 ParserState 增加 `into_errors() -> Vec<CompileError>` 方法。在 `parse_stream_with_paths` 管线末端用 `.last()` 算子提取最终状态，如果有错误则转为 Observable 错误。

但由于时间紧迫，实际最低风险方案：将 Parser 拆分为两步——scan_map 管线产 AstNode，然后用 `.collect::<Vec<_>>()` 收集结果同时检查 unclosed delimiters 并转为 `Result<Vec<AstNode>, CompileError>`再重新包装为 Observable。但这不又回到了 collect 吗？

实际上，我们可以让 `scan_map` 生成 `Vec<AstNode>` 或 `CompileError` 两种 item，下游用 filter_map 分流。或者采用 rxrust 的 `map` 将 scan_map 的 output 转为 Result，下游用 `and_then` 处理错误。

但是 rxrust 的 `.and_then` 不存在。最佳方案：让 scan_map 的返回类型为 `Vec<Result<AstNode, CompileError>>`，在内部 push Ok(node) 或 vec![Err(error)]，下游用 `flat_map(Shared::from_iter)` + `map(|r| r)` 保留 Result，最终由 collect_boxed 统一处理错误。

综合权衡，鉴于 rxrust 算子链的错误传播限制，**最实际的方案**是：

`parser_feed` 在检测 token 进程中，如果遇到未闭合分隔符（只在 Eof 时判断），通过 `std::sync::mpsc::channel` 的 side channel 发射错误事件——但这破坏了纯响应式。

**结论**：采用 `tokio::sync::oneshot` 或共享 `Arc<Mutex<Option<CompileError>>>` 在管线末端检查。这是可接受的折中，因为 Parser 不可能流式地发现未闭合错误——必须等所有 token 到齐才知道某 `{` 未闭合。但这与 "全链路响应式" 矛盾吗？不矛盾：scan_map 仍逐 token 增量产出 AstNode（每个完整语句即时 emit），未闭合检测只在流结束时进行一次检查——这是"全链路响应式"的合理边界条件。

最终决定：ParserState 增加 `check_error: Option<CompileError>` 字段。Eof 时检查。管线结构：
```
scan_map(ParserState, parser_feed) -> Vec<AstNode>
  .flat_map(Shared::from_iter)
  .tap(|node| { /* 透传 */ })
  .box_it()
```
然后用 `Arc<Mutex<ParserState>>` 在管线末端检查 check_error，有错误时通过 `do_unchecked` 中断——不行。

**最简洁实用方案**：在 scan_map 完成后用 `.tap()` 检查 `state.check_error`，但由于 tap 不能 emit 错误，改用 `.and_then(...)` 不存在。

**最终方案**：让 scan_map 的 Output = `Vec<Result<AstNode, CompileError>>`。parser_feed 的每次调用 push Ok(node) 列表；Eof 时如果检测到未闭合，push `vec![Err(CompileError::Parse {...})]`；下游 flat_map 展平后得到 `Observable<Result<AstNode, CompileError>>`，再 `.map(|r| r.unwrap_or_else(|e| { /* 无法处理 */ panic!() })`——不好。

换种方式：使用 `Shared::create` 替代 scan_map。手动订阅 TokenStream，按增量模式处理，产出 AstNode/Error via subscriber。这是唯一保证正确性的方案。

**最终最终方案**：parse_stream_with_paths 内部用 `Shared::create` 包装，手动管理 ScanState：

```rust
pub fn parse_stream_with_paths(
    token_stream: TokenStream,
    scope_id: u64,
    include_paths: Vec<std::path::PathBuf>,
) -> AstStream {
    let mut child_state = ParserState::with_include_paths(include_paths);
    child_state.scope_id_counter = scope_id;
    Shared::create(move |subscriber: &mut dyn Observer<AstNode, CompileError>| {
        let mut state = child_state;
        // 订阅 token_stream
        token_stream.subscribe_all(
            move |tok| {
                let emitted = parser_feed(&mut state, tok);
                for node in emitted {
                    subscriber.next(node);
                }
            },
            move |err: CompileError => {
                subscriber.error(err);
            },
            move || {
                // 流结束：检查未闭合分隔符
                if let Err(e) = check_unclosed_delimiters(&state) {
                    subscriber.error(e);
                } else {
                    subscriber.complete();
                }
            },
        );
        ()  // 返回 Subscription (empty tuple)
    }).box_it()
}
```

其中 `parser_feed` 和 scan_map 版本一样——推入 token 后循环 try_step 产出 AstNode。

但这又陷入 GC 反模式吗？不，`Shared::create` 是 rxrust 推荐的**自定义源创建方式**（参见 rxrust/SKILL.md "Creating Observable" 章节）。这不是 GC——GC 是 `Arc<Mutex<Vec>> + subscribe(push)`。`Shared::create` 是原生算子，正确地把上游订阅桥接到下游 subscriber。

**这确实是正确答案**：用 `Shared::create` 替代 scan_map 做 Parser 的自定义 rxrust 源。原因：
1. scan_map 无法在检测到错误时调用 `subscriber.error()`
2. Shared::create 可以手动桥接上游 subscribe + 下游 subscriber
3. 这是 rxrust SKILL.md 推荐的自定义源模式
4. 上游 unsubscribe 通过 Subscription teardown 正确传播

#### Scenario: Parser uses Shared::create not scan_map / Parser 用 Shared::create 不用 scan_map
- **WHEN / 当** 检查 `parse_stream_with_paths` 的函数体
- **THEN / 那么** 发现 `Shared::create(move |subscriber| { ... })` 包装了手动 token bridge
- **THEN / 那么** 内部订阅 `token_stream.subscribe_all(...)` 将 token 推入 parser_feed

#### Scenario: Parser emits then checks unclosed on complete / Parser 完成后检查未闭合
- **WHEN / 当** upstream TokenStream 发出 `complete`
- **THEN / 那么** Parser 的订阅 on_complete 调用 `check_unclosed_delimiters`
- **THEN / 那么** 若有未闭合分隔符，调用 `subscriber.error(CompileError::Parse {...})`
- **THEN / 那么** 若无未闭合分隔符，调用 `subscriber.complete()`

---

### Requirement: Eval SHALL use expand not flat_map / Eval SHALL 使用 expand 而非 flat_map

`src/eval/mod.rs` 的 `eval_stream` 函数 SHALL 重构为：

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

**关键变更**：
1. 删除 `let nodes: Vec<AstNode> = crate::collect_boxed(ast_stream);` — 消除断裂点
2. 删除 `build_css_stream(nodes, ...)` 包装函数（或改为内联）
3. 将 `flat_map` 改为 `expand`（深度优先语义，对嵌套规则正确）
4. 删除 `eval_nodes_sync`, `eval_ast_stream_sync` 同步包装函数（不再需要）

`src/eval/emit.rs` 的 `emit_events` 函数签名 SHALL 从：
```rust
pub(crate) fn emit_events(
    node: AstNode,
    parent_sel: Option<String>,
    ctx: Arc<EvalContext>,
    bus: CompilerBus,
) -> SharedBoxedObservable<'static, EvalEvent, Infallible>
```
变为：
```rust
pub(crate) fn emit_events(
    node: AstNode,
    parent_sel: Option<String>,
    ctx: Arc<EvalContext>,
    bus: CompilerBus,
) -> SharedBoxedObservable<'static, EvalEvent, CompileError>
```

函数内部的 `Infallible` 出现 SHALL 全部替换为 `CompileError`。

#### Scenario: eval_stream uses expand / eval_stream 使用 expand
- **WHEN / 当** 检查 `src/eval/mod.rs` 的 `eval_stream` 函数
- **THEN / 那么** 发现 `.expand(move |node| emit_events(...))` 调用
- **THEN / 那么** 不发现 `.flat_map(move |node| emit_events(...))`
- **THEN / 那么** 不发现 `collect_boxed(ast_stream)` 调用

#### Scenario: emit_events return type is CompileError / emit_events 返回类型为 CompileError
- **WHEN / 当** 检查 `src/eval/emit.rs` 的 `emit_events` 函数签名
- **THEN / 那么** 返回类型为 `SharedBoxedObservable<'static, EvalEvent, CompileError>`

---

### Requirement: SerializeStream SHALL be reactive fold / SerializeStream 必须是响应式 fold

`src/serialize/mod.rs`  SHALL 新增 `serialize_stream` 函数和 `SerializeState` 结构体：

```rust
pub struct SerializeState {
    stmts: Vec<CssStmt>,
    options: Options,
}

impl SerializeState {
    pub fn new(options: Options) -> Self {
        Self { stmts: Vec::new(), options }
    }
    pub fn push(&mut self, stmt: CssStmt) {
        self.stmts.push(stmt);
    }
    pub fn render(self) -> String {
        crate::serialize::serialize(&self.stmts, &self.options)
    }
}

pub fn serialize_stream<S>(css_stream: S, options: Options) -> OutputStream
where
    S: Observable<Item = CssStmt, Err = CompileError> + Send + 'static,
{
    css_stream
        .fold(SerializeState::new(options), |mut state, stmt| {
            state.push(stmt);
            state
        })
        .map(|state| state.render())
        .box_it()
}
```

**关键**：`serialize_stream` 管线末端只产生**一个** String 块（所有 CssStmt 累积后一次 render）。这与原 `serialize()` 的 Expanded/Compressed 渲染语义一致。

#### Scenario: serialize_stream emits single String / serialize_stream 产出单个 String
- **WHEN / 当** 订阅 `serialize_stream(css_stream, opts)`
- **THEN / 那么** 在 CSS 流完成后，receive 一个 `on_next(String)` 然后 `on_complete()`
- **THEN / 那么** 该 String 等同于原 `serialize(&stmts, &opts)` 输出

#### Scenario: SerializeState wraps serialize / SerializeState 包装 serialize
- **WHEN / 当** 检查 `src/serialize/mod.rs`
- **THEN / 那么** 存在 `struct SerializeState { stmts: Vec<CssStmt>, options: Options }`
- **THEN / 那么** 存在 `impl SerializeState { fn new / fn push / fn render }` 三个方法
- **THEN / 那么** `render` 方法内部调用 `crate::serialize::serialize(&self.stmts, &self.options)`

---

### Requirement: collect_boxed SHALL handle CompileError / collect_boxed SHALL 处理 CompileError

`src/observable_ext.rs` 的 `collect_boxed` 函数 SHALL 重构为：

```rust
pub fn collect_boxed<T: Send + 'static>(
    stream: SharedBoxedObservable<'static, T, CompileError>,
) -> Result<Vec<T>, CompileError> {
    let (tx, rx) = mpsc::channel::<Vec<T>>();
    let (err_tx, err_rx) = mpsc::channel::<CompileError>();
    stream.collect::<Vec<_>>().subscribe(
        move |v| { let _ = tx.send(v); },
        move |e| { let _ = err_tx.send(e); },
    );
    // 先检查是否有错误
    if let Ok(e) = err_rx.try_recv() {
        return Err(e);
    }
    Ok(rx.recv().unwrap_or_default())
}
```

**关键**：rxrust 的 `.subscribe(on_next, on_error)` 双参数版本 SHALL 被使用，捕获 CompileError 并向上传播。

#### Scenario: collect_boxed propagates errors / collect_boxed 传播错误
- **WHEN / 当** `stream` 发射了 `on_error(CompileError::Parse {...})`
- **THEN / 那么** `collect_boxed` 返回 `Err(CompileError::Parse {...})`
- **THEN / 那么** 调用者的 `from_string` 返回 `Err(...)` 给最终用户

#### Scenario: collect_boxed success / collect_boxed 成功
- **WHEN / 当** `stream` 正常完成
- **THEN / 那么** `collect_boxed` 返回 `Ok(Vec<T>)`

---

### Requirement: pipeline.rs SHALL orchestrate reactive stages / pipeline.rs SHALL 编排响应式阶段

`src/pipeline.rs` SHALL 重构为通过管线编排全部阶段。精确代码结构：

```rust
use std::path::Path;
use rxrust::prelude::*;
use crate::eval::eval_stream;
use crate::lexer::scan;
use crate::parser::parse_stream_with_paths;
use crate::runtime::create_runtime;
use crate::serialize::{self, Options, serialize_stream};
use crate::types::*;

/// 组装全链路响应式管线
fn compile_pipeline(source: &str, options: &Options, include_paths: Vec<std::path::PathBuf>) -> OutputStream {
    let tokens = scan(source);
    let (ctx, _bus) = create_runtime();
    let ast = parse_stream_with_paths(tokens, ctx.scope_id(), include_paths);
    let css = eval_stream(ast, ctx);
    serialize_stream(css, options.clone())
}

pub fn from_string(source: &str, options: &Options) -> Result<String, CompileError> {
    from_string_with_paths(source, options, Vec::new())
}

pub fn from_string_with_paths(source: &str, options: &Options, include_paths: Vec<std::path::PathBuf>) -> Result<String, CompileError> {
    let stream = compile_pipeline(source, options, include_paths);
    // 终端阻塞收集
    let chunks = collect_boxed(stream)?;
    Ok(chunks.join(""))
}

pub fn from_path(path: &Path, options: &Options) -> Result<String, CompileError> {
    let source = std::fs::read_to_string(path)
        .map_err(|e| CompileError::Io(format!("Failed to read {:?}: {}", path, e)))?;
    let mut include_paths = vec![];
    if let Some(parent) = path.parent() {
        include_paths.push(parent.to_path_buf());
    }
    from_string_with_paths(&source, options, include_paths)
}
```

同时删除原有的 `collect_stream` 辅助函数（已被 `collect_boxed` 吸收）。

#### Scenario: pipeline uses pure reactive stages / pipeline 用纯响应式阶段
- **WHEN / 当** 阅读 `compile_pipeline` 函数
- **THEN / 那么** 函数体为 `scan → parse_stream_with_paths → eval_stream → serialize_stream` 的函数组合链
- **THEN / 那么** 中间无 `collect_boxed`、无 `Vec`、无命令式 for 循环

#### Scenario: from_string blocks at terminal / from_string 在终端阻塞
- **WHEN / 当** 调用 `from_string(source, opts)`
- **THEN / 那么** `compile_pipeline` 组装惰性 Observable（不发射任何数据直到被订阅）
- **THEN / 那么** `collect_boxed(stream)` 订阅管线，阻塞等待最终结果
- **THEN / 那么** 返回 `Ok(String)` 或 `Err(CompileError)`

---

### Requirement: lib.rs SHALL remove sync API exports / lib.rs SHALL 删除同步 API exports

`src/lib.rs` SHALL 删除对 `eval_ast_stream_sync` 的引用（该函数在 eval/mod.rs 中已删除）。

```rust
pub mod builder;
pub mod bus;
pub mod eval;
pub mod lexer;
pub mod observable_ext;
pub mod parser;
pub mod pipeline;
pub mod runtime;
pub mod serialize;
pub mod telemetry;
pub mod types;

pub use builder::CompileBuilder;
pub use observable_ext::{ObservablePipe, collect_boxed};
pub use pipeline::{from_path, from_string};
pub use serialize::Options;
pub use types::{AstNode, CssStmt, OutputStyle, Token, Value, InputSyntax, CompileError};
```

`pipeline.rs` 不再导出 `from_string_with_paths`（仅内部使用）。

#### Scenario: lib.rs exports / lib.rs exports
- **WHEN / 当** 检查 `src/lib.rs` 的 `pub use pipeline::{...}` 行
- **THEN / 那么** 导出 `from_path` 和 `from_string`
- **THEN / 那么** 不导出 `collect_stream` 或 `from_string_with_paths`

---

### Requirement: Infallible SHALL be purged from all files / Infallible SHALL 从所有文件中清除

搜索整个 `src/` 目录，确认：

1. **`src/types.rs`** — 无 `use std::convert::Infallible;`
2. **`src/lexer/mod.rs`** — 无 `Infallible` 出现
3. **`src/parser/mod.rs`** — 无 `Infallible` 出现（如有）
4. **`src/parser/state.rs`** — 无 `Infallible` 出现
5. **`src/eval/mod.rs`** — 无 `Infallible` 出现
6. **`src/eval/emit.rs`** — 无 `Infallible` 出现
7. **`src/eval/expr.rs`** — 无 `Infallible` 出现（如有引用）
8. **`src/eval/prefixer.rs`** — 无 `Infallible` 出现
9. **`src/eval/builtin.rs`** — 无 `Infallible` 出现（如有引用）
10. **`src/serialize/mod.rs`** — 无 `Infallible` 出现
11. **`src/pipeline.rs`** — 无 `Infallible` 出现
12. **`src/observable_ext.rs`** — 无 `Infallible` 出现
13. **`src/runtime.rs`** — 无 `Infallible` 出现（如有引用）
14. **`src/bus.rs`** — 无 `Infallible` 出现（如有引用）

**实现方法**: 在修改完 `types.rs` 后，全局搜索 `Infallible` 并逐个文件确认。如有 function signatures 仍使用 `Infallible`，改为 `CompileError`。

同时检查 `tests/` 目录中的测试文件：所有引用 `Infallible` 的地方 SHALL 改为 `CompileError`。测试文件的错误类型适配 SHALL 同步完成。

#### Scenario: No Infallible in src/ / src/ 中无 Infallible
- **WHEN / 当** `grep -r "Infallible" src/` 运行
- **THEN / 那么** 返回空结果（无任何匹配行）

#### Scenario: No Infallible in tests/ / tests/ 中无 Infallible
- **WHEN / 当** `grep -r "Infallible" tests/` 运行
- **THEN / 那么** 返回空结果（无任何匹配行）
