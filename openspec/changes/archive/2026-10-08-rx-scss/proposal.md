# Proposal

## Why

目前 Rust 生态缺少一个响应式风格的 SCSS 编译器：Grass 使用 `Rc<RefCell<Environment>>` + 递归下降 Visitor 模式，不符合零 GC 目标。rx-scss 用 Rust 所有权语义 + rxrust Observable 模式从零构建完整 SCSS 编译器——Shared 多线程调度加速编译，SharedSubject 多播实现模块/变量/事件分发，Observable 数据流贯穿 Lexer → Parser → Eval → Serializer 全阶段。目标是将 Bootstrap 5.3.x 的编译覆盖率提升至 99%+。

## What Changes

- **新增 `rx-scss`  crate** (`/Users/panglijun/rust/rx-scss/`) — 独立的响应式 SCSS 编译器 crate，与 Lightforger 管解耦，直接对标 sass-spec 规则集
- **构造器模式 (Builder)** — 引入 `CompileBuilder` 作为核心构造器，提供链式 API：`.syntax(InputSyntax::Scss).include_path(path).scheduler(config).serialize_style(Expanded).compile(source)`。所有阶段（Lexer/Parser/Eval/Serializer）通过 Builder 的字段配置，实现管线组件的可替换和可扩展
- **新增 Lexer 模块** — 用 `Shared::create` + `scan(LexerState)` 实现流式词法分析，覆盖 SCSS/indented-Sass/CSS 三种语法模式，输出 `Shared Observable<Token>` 流
- **新增 Parser 模块** — Pratt 优先级 climbing 解析器（`mod.rs` + `state.rs`），完整支持 SCSS 语法（变量声明、选择器嵌套、插值 `#{}`、父引用 `&`、@-rules、表达式运算、逗号分隔列表）
- **新增 Eval 模块** — 工作队列 + 事件流架构（`mod.rs` + `expr.rs` + `builtin.rs`），支持所有指令（@if/@for/@each/@while/@mixin/@include/@function/@return/@media/@supports/@warn/@debug），内置函数覆盖 sass:color/math/string/list/map/selector 模块
- **CompilerBus 多播总线** — 三个 `SharedSubject` 通道，Arc<HashMap> 注册表，parent_map 原子计数器管理作用域层级
- **新增值系统** — `Value` enum（Number/String/Color/List/Map/Bool/Null）+ `Display` trait + `truthy()` 语义
- **新增 Serializer 模块** — Expanded / Compressed / Nested 三种 OutputStyle
- **新增 Pipeline 公共 API** — `from_string(source, options)` / `from_path(path, options)` 两个入口
- **EvalContext Runtime** — 持有的 scope_counter: ScopeCounter（Arc<AtomicU64>），child_scope() 递增派生
- **Filesystem 抽象** — `Fs` trait（`read_to_string`），生产用 `RealFs`，测试可 mock
- **完整测试套件** — tests/ 目录下 8 个测试文件 + bootstrap_test.rs
- **Bootstrap 5.3.x 端到端验证** — `bootstrap_test.rs` 编译 Bootstrap SCSS 并与 dist CSS 比对（当前 coverage 1.25%，根因：序列化缩进 bug + `!important` 缺失 + 厂家前缀 + Sass Maps 格式; 完整诊断进行中）

## Capabilities

### New Capabilities

- `lexer`: 词法分析器 — 将 SCSS 源码转为 Token 流（Vec<Token>），覆盖所有 token 类型（插值、@-rules、操作符、字符串转义、HashId）
- `parser`: Pratt 语法分析器 — Pratt climbing 算法 + ParserState（mod.rs + state.rs），输出 Vec<AstNode>，覆盖 SCSS 全部语法
- `eval`: 求值器 — 工作队列 + 事件流 + scan 累积（mod.rs + expr.rs + builtin.rs），CompilerBus + parent_map 作用域链
- `serialize`: CSS 序列化 — Expanded / Compressed / Nested 三种 OutputStyle
- `pipeline`: 公共 API — `from_string` / `from_path` 同步入口，内部 CompileBuilder 组装管线
- `multicast-bus`: 多播事件总线 — SharedSubject 三通道 + parent_map 原子计数器
- `builder-pattern`: 构造器模式 — 链式 setter + `build(source)` 同步编译
- `value-system`: Sass 值系统 — Number/String/Color/List/Map/Bool/Null 七种类型 + Display + truthy
- `runtime`: 运行时上下文 — EvalContext（bus + ScopeCounter），child_scope 递增派生
- `bootstrap-validation`: Bootstrap 5.3.x 端到端验证 — 编译 SCSS 与 dist CSS 比对（当前 coverage 1.25%，6 大根因已定位，进行中）

### Modified Capabilities

- _（无既有 capability 需要修改——rx-scss 是全新 crate）_

## Impact

- **代码**: 新增 `/Users/panglijun/rust/rx-scss/` 目录，内含 `src/`（lib.rs + 9 个子模块：bus/types/runtime/lexer/parser/eval/serialize/pipeline/builder）和 `tests/`（8 个测试文件 + bootstrap_test.rs）。单文件严格 ≤ 500 行
- **依赖**: `rxrust = "1.0.0-rc.5"` + `tracing = "0.1"`；dev-dependency `tracing-subscriber`（测试用）+ `similar`（Bootstrap diff 输出用）
- **Workspace 影响**: rx-scss 是独立 crate 独立项目，不加入任何 workspace，不依赖任何外部内部 crate
- **API**: 公共 API 为 `from_string` / `from_path` / `Options` / `CompileError` / `CompileBuilder`
- **测试策略**: 所有测试在 tests/ 目录；Bootstrap 验证覆盖率 1.25%（进行中，6 大根因已定位，详见 design.md §Recent Bugfixes）
