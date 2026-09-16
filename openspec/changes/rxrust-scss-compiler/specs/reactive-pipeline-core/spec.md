## ADDED Requirements

### Requirement: 编译管线是 Observable 到 Observable 的转换链
编译管线 SHALL 由四个标准 stage 组成，每个 stage 都是一个 `Observable<In> → Observable<Out>` 的纯算子变换，通过 `.pipe()` 或 `.map()` 串联。

#### Scenario: tokenize stage 变换 char 流为 Token 流
- **WHEN** 源 Observable 发射 char 序列 `['$', 'c', 'o', 'l', 'o', 'r', ':']`
- **THEN** TOKENIZE stage 通过 `scan(Scanner::next)` + `filter(is_emittable)` 将其转换为 `Observable<Token>`，依次发射 `Dollar, Ident("color"), Colon, ...`
- **AND** 单个 char MUST NOT 直接映射为单个 Token（Identifier 由多个 char 聚合），必须用 `scan` + `filter` 模式

#### Scenario: parse stage 变换 Token 流为 Node 流
- **WHEN** Token Observable 发射 `Ident("a"), Whitespace, LBrace, ...`
- **THEN** PARSE stage 通过 `scan(ASTBuilder::feed)` + `filter(has_complete_node)` 产出 `Observable<Node>`
- **AND** 同一组 Token 可能被 ASTBuilder 合并为一条 Rule 节点（1:N 映射对应 scan+filter 模式）

#### Scenario: evaluate stage 变换 Node 流为 CssNode 流
- **WHEN** Node Observable 发射规则声明序列
- **THEN** EVALUATE stage 通过 `flat_map(Directive::expand)` 展开指令树
- **AND** `@extend` 展开 1:N 映射: `flat_map(expand_extend)` 返回 Observable<Node>
- **AND** `@mixin/@include` 展开: `flat_map(apply_mixin)` 返回 Observable<Node>

#### Scenario: serialize stage 变换 CssNode 流为 char 流
- **WHEN** CssNode Observable 发射规则和声明
- **THEN** SERIALIZE stage 通过 `map(|node| node.render())` 渲染
- **AND** `flat_map(|chunk| Local::from_iter(chunk.chars()))` 将字符串产出 flat 为 char

#### Scenario: 管道整体通过 box_it 类型擦除
- **WHEN** 在 pipeline.rs 中链式调用各 stage
- **THEN** 每个 `.pipe(stage)` 返回后 MUST 跟随 `.box_it()` 调用
- **AND** 最终 subscribe 接口 MUST 统一为 `LocalBoxedObservable<char, Infallible>`

### Requirement: 单文件 ≤ 500 行的编译模块拆分
每个编译 stage 的实现 MUST 独立存在于单文件中，单文件不超过 500 行源码（不含 tests/）。超出 MUST 被拆分为新的文件，拆分不改变对外 `fn(Observable<In>) -> Observable<Out>` 的签名。

#### Scenario: tokenize_dst.rs 超出行数
- **WHEN** `src/tokenize_dst.rs` 的源码行数超过 500
- **THEN** MUST 拆分为 `lexer_state.rs`（扫描器状态机）+ `lex_feed.rs`（char 喂入函数）
- **AND** 每个子文件 MUST ≤ 500 行

#### Scenario: evaluate_dst.rs 按 directive 拆分
- **WHEN** `src/evaluate_dst.rs` 超过 500 行
- **THEN** MUST 将 `@extend`、`@mixin/@include`、`@use/@import` 各拆出为 `evaluate/extend.rs`、`evaluate/mixin.rs` 等
- **AND** `evaluate_dst.rs` MUST 仅保留 enum dispatch + flat_map 调度

#### Scenario: pipeline.rs 仅负责编排
- **WHEN** `pipeline.rs` 编排各 stage
- **THEN** pipeline.rs 必须 ≤ 150 行
- **AND** MUST 仅导入 stage 函数并通过 `.pipe().box_it().last().subscribe()` 串联
- **AND** MUST NOT 包含任何 stage 内部逻辑

### Requirement: 禁止内联测试
所有测试 MUST 在 `tests/` 目录实现，`src/` 下的源码 MUST 不包含 `#[test]`、`#[cfg(test)]` 或任何测试逻辑。

#### Scenario: 添加 tokenize 单元测试
- **WHEN** 需要验证 tokenize 行为
- **THEN** MUST 在 `tests/tokenize.rs` 中编写 `#[test]` 函数
- **AND** MUST 使用 rxrust 的测试宏 `#[rxrust_macro::test(local)]`

#### Scenario: 添加端到端集成测试
- **WHEN** 验证完整管线（tokenize → parse → evaluate → serialize）
- **THEN** MUST 在 `tests/integration.rs` 中编写
- **AND** 使用 HRX spec 文件作为 input + 比较 output

### Requirement: 必须使用 tracing span 跨阶段追踪
所有跨越编译阶段的调用 MUST 通过 `tap` 桥接 `tracing::span!`，不得直接在 map/scan 闭包内使用 tracing 宏（避免副作用污染纯变换）。

#### Scenario: 通过 tap 桥接 tracing span
- **WHEN** pipeline 调用 tokenize
- **THEN** MUST 使用：
  ```rust
  .pipe(tokenize)
  .tap(|token| debug!(?token, stage = "tokenize"))
  ```
- **AND** entry/exit tracing MUST 在 tap 闭包内实现 span 记录

#### Scenario: 使用 Instrument trait
- **WHEN** stage 需要自动 span entry/exit
- **THEN** MAY 使用 rxrust 的 `tracing` 集成 `.instrument(span)` 替代手写 tap
- **AND** span MUST 包含字段：`stage`、`module`、`path`、`elapsed_ms`

#### Scenario: span 字段命名约定
- **WHEN** 所有 stage 入口 tap
- **THEN** span 字段 MUST 遵循：`stage = %"tokenize"`, `module = %"feed"`, `id = node_id`, `token.kind = ?`
