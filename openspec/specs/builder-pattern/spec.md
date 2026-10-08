# Builder Pattern Specification

## Purpose

构造器模式 (Builder Pattern) 定义 `CompileBuilder` 结构体作为核心构造器，通过链式 API 配置和组装 SCSS 编译管线。所有阶段（Lexer/Parser/Eval/Serializer）通过 Builder 字段配置，实现管线组件的可替换、可扩展、可测试。

## Requirements

### Requirement: CompileBuilder 结构体
`CompileBuilder` SHALL 定义为包含以下字段的结构体：(1) `syntax: InputSyntax`（默认 `Scss`）；(2) `include_paths: Vec<PathBuf>`（默认空）；(3) `serialize_style: OutputStyle`（默认 `Expanded`）；(4) `scheduler_config: Option<SchedulerConfig>`（默认 None）；(5) `bus: Option<Arc<CompilerBus>>`（默认 None）。结构体 SHALL 实现 `Default` trait。

#### Scenario: Default builder creation
- **WHEN** `CompileBuilder::new()` 被调用
- **THEN** SHALL 返回 `syntax=Scss, serialize_style=Expanded, include_paths=[], scheduler_config=None, bus=None` 的实例

#### Scenario: Default trait
- **WHEN** `CompileBuilder::default()` 被调用
- **THEN** SHALL 返回与 `CompileBuilder::new()` 等价的实例

### Requirement: 链式 Setter API
`CompileBuilder` SHALL 提供以下链式 setter 方法，每个方法 SHALL 消费 `self` 返回 `Self`：
- `.syntax(syntax: InputSyntax) -> Self`
- `.include_path(path: impl Into<PathBuf>) -> Self`（追加路径）
- `.include_paths(paths: Vec<PathBuf>) -> Self`（替换全部路径）
- `.serialize_style(style: OutputStyle) -> Self`
- `.scheduler(config: SchedulerConfig) -> Self`
- `.bus(bus: Arc<CompilerBus>) -> Self`

#### Scenario: Chain setters
- **WHEN** `CompileBuilder::new().syntax(InputSyntax::Scss).include_path("src/scss").serialize_style(OutputStyle::Compressed)` 被调用
- **THEN** SHALL 返回配置完成的可编译 Builder 实例

### Requirement: SchedulerConfig 枚举
`SchedulerConfig` SHALL 定义为 `SingleThread | ThreadPool(usize)` 枚举，用于配置 rxrust Observable 的 `observe_on` 调度策略。`ThreadPool(n)`  SHALL 表示 n 个工作线程。

#### Scenario: ThreadPool config
- **WHEN** 用户调用 `.scheduler(SchedulerConfig::ThreadPool(4))`
- **THEN** eval 阶段 SHALL 在 4 个工作线程上通过 `observe_on` 调度

### Requirement: Build 方法
`CompileBuilder` SHALL 提供 `build(self, source: &str) -> CssOutputStream` 方法。`build` SHALL 内部按以下顺序组装 Observable 管线：
1. 创建或复用 `Arc<CompilerBus>`
2. 用 `Shared::create` + `scan(LexerState)` 构建 Token Observable
3. 用 `scan(ParserState)` + `flat_map` 构建 AstNode Observable
4. 用 `flat_map(dispatch_op)` 构建 CssStmt Observable（eval 阶段）
5. 若 `scheduler_config` 为 `ThreadPool(n)`，eval 阶段 SHALL 调用 `observe_on(ThreadPool(n))`
6. 用 `collect + flat_map(format)` 构建 String Observable（serialize 阶段）

监听流 SHALL 在 `build` 返回后被订阅时才开始执行（冷 Observable 语义）。分发函数 SHALL 作为独立函数 `dispatch_op(node: AstNode, ctx: Arc<EvalContext>) -> AstStream` 定义。

#### Scenario: Build pipeline from string
- **WHEN** `CompileBuilder::new().build("a { color: red; }")` 被调用
- **THEN** SHALL 返回 `SharedBoxedObservable<String, Infallible>` 冷流

### Requirement: Build from path
`CompileBuilder` SHALL 提供 `build_from_path(self, path: impl AsRef<Path>) -> Result<CssOutputStream, CompileError>` 方法。内部 SHALL 读取文件内容为字符串，然后委托 `build(source)`。读取 SHALL 使用 `Fs` trait 以支持测试注入。

#### Scenario: Build from valid path
- **WHEN** `CompileBuilder::new().build_from_path("styles/app.scss")` 被调用且文件存在
- **THEN** SHALL 返回 Ok(CssOutputStream)

#### Scenario: Build from missing path
- **WHEN** 文件不存在
- **THEN** SHALL 返回 Err(CompileError::Io(...))

### Requirement: IncludePath 注册
`CompileBuilder::build` SHALL 将所有 `include_paths` 注册到 `Fs` 实现（生产用 `RealFs`），确保 `@use`/`@import` 能解析注册路径。 SHALL 遵循 Sass 模块解析规则：先查找相对路径，再遍历 `include_paths` 列表查找。

#### Scenario: Bootstrap import resolution
- **WHEN** `.include_path("bootstrap/scss/")` 后编译 `@import "variables"`
- **THEN** SHALL 在 `bootstrap/scss/_variables.scss` 查找到文件

### Requirement: Bus 注入
`CompileBuilder::build` SHALL 优先使用 `.bus()` 注入的 bus，若无则创建新的 `Arc<CompilerBus::default>()`。注入的 bus 允许测试中替换为 `BusTestHarness` 或共享全局 bus。

#### Scenario: Shared bus across compilations
- **WHEN** 两次 `compile` 调用使用同一 `.bus(my_bus.clone())`
- **THEN** 第二次 compile SHALL 能看到第一次注册的所有 mixin/function

### Requirement: Send + Sync 约束
`CompileBuilder` 及其所有字段类型 SHALL 满足 `Send + 'static` 约束，确保可在多线程 `Shared` 调度器中通过 `Arc<dyn ...>` 传递。`SchedulerConfig`、`InputSyntax`、`OutputStyle` 同样 SHALL 满足 `Send + Sync + 'static`。

#### Scenario: Builder used with ThreadPool scheduler
- **WHEN** Builder 被 `Arc::new` 包装后跨线程传递
- **THEN** 编译 SHALL 成功（满足 Send + Sync）

### Requirement: OutputStyle 枚举
`OutputStyle` SHALL 定义为 `Expanded | Compressed | Nested` 枚举。`Expanded` 使用 2 空格缩进；`Compressed` 去除所有不必要空白；`Nested` 与 Expanded 对齐（后续可区分）。序列化阶段 SHALL 根据此字段选择格式化器。

#### Scenario: Compressed output
- **WHEN** `.serialize_style(OutputStyle::Compressed)` 编译 `.a { color: red; }`
- **THEN** 输出 SHALL 为 `.a{color:red}` (无空格、无换行)

#### Scenario: Expanded output
- **WHEN** `.serialize_style(OutputStyle::Expanded)` 编译 `.a { color: red; }`
- **THEN** 输出 SHALL 包含换行和 2 空格缩进，如 `.a {\n  color: red;\n}\n`
