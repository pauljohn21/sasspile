## ADDED Requirements

### Requirement: 所有权三态是闭包选择的唯一判据
AI 选择 rxrust 算子时，MUST 依据 Rust 闭包所有权语义（move / & / &mut），不得依据 Flux/Reactor 等 GC 框架的思维模式。

#### Scenario: 消费 T 产出 U 时选择 map
- **WHEN** AI 需要将流中每个元素从类型 T 转换为类型 U
- **THEN** MUST 使用 `.map(|t: T| -> U { ... })`，闭包接收 T（所有权转移），不得使用 `for x in stream { out.push(f(x)) }`

#### Scenario: 借用判断时选择 filter
- **WHEN** AI 需要对流中元素做条件过滤
- **THEN** MUST 使用 `.filter(|t: &T| -> bool { ... })`，闭包借用 &T，不得使用 `if condition { vec.push(x) }` 累积模式

#### Scenario: 累积状态时选择 scan_map
- **WHEN** AI 需要在流处理中维护可变状态（计数器、累加器、构建器）
- **THEN** MUST 使用 `.scan_map(Acc::new(), |acc: &mut Acc, item| -> Out { ... })`，不得使用外部 `let mut state` + `map` 闭包捕获

#### Scenario: 终态转移时使用 move
- **WHEN** AI 在 subscribe 闭包中转移终态（如 mpsc::Sender）
- **THEN** MUST 使用 `.subscribe(move |v| { tx.send(v) })`，通过 move 将 tx 所有权移入闭包，不得使用 `Arc<Mutex<Sender>>` + `.lock().send()`

### Requirement: 算子选择表作为出发点
AI 在编写任何管线代码前，MUST 先对照以下三态表决定算子：

| 意图 | 闭包签名 | rxrust 算子 |
|------|---------|------------|
| 消费 T → 产出 U | `FnMut(T) -> U` | `map` |
| 借用 T 判断 bool | `FnMut(&T) -> bool` | `filter` |
| &mut 持有状态 | `FnMut(&mut Acc, T) -> Out` | `scan_map` |
| T → 1:N 子流 | `FnMut(T) -> Inner: Observable` | `flat_map` |
| 副作用不改流 | `FnMut(&T)` | `tap` |
| 终态转移 | `move` closure | `subscribe` |
| 汇聚成集合 | `C: Extend<T>` | `collect::<C>()` |

#### Scenario: 新增功能前查表
- **WHEN** AI 需要新增一个涉及数据流转换的功能
- **THEN** MUST 先从上表选择对应算子，不得直接写 `for`/`while` 循环

### Requirement: 禁止 GC 思维反模式（硬禁令）
以下模式在 `src/` 代码中严格禁止，AI MUST NOT 产出：

| # | 禁止模式 | 正确替代 |
|---|---------|---------|
| 1 | `while let Some(x) = ... { result.replace_range(...) }` 文本手术 | `map` + 枚举 token 替换 |
| 2 | `for x in items { out.push(f(x)) }` 命令式累积 | `items.flat_map(f).collect::<Vec<_>>()` |
| 3 | `let mut v = vec![]; stream.map(|x| { v.push(x); ... })` 共享可变 | `scan_map(Vec::new(), \|acc, x\| { ... })` |
| 4 | `state.clone()` 在流操作中克隆整个状态 | `scan_map` 的 `&mut Acc` 就地修改 |
| 5 | `if take { return body.clone(); }` 提前返回破坏线性流 | 用 `filter` + `map` 组合 |
| 6 | `Arc<Mutex<T>>` + `lock()` | `std::sync::mpsc::channel` 单次值转移 |
| 7 | `Rc<RefCell<T>>` + `borrow_mut()` | `scan_map(State::new(), reducer)` |
| 8 | `flat_map(\|x\| vec![x])`（非 Observable） | `flat_map(\|x\| Shared::from_iter(vec![x]))` |
| 9 | 用 Flux 概念设计再"翻译"成 rxrust | 直接用 Rust ownership 三态思考 |

#### Scenario: AI 自检发现 for 循环累积
- **WHEN** AI 写出 `let mut result = vec![]; for x in items { result.push(f(x)); }`
- **THEN** MUST 在提交前自我纠正为 `items.map(f).collect::<Vec<_>>()`

#### Scenario: AI 自检发现 mutable 闭包捕获
- **WHEN** AI 写出 `let mut state = State::new(); source.map(|x| { state.update(x); ... })`
- **THEN** MUST 在提交前自我纠正为 `source.scan_map(State::new(), |s, x| { s.update(x); ... })`

### Requirement: scan_map 是状态的唯一栖息地
AI MUST NOT 在 `scan_map` 闭包外部持有任何可变状态（结构体字段除外，但结构体必须作为 Acc 由 scan_map 持有）。

#### Scenario: 跨行状态依赖
- **WHEN** AI 需要在前一行基础上处理当前行（如多行规则嵌套）
- **THEN** MUST 将状态放入 `scan_map` 的 `Acc` 参数中，通过 `&mut Acc` 就地修改
