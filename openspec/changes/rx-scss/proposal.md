# Proposal

## Why

目前 Rust 生态缺少一个纯响应式风格的 SCSS 编译器：Grass 使用传统的 `Rc<RefCell<Environment>>` + 递归下降 Visitor 模式，无法利用多线程并行编译和响应式的增量更新能力；Lightforger 已有初步响应式骨架（Shared + CompilerBus），但缺少完整 Lexer/Parser，且 AbNode 语法覆盖不足。用户需要在不依赖 dart-sass 兼容性的前提下，以 sass-spec 为规则参考，用 Rust 所有权语义和 rxrust Observable 模式从零构建一个完整的 SCSS 编译器——共享多线程调度（Shared Scheduler）加速编译，多播（SharedSubject）实现模块/变量/事件的广播分发，Observable 数据流贯穿 Lexer → Parser → Eval → Serializer 全阶段。

## What Changes

- **新增 `rx-scss`  crate** (`/Users/panglijun/rust/rx-scss/`) — 独立的响应式 SCSS 编译器 crate，与 Lightforger 管解耦，直接对标 sass-spec 规则集
- **构造器模式 (Builder)** — 引入 `CompileBuilder` 作为核心构造器，提供链式 API：`.syntax(InputSyntax::Scss).include_path(path).scheduler(config).serialize_style(Expanded).compile(source)`。所有阶段（Lexer/Parser/Eval/Serializer）通过 Builder 的字段配置，实现管线组件的可替换和可扩展
- **新增 Lexer 模块** — 用 `Shared::create` + `scan(LexerState)` 实现流式词法分析，覆盖 SCSS/indented-Sass/CSS 三种语法模式，输出 `Shared Observable<Token>` 流
- **新增 Parser 模块** — 用 `Shared::create` + `scan(ParserState)` 实现增量式语法分析，产出 `Observable<SassAstNode>` 流，支持插值 `#{}`、父选择器 `&`、嵌套规则、`@`-rules 全部语法
- **新增 Lexer → Parser → Eval 全 Observable 管道** — 三个阶段通过 `flat_map` / `box_it()` 组成端到端 `SharedBoxedObservable` 链，每阶段都可在独立线程池上调度 (observe_on)
- **新增 `CompilerBus` 多播总线** — 三个 `SharedSubject` 通道（var_events / module_events / scope_events）实现多播；Mixin/Function/Module 注册表用 `Arc<Mutex<HashMap>>` 实现线程安全共享
- **新增 Evaluator 模块** — 每个指令（@if/@for/@each/@while/@mixin/@include/@function/@return）用 `Shared::create` 构建原生 rxrust 算子，接入调度/背压/取消系统，消除 `Box<dyn Fn>` + `Rc<RefCell>` 历史实现
- **新增值系统** — `Value` enum（Number/String/Color/List/Map/Bool/Null）+ `Display` trait 实现 Sass 值语义
- **新增 Serializer 模块** — `Observable<CssStmt>` → `String` 流转换，支持 Expanded / Compressed / Nested 三种 OutputStyle，用 `collect::<Vec<_>>` + `flat_map` 实现流式序列化
- **新增 Pipeline 公共 API** — `from_string(source, options)` / `from_path(path, options)` 两个入口，内部用 `CompileBuilder::new().build()` 组装全管线
- **新增 eval.rs Runtime** — `EvalContext` 不可变上下文（bus 引用 + scope_id），child_scope 通过 scope_id 算术派生（`parent * 1000 + idx`）；变量查找通过 `CompilerBus::get_var` 向上遍历
- **Filesystem 抽象** — `Fs` trait（`read_to_string`），生产用 `RealFs`，测试可 mock
- **完整测试套件** — tests/ 目录下按阶段划分：`lexer_test.rs`、`parser_test.rs`、`eval_test.rs`、`serialize_test.rs`、`pipeline_test.rs`、`bus_test.rs`、`integration_test.rs`
- **全量 Bootstrap 验证** — 使用 Bootstrap 5.3.x SCSS 源码作为端到端验收标准，`bootstrap_test.rs` 编译 Bootstrap 全部 SCSS 文件（约 130 个）并逐字节比对 dist CSS 输出

## Capabilities

### New Capabilities

- `lexer`: 流式词法分析器 — 将 SCSS 源码转为 `Shared Observable<Token>` 流，覆盖所有 SCSS token 类型（含插值、@-rules、操作符、字符串转义）
- `parser`: 增量式语法分析器 — 将 `Token` 流转为 `Observable<SassAstNode>` 流，完整表达 SCSS 语法树（变量声明、规则嵌套、@media、@supports、控制流、混入、函数、模块）
- `eval`: 响应式求值器 — 将 `AstNode` 流通过 `flat_map(dispatch_op)` 转为 `CssStmt` 流，每个指令作为独立 rxrust 算子，CompilerBus 多播变量/模块/作用域事件
- `serialize`: CSS 序列化器 — 将 `CssStmt` 流转为格式化的 CSS 字符串，支持 Expanded/Compressed/Nested 风格
- `pipeline`: 端到端编译管线 — 串联 Lexer → Parser → Eval → Serializer 为一条 Observable 管道，所有阶段 Shared 调度
- `multicast-bus`: 多播事件总线 — 通过 `SharedSubject` 实现变量绑定、模块加载、CSS 作用域的开闭事件的多播分发；Arc 注册表提供线程安全的 Mixin/Function/Module 存储
- `builder-pattern`: 构造器模式 — `CompileBuilder` 通过链式方法组装全管线，所有 Observable 组件通过 Builder 字段配置创建，支持 `.syntax()` / `.include_path()` / `.scheduler()` / `.serialize_style()` 链式调用
- `value-system`: Sass 值系统 — Number/String/Color/List/Map/Bool/Null 七种值类型，实现 Display/PartialEq/Clone，作为 Eval 阶段的运行时值
- `runtime`: 运行时上下文 — EvalContext（bus + scope_id 不可变对），提供 child_scope/var/bind_var 纯函数式 API
- `bootstrap-validation`: Bootstrap 5.3.x 全量端到端验证 — 编译 Bootstrap SCSS（含所有 mixin 嵌套、@use 模块系统、内置函数调用），逐字节比对 dist CSS 确认编译正确性

### Modified Capabilities

- _（无既有 capability 需要修改——rx-scss 是全新 crate）_

## Impact

- **代码**: 新增 `/Users/panglijun/rust/rx-scss/` 目录，内含 `src/`（lib.rs + 9 个子模块：bus/types/runtime/lexer/parser/eval/serialize/pipeline/builder）和 `tests/`（8 个测试文件 + bootstrap_test.rs）。单文件严格 ≤ 500 行
- **依赖**: `rxrust = "1.0.0-rc.5"` + `tracing = "0.1"`；dev-dependency `tracing-subscriber`（测试用）。Bootstrap 验证额外需要 `similar`（dev-dep，用于 diff 输出）。无其他外部依赖
- **Workspace 影响**: rx-scss 是独立crate独立项目（`/Users/panglijun/rust/rx-scss/`），不加入任何 workspace，不依赖 Lightforger / Grass 或其他任何内部 crate。所有类型（Token / Value / AstNode / CssStmt / CompileError）从零定义，Observable 调度器直接依赖 rxrust 官方 crate 而非 Lightforger 封装
- **API**: 公共 API 只有 `from_string` / `from_path` / `Options` / `CompileError` 四个导出项；内部模块通过 `pub(crate)` 可见性限制
- **测试策略**: 所有测试在 tests/ 目录（禁止 src/ 内联测试）；test 编译需 `tracing-subscriber` 初始化
- **Bootstrap 验证策略**: 通过项目 `.gitmodules` 浅拷贝引入 Bootstrap 5.3.x SCSS 到 `bootstrap/scss/` 目录（`shallow = true`, depth=1，节省磁盘和克隆时间），`tests/bootstrap_test.rs` 使用 `CompileBuilder::new().include_path("bootstrap/scss/")` 直接编译，与官方 dist `bootstrap.css` 逐字节比对
- **性能**: Shared 调度器在 observe_on 后将工作分布到线程池；multicast 确保模块只加载一次、多消费者共享
