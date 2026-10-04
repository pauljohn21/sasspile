# Spec Delta

## Purpose

定义核心响应式编译管道（reactive compilation pipeline），通过 rxrust Observable 流将 Sass 输入字符依次经过四个阶段变换（Lexer、Parser、Evaluator、Serializer）。管道支持流式输出（streaming output）——在完整编译完成之前，CSS 字节就已经开始输出。

Defines the core reactive compilation pipeline that transforms Sass input characters through four connected stages (Lexer, Parser, Evaluator, Serializer) via rxrust Observable streams, supporting streaming output where CSS bytes are emitted before full compilation completes.

## ADDED Requirements

### Requirement: Pipeline SHALL connect all four compilation stages via Observable streams / 管道 SHALL 通过 Observable 流连接全部四个编译阶段

系统 SHALL 将 Lexer → Parser → Evaluator → Serializer 连接成单一 Observable 管道。每个阶段消费上游的产出 Observable，并产出新的 Observable 给下一阶段消费。数据在每个阶段之间单向流动（character → Token → AstNode → CssStmt → String）。

The system SHALL connect Lexer → Parser → Evaluator → Serializer as a single Observable pipeline where each stage consumes the previous stage's output Observable and produces a new Observable for the next stage.

#### Scenario: Basic pipeline connection / 基本管道连接
- **WHEN / 当** 调用 `compile("a { color: red; }")` 时
- **THEN / 那么** 字符流经 Lexer 产出 Token，Token 流经 Parser 产出 AstNode，AstNode 流经 Evaluator 产出 CssStmt，CssStmt 流经 Serializer 产出 CSS 字符串块

#### Scenario: Pipeline produces output incrementally / 管道渐进式输出
- **WHEN / 当** 编译一条包含 100 个顶层规则（top-level rules）的样式表时
- **THEN / 那么** 第一个 CSS 块在所有 100 个规则被解析完成之前就已经被发出（emit）

### Requirement: Lexer SHALL emit Tokens via Observable from character input / Lexer SHALL 从字符输入通过 Observable 发射 Token

系统 SHALL 提供 Lexer，以 `Observable<char>` 作为输入，产出 `Observable<Token>`。每个 Token 携带 `kind: char` 和 `pos: u32`。Lexer 使用 `Local::from_iter(input_chars)` 作为源 Observable。Lexer 还负责字符标准化：将 `\r\n` 和 `\x0C`（form feed）统一转换为 `\n`。

The system SHALL provide a Lexer that takes `Observable<char>` as input and produces `Observable<Token>`. Each Token carries a `kind: char` and `pos: u32`. The lexer SHALL normalize characters: `\r\n` and `\x0C` SHALL be converted to `\n`.

#### Scenario: Lexer emits token per character / 每个字符发射一个 Token
- **WHEN / 当** 输入为 `"abc"` 时
- **THEN / 那么** Lexer Observable 按顺序发射三个 Token：`Token { kind: 'a', pos: 0 }`、`Token { kind: 'b', pos: 1 }`、`Token { kind: 'c', pos: 2 }`

#### Scenario: Lexer handles newline normalization / 处理换行符标准化
- **WHEN / 当** 输入包含 `"\r\n"` 时
- **THEN / 那么** Lexer 在 `'\r'` 位置发射单个 Token `'\n'`

#### Scenario: Lexer handles form feed / 处理换页符
- **WHEN / 当** 输入包含 `'\x0C'`（form feed）时
- **THEN / 那么** Lexer 发射 Token `'\n'`

### Requirement: Parser SHALL emit AstNodes via Observable from Token stream / Parser SHALL 通过 Token 流产出 AstNode

系统 SHALL 提供 Parser，接受 `Observable<Token>` 并产出 `Observable<AstNode>`。Parser SHALL 使用 `scan` 操作符，内部维护一个 `ParserState` 缓冲 token，并通过 `filter_map(Option::AstNode)` 发射完成的 AST 节点。

The system SHALL provide a Parser that takes `Observable<Token>` and produces `Observable<AstNode>`. The parser SHALL use `scan` with an internal `ParserState` that buffers tokens and emits completed AST nodes via `filter_map(Option::AstNode)`.

#### Scenario: Parser emits style rule AST / 发射样式规则 AST
- **WHEN / 当** token 序列代表 `"a { color: red; }"` 时
- **THEN / 那么** Parser 发射 `AstNode::RuleSet { selector: "a", body: [StyleDecl("color", "red")] }`

#### Scenario: Parser supports lookahead via internal buffer / 支持前瞻（lookahead）
- **WHEN / 当** 解析歧义的 token 序列时
- **THEN / 那么** ParserState SHALL 缓冲 token 并支持 `peek_n(N)` 实现任意长度的前瞻，且不消费（consume）任何 token

#### Scenario: Parser supports backtracking via internal buffer / 支持回溯（backtracking）
- **WHEN / 当** 某个产生式规则匹配失败时
- **THEN / 那么** ParserState SHALL 支持 `set_cursor(position)` 重置 token 游标到已保存的位置

### Requirement: Evaluator SHALL emit CssStmts from AstNode stream with multicast wiring / Evaluator SHALL 从 AstNode 流产出 CssStmt，并连接多播总线

系统 SHALL 提供 Evaluator，接受 `Observable<AstNode>` 并产出 `Observable<CssStmt>`。Evaluator SHALL 连接 `CompilerBus` 的 `var_events` 多播 Subject 进行环境查找。

The system SHALL provide an Evaluator that takes `Observable<AstNode>` and produces `Observable<CssStmt>`. The evaluator SHALL connect to the `CompilerBus` multicast variable-events Subject for environment lookups.

#### Scenario: Evaluator produces CSS for a simple rule / 简单规则生成 CSS
- **WHEN / 当** `AstNode::RuleSet { selector: "a", body: [StyleDecl("color", "red")] }` 到达时
- **THEN / 那么** Evaluator 发射 `CssStmt::Rule { selector: "a", declarations: [("color", "red")] }`

#### Scenario: Evaluator reads variables from multicast bus / 从多播总线读取变量
- **WHEN / 当** 求值 `AstNode::VariableDecl("$x", Value::Number(42))` 时
- **THEN / 那么** Evaluator 向 `var_events` Subject 发射 `VarEvent::Bind { name: "$x", value: 42 }`
- **THEN / 那么** 后续引用 `$x` 的 AstNode 通过订阅 `var_events` 观察到该绑定

### Requirement: Serializer SHALL emit CSS string chunks from CssStmt stream / Serializer SHALL 从 CssStmt 流产出 CSS 字符串块

系统 SHALL 提供 Serializer，接受 `Observable<CssStmt>` 并产出 `Observable<String>`（CSS chunks）。Serializer SHALL 处理输出样式（Expanded / Compressed / Nested）以及正确的空白符/分号插入。

The system SHALL provide a Serializer that takes `Observable<CssStmt>` and produces `Observable<String>` (CSS chunks). The Serializer SHALL handle output style (Expanded/Compressed/Nested) and proper whitespace/semicolon insertion.

#### Scenario: Serializer produces compressed CSS / 压缩模式
- **WHEN / 当** Evaluator 在 Compressed 样式下发射 `CssStmt::Rule { selector: "a", declarations: [("color", "red")] }` 时
- **THEN / 那么** Serializer 发射 `"a{color:red}"`

#### Scenario: Serializer produces expanded CSS / 展开模式
- **WHEN / 当** 相同输入使用 Expanded 样式时
- **THEN / 那么** Serializer 发射 `"a {\n  color: red;\n}\n"`

### Requirement: Pipeline SHALL support streaming output API / 管道 SHALL 支持流式输出 API

系统 SHALL 提供 `from_string_stream(input: &str, options: &Options) -> Local<String>`，返回 CSS 字符串块的 Observable，允许调用方在 CSS 块产生时立即接收。调用方可以通过取消订阅（unsubscribe）来中止编译。

The system SHALL expose `from_string_stream(input: &str, options: &Options) -> Local<String>` that returns an Observable of CSS string chunks. Callers can cancel compilation via unsubscribe.

#### Scenario: Caller receives first chunk before compilation completes / 调用方在编译完成前收首块
- **WHEN / 当** 调用方对一个 1000 行的 Sass 文件订阅 `from_string_stream` 时
- **THEN / 那么** 在文件全部解析完之前，第一个 CSS 块已交付到订阅者的 `on_next`

#### Scenario: Caller can cancel compilation via unsubscribe / 调用方可取消编译
- **WHEN / 当** 调用方在编译中途取消订阅时
- **THEN / 那么** 所有上游阶段 SHALL 停止处理（通过 Observable 背压传播）

### Requirement: Pipeline SHALL preserve backward-compatible blocking API / 管道 SHALL 保持向后兼容的阻塞式 API

系统 SHALL 继续提供 `from_string(input: &str, options: &Options) -> Result<String>` 和 `from_path(path: &Path, options: &Options) -> Result<String>`。这两个函数内部订阅响应式管道并将所有块收集到单个 String 中。

The system SHALL continue to expose `from_string()` and `from_path()` which internally subscribe to the reactive pipeline and collect all chunks into a single String.

#### Scenario: from_string produces identical output to grass v0.13 / 输出与 v0.13 完全一致
- **WHEN / 当** 调用 `from_string("$x: 42; a { width: $x }", &Options::default())` 时
- **THEN / 那么** 结果为 `Ok("a {\n  width: 42;\n}\n")`，与参考实现逐字节匹配

#### Scenario: from_path reads file and compiles / 从路径读取文件并编译
- **WHEN / 当** 调用 `from_path("input.scss", &Options::default())` 时
- **THEN / 那么** 文件被读取并通过响应式管道编译

### Requirement: Error propagation SHALL flow through Observable error channel / 错误传播 SHALL 通过 Observable 错误通道流动

系统 SHALL 将解析错误（parse errors）、求值错误（evaluation errors）和运行时错误（runtime errors）通过 Observable 的错误通道（`on_error`）传播，允许调用方通过 `subscribe` 的错误处理程序捕获错误。

The system SHALL propagate parse errors, evaluation errors, and runtime errors through the Observable's error channel (`on_error`).

#### Scenario: Parse error terminates pipeline / 解析错误终止管道
- **WHEN / 当** 输入为 `"a { color: }"`（不完整的声明）时
- **THEN / 那么** Parser 在错误通道上发射错误
- **THEN / 那么** 下游阶段 SHALL 接收错误并不再发射任何值

#### Scenario: Custom function @error propagates / 自定义 @error 传播
- **WHEN / 当** Sass 代码在求值过程中包含 `@error "invalid value"` 时
- **THEN / 那么** 错误通过 Observable 错误通道传播到调用方
