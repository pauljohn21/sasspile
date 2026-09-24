---
description: src/directive 编译管线专用规则
---

# src/directive/ 管线代码规则

> 本目录包含 sasspile 核心编译管线（pipeline.rs / eval.rs / parse.rs 等）
> 管线使用 **Shared** 多线程上下文（非 Local），入口 String 因 'static + Send 约束

## ⛔ 反模式（见了就改）

| 反模式 | 正确做法 |
|--------|---------|
| `let mut state = State::default(); source.map(\|x\| state.update(x))` | `source.scan_map(State::default(), \|s, x\| s.update(x))` |
| `let mut v = vec![]; for x in items { v.push(f(x)) }` | `items.flat_map(f).collect::<Vec<_>>()` |
| `Rc<RefCell<CompileState>>` | `scan_map(CompileState::default(), reducer)` |
| `flat_map(\|x\| vec![x])` | `flat_map(\|x\| Shared::from_iter(vec![x]))` |
| `subscribe` 闭包内再 `subscribe` | 改 `flat_map` + 子流 |
| Shared 管线中试图用 `&str` 绕过 'static | 接受 `String` 入口，保证中间零额外 clone |
| `Arc<Mutex<Sender>>` + `lock().send()` | `std::sync::mpsc::channel` + `tx.send()` |
| `from_iter` 循环无 `is_closed` 检查 | 循环顶加 `if observer.is_closed() { break; }` |
| `Local::subject::<&str, ...>()` 在 Shared 场景 | `Shared::subject::<String, ...>()` |

## ✅ 必须遵守

### 1. Shared 管线的数据流

```rust
// ✅ 正确：Shared 管线用 String 入口，中间零 clone，终点才 owned
subject
    .scan_map(CompileState::new(), dispatch_pass)         // &mut State + String → Vec<String>
    .flat_map(|v: Vec<String>| Shared::from_iter(v))      // Vec<String> → String
    .scan_map(CssBuilder::new(), |b: &mut _, l: String|  // &mut Builder + String → Vec<CssNode>
        b.feed(&l))                                       // &l 借用传参
    .flat_map(|v: Vec<CssNode>| Shared::from_iter(v))     // Vec<CssNode> → CssNode
    .map(|node: CssNode| render_node(&node))              // &CssNode → String（终点才出现）
    .collect::<Vec<String>>()
    .last()
    .subscribe(|v: Vec<String>| v.join("\n"))             // 终点产物
```

### 2. scan_map 是状态机的唯一位置

```rust
source.scan_map(CompileState::default(), |state: &mut CompileState, token: String| -> Vec<String> {
    state.line_count += 1;           // &mut 就地修改
    state.dispatch(token)            // 消费 token, 返回 Vec<String>
});
```

### 3. tracing span 在疑似路径

```rust
fn dispatch_pass(state: &mut CompileState, token: String) -> Vec<String> {
    let _span = info_span!("dispatch_pass",
        phase = ?state.phase,
        line = state.line_count,
        token = %token
    ).entered();
    // ...
}
```

字段命名约定：`stage`, `module`, `id`, `elapsed_ms`, `phase`, `token`, `variant`

### 4. 终态模式固定

管线出口固定用 `collect::<Vec<T>>().last().subscribe(closure)` 模式。
subscribe 闭包内通过 `std::sync::mpsc::channel` 转移终态产物。

### 5. flat_map 的 Inner 必须是 Observable

```rust
.flat_map(|v: Vec<String>| Shared::from_iter(v))
.flat_map(|v: Vec<CssNode>| Shared::from_iter(v))
```

### 6. 入口 Subject 类型

```rust
// ✅ Shared 多线程上下文
let subject = Shared::subject::<String, Infallible>();

// ❌ 不要用在 Shared 管线
// let subject = Local::subject::<&str, Infallible>();
```

## 📦 算子签名速查

```rust
fn scan_map<Acc, Output, F>(self, initial: Acc, f: F)
where F: for<'a> FnMut(&mut Acc, Self::Item<'a>) -> Output

fn flat_map<F, Inner>(self, f: F)
where F: for<'a> FnMut(Self::Item<'a>) -> Inner,
      Inner: Context<Inner: ObservableType<Err = Self::Err>>

fn collect<C>(self) -> Self::With<Collect<Self::Inner, C>> where C: Default
fn last(self) -> Self::With<Last<Self::Inner>>
```

## 📦 终端模式

```rust
// std::sync::mpsc — 同步管线不需要 tokio
let (tx, rx) = std::sync::mpsc::channel::<String>();

// subscribe 转移终态
.subscribe(move |css_vec: Vec<String>| {
    let _ = tx.send(css_vec.join("\n"));
});

// 注入数据 + complete
input.lines().for_each(|line| subject.clone().next(line.to_string()));
subject.clone().complete();

// 消费终态
rx.recv().unwrap_or_default()
```
