# Design

## Context

rx-scss 是全新独立项目，从零构建完整 SCSS 编译器。Grass 使用 `Rc<RefCell<Environment>>` + Visitor 模式，不符合响应式风格。

**关键约束**: (1) 单文件 ≤ 500 行；(2) 零 `Rc<RefCell<>>`，禁止 GC 思维；(3) 全部 Observable 组件通过 Builder 模式组装；(4) 错误类型 `CompileError` 三阶段区分；(5) 不追求 dart-sass 兼容，以 sass-spec 为规则参考；(6) **完全独立项目** — 所有 AST/Token/Value 从零定义，仅依赖 rxrust + tracing 两个外部 crate。

## Goals / Non-Goals

**Goals:**
- 完整 SCSS 编译器：`from_string` / `from_path` 产出合规 CSS
- 纯响应式管线：Lexer → Parser → Eval → Serializer 全部通过 `Shared Observable` 连接
- 构造器模式：`CompileBuilder` 链式 API 组装所有组件
- Shared 多线程调度：eval 阶段支持 `observe_on(ThreadPool(n))`
- 多播：CompilerBus 三个 SharedSubject 通道实现事件广播
- Rust 所有权：不可变 EvalContext + Arc<CompilerBus> 零 GC 风格
- 全量 Bootstrap 验证：`bootstrap_test.rs` 编译 Bootstrap 5.3.x SCSS 产出与 dist CSS 逐字节一致的输出（当前 1.25% < 目标 99%，6 大根因已定位）

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

### Decision 3: Parser — 模块内 Pratt 解析器

**选择**: Parser 模块组织为 `mod.rs`（核心逻辑：parse_atom + parse_expression Pratt 解析器 + parse_selector + parse_property_name + 语句分发）和 `state.rs`（ParserState：tokens/peek/brace_stack/scope_id 管理）。ParserState 内部使用 token peek buffer + Pratt 优先级 climbing 算法，支持完整的 SCSS 语法（变量声明、选择器嵌套、@rules、插值 `#{}`、父引用 `&`）。输出不通过 rxrust Observable，而是直接 `Vec<AstNode>` 求全权。模块拆分：`mod.rs`（核心解析）+ `state.rs`（状态管理），单文件 ≤ 500 行约束。

**替代方案**:
- (A) `scan(ParserState)` + rxrust `flat_map` 流式产出 — 被否决，Parser 上下文复杂度高（嵌套括号/插值/字符串），不适合简单 scan
- (B) LALR/PEG parser 生成器 — 引入外部依赖

**理由**: Pratt 解析器天然处理 SCSS 的操作符优先级和嵌套结构；ParserState 用 `Vec<Token>` peek buffer + index 推进；模块按职责拆分，保持单文件行数约束。

### Decision 4: Eval — 工作队列 + 事件流 + scan 累积

**选择**: Eval 模块组织为 `mod.rs`（expand_nodes_to_events 工作队列 + apply_event scan 累积 + 根入口函数）、`expr.rs`（eval_expr 表达式求值 + resolve_selector + resolve_property + value_to_string + truthy）、`builtin.rs`（Sass 内置函数：color/math/string/list/map/selector dispatch）。工作队列展开嵌套指令 '@if/@for/@each/@while/@mixin/@include/@media/@supports' → EvalEvent 序列（Enter/Leave + Terminal），scan 折叠为 `Vec<CssStmt>`。表达式求值是纯函数（&AstNode + &EvalContext → Value），所有指令语义（变量绑定、作用域传播、null 过滤）在工作队列中处理。

**替代方案**:
- (A) 每个指令独立 `Shared::create` + `flat_map` — 被否决，指令作用域传播需要共享 CompilerBus，cross-cutting 关注点散布在多个算子中
- (B) Visitor 模式 — 被否决，GC 思维

**理由**: 工作队列统一处理嵌套展开和所有指令语义；事件 + scan 累积自然处理选择器嵌套（EnterRule/LeaveRule frame 栈）；表达式求值保持纯函数，便于独立测试。

### Decision 5: CompilerBus 多播设计

**选择**: 三个 `SharedSubject<'static, Event, Infallible>` 通道（var_events / module_events / scope_events）。注册表（mixin_fn/module HashMap）用 `Arc<Mutex<HashMap>>` 实现线程安全共享。变量读写通过 `set_var / get_var` 方法委托。

**替代方案**:
- (A) `tokio::sync::broadcast` — 引入 tokio 依赖
- (B) 单个主题 + 事件枚举 — 丧失多播灵活性

**理由**: `SharedSubject` 是 rxrust 原生多播原语；`Arc<Mutex<HashMap>>` 是 Rust 标准共享模式。模块加载后广播 `ModuleEvent::Loaded`，所有等待该模块的 Eval 节点无需轮询。

### Decision 6: 变量作用域 — 原子计数器 + 父链映射

**选择**: `EvalContext` 持有 `counter: ScopeCounter`（Arc<AtomicU64>），`child_scope()` 调用 `counter.allocate()` 生成全局唯一递增 u64 scope_id。作用域层级关系通过 `CompilerBus::register_parent(child_id, parent_id)` 记录，变量查找通过 `get_var(ctx, name)` 沿 parent_map 向上遍历。

**替代方案**:
- (A) `scope_id = parent_id * 1000 + idx` 算术派生 — 被否决，溢出风险且无法支持动态嵌套深度
- (B) 栈式 Environment (Rc<RefCell>) — 被否决，GC 思维
- (C) 静态作用域链 — 复杂度高

**理由**: 原子计数器避免锁竞争，全局唯一且支持任意嵌套深度；parent_map 在 Bus 中集中管理，支持跨上下文的变量查找。

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

| Risk | 影响 | Mitigation | Status |
|------|------|------------|--------|
| CompilerBus 锁竞争 | 变量/set_var 多线程争用 | 类型拆分（Scope/Mixin/Function 各自 Mutex） | ✅ 已化解 |
| Parser 状态持有 tokens Vec | 内存消耗 | tokens 在 Parser 消费后即释放 | ✅ 已内置 |
| 选择器嵌套深度 N 导致 O(N²) 输出 | 极端场景性能 | `expand_block_contents` EnterRule/LeaveRule frame 栈 + |frame.chain| 前缀追加 | ✅ 已缓解 |
| `@media` + `@supports` 多参 | 参数传播 | 表达式独立求值 | ✅ 已实现 |
| 单文件 ≤ 500 行 | parser/eval 拆分 | parser = mod.rs + state.rs；eval = mod.rs + expr.rs + builtin.rs | ✅ 已拆分 |
| Bootstrap dist 全量对齐 | 扫描遗漏 | 当前 coverage 1.25%（67/5342 行匹配），6 大根因：(B6) 缩进 double-counting (B7) 选择器缺空格 (B8) !important 未处理（651 行） (B9) 厂家前缀缺失 (B10) Maps 格式错误 (B11) 复杂 mixin 展开不全（@media 差 84 条） | ⚠️ 根因已定位，修复进行中 |

## Recent Bugfixes / Lessons Learned

### B1: Null 值泄漏
- **问题**：SCSS 中 `null` 是占位符（Bootstrap 中大量使用 `!default` 变量），不应输出到 CSS
- **修复**：`value_to_string` 返回 `"null"` 时跳过 StyleDecl 求值；prop_name 含 `"null"` 也跳过
- **数据**：null 泄漏行从 439 降到 14（-97%）

### B2: Spaced var() 函数
- **问题**：`var(-- bs-name)` 空格被序列化输出，导致 CSS 无效
- **修复**：在 var 函数计算前将属性名中的空格剥离

### B3: HashId 颜色解析
- **问题**：parse_atom 缺少 `Token::HashId` 分支，`#ffffff` 颜色字面值无法解析
- **修复**：在 parse_atom 中新增 `Token::HashId(s)` 分支调用 parse_hex_color

### B4: 逗号分隔列表变量
- **问题**：`$a: x, y, z` 只解析第一个元素，导致 `@each` 循环只迭代一次
- **修复**：新增 `parse_list_or_expr` 函数替代 `parse_value` 用于变量声明，正确处理逗号分隔列表

### B5: 工具类生成
- **状态**：Bootstrap 工具类（.col-md-* 等）部分选择器含未解析 `#{$name}` — resolve_selector 需支持 `#{}` 插值内的 `$var$
- **根因**：lexer 在 `#{` 时将变量名存入 selector 为 `$var` 形式（parse_selector InterpolationStart handler），但 resolve_selector `$var$ 查找在某些作用域链中失败

### B6: Decl 缩进 double-counting（影响 ALL 行）
- **问题**：`format_expanded` 中 `depth` 每层 +2 而非 +1，根节点 decl 从 depth=2 开始（应为 depth=1），导致 decl 缩进 8 空格（应为 4 空格）
- **证据**：输出中所有 decl `        prop: val;` (8 空格) vs reference `    prop: val;` (4 空格)
- **影响**：覆盖率直接受损 ~50%（所有 decl 行均不匹配）

### B7: 选择器缺失空格 before `{`（影响 ALL 行）
- **问题**：`.selector{` 而非 `.selector {`，所有含 `{` 的行格式错误
- **证据**：reference `.a {` vs output `.a{`
- **影响**：所有规则选择器行均不匹配

### B8: `!important` 标记未处理（651 唯一受影响行）
- **问题**：`prop: val !important;` 未解析，导致 decl 丢失
- **证据**：Bootstrap utility classes 中 `!important` 密集出现
- **影响**：651 行缺失

### B9: 厂家前缀缺失（-webkit-, -moz-, -ms-）
- **问题**：`_mixins.scss` 中 `webkit-prefixer` 等 mixin 未完整展开
- **证据**：output @media 规则 25 vs reference 109
- **影响**：transition/transform 相关 decl 缺失前缀变体

### B10: Sass Maps 格式输出错误
- **问题**：map 值在 CSS decl 中输出为 Sass 内部格式而非 `null`
- **影响**：部分 helper 区块 decl 值格式错误

### B11: 复杂 Mixin 展开不全
- **问题**：`media-breakpoint-up` 调用链未完整展开为 @media 规则
- **证据**：output @media 25 vs reference 109（差 84 条）
- **影响**：responsive 规则大量缺失

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

