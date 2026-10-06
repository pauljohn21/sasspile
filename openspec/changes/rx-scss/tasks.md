# Tasks

## 1. Setup — Crate 骨架与依赖

- [ ] 1.1 创建 `/Users/panglijun/rust/rx-scss/Cargo.toml` 配置 `edition=2024`、`rust-version=1.99`、依赖 `rxrust = "1.0.0-rc.5"` 和 `tracing = "0.1"`、dev-dep `tracing-subscriber`。验证：`cargo check` 通过
- [ ] 1.2 创建 `src/lib.rs` 导出 `pub mod bus; pub mod types; pub mod runtime; pub mod lexer; pub mod parser; pub mod eval; pub mod serialize; pub mod pipeline; pub mod builder;` 和 `pub use pipeline::{from_string, from_path}; pub use serialize::Options; pub use builder::CompileBuilder;`。验证：`cargo build` 通过（空模块编译）
- [ ] 1.3 创建 `src/types.rs` 定义 `Token` enum（40+ 变体）、`Value` enum（7 种+Display）、`CssStmt` enum、`AstNode` enum（20+ 变体）、`BinOp`/`UnaryOp`、`Param` 结构体、类型别名 `TokenStream`/`AstStream`/`CssStream`。验证：`#[test] token_coverage` 在 `tests/types_test.rs` 通过
- [ ] 1.4 创建 `tests/` 目录和 `tests/integration_test.rs` 为空文件占位。在 `Cargo.toml` dev-dependencies 中预加 `similar = \"2\"`（Bootstrap diff 输出用）。验证：目录存在

## 2. Value System + Multicast Bus + Runtime

- [ ] 2.1 实现 `src/bus.rs`: `CompilerBus` 结构体（三个 `SharedSubject` + `Arc<Mutex<BusInner>>`）、`VarEvent`/`ModuleEvent`/`ScopeEvent` 枚举、`ScopeKind` 枚举、`MixinDef`/`FnDef`/`ModuleDef` 结构体。`set_var/get_var/register_mixin/lookup_mixin/register_fn/lookup_fn/register_module/lookup_module` 方法。验证：`tests/bus_test.rs` 测试多播和线程安全通过
- [ ] 2.2 实现 `src/runtime.rs`: `EvalContext`（bus + scope_id）、`child_scope(local_idx)` 派生、`var(name)`/`bind_var(name, value)` 方法、`create_runtime()` 函数。验证：`tests/runtime_test.rs` 测试 child_scope 算术派生和变量作用域隔离通过
- [ ] 2.3 为 `Value` 实现 `Display` trait（Number/String/Color RGBA/List/Map/Bool/Null 七种）。验证：`tests/value_test.rs` 每种类型 Display 输出断言通过
- [ ] 2.4 为 `CssStmt` 实现 `is_invisible()` 方法。验证：空 inner 的 Rule 返回 true

## 3. Lexer — 流式词法分析

- [ ] 3.1 创建 `src/lexer/` 目录结构：`mod.rs` + `state.rs`（LexerState）+ `scan.rs`（scan 闭包逻辑）。单文件 ≤ 500 行约束。验证：目录结构正确
- [ ] 3.2 实现 `LexerState`：维护 `pos: u32`、`in_double_quote: bool`、`in_single_quote: bool`、`interp_depth: u32`、`in_comment: bool`、`buf: String`。`feed(ch: char) -> Option<Token>` 方法。验证：`tests/lexer_test.rs` 基本 token 产出通过
- [ ] 3.3 实现 `scan.rs`：处理字符串上下文（`\"` 开始/结束字符串）、转义字符 `\\n` `\\"`、插值 `#{` 增加 depth、`}` 减少 depth、`//` 单行注释跳过、`/* */` 多行注释跳过。验证：字符串转义 + 插值混合测试通过
- [ ] 3.4 实现 `@`-rules 识别（`@media`/`@supports`/`@if`/`@for`/`@each`/`@@while`/`@mixin`/`@include`/`@function`/`@return`/`@use`/`@forward`/`@extend`/`@warn`/`@debug`）。验证：@-rule 识别测试通过
- [ ] 3.5 实现数字和单位识别（`16px`、`1.5em`、`50%`、`-3rem`）。验证：带单位数字产出测试通过
- [ ] 3.6 实现操作符双字符识别（`==`、`!=`、`<=`、`>=`）。验证：比较操作符不拆分为单字符通过

## 4. Parser — 增量式语法分析

- [ ] 4.1 创建 `src/parser/` 目录：`mod.rs` + `state.rs`（ParserState）+ `at_rules.rs`（@规则解析）+ `selectors.rs`（选择器解析）+ `values.rs`（值类型解析）。验证：目录结构正确
- [ ] 4.2 实现 `ParserState`：维护 `tokens: Vec<Token>` peek buffer、`brace_stack: Vec<char>`、`scope_id_counter: u64`、`errors: Vec<ParseError>`。`peek()`/`peek_n()`/`next()`/`expect()` 方法。验证：基本 peek/next 操作通过
- [ ] 4.3 实现 `scan.rs` 增量解析核心：每次 scan 接收 Token 并尝试解析完整语句（VariableDecl、StyleDecl、Rule、@-rule）；产出 `Vec<SassAstNode>`。验证：单个声明解析通过
- [ ] 4.4 实现 values.rs：数字、字符串、颜色（十六进制+命名）、布尔、null、列表、Maps、函数调用、算术表达式解析。验证：每种值类型的解析测试通过
- [ ] 4.5 实现 selectors.rs：类选择器 `.class`、ID `#id`、元素 `div`、伪类 `:hover`、伪元素 `::before`、属性 `[attr]`、组合 `>` `+` `~`、插值 `#{$var}`、父引用 `&`。验证：复合选择器 `.a > .b:hover` 解析通过
- [ ] 4.6 实现 at_rules.rs：`@media {<query>}`、`@supports {<query>}`、`@if <cond> {} @else {}`、`@for $var from N to/through M {}`、`@each $var in list {}`、`@while <cond> {}`、`@mixin name($args) {}`、`@include name(args)`、`@function name($args) {}`、`@return <expr>`、`@use "path"`、`@forward "path"`、`@warn <expr>`、`@debug <expr>`。验证：每种 @-rule 的解析测试通过
- [ ] 4.7 实现错误恢复：遇到无法识别语法时从同步点（`;` 或 `}`）继续。未闭合分隔符检测。验证：错误恢复测试 + ParseError 包含正确 `pos`

## 5. Evaluator — 响应式指令算子

- [ ] 5.1 创建 `src/eval/` 目录：`mod.rs` + `ops.rs`（指令算子映射表）+ `cond.rs`（条件求值辅助）。验证：目录结构正确
- [ ] 5.2 实现 `dispatch_op` 函数：`fn dispatch_op(node: AstNode, ctx: Arc<EvalContext>) -> AstStream` 根据 AstNode variant 返回对应的 `Shared::create` 算子。支持全部变体。验证：dispatch 映射覆盖测试通过
- [ ] 5.3 实现 `@if/@else` 算子：cond 求值为布尔，选择分支，`child_scope + from_iter + flat_map` 递归展开。验证：正/反条件测试通过
- [ ] 5.4 实现 `@for` 算子：计算范围，inclusive 区分 through/to，每次迭代 `child_scope(idx).bind_var + 递归展开 + collect`。验证：inclusive/exclusive range 测试通过
- [ ] 5.5 实现 `@each` 算子：list 求值，绑定 var，递归展开。验证：单变量 + 多变量 each 测试通过
- [ ] 5.6 实现 `@while` 算子：cond 求值，MAX_WHILE_ITERATIONS 安全阀。验证：计数器循环 + 无限循环保护测试通过
- [ ] 5.7 实现 `@mixin` 注册算子：register_mixin + 空流返回。验证：MixinDef 注册到 bus 后 lookup 成功
- [ ] 5.8 实现 `@include` 算子：lookup_mixin，child_scope 绑定参数，递归展开 body，未找到 @warn。验证：完整 mixin 定义+调用 CSS 输出测试通过
- [ ] 5.9 实现 `@function/@return` 算子：register_fn + Return 求值。函数调用通过 `dispatch_op` 触发 FnDef body 求值。验证：自定义函数定义+调用返回值测试通过
- [ ] 5.10 实现 `@media/@supports` 算子：child_scope 递归展开 inner + collect 包装 CssStmt。验证：嵌套 @media CSS 输出测试通过
- [ ] 5.11 实现 `@warn/@debug` 算子：tracing event + 空流。验证：tracing 输出捕获测试通过
- [ ] 5.12 实现 `eval_stream(ast_stream, ctx) -> CssStream`：通过 `flat_map(dispatch_op)` 连接 ast 流到 css 流。验证：端到端 eval 管线测试通过

## 6. Serializer + Pipeline

- [ ] 6.1 创建 `src/serialize/` 目录：`mod.rs` + `format.rs`（格式化引擎）。验证：目录结构正确
- [ ] 6.2 实现 `format_compressed(stmt) -> String`：最小空白，`prop:val;`，`sel{inner}`，`@media q{inner}`。验证：压缩格式断言测试通过
- [ ] 6.3 实现 `format_expanded(stmt, indent_width, depth) -> String`：缩进、换行、正确 depth 管理。验证：缩进深度正确性断言测试通过
- [ ] 6.4 实现 `format_nested(stmt, indent_width, depth) -> String`：同 expanded（后续可区分）。验证：与 expanded 对齐
- [ ] 6.5 实现 `serialize(stream, options) -> Observable<String>`：`collect::<Vec<_>>.flat_map(from_iter + map(format))`。验证：流输出与期望字符完全匹配
- [ ] 6.6 实现 `serialize_to_string` 便捷方法，处理 `@charset "UTF-8";` 头部和 `suppress_charset` 标志。验证：默认包含 @charset，suppress_charset=true 不包含
- [ ] 6.7 实现 `src/pipeline.rs`：`from_string`/`from_path`/`Fs` trait/`RealFs`。内部使用 `CompileBuilder::new().build(source)` 组装管线。验证：`from_string("a { color: red; }", &Options::default())` 返回正确 CSS
- [ ] 6.8 实现 `collect_stream(stream) -> String`：`Arc<Mutex<Vec<String>>>` 同步收集 + `join("")`。验证：多 chunk 拼接测试通过

## 7. Builder Pattern (构造器模式)

- [ ] 7.1 创建 `src/builder.rs`：`CompileBuilder` 结构体定义（字段：`syntax: InputSyntax`、`include_paths: Vec<PathBuf>`、`serialize_style: OutputStyle`、`scheduler_config: Option<SchedulerConfig>`、`bus: Option<Arc<CompilerBus>>`）、`SchedulerConfig` enum（`SingleThread` / `ThreadPool(usize)`）。验证：`cargo check` 通过
- [ ] 7.2 实现 `Default` impl：`CompileBuilder::new()` 默认 `syntax=Scss, serialize_style=Expanded, include_paths=[], scheduler_config=None, bus=None`。验证：`CompileBuilder::default()` 创建成功
- [ ] 7.3 实现链式 setter：`.syntax(s)` / `.include_path(p)` / `.serialize_style(s)` / `.scheduler(c)` / `.bus(b)`，每个 setter 消费 self 返回 Self。验证：链式调用 `CompileBuilder::new().syntax(InputSyntax::Scss).serialize_style(OutputStyle::Compressed)` 编译通过
- [ ] 7.4 实现 `.build(source: &str) -> CssOutputStream` 方法：内部串联 Lexer → Parser → Eval → Serializer 管线。根据 `scheduler_config` 调用 `observe_on` 注入调度器；根据 `serialize_style` 选择格式化器；根据 `bus` 字段或创建默认 bus。分发函数 `dispatch_op` 作为独立函数定义。验证：`CompileBuilder::new().build(".a{color:red}")` 编译并通过 collect_stream 产出 CSS 字符串
- [ ] 7.5 实现 `.build_from_path(path: &str) -> Result<CssOutputStream, CompileError>` 方法：读取文件后委托 `.build()`。使用 `Fs` trait 读取。验证：从临时文件读取 SCSS 并编译
- [ ] 7.6 确保 `CompileBuilder` 满足 `Send + Sync + 'static` 约束。验证：`fn assert_send<T: Send + Sync + 'static>() {}` 编译通过
- [ ] 7.7 编写 `tests/builder_test.rs`：覆盖所有 setter 默认值、链式调用、scheduler 注入、bus 注入、include_path 编译。验证：`cargo test --test builder_test` 全部通过

## 8. Integration + Tests

- [ ] 8.1 编写 `tests/integration_test.rs`：端到端测试覆盖——嵌套规则、变量+插值、`@if/@else`、`@for`、`@each`、`@mixin`/`@include`、`@media`、Expanded/Compressed 输出。验证：`cargo test --test integration_test` 全部通过
- [ ] 8.2 编写 `tests/lexer_test.rs`：覆盖 SCSS/CSS 模式切换、@-rules、插值、字符串转义、注释跳过、错误位置。验证：`cargo test --test lexer_test` 全部通过
- [ ] 8.3 编写 `tests/parser_test.rs`：覆盖所有值类型、选择器、@-rules、错误恢复、未闭合分隔符。验证：`cargo test --test parser_test` 全部通过
- [ ] 8.4 编写 `tests/eval_test.rs`：覆盖所有指令算子、变量作用域隔离、child_scope 继承。验证：`cargo test --test eval_test` 全部通过
- [ ] 8.5 编写 `tests/pipeline_test.rs`：覆盖 `from_string`/`from_path`、空输入、错误传播、Fs trait 注入。验证：`cargo test --test pipeline_test` 全部通过
- [ ] 8.6 编写 `tests/bus_test.rs`：覆盖多播、Arc 并发读写、registry 线程安全。验证：`cargo test --test bus_test` 全部通过
- [ ] 8.7 编写 `tests/builder_test.rs`：覆盖 CompileBuilder 所有 setter + 默认实现 + mock bus 注入。验证：`cargo test --test builder_test` 全部通过
- [ ] 8.8 全量验证：`cargo test` 所有测试套件通过、`cargo clippy -- -D warnings` 无警告、`cargo doc` 文档生成成功。验证：CI 命令全部零退出

## 9. Bootstrap 全量验证

- [ ] 9.1 创建 `tests/bootstrap_test.rs`：定义 `assert_css_eq(actual, expected)` 逐字节比对辅助函数；使用 `CompileBuilder::new().include_path("bootstrap/scss/")` 编译。用 `#[ignore]` 标记。验证：文件已创建，`cargo test --test bootstrap_test -- --list` 列出被 ignore 的测试
- [ ] 9.2 确认 `.gitmodules` 中 bootstrap submodule 配置 `shallow = true`。CI/本地执行 `git submodule update --init --depth 1 bootstrap` 后 `bootstrap/scss/bootstrap.scss` 存在。验证：`ls bootstrap/scss/bootstrap.scss` 存在。`BOOTSTRAP_FORCE_REFRESH=1` 时 `rm -rf bootstrap && git submodule update --init --depth 1 bootstrap`
- [ ] 9.3 准备 fixtures 文件 `tests/fixtures/bootstrap-5.3.x-dist.css`（Expanded 模式期望输出）和 `tests/fixtures/bootstrap-5.3.x-dist.min.css`（Compressed 模式期望输出）。可通过 Bootstrap 官方 GitHub release file 或 dart-sass 编译产出。验证：fixtures 目录存在且文件 > 100KB
- [ ] 9.4 实现 `compile_bootstrap_full()` 测试：使用 `CompileBuilder::new().include_path("bootstrap/scss/").build("bootstrap/scss/bootstrap.scss")`，Expanded 模式。将产物与 `bootstrap-5.3.x-dist.css` 逐字节比对。失败时将实际输出 dump 到 `target/bootstrap-output-expanded.css`。验证：`cargo test --test bootstrap_test compile_bootstrap_full -- --ignored` 通过
- [ ] 9.5 实现 `compile_bootstrap_compressed()` 测试：同上但使用 Compressed 模式。产物与 `bootstrap-5.3.x-dist.min.css`（trim 末尾空白）比对。失败 dump 到 `target/bootstrap-compressed.css`。验证：`cargo test --test bootstrap_test compile_bootstrap_compressed -- --ignored` 通过
- [ ] 9.6 实现 diff 上下文辅助：当比对失败时输出 `diff` 格式的具体差异位置。使用 `similar` crate 或类似工具。验证：在 fixtures 被故意篡改 1 字节后测试能打印差异行号
- [ ] 9.7 实现 IncludePath 注册测试：通过 `.include_path("bootstrap/scss/")` 注册作为 IncludePath，验证 `@use \"sass:color\"` 和 `@import \"variables\"` 都能正确解析。验证：Bootstrap 内部模块导入解析通过
- [ ] 9.8 实现模块覆盖度断言：编译完成后 > 100,000 字节输出、包含关键选择器 (`btn`、`container`、`modal`、`navbar`)。验证：编译产物断言通过
- [ ] 9.9 Bootstrap 测试通过全量编译：完整 Bootstrap 5.3.x SCSS（含所有 mixin/@use/循环/条件/内置函数）编译产物与官方 dist CSS 逐字节一致。验证：`cargo test --test bootstrap_test -- --ignored` 零失败

