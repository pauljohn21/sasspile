## ADDED Requirements

### Requirement: 编译错误作为流值继续传播（Infallible 管道）
编译器管道的 Observable 错误类型 MUST 为 `std::convert::Infallible`。所有可预见的编译错误（语法错误、未定义变量、类型不匹配等）MUST 被封装进 `Result<T, CompileError>` 枚举作为流的正常元素继续传播，不得作为 `error` 事件终止 Observable。

#### Scenario: 遇到语法错误
- **WHEN** 输入为 `"a { color: }"`（缺少值）
- **THEN** MUST 产出 `Result::Err(CompileError::MissingValue { ... })` 作为 Observable 的下一个元素，管道继续运行；MUST NOT 调用 `on_error` 或终止管道

#### Scenario: 编译多文件时单个文件失败
- **WHEN** 流里即使多个输入，其中一个产生语法错误
- **THEN** Observable MUST 继续处理后续元素失败的 case，不可终止

#### Scenario: 用户使用了未定义变量
- **WHEN** 输入为 `"a { color: $undefined; }"`
- **THEN** MUST 返回 `Result::Err(CompileError::UndefinedVariable { name: "$undefined" })`

### Requirement: 全部管道签名必须反映错误类型
四个 stage 的函数签名 MUST 将 `Result<T, CompileError>` 纳入流的 Item 类型，或使用明确的错误枚举作为第二参数。错误类型的变化 MUST 被用户显式可观测。

#### Scenario: tokenize 处理错误字符
- **WHEN** 遇到 UTF-8 非法字节或不识别字符
- **THEN** MUST 返回 `Token::Error(CompileError::InvalidInput { byte })`，或包装为 `Result<Token, CompileError>` 作为 Token Item

#### Scenario: evaluate 处理未知的 @-规则
- **WHEN** 遇到 sass-spec 之外的不识别指令
- **THEN** MUST 返回 Error item 而非 panic

### Requirement: 错误类型命名和有意义的上下文
`CompileError` 必须携带尽可能多的上下文信息：源文件路径、起止行列、错误码短语（可选 M:N 行号）、stage 标识。错误 MUST 能格式化为人类可读字符串，便于记录到 tracing 系统。

#### Scenario: 报告语法错误
- **WHEN** 出错
- **THEN** 错误信息 MUST 类似 `compile error: missing value for declaration at 1:10 in input.scss (stage=evaluate)`

#### Scenario: tracing 集成
- **WHEN** 使用了 tracing span
- **THEN** span MUST 打印错误字段：`error = compile_error.to_string()`
