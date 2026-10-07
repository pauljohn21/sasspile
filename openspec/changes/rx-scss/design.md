# Design

## Context

Lightforger 现有 reactive-pipeline change 已完成全部 proposal/specs/design/tasks（`reactive-pipeline-v2`），但实际实现仍缺少 Lexer/Parser，AstNode 语法覆盖不足。Grass 使用 `Rc<RefCell<Environment>>` + Visitor 模式，不符合响应式风格。rx-scss 是全新独立项目，从零构建完整 SCSS 编译器，**不与 Lightforger 共享任何代码或类型**。

**关键约束**: (1) 单文件 ≤ 500 行；(2) 零 `Rc<RefCell<>>`，禁止 GC 思维；(3) 全部 Observable 组件通过 Builder 模式组装；(4) 错误类型从 `Infailable` 改为泛型 `E`；(5) 不追求 dart-sass 兼容，以 sass-spec 为规则参考；(6) **完全独立项目** — 不依赖 Lightforger 任何 crate、不复用 Lightforger 任何类型定义（Token / Value / AstNode 全部重新定义），仅借鉴其响应式管线设计哲学。

## Goals / Non-Goals

**Goals:**
- 完整 SCSS 编译器：`from_string` / `from_path` 产出合规 CSS
- 纯响应式管线：Lexer → Parser → Eval → Serializer 全部通过 `Shared Observable` 连接
- 构造器模式：`CompileBuilder` 链式 API 组装所有组件
- Shared 多线程调度：eval 阶段支持 `observe_on(ThreadPool(n))`
- 多播：CompilerBus 三个 SharedSubject 通道实现事件广播
- Rust 所有权：不可变 EvalContext + Arc<CompilerBus> 零 GC 风格
- 全量 Bootstrap 验证：`bootstrap_test.rs` 编译 Bootstrap 5.3.x SCSS 产出与 dist CSS 逐字节一致的输出

**Non-Goals:**
- 不支持 `.sass` 缩进语法（仅 SCSS + 简化 CSS）
- 不支持 `@extend`（最复杂，后续单独规划）
- 不支持 source map 生成
- 不支持 `--watch` / interactive REPL
- 不引入 `tokio` / async 运行时

## Decisions

### Decision 1: 构造器模式 (Builder)

**选择**: 定义 `CompileBuilder` 结构体，包含 `syntax: InputSyntax`、`include_paths: Vec<PathBuf>`、`scheduler_config: Option<SchedulerConfig>`、`serialize_style: OutputStyle`、`bus: Option<Arc<CompilerBus>>` 字段。提供链式 setter：`.syntax()` / `.include_path()` / `.scheduler()` / `.serialize_style()` / `.bus()`。`build(source)` 方法内部串联 Lexer → Parser → Eval → Serializer 全管线。

**替代方案**:
- (A) 直接函数调用链 (`lex(source).and_then(parse).map(eval)`) — 被否决，无法注入自定义调度/策略
- (B) 四个 StreamFactory / OperatorFactory / SerializerFactory / PipelineFactory trait — 被否决，trait 对象在多阶段管线中造成过度抽象和类型爆炸
- (C) 全局 singleton 构造器 — 被否决，不利于测试注入

**理由**: Builder 模式是 Rust 生态最自然的组合 API（类似 `Command::new()`、`tokio::runtime::Builder`）。字段集合在一个结构体，类型推导友好；链式调用表达力强；`.bus()` 注入测试用 mock bus；`build()` 返回 `CssOutputStream` 与现有 `collect_stream` 集成。单 struct + impl 即可满足，无需 trait 对象间接层。

### Decision 2: Lexer — `Shared::create` + `scan(LexerState)`

**选择**: `CompileBuilder` 的 `build()` 方法内部调用 `Shared::create` 构建 Token Observable，使用 `scan(LexerState::default(), |state, ch| -> Option<Token>)` 逐字符扫描。`scan` 产出 `Option<Token>` 后通过 `filter_map` 过滤 None。

**替代方案**:
- (A) 一次性 `Vec<Token>` + `Shared::from_iter` — 被否决，无法流式
- (B) `Observable::create` 手动 `subscriber.next()` — 可行但不如 `scan` 简洁

**理由**: `scan` 维护 `LexerState`（字符串上下文、插值深度），状态所有权归 rxrust 管理，无 `RefCell` 残留。

### Decision 3: Parser — `scan(ParserState)` + `flat_map`

**选择**: `CompileBuilder` 的 `build()` 方法内部使用 `scan(ParserState::default(), |state, token| -> Vec<SassAstNode>)` 增量解析。ParserState 内部维护 token peek buffer、@rule 嵌套栈、插值上下文。每次 scan 产出零或多个节点，通过 `flat_map(Shared::from_iter)` 展平为 `Observable<SassAstNode>`。

**替代方案**:
- (A) 递归下降完整 AST — 流式复杂度高
- (B) LALR/PEG parser 生成器 — 引入外部依赖

**理由**: `scan` 是 Rust/FRP 风格的状态化流操作，ParserState 的所有权归 rxrust。`flat_map` 自然展平 `Vec<SassAstNode>`。

### Decision 4: Eval — `dispatch_op` 函数 + 每个指令独立 `Shared::create` 算子

**选择**: `CompileBuilder` 内部使用闭包 `dispatch_op(node: AstNode, ctx: Arc<EvalContext>) -> AstStream` 根据 AstNode variant 分发到对应指令算子。每个算子是一个 `Shared::create(|subscriber| { ... })` 闭包。求值管线通过 `ast_stream.flat_map(|node| dispatch_op(node, ctx.clone()))` 组合。dispatch_op 作为独立函数定义，Builder 内部调用。

**替代方案**:
- (A) Visitor 模式 — 被否决，传统命令式不符合响应式目标
- (B) 中央 match 调度 — 可行但无法利用 rxrust 算子调度/背压
- (C) `async fn` 算子 — 被否决，引入 async 运行时

**理由**: `Shared::create` 让每个算子接入 rxrust 调度系统（ThreadPool、backpressure、cancellation）。`flat_map` 天然组合子管线。不同指令可独立 observe_on 不同调度器。

### Decision 5: CompilerBus 多播设计

**选择**: 三个 `SharedSubject<'static, Event, Infallible>` 通道（var_events / module_events / scope_events）。注册表（mixin_fn/module HashMap）用 `Arc<Mutex<HashMap>>` 实现线程安全共享。变量读写通过 `set_var / get_var` 方法委托。

**替代方案**:
- (A) `tokio::sync::broadcast` — 引入 tokio 依赖
- (B) 单个主题 + 事件枚举 — 丧失多播灵活性

**理由**: `SharedSubject` 是 rxrust 原生多播原语；`Arc<Mutex<HashMap>>` 是 Rust 标准共享模式。模块加载后广播 `ModuleEvent::Loaded`，所有等待该模块的 Eval 节点无需轮询。

### Decision 6: 变量作用域 — 算术派生 scope_id

**选择**: `child_scope(local_idx)` 生成 `scope_id = parent_id * 1000 + idx`。变量查找先精确匹配 scope_id，未找到则 `scope_id /= 1000` 向上遍历。全局 scope_id=1 为根。

**替代方案**:
- (A) 栈式 Environment (Rc<RefCell>) — 被否决，GC 思维
- (B) 静态作用域链 — 复杂度高

**理由**: 整数算术派生保证 scope_id 全局唯一且有序，无需可变链表。向上遍历 O(depth) 在 SCSS 典型嵌套深度（< 10）下性能忽略不计。

### Decision 7: Value 类型设计

**选择**: `Value` enum（Number/String/Color/List/Map/Bool/Null）+ `Display` trait。Color 用 RGBA u8 四元组，List 用 `Vec<Value>`，Map 用 `Vec<(String, Value)>`。不支持 Sass Function/ArgumentList/Selector 等复杂类型（不在核心范围内）。

**理由**: 覆盖 sass-spec 核心值类型。Display trait 负责 CSS 输出格式化（如 Color → `#rrggbb`）。Map 用 Vec 而非 HashMap 保持插入顺序（Sass map 有序）。

### Decision 8: Serializer 流式输出

**选择**: `CompileBuilder` 的序列化阶段使用 `stream.collect::<Vec<_>>().flat_map(|stmts| Shared::from_iter(stmts.into_iter().map(format_stmt)))`。每个格式化的 CssStmt 对应流中一个 String 元素。OutputStyle 从 Builder 字段获取并在闭包捕获。

**替代方案**:
- (A) 单 String 累积 — 失去流式意义
- (B) `scan` 维护缩进状态 — 可行，但 `collect` + `flat_map` 更直观

**理由**: 分两阶段（先收集后格式化）简化格式化逻辑。flat_map 展平后的 String 流下游可逐个消费或 join。

### Decision 9: 错误类型 — `CompileError` 枚举

**选择**: `CompileError { Io(String), Parse(String), Eval(String) }`。内部各阶段错误最终转为 `CompileError`。`CompileError` 实现 `std::error::Error` 以支持 `?` 传播。

**理由**: 三阶段（IO/Parse/Eval）错误类型够用。未来可细分但当前保持简洁。

### Decision 10: Bootstrap 全量验证策略（Git Submodule 浅拷贝）

**选择**: 通过项目 `.gitmodules` 配置 Bootstrap submodule (`git@github.com:twbs/bootstrap.git`)，使用 `shallow = true` (depth=1) 浅拷贝，仅获取最新 commit（~25MB 而非完整历史 ~200MB+）。`tests/bootstrap_test.rs` 直接使用 `CompileBuilder::new().include_path("bootstrap/scss/").build("bootstrap/scss/bootstrap.scss")` 编译，对 Expanded 输出与官方 `bootstrap.css` fixtures 逐字节比对。`BOOTSTRAP_FORCE_REFRESH=1` 时 `rm -rf bootstrap && git submodule update --init --depth 1 bootstrap`。

**替代方案**:
- (A) 将 Bootstrap SCSS 嵌入测试二进制 — 文件过大（~5MB 源码），不宜
- (B) 依赖系统全局安装 — 无法保证版本一致性
- (C) 运行时从 GitHub raw 下载 — 测试不稳定性风险

**理由**: Git submodule 浅拷贝保证版本锁定和一致性；depth=1 节省 80%+ 磁盘空间和克隆时间；`bootstrap/` 路径是标准 submodule 位置，由 git 管理；Bootstrap 5.3 是工业级 Sass 代码库，其 SCSS 使用了 mixin 系统、`@use` with 配置、内置模块、Maps、循环、条件等全部高级特性——能一次性发现编译器所有缺陷。

**验证范围**: Bootstrap SCSS 主要模块（约 130 个.scss 文件）：`_variables.scss`、`_mixins.scss`、`_buttons.scss`、`_forms.scss`、`_grid.scss`、`_modal.scss`、`_navbar.scss`、`_functions.scss`、`_maps.scss`。

### Decision 11: Shared Scheduler 生命周期

**选择**: `CompileBuilder::build(source)` 在调用时组装管线但直到被 `collect_stream` 订阅时才启动执行（冷 Observable 语义）。Shared 调度器由 `.scheduler(config)` 配置（传入 `observe_on` 时）。`collect_stream` 函数订阅流并阻塞至 complete。生产代码通过 `from_string` 同步入口：`CompileBuilder::new().build(source).pipe(collect_stream)`。测试可替换 bus、scheduler。

**理由**: 同步 API 入口简化使用；内部 Shared 调度器由 rxrust 管理，无手动线程 join。

## Risks / Trade-offs

| Risk | 影响 | Mitigation |
|------|------|------------|
| `Shared::create` 闭包生命周期 | 编译错误风险高 | 严格 `'static` move 语义，ctx 通过 Arc::clone 共享 |
| `flat_map` 深层指令嵌套 | 调用栈过深 | 限制 @-rule 嵌套深度 ≤ 100 |
| `scan(ParserState)` 状态消耗 | 每 Token clone | ParserState 内部使用索引而非 Token clone |
| 流式 Serializer 正确性 | 嵌套规则输出错误 | Unit tests 覆盖边界条件 |
| 单文件 ≤ 500 行 | parser/eval 拆分 | parser/ 拆分为 mod + at_rules + selectors + values |

## Migration Plan

Phased implementation:

1. **Phase 1 — 值系统 + 总线**: types.rs + bus.rs + runtime.rs + 测试
2. **Phase 2 — Lexer + Parser**: lexer/ + parser/ 模块 + CompileBuilder 集成测试
3. **Phase 3 — Evaluator**: eval/ 模块 + dispatch_op 函数 + 测试
4. **Phase 4 — Serializer + Pipeline**: serialize/ + pipeline.rs + CompileBuilder::build + 集成测试
5. **Phase 5 — 构造器模式**: CompileBuilder 完整实现 + 链式 API + 依赖注入

每个 Phase 单独 commit，可在任意点回滚到上一 Phase。

## Open Questions

- [Q1] @extend 是否后续支持？当前 Non-Goals 排除，但 sass-spec 有测试 → 推迟决定
- [Q2] CSS 模式是否完全跳过解析？→ 决定：CSS 模式走同一 Parser 但禁用 Sass 语法
- [Q3] 错误类型是否支持 source location？→ 决定：Token 携带 pos，错误尽量携带
- [Q4] rx-scss 与 Lightforger 是否共享代码？→ 决定：**完全独立项目** — 不复用任何 Lightforger 类型，所有 AST/Token/Value 从零定义

