# Design

## Context

当前 reactive-pipeline 有 5 个结构性缺陷（见 proposal.md Why 章节），阻塞了编译器的完整功能构建。实现基础：`rxrust = "1.0.0-rc.5"`、AST 节点（`AstNode`）、`CompilerBus` Subject 多播。本次重构是一次性全量改造——对标 Grass 参考实现，从零构建完整 Sass 编译器，同时完成 OpenSpec 规划的 reactive-pipeline + directive-ops + multicast-bus 三重构。

**关键约束**: 单文件 ≤ 500 行；输出 CSS 与 Grass v0.13.x 逐字节一致；禁止使用 `Box<dyn Fn>` 函数指针实现管线连接；禁止使用 `Infallible` 错误类型。

## Goals / Non-Goals

**Goals:**
- 完整 Sass 编译器：`from_string` / `from_path` 产出与参考实现一致的 CSS 输出
- 原生 rxrust 算子：Lexer/Parser/Evaluator/Serializer 每个阶段用 `Observable::create` 构建，接入调度/背压/取消
- 独立 `SassAstNode` 解析树 + lowering pass，完整表达 SCSS 语法
- 泛型错误类型 `E: crate::Error` 贯穿所有 Observable 通道
- 消除所有 `Rc<RefCell>` 残留，纯响应式收集逻辑
- Bootstrap 5.3.8 dist 验证（编译结果逐字节一致）

**Non-Goals:**
- 不支持 `.sass` 缩进语法文件的完整编译（仅 Parser 接口预留 mode，先做 SCSS）
- 不支持 `@extend`（Bootstrap 不使用，后续单独规划）
- 不支持 source map 生成
- 不支持 `--watch` / interactive REPL
- 不使用 `ractor` / `tokio` actor 框架（纯 rxrust 单线程管线）

## Decisions

### Decision 1: Lexer — `Observable::create` + `scan(LexerState)`

**选择**: 用 `Observable::create` 构建原生算子，内部通过 `scan(LexerState)` 逐字符扫描产出 Token。

**替代方案**:
- (A) `Observable::from_iter` + `map` — 被否决，无法处理需要状态机的 token（如字符串内的转义）
- (B) 一次性词法分析产出 `Vec<Token>` — 被否决，违背流式处理原则

**理由**: `Observable::create` 将 full 闭包生命周期交给 rxrust 管理，支持取消；`scan` 维护 `LexerState`，符合"无 `RefCell`"约束。每个 Token 携带 `pos: u32` 用于错误定位。

### Decision 2: Parser — `Local::create` + `scan_map(ParserState)` + `flat_map` + `collect`

**选择**: Parser 使用 `Local::create` 包装 `scan_map(ParserState)` + `flat_map` + `collect` 的响应式管道。`scan_map` 增量接收 Token（闭包接收 `&mut ParserState`），每次尝试解析完整语句输出 `Vec<SassAstNode>`；`flat_map` 将 Vec 展平为单个节点流；`collect` 收集所有节点；`subscribe` 转发到 downstream subscriber。流结束后通过 `check_unclosed_delimiters` 扫描 ParserState 检查未闭合分隔符。

**替代方案**:
- (A) `lalrpop` / `pest` 外部 Parser 生成器 — 被否决，增外部依赖、不支持 Observable 流式消费
- (B) 一次性解析产出 `Vec<SassAstNode>` — 被否决，无法流式
- (C) `Observable::create` 无 `scan` — 被否决，`scan_map` 更自然支持增量解析且支持 `&mut` 状态

**理由**: `scan_map` 闭包接收 `&mut Acc` 允许直接修改 ParserState；`flat_map` 自然展平 `Vec<SassAstNode>`；错误传播通过流结束后的 `check_unclosed_delimiters` 检查实现，扫描 ParserState 中所有 token（包括已消费的）以发现未闭合的 `(` 或 `{`。

### Decision 3: `SassAstNode` 独立解析树 + Lowing

**选择**: Parser 产出 `SassAstNode`（含 `Interpolated`、`ParentSelector`、`MapLiteral` 等变体），经独立 lowering pass 转为 `AstNode`。

**替代方案**:
- (A) Parser 直接产出 `AstNode` — 被否决，AstNode 是 Evaluator 消费型 AST，无法表达源码级语法差异
- (B) 复用 Grass 的 `SassAstNode` 定义 — 被否决，独立实现路径

**理由**: 解析树 / 消费型 AST 分离是编译器经典架构（参考 rustc HIR → MIR）。lowering 集中处理插值展开、& 展开、Map→Value 转换，使 Evaluator 逻辑保持简洁。

### Decision 4: Evaluator — `SassOp` 返回 `Observable<AstNode>`

**选择**: `SassOp` trait 定义 `fn into_operator(self, ctx: Rc<EvalContext>) -> Observable<AstNode>`。每个指令用 `Observable::create` 构建算子。

**替代方案**:
- (A) 保持 `Box<dyn Fn>` 函数指针 — 被否决，绕过了 rxrust 调度/背压/取消
- (B) `async fn` + tokio — 被否决，引入 async 运行时依赖，rxrust 已提供充足流式原语

**理由**: `Observable::create` 返回的算子与 `switch_map` / `flat_map` / `tap` 原生组合，错误通过 `on_error` 通道统一传播。

### Decision 5: 错误类型 — 泛型 `E: crate::Error`

**选择**: Observable 通道错误类型从 `Infallible` 改为泛型 `E: crate::Error`。`crate::Error` 新增 `LexerError`、`ParserError`、`LoweringError`、`EvalError`、`SerializerError` 变体。

**替代方案**:
- (A) `Box<dyn std::error::Error>` — 被否决，失去具体类型信息
- (B) 各阶段各自错误类型 + `From` 转换 — 可行，但增加类型参数数量

**理由**: 泛型 `E` 允许 `CompilerBus<E>` 各 Subject 携带错误事件，同时保持类型安全。

### Decision 6: CssStmt 收集 — `Observable::create` + `scan(CssBuffer)`

**选择**: 消除 `collect_css` 中的 `Rc<RefCell<CssBuffer>>`。改为 `Observable::create` + `scan(CssBuffer)` 实现。当收到 `ScopeEvent::Exit` 时 flush 缓冲区，发射包装后的 `CssStmt::Media`。

**替代方案**:
- (A) `std::iter::from_fn` + 迭代器 — 被否决，非响应式
- (B) `tokio::sync::Mutex` — 被否决，引入 async

**理由**: 纯 rxrust 算子，无运行时共享可变状态。

### Decision 7: OutputStyle 支持

**选择**: Serializer 接受 `OutputStyle` 参数（Expanded/Compressed/Nested），内部用 scan + state 管理缩进层级。

**理由**: 覆盖 Bootstrap dist（Compressed）和开发调试（Expanded）两大用例。

### Decision 8: Filesystem 抽象

**选择**: 定义 `pub trait Fs { fn resolve(&self, path: &Path) -> io::Result<String> }`。生产用 `StdFs`，测试用 `NullFs`（空实现）。

**理由**: 解耦文件系统访问，使 `@import` 可在测试中 mock。

### Decision 9: 内置函数模块化

**选择**: `src/builtin/` 目录下按模块分文件：`color.rs`、`math.rs`、`string.rs`、`list.rs`、`map.rs`、`meta.rs`、`selector.rs`。每个文件导出一个 `register_functions(scope: &mut Scope)` 函数。

**理由**: 符合"单文件 ≤ 500 行"约束，模块化易于扩展新函数。

## Risks / Trade-offs

| Risk | 影响 | Mitigation |
|------|------|------------|
| `Observable::create` 闭包生命周期复杂 | 编译错误风险高 | 严格遵循 `'static` move 语义；`ctx` 通过 `Rc::clone` 共享 |
| 错误类型泛型 `E` 传染所有 API | 类型签名膨胀 | 在 `src/lib.rs` 中一次性 type alias：`type SassResult<T> = Result<T, SassError>` |
| 流式 Serializer 的正确性 | 复杂嵌套规则输出可能错误 | Bootstrap dist 逐字节验证作为最高警戒测试 |
| 500 行限制下 parser 拆分 | 模块间共享 ParserState | parser/ 拆分为 mod.rs + selectors.rs + at_rules.rs + values.rs |
| rxrust `scan` 状态消耗 | 每 Token 需 clone 检查 | LexerState / ParserState 内部使用 `&str` 切片，仅在 Token clone |

## Migration Plan

1. **Phase 1 — Lexer/Parser 骨架**: 新增 `src/lexer/`、`src/parser/`，不破坏现有 SaaSOp 系统；`from_string` 仍为 stub
2. **Phase 2 — `SassAstNode` + lowering**: 新增 `src/lowering/`，Parser 产出 `SassAstNode`，lowering 为 `AstNode` 后进入现有 Evaluator
3. **Phase 3 — Evaluator 改造**: 将 `SassOp` 从 `Box<dyn Fn>` 改为 `Observable::create` 算子，接入新 Lexer/Parser 管线
4. **Phase 4 — 错误类型泛型化**: 将错误类型从 `Infailable` 改为 `E: crate::Error`，更新全部 Observable 通道签名
5. **Phase 5 — 内置函数 + CSS 输出**: 实现 `src/builtin/` 所有模块，Serializer 实现 OutputStyle
6. **Phase 6 — Bootstrap 验证**: 用 Bootstrap 5.3.8 dist 逐字节验证输出正确性

**回滚策略**: 每个 Phase 单独一个 PR；Phase 1-2 不破坏现有功能，可独立 rollback；如 Phase 3+ 失败，回滚到 Phase 2 的 `SassAstNode` + 旧 Evaluator。

## Open Questions

- [Q1] `Observable::create` 在处理多 token 前瞻时是否需要特殊的 backpressure 策略？ → 可在实现 Phase 2 时决定
- [Q2] Map 值类型的 `Value` 枚举设计（flat vs nested）是否足以表达 Sass 所有值类型？ → Phase 2 lowering 实现时评估
- [Q3] 是否需要在 `SassAstNode` 中保留注释节点？（当前认为不需要，Bootstrap 不使用 `//` 注释输出） → 可在 Phase 2 中确认
