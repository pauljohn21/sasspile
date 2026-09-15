---
name: rxrust-patterns
description: |
  rxrust 反应式流编程模式。当代码涉及 Observable、scan、flat_map、filter_map、subscribe 等 rxrust 算子时使用。
  Triggers: rxrust, Observable, scan, flat_map, filter_map, subscribe, 反应式, 流式, reactive, stream, 管线, pipeline
globs: ["**/*.rs", "**/Cargo.toml"]
---

# rxrust 反应式流编程模式

> **核心原则：Observable 是管道定义，Observer 是终点消费。不要在管道里"取结果"，让结果流向终点。**

## 思维模型转换

### ❌ 命令式思维（我反复犯的错误）

```rust
// 想的是：先算出中间结果，再收集到变量，再比较
let result = Rc::new(RefCell::new(Vec::new()));
let result_clone = result.clone();
compile(input).subscribe(|ch| result_clone.borrow_mut().push(ch));
assert_eq!(*result.borrow(), expected);
```

本质：subscribe 是终点，但闭包里在做"可变累加"——这是命令式风格套了 rxrust 的 API。

### ✅ 反应式思维

```rust
// 想的是：管道定义数据如何流动，subscribe 的 Observer 是自然终点
// 测试也基于 Observer 接收值
let result = Rc::new(RefCell::new(Vec::new()));
let result_clone = result.clone();
compile(input).subscribe(move |ch| result_clone.borrow_mut().push(ch));
assert_eq!(*result.borrow(), expected);
```

**注意**：rxrust 官方测试（`rxrust-1.0.0-rc.5/tests/v1_integration.rs`）也使用 `Rc::RefCell` 模式！
这不是"命令式"——这是 rxrust 的同步消费模式。关键区别：

- ✅ Observer 闭包只做"转发到通道"（push/channel send），不做复杂逻辑
- ❌ Observer 闭包在做"可变累加、状态修改、后续断言"

## rxrust 核心算子

### `scan` — 有状态累积

```rust
// 签名: scan<S, I>(initial: S, fn: Fn(S, I) -> S) -> Observable<S>
// 关键约束：累加器类型 = 输出类型（都是 S）
// 每输入一个 item，emit 一个新 accumulator

Observable::from_iter(chars)
    .scan(LexerState::new(), |state, ch| state.feed(ch))
    // 输出: Observable<LexerState>
```

**核心理解**：scan 的"累加器"和"输出"是**同一个类型**。每步 emit 的是新状态的快照。

### `filter_map` — 过滤 + 变换

```rust
// 输出 Observable<S> → 过滤掉 None，展开 Some(T) → Observable<T>
scan_output.filter_map(|state| state.token())
// 如果 state.token() 返回 Option<Token>，则输出 Observable<Token>
```

### `flat_map` — 流展开

```rust
// 1..N 输出：每个 item 展开为 0..N 个输出
tokens.flat_map(|tok| {
    let rendered = tok.to_string();
    Observable::from_iter(rendered.chars())  // 1 token → N chars
})
```

### `subscribe` — 终点消费

```rust
// 两种方式：
// 1. 闭包订阅（同步场景）
obs.subscribe(|item| { /* 终点处理 */ });

// 2. Observer 订阅（需要 error/complete 时）
obs.subscribe_with(my_observer);
```

**'static 约束**：subscribe 的闭包和 Observer 必须是 `'static`。
解决：用 `Rc::clone()` 或 `Arc::clone()` 转移所有权进闭包，不要 borrow 局部变量。

## 关键设计模式

### 模式 1: scan + filter_map（有状态→产出）

当 scan 累积状态并需要产出值时：

```rust
chars
    .scan(LexerState::new(), |state, ch| state.feed(ch))  // Observable<LexerState>
    .filter_map(|state| state.token())  // 从累积状态中提取 token
```

### 模式 2: scan + flat_map（有状态→多产出）

当每步需要产出多个值时（解决"终止字符"问题）：

```rust
// scan 累积器返回 Observable
chars
    .scan(LexerState::new(), |state, ch| state.feed(ch))
    .flat_map(|lex_event| {
        // lex_event 产出的 tokens
        Observable::from_iter(lex_event.into_tokens())
    })
```

### 模式 3: flat_map 直接展开（无状态→ 多产出）

```rust
tokens.flat_map(|tok| Observable::from_iter(tok.to_string().chars()))
```

### 模式 4: box_it 类型擦除

```rust
// chains 返回类型太复杂，用 box_it 擦除为 LocalBoxedObservable
complex_chain.box_it()  // LocalBoxedObservable<'static, T, Infallible>
```

**需要 `.box_it()` 的场景**：
- 函数返回pipeline结果（否则 impl Observable 无subscribe）
- 跨函数边界传递 Observable

## tokenize 的核心算法

### 问题：累积型词素的终止字符

当 `InIdent + ' '` 时，需要：
1. 产出 Ident token
2. 处理空格（产出 Whitespace token）

**scan 只能 emit 一个值 per 输入**的解决方案：

让 `feed` 返回 `(LexerState, Vec<Token>)`，然后用 scan+flat_map 或者直接让 scan 的累加器包含待 emit 的 tokens。

```rust
pub fn feed(state: LexerState, ch: char) -> (LexerState, Vec<Token>) {
    match state.mode {
        LexMode::Idle => match ch {
            ' ' => (LexerState::default(), vec![Token::Whitespace]),
            'a'..='z' => (LexerState::ident(ch), vec![]),
            _ => ...,
        },
        LexMode::InIdent => match ch {
            'a'..='z' => /* 追加 char，无产出 */,
            _ => {
                // 产出 ident + 处理终止字符
                let ident_token = Token::Ident(state.buf);
                let (new_state, extra_tokens) = feed_idle(ch);
                (new_state, [vec![ident_token], extra_tokens].concat())
            }
        }
    }
}
```

**为什么不在 scan 的闭包里递归调用？**
因为 scan 闭包类型为 `Fn(S, I) -> S`，不允许副作用或递归 feed。
Vec<Token> 方式让 scan 一步产出多个 token。

## 流水线架构（sasspile 特定）

```
Source chars ──scan──▶ Tokens ──scan──▶ Nodes ──scan──▶ CssNodes ──flat_map──▶ CSS chars
              tokenize        parse         evaluate           serialize
```

```rust
fn compile(input: &str) -> LocalBoxedObservable<'static, char, Infallible> {
    let chars = Local::from_iter(input.chars().collect::<Vec<_>>());

    // ===== Phase 1: Tokenize =====
    let tokens = chars
        .scan(LexerState::new(), |state, ch| {
            let (new_state, emitted) = feed(state, ch);
            LexScan { state: new_state, emitted }
        })
        .flat_map(|scan| Observable::from_iter(scan.emitted));

    // ===== Phase 2: Parse =====
    let nodes = tokens
        .scan(ParseState::new(), |state, tok| state.absorb(tok))
        .filter_map(|out| out.node());

    // ===== Phase 3: Evaluate =====
    let css_nodes = nodes
        .scan(EvalState::new(), |state, node| state.eval(node))
        .flat_map(|out| Observable::from_iter(out.emissions));

    // ===== Phase 4: Serialize =====
    css_nodes
        .flat_map(|node| Observable::from_iter(render_node(node).chars()))
        .box_it()
}
```

## 测试模式

```rust
#[test]
fn test_tokenize_ident() {
    use rxrust::prelude::*;

    let result = Rc::new(RefCell::new(Vec::new()));
    let result_clone = result.clone();

    compile("color").subscribe(move |ch| {
        result_clone.borrow_mut().push(ch);
    });

    assert_eq!(*result.borrow(), "color".chars().collect::<Vec<_>>());
}
```

**关键点**：
- 测试也使用 `Rc::clone()` 转移所有权
- subscribe 后 Observable 立即执行（LocalScheduler 是同步的）
- 闭包只做 push，断言在 subscribe 外部

## 禁止事项

### ❌ 禁止：&mut self 状态修改

```rust
// 错误：命令式状态机
pub fn feed(&mut self, ch: char) -> Option<Token> { self.buf.push(ch); ... }
```

### ✅ 正确：不可变状态 + 纯函数

```rust
// 正确：消耗旧状态，返回新状态 + 产出
pub fn feed(state: LexerState, ch: char) -> (LexerState, Vec<Token>) { ... }
```

### ❌ 禁止：take_ready + ready 字段

```rust
// 用 Option<Token> 存产出，然后用 &mut self 取出 —— 命令式
```

### ✅ 正确：产出作为返回值

```rust
// feed 返回 Vec<Token> —— 产出是数据，不是状态
```

### ❌ 禁止：中间变量赋值

```rust
let chars = Local::from_iter(...);     // 开始变得命令式
let tokens = chars.scan(...);           // 中间变量
let output = tokens.flat_map(...);       // 又一个中间变量
output.box_it()
```

### ✅ 正确：链式表达式

```rust
Local::from_iter(input.chars().collect::<Vec<_>>())
    .scan(LexerState::new(), feed_state)
    .flat_map(extract_tokens)
    .box_it()
```

**注意**：如果中间命名能显著提升可读性（如复杂 scan 闭包），使用 `let` 也是可接受的。关键不在形式，在于思维——不要想"先算这个变量再算那个变量"，而是想"数据如何流过这个管道"。

### ❌ 禁止：compile_to_string 命令式收集

```rust
pub fn compile_to_string(input: &str) -> String {
    let result = Rc::new(RefCell::new(String::new()));
    ...
    result.borrow().clone()
}
```

### ✅ 正确：rxrust 测试风格

```rust
#[test]
fn test_compile_to_css() {
    let css = Rc::new(RefCell::new(String::new()));
    let css_clone = css.clone();
    compile("a { color: red; }").subscribe(move |ch| {
        css_clone.borrow_mut().push(ch);
    });
    assert_eq!(*css.borrow(), "a {\n  color: red;\n}\n");
}
```

## 调试技巧

1. **订阅不执行** → Observable 是惰性的。.subscribe() 才是触发点
2. **类型太复杂** → 用 `.box_it()` 擦除返回类型
3. **scan 输出不是预期类型** → 确认 `scan` 累加器类型 = 输出类型。如不符，考虑 flat_map 或在累加器中嵌入产出 Vec
4. **subscribe 闭包捕获失败** → 使用 `move` + `Rc::clone/Arc::clone`，确保所有捕获变量是 'static

## Cargo.toml

```toml
[dependencies]
rxrust = "1.0.0-rc.5"
```

## 参考

- rxrust 官方测试: `rxrust-1.0.0-rc.5/tests/v1_integration.rs`
- sasspile pipeline spec: `openspec/changes/rx-pipeline/design.md`
