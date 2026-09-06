## ADDED Requirements

### Requirement: 解析器路径禁止 unwrap()
所有 `src/css/selector_parser.rs`、`src/eval/value/calc_ast.rs`、`src/eval/value/calc_simplify.rs` 中的 `.unwrap()` 调用 SHALL 被替换为 `?` 错误传播或 `.expect("context message")`。

#### Scenario: EOF 时 take_ident 返回错误而非 panic
- **WHEN** selector_parser.rs::take_ident 调用 `self.chars.next()` 返回 None
- **THEN** 函数 SHALL 返回 `Err(SassError::Parse { expected: "ident", found: "EOF", pos })` 而非 panic

#### Scenario: calc_ast 解析数字时优雅失败
- **WHEN** calc_ast.rs::parse_number 调用 `self.advance()` 返回 None
- **THEN** 函数 SHALL 返回 `Err(SassError::Parse { expected: "number", found: "EOF" })`

#### Scenario: calc_simplify 空列表时返回原始节点
- **WHEN** calc_simplify.rs 收到空 CalcNode 列表
- **THEN** 函数 SHALL 返回包含错误信息的 Result，使用 `.expect("calc_simplify: nums checked non-empty above")` 标注不变量

### Requirement: expect() 必须携带上下文消息
所有 SHALL 保留的 `.expect()` 调用 MUST 包含描述性消息，说明该不变量为何成立。

#### Scenario: selector_parser expect 上下文
- **WHEN** 修复后的代码中保留 `.expect()`
- **THEN** 消息 SHALL 包含位置信息和期望条件，如 `expect("hex digit after #")`
