# Requirement: 消除 `RxmRefCell` + `borrow_mut` 的 GC 思维

`src/` 目录下所有业务函数 MUST NOT 使用 `Rc<RefCell<T>>` + `borrow_mut()` 模式来模拟 GC 语言的共享可变状态。这是所有权的根本约束。

#### Scenario: 函数需要累积结果

- **WHEN**  helper 函数需要遍历输入并累积到集合
- **THEN** MUST 使用 `Iterator::map` / `Iterator::filter` / `Iterator::fold` / `scan().collect()`
- **AND** MUST NOT 使用 `let mut v = Vec::new(); for x in items { v.push(f(x)) }; v`

#### Scenario: 状态需要在迭代中传播

- **WHEN**  reducer 需要在多步处理中传播可变状态
- **THEN** MUST 使用 `scan_map(State::new(), reducer)` 消费旧状态产出新状态
- **AND** MUST NOT 使用 `let mut state = State::default(); for item in items { state = f(state, item) }; state`

#### Scenario: 终点收集流值

- **WHEN**  需要从 Observable 取最终值 / 全部值
- **THEN** MUST 使用 `.last().subscribe(|v| ...)` 或 `.collect().subscribe(|v: Vec<_>| ...)`
- **AND** MUST NOT 使用 `let r = Rc::new(RefCell::new(Vec::new())); obs.subscribe(|x| r.borrow_mut().push(x)); r.borrow().iter().collect()`

#### Scenario: 副作用（tracing/debug）

- **WHEN**  需要在流中记录某 item 进入/离开
- **THEN** MUST 使用 `.tap(|item| tracing::debug!(...))`
- **AND** MUST NOT 在 `map` / `flat_map` 闭包内部调用 `tracing::` 宏

---

# Requirement: 消除 while + 手动索引循环

函数 MUST NOT 使用 `while i < items.len() { ...; i += 1 }` 模式——这是命令式/GC 思维的典型症状。

#### Scenario: 字符串字符级处理

- **WHEN**  需要按字符扫描字符串并分段产出
- **THEN** MUST 使用 `chars().scan_map(State::new(), |s, ch| s.feed(ch)).flat_map(|segs| ...).last()`
- **AND** MUST NOT 使用 `let chars_vec: Vec<(usize, char)> = s.char_indices().collect(); let mut i = 0; while i < chars_vec.len() { ... }`

#### Scenario: 切片/分割为子段

- **WHEN**  需要把 vector 或 string 按某个分隔符拆分为子段
- **THEN** MUST 使用 `split(',').map(trim).collect()`（简单）或 `chars().scan(...).filter_map(...).collect()`（带嵌套跟踪）
- **AND** MUST NOT 手动维护 `depth` 计数器 + `match ch { '(' => depth += 1, ... }`

---

# Requirement: 跨节点展开使用 flat_map

pipeline 内的 1→N 展开 MUST 由 `flat_map` 算子完成,而不是内部的 `for node in body { out.extend(eval)? }` 递归累积。

#### Scenario: @for / @if / @mixin 展开

- **WHEN**  evaluate 阶段需要把一个 Node 展开为多个 CssNode
- **THEN** match 分支 MUST 返回 impl Observable 或 into_local_obs().boxed()
- **AND** MUST NOT 使用 `let mut out = Vec::new(); for node in body { out.extend(evaluate_node_with_locals(ctx, node)?); }`

#### Scenario: mixin body / @for body 嵌套展开

- **WHEN**  body 内的子节点各自展开
- **THEN** MUST 使用 `body.iter().flat_map(|node| expand_node_owned(ctx, node))`
- **AND** MUST 让所有权沿流传播,而不是复制到局部 Vec 再返回
