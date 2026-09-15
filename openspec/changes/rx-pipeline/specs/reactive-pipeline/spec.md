# Spec: Reactive Pipeline

## ADDED Requirements

### Requirement: 流变换函数作为编译器的核心抽象

编译器 SHALL 以流变换函数（而非对象 + 方法）作为核心抽象单元。

#### Scenario: tokenize 是纯流变换

- **WHEN** 调用 `tokenize(Observable<char>)`
- **THEN** 返回 `Observable<Token>`，不持有 Lexer 结构体实例
- **AND** 内部使用 `scan(LexerState, feed)` 累积词素状态

#### Scenario: parse 是纯流变换

- **WHEN** 调用 `parse(Observable<Token>)`
- **THEN** 返回 `Observable<Node>`，不持有 Parser 结构体实例
- **AND** 内部使用 `scan(ParseState, absorb)` 累积结构状态

#### Scenario: evaluate 是纯流变换

- **WHEN** 调用 `evaluate(Observable<Node>)`
- **THEN** 返回 `Observable<CssNode>`，不持有 Evaluator 结构体实例
- **AND** 内部使用 `scan(EvalState, eval)` 自动传播 Env

#### Scenario: serialize 是纯流变换

- **WHEN** 调用 `serialize(Observable<CssNode>)`
- **THEN** 返回 `Observable<char>`，不持有 Serializer 结构体实例
- **AND** 内部使用 `flat_map(render_node)` 展开 CSS 字符

---

### Requirement: scan 累加器自动传播状态

编译器 SHALL 使用 scan 操作符在流变换中自动传播状态，无需手动线程化。

#### Scenario: ParseState 管理 brace 嵌套

- **WHEN** token 流包含 `{ ... }`
- **THEN** ParseState 内部 brace_stack 自动 push/pop
- **AND** 遇到 `}` 时 emit 完整 Node

#### Scenario: EvalState 管理 Env 作用域

- **WHEN** 节点流包含 `Define($x, 1)` 后跟 `Prop(color, $x)`
- **THEN** EvalState 中的 Env 从第一个节点自动传递到第二个
- **AND** 第二个节点能正确查找到 `$x` 的值

#### Scenario: 作用域进入/退出

- **WHEN** 节点流进入嵌套规则
- **THEN** EvalState 自动创建子作用域
- **AND** 退出时自动恢复到父作用域

---

### Requirement: 全管线组装

编译器 SHALL 提供 `compile(source: &str) -> Observable<char>` 作为端到端编译入口。

#### Scenario: 基础编译

- **WHEN** 调用 `compile("a { color: red }")`
- **THEN** 订阅后产出 CSS 字符流
- **AND** 收集结果为 `"a {\n  color: red;\n}\n"`

#### Scenario: 惰性求值

- **WHEN** 调用 `compile(source)` 但不订阅
- **THEN** 不执行任何计算（零开销）

#### Scenario: 错误传播

- **WHEN** 输入包含语法错误
- **THEN** Observable 发出 `Err(SassError)` 后 complete

---

### Requirement: 模块流合并

编译器 SHALL 通过 flat_map 将依赖文件的流合并入主 Node 流。

#### Scenario: @use 流 merge

- **WHEN** 主文件 `@use 'lib'` 且 lib 文件定义 `$x: 1`
- **THEN** lib 文件的 Node 流 merge 到主流
- **AND** 主流中后续节点可使用 `$x`

#### Scenario: 模块缓存

- **WHEN** 多个文件 `@use 'lib'`
- **THEN** lib 只加载一次，流被共享

---

### Requirement: OTel 操作符

编译器 SHALL 提供 `otel_span` 作为 Observable 操作符。

#### Scenario: OTel span 作为流节点

- **WHEN** 在 tokenize 后接入 `.pipe(otel_span("lex"))`
- **THEN** 每个通过该点的 Token 触发一次 OTel 事件
- **AND** 不影响 Token 的值传递

#### Scenario: 条件编译

- **WHEN** 未启用 `otel` feature
- **THEN** `otel_span` 编译为空操作（零开销）
