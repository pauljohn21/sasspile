# Spec Delta

## MODIFIED Requirements

### Requirement: Pipeline SHALL connect all four compilation stages via Observable streams / 管道 SHALL 通过 Observable 流连接全部四个编译阶段

系统 SHALL 将 Lexer → Parser → Lowering → Evaluator → Serializer 连接成单一 Observable 管道。每个阶段消费上游的产出 Observable，并产出新的 Observable 给下一阶段消费。数据在每个阶段之间单向流动（character → Token → SassAstNode → AstNode → CssStmt → String）。**MODIFIED**: 新增 Lowering 阶段（SassAstNode → AstNode），原有的 Character → Token → AstNode → CssStmt → String 流变为 Character → Token → SassAstNode → AstNode → CssStmt → String。

The system SHALL connect Lexer → Parser → Lowering → Evaluator → Serializer as a single Observable pipeline where each stage consumes the previous stage's output Observable and produces a new Observable for the next stage. **MODIFIED**: Added Lowering stage (SassAstNode → AstNode), changing the original Character → Token → AstNode → CssStmt → String flow to Character → Token → SassAstNode → AstNode → CssStmt → String.

#### Scenario: Basic pipeline connection / 基本管道连接
- **WHEN / 当** 调用 `compile("a { color: red; }")` 时
- **THEN / 那么** 字符流经 Lexer 产出 Token，Token 流经 Parser 产出 SassAstNode，SassAstNode 经 Lowering 转为 AstNode，AstNode 流经 Evaluator 产出 CssStmt，CssStmt 流经 Serializer 产出 CSS 字符串块

#### Scenario: Pipeline produces output incrementally / 管道渐进式输出
- **WHEN / 当** 编译一条包含 100 个顶层规则（top-level rules）的样式表时
- **THEN / 那么** 第一个 CSS 块在所有 100 个规则被解析完成之前就已经被发出（emit）

### Requirement: Lexer SHALL emit Tokens via Observable from character input / Lexer SHALL 从字符输入通过 Observable 发射 Token

系统 SHALL 提供 Lexer，以 `Observable<char>` 作为输入，产出 `Observable<Token>`。每个 Token 携带 `kind: char` 和 `pos: u32`。Lexer SHALL 使用 `Observable::create` 构建原生操作符，内部维护 `LexerState` 并通过 `scan` 逐字符产出 Token。Lexer 还负责字符标准化：将 `\r\n` 和 `\x0C`（form feed）统一转换为 `\n`。**MODIFIED**: 原规定使用 `Local::from_iter(input_chars)` 改为 `Observable::create` 原生操作符，以接入 rxrust 调度/背压/取消系统。

The system SHALL provide a Lexer that takes `Observable<char>` as input and produces `Observable<Token>`. Each Token carries a `kind: char` and `pos: u32`. The lexer SHALL use `Observable::create` with internal `LexerState` and `scan` to emit tokens character by character. **MODIFIED**: Original specification used `Local::from_iter(input_chars)`, changed to native `Observable::create` operator for rxrust scheduling/backpressure/cancellation integration.

#### Scenario: Lexer emits token per character / 每个字符发射一个 Token
- **WHEN / 当** 输入为 `"abc"` 时
- **THEN / 那么** Lexer Observable 按顺序发射三个 Token：`Token { kind: 'a', pos: 0 }`、`Token { kind: 'b', pos: 1 }`、`Token { kind: 'c', pos: 2 }`

#### Scenario: Lexer handles newline normalization / 处理换行符标准化
- **WHEN / 当** 输入包含 `"\r\n"` 时
- **THEN / 那么** Lexer 在 `'\r'` 位置发射单个 Token `'\n'`

#### Scenario: Lexer handles form feed / 处理换页符
- **WHEN / 当** 输入包含 `'\x0C'`（form feed）时
- **THEN / 那么** Lexer 发射 Token `'\n'`

### Requirement: Parser SHALL emit SassAstNodes via reactive pipeline from Token stream / Parser SHALL 通过响应式管道从 Token 流产出 SassAstNode

系统 SHALL 提供 Parser，接受 `LocalBoxedObservableClone<'static, Token, Infallible>` 并产出 `LocalBoxedObservableClone<'static, SassAstNode, Error>`。Parser SHALL 使用 `Local::create` 包装 `scan_map(ParserState)` + `flat_map` + `collect` 的响应式管道：`scan_map` 增量接收 Token 并维护 ParserState，每次尝试解析完整语句输出 `Vec<SassAstNode>`；`flat_map` 将 Vec 展平为单个节点流；`collect` 收集所有节点到 Vec；`subscribe` 转发到 downstream subscriber。流结束后通过 `check_unclosed_delimiters` 检查未闭合分隔符，决定调用 `error()` 或 `complete()`。

The system SHALL provide a Parser that takes a Token stream and produces a `SassAstNode` stream with error propagation. The parser SHALL use `Local::create` wrapping a reactive pipeline of `scan_map(ParserState)` + `flat_map` + `collect`: `scan_map` incrementally receives tokens maintaining ParserState, attempting to parse complete statements and emitting `Vec<SassAstNode>`; `flat_map` flattens the Vec into individual node streams; `collect` gathers all nodes; `subscribe` forwards to the downstream subscriber. After stream completion, `check_unclosed_delimiters` inspects the ParserState for unclosed `(` or `{` and emits `error()` or `complete()` accordingly.

#### Scenario: Parser emits style rule AST / 发射样式规则 AST
- **WHEN / 当** token 序列代表 `"a { color: red; }"` 时
- **THEN / 那么** Parser 通过 `subscriber.next()` 依次发射 `SassAstNode::Rule { selector: "a", inner: [SassAstNode::StyleDecl { prop: "color", value: "red" }] }`，最后调用 `subscriber.complete()`

#### Scenario: Parser detects unclosed delimiters at stream end / Parser 在流结束时检测未闭合分隔符
- **WHEN / 当** token 序列代表 `"a { color: red;"`（缺少 `}`）时
- **THEN / 那么** Parser 转发所有已解析节点后，通过 ParserState 扫描发现未闭合 `{`，调用 `subscriber.error(Error::parser("unclosed '{'"))`

#### Scenario: Parser supports lookahead via internal buffer / 支持前瞻（lookahead）
- **WHEN / 当** 解析歧义的 token 序列时
- **THEN / 那么** ParserState SHALL 缓冲 token 并支持 `peek_n(N)` 实现任意长度的前瞻，且不消费（consume）任何 token

#### Scenario: Parser supports backtracking via internal buffer / 支持回溯（backtracking）
- **WHEN / 当** 某个产生式规则匹配失败时
- **THEN / 那么** ParserState SHALL 支持 `set_cursor(position)` 重置 token 游标到已保存的位置

### Requirement: Evaluator SHALL emit CssStmts from AstNode stream with multicast wiring / Evaluator SHALL 从 AstNode 流产出 CssStmt，并连接多播总线

系统 SHALL 提供 Evaluator，接受 `Observable<AstNode>` 并产出 `Observable<CssStmt>`。Evaluator SHALL 连接 `CompilerBus` 的 `var_events` 多播 Subject 进行环境查找。**MODIFIED**: 无行为变化，但 `CompilerBus` 的 Subject 错误类型从 `Infallible` 变为泛型 `E`，使错误可经通道传播。

The system SHALL provide an Evaluator that takes `Observable<AstNode>` and produces `Observable<CssStmt>`. The evaluator SHALL connect to the `CompilerBus` multicast variable-events Subject for environment lookups. **MODIFIED**: No behavior change, but `CompilerBus` Subject error type changes from `Infallible` to generic `E` for error propagation.

#### Scenario: Evaluator produces CSS for a simple rule / 简单规则生成 CSS
- **WHEN / 当** `AstNode::RuleSet { selector: "a", body: [StyleDecl("color", "red")] }` 到达时
- **THEN / 那么** Evaluator 发射 `CssStmt::Rule { selector: "a", declarations: [("color", "red")] }`

#### Scenario: Evaluator reads variables from multicast bus / 从多播总线读取变量
- **WHEN / 当** 求值 `AstNode::VariableDecl("$x", Value::Number(42))` 时
- **THEN / 那么** Evaluator 向 `var_events` Subject 发射 `VarEvent::Bind { name: "$x", value: 42 }`
- **THEN / 那么** 后续引用 `$x` 的 AstNode 通过订阅 `var_events` 观察到该绑定

### Requirement: Error propagation SHALL flow through Observable error channel / 错误传播 SHALL 通过 Observable 错误通道流动

系统 SHALL 将解析错误（parse errors）、求值错误（evaluation errors）和运行时错误（runtime errors）通过 Observable 的错误通道（`on_error`）传播。Parser 错误类型从 `Infallible` 改为 `crate::Error`，使错误可经通道传播。Parser SHALL 在流结束后检查未闭合分隔符（`check_unclosed_delimiters`），通过 `scan_tokens` 遍历 ParserState 中所有 token，若存在未闭合的 `(` 或 `{` 则调用 `subscriber.error()`。

The system SHALL propagate parse errors, evaluation errors, and runtime errors through the Observable's error channel (`on_error`). The Parser's error type changes from `Infallible` to `crate::Error`. The Parser SHALL check for unclosed delimiters via `check_unclosed_delimiters` after stream completion, scanning all tokens in ParserState via `scan_tokens`, and emit `subscriber.error()` if unclosed `(` or `{` is detected.

#### Scenario: Unclosed paren detected at stream end / 流结束时检测未闭合括号
- **WHEN / 当** 输入为 `"(blue: #0d6efd"`（缺少 `)`）时
- **THEN / 那么** Parser 转发已解析节点后，`check_unclosed_delimiters` 发现未闭合 `(`，调用 `subscriber.error(Error::parser("unclosed '('"))`
- **THEN / 那么** 下游通过 `on_error` 接收错误，流终止

#### Scenario: Unclosed brace detected at stream end / 流结束时检测未闭合花括号
- **WHEN / 当** 输入为 `"a { color: red;"`（缺少 `}`）时
- **THEN / 那么** Parser 转发已解析节点后，`check_unclosed_delimiters` 发现未闭合 `{`，调用 `subscriber.error(Error::parser("unclosed '{'"))`

#### Scenario: Valid input completes normally / 正常输入正常完成
- **WHEN / 当** 输入为 `"a { color: red; }"` 时
- **THEN / 那么** Parser 转发所有节点后，`check_unclosed_delimiters` 未发现未闭合分隔符，调用 `subscriber.complete()`
