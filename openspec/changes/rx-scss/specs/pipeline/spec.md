# Spec Delta

## Purpose

通过 `CompileBuilder` 将 Lexer → Parser → Eval → Serializer 组装为端到端 Observable 管道，提供 `from_string` / `from_path` 公共 API。所有组件通过 Builder 的 `.build()` 方法组装。

## ADDED Requirements

### Requirement: CompileBuilder 组装
`CompileBuilder::build(source)` SHALL 串联：Lexer → Parser → Eval → Serializer 全阶段。每阶段通过 `flat_map` / `box_it()` 连接成 SharedBoxedObservable 链。Builder 的字段（`syntax`, `serialize_style`, `scheduler_config`, `bus`, `include_paths`）控制各阶段行为。

#### Scenario: End-to-end pipeline for simple rule
- **WHEN** `CompileBuilder::new().build("a { color: red; }")` 被调用
- **THEN** SHALL 返回 Observable 产出 CSS 字符串

#### Scenario: Pipeline with compressed style
- **WHEN** `CompileBuilder::new().serialize_style(OutputStyle::Compressed)`
- **THEN** SHALL 输出 `"a{color:red}"`

### Requirement: 调度策略注入
`.scheduler(config)` 链式方法 SHALL 接受 `SchedulerConfig` 控制 eval 阶段线程调度。默认使用 `Shared::current_thread()`。

#### Scenario: Current thread default
- **WHEN** 不指定 SchedulerConfig
- **THEN** 全部求值 SHALL 在 subscribe 调用者同一线程执行

### Requirement: from_string 公共 API
`from_string(source: &str, options: &Options) -> Result<String, CompileError>` SHALL 调用 `CompileBuilder::new()` 配置后 + `build` + 同步收集。空输入返回 `Ok(String::new())`。

#### Scenario: Empty input
- **WHEN** from_string("", &Options::default())
- **THEN** SHALL 返回 Ok("")

#### Scenario: Valid SCSS
- **WHEN** from_string("a { color: red; }", &Options::default())
- **THEN** SHALL 返回 Ok("@charset \"UTF-8\";\na {\n  color: red;\n}\n")

### Requirement: from_path 公共 API
`from_path<F: Fs>(path: &Path, options: &Options, fs: &F) -> Result<String, CompileError>` SHALL 通过 `fs.read_to_string(path)` 读取，内部调用 `from_string`。IO 错误包装为 `CompileError::Io`。

#### Scenario: Valid file compilation
- **WHEN** from_path 读取包含 "a { color: red; }" 的路径
- **THEN** SHALL 返回 Ok 包含 CSS 输出

#### Scenario: I/O error
- **WHEN** from_path 读取不存在的路径
- **THEN** SHALL 返回 Err(CompileError::Io(...))

### Requirement: 错误传播
管线内部任阶段的错误 SHALL 通过 `CompileError` 传播到 API 层。Lexer/Parser/Evaluator 各自错误类型 SHALL 统一转换。

#### Scenario: Parse error propagation
- **WHEN** from_string 遇到语法错误
- **THEN** SHALL 返回 Err(CompileError::Parse(...))

### Requirement: Filesystem 注入
`from_path` SHALL 使用 `Fs` trait 抽象，不直接调用 `std::fs`。测试可以通过 mock `Fs` 实现编译测试。

#### Scenario: Mock Fs for testing
- **WHEN** Test 使用 mock Fs 返回 "a { color: red; }"
- **THEN** from_path SHALL 输出正确 CSS

### Requirement: 同步收集终止
`collect_stream(stream: SharedBoxedObservable<String>) -> String` SHALL 同步收集所有 chunks 后用 join("") 拼接。此函数 SHALL 阻塞直到流完成。

#### Scenario: Multiple chunks
- **WHEN** 流发射 "a{" 和 "color:red}" 两个 chunks
- **THEN** SHALL 返回 "a{color:red}"

