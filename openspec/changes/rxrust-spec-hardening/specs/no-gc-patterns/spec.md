## ADDED Requirements

### Requirement: 禁止文本模板式字符串替换
AI 不得使用 `while`/`for` + `result.replace()` 模式实现变量替换或插值。MUST 将输入解析为结构化 token 后使用 `map`。

#### Scenario: 变量插值 #{$var}
- **WHEN** AI 需要实现 SCSS 变量插值 `#{...}` 的替换
- **THEN** MUST 将输入解析为 token 枚举（`Text(String)` / `Interpolation(String)` / `VariableRef(String)`），然后用 `.map(|tok| match tok { ...) }` 替换
- **不得** 使用 `while let Some(start) = result.find("#{") { result.replace_range(...) }` + `break` 模式

#### Scenario: 批量变量替换
- **WHEN** AI 需要对一行中多个变量做替换
- **THEN** MUST 构建 token 流后使用 `.scan_map(VarScope::new(), |scope, tok| tok.resolve(scope))`，不得使用 `for (name, value) in &vars { result = result.replace(name, value) }`

### Requirement: 禁止命令式累积 — for + Vec::push/extend
AI 不得使用 `for x in items { result.push(f(x)) }` 或 `result.extend(...)` 累积结果。MUST 使用 `flat_map` + `collect`。

#### Scenario: 多行展开
- **WHEN** AI 需要将每行展开为多行（如 mixin 展开、循环展开）
- **THEN** MUST 使用 `lines.into_iter().flat_map(|line| expand(line)).collect::<Vec<_>>()`，不得使用循环 push

#### Scenario: BFS/队列展开
- **WHEN** AI 需要用队列做 BFS（如 @forward 链展开）
- **THEN** MUST 使用 `Shared::from_iter(queue)` + `.flat_map(...)` 或将 BFS 本身重构为 scan_map 状态机，不得使用 `while let Some(x) = queue.pop_front() { result.extend(...) }`

### Requirement: 禁止共享可变状态模式
AI 不得使用 `Rc<RefCell<T>>` / `Arc<Mutex<T>>` / 外部 `let mut state` 被多个闭包捕获。MUST 使用 `scan_map` 的 `&mut Acc`。

#### Scenario: 循环内状态更新
- **WHEN** AI 需要在 @while 展开时维护变量状态（如 `$i: $i + 1`）
- **THEN** MUST 使用 `scan_map(WhileState::new(), |state, line| { state.update(line); state.output() })`
- **不得** 使用 `let mut current_state = state.clone(); current_state.scope.variables.insert(...)` 模式

#### Scenario: 跨闭包共享发送端
- **WHEN** AI 需要在多个闭包间共享 Sender
- **THEN** MUST 使用 `mpsc::channel` + `move` 一次性转移，不得使用 `Arc<Mutex<Sender>>` + `.lock().send()`

### Requirement: 禁止 if/return 控制流链替代 filter/map
AI 不得使用连续的 `if condition { return x }` 链实现分支数据流。MUST 使用 `filter` + `map` 组合或 `scan_map`。

#### Scenario: process_line 多分支
- **WHEN** AI 编写处理多种行类型（变量定义、@include、@extend、plain CSS）的函数
- **THEN** MUST 将行类型建模为枚举（`enum LineKind { VarDef, Include, Extend, Plain }`），用 `LineKind::classify(&line)` 做 `map` + `filter` 分发
- **不得** 使用 60 行的 `if trimmed.starts_with('$') { ... return ... } if trimmed.starts_with("@include") { ... return ... }` 链

#### Scenario: @if/@else 分支选择
- **WHEN** AI 实现 @if/@else if/@else 分支
- **THEN** MUST 使用 `scan_map(IfState::new(), |state, (cond, body)| { state.eval_branch(cond, body) })` 或枚举 + `filter`，不得使用 `for (cond, body) in branches { if take { return body.clone() } }`

### Requirement: 禁止 Flux 术语在代码和 doc comment 中出现
AI 不得在 doc comment、变量名、注释、模块描述中使用 "Flux 思维" / "Flux → rxrust 转译" / "Flux.create()" 等表述。

#### Scenario: 模块级文档注释
- **WHEN** AI 编写或修改 `//!` 模块级 doc comment
- **THEN** MUST 使用 "Rust ownership 三态 (move/&/&mut) 驱动" 或 "rxrust 算子链组合" 描述架构
- **不得** 使用 "Flux 思维" / "Flux 模型" / "Flux → rxrust 转译" / "Flux Sinks.Many"

#### Scenario: 管线入口注释
- **WHEN** AI 注释 Shared Subject 入口
- **THEN** MUST 写 `// Shared Subject 入口 (String 因 Shared 需要 'static + Send)`
- **不得** 写 `// Flux 思维: 这就是 Flux.create() 的 Sinks.Many.asFlux()`
