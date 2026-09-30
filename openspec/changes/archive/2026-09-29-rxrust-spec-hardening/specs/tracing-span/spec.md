## ADDED Requirements

### Requirement: 跨函数/跨阶段路径必须使用 tracing span
AI 在涉及跨函数调用或跨编译阶段的代码路径中，MUST 使用 `tracing::info_span!` / `debug_span!` / `trace_span!` 创建 span，不得仅使用 `event!` 或 `println!`。

#### Scenario: scan_map reducer 入口
- **WHEN** AI 编写 scan_map 的 reducer 函数
- **THEN** MUST 在 reducer 入口处使用 `let _span = info_span!("stage_name", field = %value).entered();`
- **字段命名** MUST 遵循约定：`stage`, `module`, `id`, `elapsed_ms`, `phase`, `token`, `variant`

#### Scenario: 渲染函数入口
- **WHEN** AI 编写 `render_node` 或类似渲染函数
- **THEN** MUST 使用 `let _span = info_span!("render_node", variant = ?node_variant_name(node)).entered();`

### Requirement: 禁止使用 println! / eprintln!
AI 不得在任何 src/ 代码中使用 `println!` 或 `eprintln!`，MUST 使用 tracing 宏。

#### Scenario: 调试输出
- **WHEN** AI 需要打印调试信息
- **THEN** MUST 使用 `tracing::info!(field = value, "message")` 或 `tracing::debug!(...)`
- **不得** 使用 `println!("...")` 或 `eprintln!("...")`

#### Scenario: panic 前打印
- **WHEN** AI 需要在不恢复错误前输出信息
- **THEN** MUST 使用 `tracing::error!(...)` + `return Err(...)` 或 `bail!(...)`
- **不得** 使用 `eprintln!("error: ...")` + `panic!(...)`

### Requirement: span 必须有 entered/exit 边界追踪
AI MUST 通过 `.entered()` 返回的 guard 追踪进入/退出边界，不得创建 span 但不 enter。

#### Scenario: 函数级 span
- **WHEN** AI 在函数入口创建 span
- **THEN** MUST 使用 `let _span = info_span!(...).entered();`（guard 命名 `_span`），guard 在函数退出时自动 drop 触发 exit 事件
- **不得** 仅写 `info_span!(...).enter()` 不绑定变量（span 立即 drop，无追踪效果）
