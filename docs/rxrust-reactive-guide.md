# rxrust 反应式编程指南（AI 零基础执行版）

> **rxrust 是什么**：Rust 的反应式编程框架 1.0.0-rc.5。组件包括 `Observable`（生产者）、`Observer`（消费者）、`operator`（管线算子）。
>
> **反应式编程是一条工厂流水线**：上游生产 → 中间算子逐步处理 → 终态收集产出。

---

## 第 1 章：四大支柱（必须死记住）

反应式编程在 Rust 中靠**四根支柱**支撑：

### 支柱 1：借引用（&T / &mut Acc）

管线中流动的中间数据应尽可能**借用**（scan_map 内 `&mut Acc` 就地修改，渲染函数 `&T` 借用）；终态产物才拥有数据。

```rust
// scan_map 闭包内 &mut 就地修改状态
stream.scan_map(CssBuilder::new(), |builder: &mut CssBuilder, line: String| -> Vec<CssNode> {
    builder.feed(&line)  // &line 借用传参，不 clone
});

// 渲染函数借用 &T 产生 owned String
.map(|node: CssNode| render_node(&node))  // render_node(&CssNode) -> String
```

**注意**：sasspile 使用 Shared 多线程上下文（`'static + Send`），入口必须是 `String`；但中间步骤仍保持零额外 clone，渲染时借用 `&CssNode`。

### 支柱 2：消费自身（&mut Acc）

状态演化算子 `scan_map` 用 `&mut Acc` 就地修改状态：

```rust
// acc 是管线的内部状态，每步修改它的字段，但不交出所有权。
stream.scan_map(CompileState::new(), |state: &mut _, token: String| {
    state.line_count += 1;          // 就地修改
    state.dispatch(token)           // 消费 token，返回 Vec<String>
})
```

**口诀**：`&mut acc` = 我的东西我改，别人的东西我借，输出的东西归下游。

### 支柱 3：响应式思想（chain 是声明，subscribe 是边界）

```rust
// 这一行没有任何代码运行——它只是"声明"了一种关系。
let declared = source.map(|x| x * 2).filter(|x| x > 10);

// 只有 .subscribe() 才真正开始执行。
declared.subscribe(|x| println!("{}", x));
```

| 概念 | 含义 |
|------|------|
| chain | 声明"数据如何流动"，零成本，不执行 |
| subscribe | **执行边界**，从这里开始数据真正流动 |
| 类型签名 | `map` 返回 `Self::With<Map<...>>`，类型本身就是逻辑文档 |

### 支柱 4：终点收集（→ String）

**只有终态产物才真正拥有（own）数据**。管线中的中间步骤，除了累加器，能用借用的就借用；最终结果必须是 `String` / `Vec<T>` 这种 owned 类型。

```rust
source
    .scan_map(CompileState::default(), dispatch)         // &mut 就地修改
    .flat_map(|v: Vec<String>| Shared::from_iter(v))     // 1:N 展平
    .scan_map(CssBuilder::default(), build)               // &mut 就地修改
    .flat_map(|v: Vec<CssNode>| Shared::from_iter(v))     // 1:N 展平
    .map(|node: CssNode| render_node(&node))              // &CssNode → String（首次 owned）
    .collect::<Vec<String>>()
    .last()
    .subscribe(move |v: Vec<String>| {
        let output: String = v.join("\n");               // ✅ 终点产物
        tx.send(output);
    });
```

---

## 第 2 章：Observer trait 和 move 语义

### 2.1 Observer 的三个方法

```rust
trait Observer<Item, Err> {
    fn next(&mut self, value: Item);  // 借用 self，可重复调用
    fn error(self, err: Err);          // 🔑 消费 self，调用后 Observer 不复存在
    fn complete(self);                 // 🔑 消费 self，调用后 Observer 不复存在
}
```

### 2.2 为什么 error/complete 要"消费 self"？

这是 Rust 在 **类型层面** 保证：Observer 收到 error / complete 后，不可再调用 `next`。

```rust
// 编译器保证：observer.error() 后，你不能再 observer.next()
let mut obs = MyObserver;
obs.next(1);      // ✅ 可以
obs.next(2);      // ✅ 可以
obs.complete();   // ✅ 可以
// obs.next(3);   // ❌ 编译错误：obs 已被 move
```

### 2.3 推论：自定义 Observer 的包装器写法

```rust
struct MyObserver<O> {
    inner: O,  // 下游 observer
}

impl<O, Item, Err> Observer<Item, Err> for MyObserver<O>
where O: Observer<Item, Err> {
    fn next(&mut self, value: Item) {
        // 自己做点处理，然后转发给 downstream
        self.inner.next(value);
    }

    fn error(self, err: Err) {
        // 🔑 self 被 move，调用 self.inner.error() 后 self 不复存在
        self.inner.error(err);
    }

    fn complete(self) {
        // 🔑 同上
        self.inner.complete();
    }
}
```

**错误示范**：

```rust
// ❌ 编译失败：self 被 move 后仍能访问
fn error(self, err: Err) {
    self.inner.error(err);
    println!("{:?}", self.something); // ❌ self 已被 move
}
```

---

## 第 3 章：CoreObservable trait——签子就是订阅逻辑

### 3.1 trait 签名

```rust
trait CoreObservable<O>: ObservableType {
    type Unsub;
    fn subscribe(self, observer: O) -> Self::Unsub;
}
```

- `O` 是 observer context（包含调度器 + observer）
- `Unsub` 是取消订阅的句柄（of/from_iter 返回 `()`）

### 3.2 两个典型实现

#### Of：发射单值并 complete

```rust
struct Of<T>(T);

impl<C, T> CoreObservable<C> for Of<T>
where C: Context, C::Inner: Observer<T, Infallible> {
    type Unsub = ();  // 不需要 unsubscribe 句柄
    fn subscribe(self, ctx: C) -> () {
        let mut observer = ctx.into_inner();
        observer.next(self.0);  // 发射值
        observer.complete();     // 终结
        // observer.next(x);     // ❌ 编译错误
    }
}
```

#### FromIter：迭代器同步发射

```rust
impl<C, Iter> CoreObservable<C> for FromIter<Iter>
where C: Context, C::Inner: Observer<Iter::Item, Infallible> {
    type Unsub = ();
    fn subscribe(self, ctx: C) -> () {
        let mut observer = ctx.into_inner();
        for item in self.iter {
            if observer.is_closed() { break; }  // 🔑 必须检查
            observer.next(item);
        }
        if !observer.is_closed() { observer.complete(); }
    }
}
```

### 3.3 is_closed 是什么？

上游算子（如 `Take(3)`）在收到 3 个值后，会"关闭" observer。`is_closed()` 让 `FromIter` 提前停止循环，不浪费计算。

**推论：任何手写 for 循环发射值，**必须在循环顶加** `if observer.is_closed() { break; }`**。

---

## 第 4 章：Observable trait 和算子模式

### 4.1 算子的三大件结构

每个算子由三部分组成（以 `Map` 为例）：

```rust
// 1️⃣ 结构体：持有 upstream source 和变换函数
struct Map<S, F> { source: S, func: F }

// 2️⃣ Observer wrapper：包装 downstream observer，加上自己的变换
struct MapObserver<O, F> { observer: O, func: F }

impl<O, F, Item, Out, Err> Observer<Item, Err> for MapObserver<O, F>
where O: Observer<Out, Err>, F: FnMut(Item) -> Out {
    fn next(&mut self, v: Item) {
        self.observer.next((self.func)(v));  // 变换后转发
    }
    fn error(self, e: Err) { self.observer.error(e); }
    fn complete(self) { self.observer.complete(); }
    fn is_closed(&self) -> bool { self.observer.is_closed() }
}

// 3️⃣ subscribe 实现：把包装的 observer 传给 upstream
impl<S, F, C, Out> CoreObservable<C> for Map<S, F>
where C: Context,
      S: CoreObservable<C::With<MapObserver<C::Inner, F>>>,
      F: for<'a> FnMut(S::Item<'a>) -> Out {
    type Unsub = S::Unsub;  // 委托给 upstream 的 unsubscribe
    fn subscribe(self, ctx: C) -> S::Unsub {
        let Map { source, func } = self;
        let wrapped = ctx.transform(|obs| MapObserver { observer: obs, func });
        source.subscribe(wrapped)
    }
}
```

### 4.2 模式总结（所有算子遵守）

| 步骤 | 做什么 | 为什么 |
|------|--------|--------|
| 解构 | `let Self { source, field } = self;` | 拆开 source + 配置 |
| 包装 | `ctx.transform(\|obs\| Wrapper { observer: obs, ... })` | 加入自己的变换逻辑 |
| 透传 | `source.subscribe(wrapped)` | 让 upstream 把数据推过来 |

### 4.3 用 Map 变换类型的例子

```rust
// 把 &str 行 → usize 行长度
let lengths: Observable<Item = usize> = lines.map(|line: &str| line.len());

// 把 usize → String 调试说明
let debugs: Observable<Item = String> = lengths.map(|n| format!("len={}", n));
```

**关键规则**：`map` 的闭包**消费**上一个 Item，**产出**新 Item。类型就这么变了。

---

## 第 5 章：scan_map——状态机的反应式表达

### 5.1 签名

```rust
fn scan_map<Acc, Output, F>(self, initial: Acc, f: F) -> Self::With<ScanMap<Self::Inner, F, Acc>>
where F: for<'a> FnMut(&mut Acc, Self::Item<'a>) -> Output
```

**三连要素**：
- `initial: Acc` — 初始状态（编译器入 operator 内部）
- `&mut Acc` — 每步就地消费自身（不用 clone）
- `Output` — 每步产出给下游

### 5.2 编译器的 scan_map 例子

```rust
#[derive(Default)]
struct CompileState {
    line_count: usize,
    indent: usize,
    errors: Vec<String>,
}

let compiled = source
    .scan_map(CompileState::default(), |state: &mut _, line: &str| {
        state.line_count += 1;                    // 🔑 消费自身
        if line.starts_with("/* error") {
            state.errors.push(line.to_string());  // 收集错误
        }
        format!("L{}: {}", state.line_count, line) // Output
    });
```

### 5.3 scan_map vs reduce 对比

```rust
// scan_map：每步都发射，看到中间结果
stream.scan_map(0, |acc, x: &str| {
    *acc += x.len();
    *acc  // 每步都传下去
});
// 输入 ["a","bb","ccc"] → [1, 3, 6]

// reduce：只在 complete 时发射一次
stream.reduce(0, |acc: usize, x: &str| acc + x.len());
// 输入 ["a","bb","ccc"] → [6]
```

**规则**：编译管线几乎总是用 `scan_map`（需要看每步结果），很少用 `reduce`（只看终态）。

### 5.4 为什么 acc 要用 &mut 而不是 clone？

```rust
// ❌ GC 思维：每步 clone 一次整个 State
stream.scan_map(CompileState::default(), |state, x| {
    let mut new_state = state.clone();  // 浪费！
    new_state.update(x);
    (new_state, output)
});

// ✅ 消费自身：&mut 原地修改，零拷贝
stream.scan_map(CompileState::default(), |state: &mut _, x| {
    state.update(x);  // 就地改
    output
});
```

---

## 第 6 章：flat_map——有序展平 Vec<T>

### 6.1 签名

```rust
fn flat_map<F, Inner>(self, f: F) -> Self::With<FlatMap<Self::Inner, F, Inner>>
where F: for<'a> FnMut(Self::Item<'a>) -> Inner,
      Inner: Context<Inner: ObservableType<Err = Self::Err>>
```

**意思**：把上游的每个 Item 映射为一个新的 Observable（Inner），然后按顺序合并所有 Inner 的输出。

### 6.2 编译器管线的实际应用

```rust
source
    .scan_map(CompileState::default(), dispatch_pass)
    // dispatch_pass 返回 Vec<String>（多个 CSS 行）
    .flat_map(|v: Vec<String>| {
        Shared::from_iter(v)  // 把 Vec<String> 展开为逐行 String
    })
```

**为什么需要 flat_map**：`scan_map` 每步产出 `Vec<String>`，下游要的是逐行 `String`。flat_map 把"Vec 的流"展平成"元素的流"。

### 6.3 Inner 必须是一个 Observable

```rust
// ❌ 编译失败：Vec 不是 Observable
.flat_map(|x| vec![x, x+1])

// ✅ Shared 多线程上下文（sasspile 使用此模式）
.flat_map(|x| Shared::from_iter(vec![x, x+1]))
```

---

## 第 7 章：collect 和 last——终端汇聚

### 7.1 collect：把流汇聚为集合，complete 时发射一次

```rust
impl<O, Item, Err, C> Observer<Item, Err> for CollectObserver<O, C>
where O: Observer<C, Err>, C: Extend<Item> {
    fn next(&mut self, value: Item) {
        self.collection.extend(Some(value));  // 累积，不转发
    }
    fn complete(mut self) {
        self.observer.next(self.collection);  // 🔑 完成时一次性发射
        self.observer.complete();
    }
}
```

**注意**：只有流 complete 后才会发射。如果用的是 `Subject` 且从不手动 complete，**collect 永远不会发射**。

### 7.2 last：只保留最后一个值

```rust
impl<O, Item, Err> Observer<Item, Err> for LastObserver<O, Item>
where O: Observer<Item, Err> {
    fn next(&mut self, value: Item) { self.last = Some(value); }  // 覆盖
    fn complete(mut self) {
        if let Some(v) = self.last.take() {
            self.observer.next(v);
        }
        self.observer.complete();
    }
}
```

### 7.3 collect + last 组合

编译器管线的标准出口模式：

```rust
source
    .collect::<Vec<&str>>()   // 汇聚所有行
    .last()                    // 取终态（唯一一次输出）
    .subscribe(move |v: Vec<&str>| {
        let output = v.join("\n");   // 终点：借用 → own
        tx.send(output);
    });
```

### 7.4 什么时候用 collect，什么时候用 last？

| 需求 | 组合 |
|------|------|
| 要所有中间结果 | `.collect::<Vec<_>>().last()` |
| 只要最终结果 | `.collect::<Vec<_>>().last()`（跟上面一样） |
| 条件满足时取最后值 | `.filter(predicate).last()` |
| 取前 N 个 | `.take(n).collect::<Vec<_>>()` |

**规则**：编译器管线**固定用 `collect::<<_>>().last().subscribe()`**，这是唯一正确的终态模式。

---

## 第 8 章：Context 与 Local vs Shared

### 8.1 Context 是什么？

Context 是算子的运行环境，包含：
- **inner**：当前 observer / observable 的核心逻辑
- **scheduler**：控制调度时序（如 delay）

### 8.2 Local vs Shared

| | Local | Shared |
|---|---|---|
| 线程模型 | 单线程 | 多线程 |
| 内部指针 | `Rc<RefCell<T>>` | `Arc<Mutex<T>>` |
| 数据必须 'static | ❌ 不需要 | ✅ 需要 |
| 性能 | 更快（无锁） | 较慢（有锁） |
| Send/Sync | ❌ 非 Send | ✅ Send + Sync |

**对 sasspile 的规则**：**sasspile 用 Shared**，入口必须是 `String`（`'static + Send`）。管线流程实际是同步的（push → complete → terminal 已全部执行），但 Shared 提供更强的线程安全保证。

**为什么不用 Local**：
- Local 使用 `Rc<RefCell<Subscribers>>`，非 `Send`，无法跨线程传递
- Local + `&str` 存在 invariant lifetime 问题，无法与某些算子组合
- Shared 是 rxrust 更通用的选择，`'static` 约束更显式

```rust
// ✅ sasspile 实际使用 Shared
Shared::subject::<String, Infallible>()
Shared::from_iter(vec)
Shared::from_stream(futures::stream::iter(items))
```

### 8.3 Shared 多线程流程（基于源码）

```
Subject<Arc<Mutex<Subscribers>>> → next() 加锁广播 → ScanMapObserver(&mut State)
→ flat_map(MergeAll) → 子流 FromIter 同步迭代 → collect → last → subscribe terminal
```

管线虽然用 Shared，但流程是同步的：所有数据在 `next()` 调用时已同步传播，`complete()` 后 terminal observer 已执行，通过 `std::sync::mpsc::channel` 传递结果。

### 8.4 调度器（Scheduler）

需要 `delay` / `timer` / `interval` 时才涉及。Shared 需要 tokio runtime 才能跑 delay。

**对 sasspile 的规则**：**不用 delay/timer/interval 算子**，因此无需关心调度器。

---

## 第 9 章：Subscription 与生命周期管理

### 9.1 Subscription trait

```rust
trait Subscription {
    fn unsubscribe(self);   // 消费 self，调用后不可再调
    fn is_closed(&self) -> bool;
}
```

**种类**：

| 类型 | 来源 | 什么时候用 |
|------|------|-----------|
| `()` | `Of`, `FromIter` | 同步发射，不可取订 |
| `ClosureSubscription<F>` | `create` 闭包返回值 | 自定义 teardown |
| 复杂的嵌套类型 | `collect/last` 后 | 需要 `.source.source` 拆 |

### 9.2 管线 declare 时期不执行

```rust
// 这一句没有任何代码运行
let pipeline = source.map(..).filter(..).collect::<Vec<_>>().last();

// 直到 subscribe 才执行
pipeline.subscribe(|v| { ... });
```

这是反应式的 **lazy** 特性：chain 是声明，subscribe 是执行边界。

---

## 第 10 章：for<'a> HRTB 的含义

### 10.1 签名中的 for<'a>

```rust
F: for<'a> FnMut(&mut Acc, Self::Item<'a>) -> Output
```

**翻译**：闭包 `F` 必须对**所有**可能的 lifetime `'a` 都有效。

### 10.2 为什么需要它？

因为 Observable 可能在**任何子 lifetime** 被订阅。编译器必须保证：闭包不捕获任何特定 lifetime 的引用——它只操作具体的值类型。

### 10.3 静态分发 vs 动态分发

rxrust 的链式调用在**编译期单态化**，不经过 vtable：

```
.map(|x| x * 2)  编译后 → FilterObserver { func: 闭包 }
.filter(|x| x > 10)          ↓
                             MapObserver { observer: FilterObserver, func: 闭包 }
```

**零运行时开销**——不像 JavaScript 的 RxJS 需要运行时 vtable 查表。

---

## 第 11 章：FROM 迭代器 vs CREATE 自定义

### 11.1 FromIter

```rust
// 把 Iterator 变成 Observable
Local::from_iter(["a", "b", "c"].iter().copied())
//                           ^^^^^^^^^^^^^^^^ &str 迭代器（Local 可用）
//
// Shared 版本: Shared::from_iter(vec!["a".to_string(), ...])
```

**注意**：`FromIter` 内部循环**必须检查 is_closed**（源码已验证）。

### 11.2 Create 自定义 Observable

```rust
// Local 版本
Local::create(|emitter: &mut dyn Emitter<Item, Err>| {
    emitter.next("first");
    emitter.next("second");
    emitter.complete();
    ()  // teardown 逻辑
})

// Shared 版本需 'static
Shared::create(|emitter: &mut dyn Emitter<Item, Err>| {
    emitter.next("first".to_string());
    emitter.next("second".to_string());
    emitter.complete();
    ()
})
```

### 11.3 选择规则

| 场景 | 选择 |
|------|------|
| 已有 Iterator | `from_iter` |
| 需要手动控制发射时机 | `create` |
| 需要从多个来源聚合 | `create` |
| 简单数据注入 | `of` / `from_iter` |

---

## 第 12 章：Subject——手动推数据的入口

### 12.1 创建和分发（Shared 多线程上下文）

```rust
let subject = Shared::subject::<String, Infallible>();
subject.subscribe(|line: String| println!("{}", line));

subject.clone().next("hello".to_string());   // 手动推入
subject.clone().next("world".to_string());
subject.clone().complete();
```

### 12.2 Re-entrancy 禁止

```rust
// ❌ panic：在 subscribe 回调内调用 next
subject.clone().subscribe(|v| {
    subject.next(v);  // re-entrant → panic
});
```

**规则**：**不要在 subscribe 内 next 同一个 Subject**。

### 12.3 Subject 在编译管线中的角色

```rust
let subject = Shared::subject::<String, Infallible>();

// 注入 SCSS 行（to_string() 创建 'static String）
input.lines().for_each(|line| subject.clone().next(line.to_string()));
subject.clone().complete();

// 管线下游
subject
    .scan_map(CompileState::default(), dispatch_pass)
    .flat_map(|v: Vec<String>| Shared::from_iter(v))
    .collect::<Vec<String>>()
    .last()
    .subscribe(|v: Vec<String>| { /* 终态产物 */ });
```

---

## 第 13 章：编译管线完整模式（sasspile 实际编译验证模板）

### 13.1 三阶段结构（Shared 多线程上下文）

```rust
pub fn compile_pipeline(input: &str) -> String {
    let subject = Shared::subject::<String, Infallible>();
    let (tx, rx) = std::sync::mpsc::channel::<String>();

    // 注入 + 驱动
    input.lines().for_each(|line| subject.clone().next(line.to_string()));
    subject.clone().complete();

    // 阶段 1：编译 (CompileState 演化)
    subject.clone()
        .scan_map(CompileState::new(), dispatch_pass)
        .flat_map(|v: Vec<String>| Shared::from_iter(v))

        // 阶段 2：构建 AST (CssBuilder 演化)
        .scan_map(CssBuilder::new(),
                  |builder: &mut CssBuilder, line: String| -> Vec<CssNode> {
                      builder.feed(&line)
                  })
        .flat_map(|v: Vec<CssNode>| Shared::from_iter(v))

        // 汇聚 CssNode
        .collect::<Vec<CssNode>>()
        .last()
        .map(|nodes| post_process(nodes))
        .flat_map(|nodes| Shared::from_iter(nodes))

        // 阶段 3：渲染 + 收集
        .map(|node: CssNode| render_node(&node))
        .collect::<Vec<String>>()
        .last()
        .subscribe(move |v: Vec<String>| {
            let css: String = v.join("\n");  // 终点产物
            let _ = tx.send(css);
        });

    // 同步等待终态产物
    rx.recv().unwrap_or_default()
}
```

### 13.2 流式图

```
input: &str (借用)
   │
   ▼
lines() → for_each → line.to_string() (Shared 入口)
   │
   ▼
Shared::subject::<String, Infallible> (入口)
   │
   ▼
scan_map(CompileState)       ← 消费自身：&mut CompileState
   │ 产出 Vec<String>        ← dispatch_pass 返回值
   ▼
flat_map(Shared::from_iter)  ← Vec<String> → String 逐行
   │
   ▼
scan_map(CssBuilder)         ← 消费自身：&mut CssBuilder, &line 借用传参
   │ 产出 Vec<CssNode>
   ▼
flat_map(Shared::from_iter)  ← Vec<CssNode> → CssNode 逐个
   │
   ▼
collect::<Vec<CssNode>>().last() ← 汇聚所有 CSS AST 节点
   │
   ▼
map(post_process)            ← @media 合并等汇聚后处理
   ▼
flat_map(Shared::from_iter)  ← Vec<CssNode> → CssNode 逐个
   │
   ▼
map(render_node)             ← &CssNode → String（首次出现 owned String）
   │
   ▼
collect::<Vec<String>>().last() ← 汇聚所有终态产物
   │
   ▼
subscribe(tx.send(v.join("\n"))) ← 终点：Vec<String> → String（唯一拥有产物）
   │
   ▼
rx.recv()                    ← 消费 channel，返回最终 String
```

---

## 第 14 章：AI 执行清单（写代码前对照）

### 14.1 管线流动数据能用借用的就用借用

```
- [ ] 输入数据是否能在管线外活？ → 用 &str / &T
- [ ] 是否在管线中间步骤使用 .clone()？ → 删掉，改为借用
- [ ] 中间数据类型是否 String 而非 &str？ → 考虑改 &str
```

### 14.2 状态演化统一用 scan_map + &mut

```
- [ ] 是否有外部 `let mut state` 被闭包捕获？ → 改 scan_map 的 acc
- [ ] acc 是否用了 `&T` / `Rc<RefCell<T>>`？ → 改 `&mut T`
- [ ] 是否在每步 clone acc？ → 删掉，&mut 就地改
```

### 14.3 Chain 只有 subscribe 才执行

```
- [ ] 是否在 chain 中间 collect？ → subscribe 只放最后
- [ ] 是否在 subscribe 内再 subscribe？ → 改 flat_map + 子流
```

### 14.4 终点收集只有一处

```
- [ ] String / Vec 类型是否出现在中间步骤？ → 往后推到 collect/last 后
- [ ] 最终产物是否消费了借用值？ → `v.join("\n")` 而不是逐行 own
```

### 14.5 for<'a> HRTB 不对时

```
- [ ] 闭包是否捕获了特定 lifetime 的引用？ → 改为 move 捕获 owned 值
```

---

## 第 15 章：脚本参考代码（Shared 多线程上下文）

### 15.1 最小可运行示例

```rust
// tests/min_rxrust_test.rs
use rxrust::prelude::*;

#[test]
fn basic_shared_reactive_pipeline() {
    let input = "body { color: red; }";
    let mut collected = vec![];

    Shared::from_iter(input.lines().map(|l| l.to_string()))  // String for Shared
        .map(|line: String| line.trim().to_string())
        .filter(|line: &String| !line.is_empty())
        .scan_map(0_usize,
                  |count: &mut _, line: String| -> String {
                      *count += 1;
                      format!("/* line {} */ {}", count, line)
                  })
        .collect::<Vec<String>>()
        .last()
        .subscribe(|v: Vec<String>| {
            collected = v;
        });

    assert_eq!(collected, vec!["/* line 1 */ body { color: red; }".to_string()]);
}
```

### 15.2 用 scan_map 做 FizzBuzz

```rust
use rxrust::prelude::*;

#[test]
fn fizzbuzz() {
    let mut out = vec![];

    Shared::from_iter((1..=15_i32).map(|n| n.to_string()))  // 'static String
        .scan_map((), |_: &mut (), n: String| -> String {
            let num: i32 = n.parse().unwrap_or(0);
            match (num % 3 == 0, num % 5 == 0) {
                (true, true) => "FizzBuzz".to_string(),
                (true, _) => "Fizz".to_string(),
                (_, true) => "Buzz".to_string(),
                _ => num.to_string(),
            }
        })
        .collect::<Vec<String>>()
        .last()
        .subscribe(|v: Vec<String>| out = v);

    assert_eq!(
        out,
        vec!["1","2","Fizz","4","Buzz","Fizz","7","8","Fizz","Buzz",
             "11","Fizz","13","14","FizzBuzz"]
    );
}
```

### 15.3 flat_map 展开 Vec

```rust
use rxrust::prelude::*;

#[test]
fn flat_map_demo() {
    let mut out = vec![];

    Shared::from_iter(vec![1.to_string(), 2.to_string(), 3.to_string()])
        .scan_map((), |_: &mut (), n: String| -> Vec<String> {
            let num: i32 = n.parse().unwrap_or(0);
            vec![n, (num * 10).to_string(), (num * 100).to_string()]
        })
        .flat_map(|v: Vec<String>| Shared::from_iter(v))
        .collect::<Vec<i32>>()
        .last()
        .subscribe(|v: Vec<i32>| out = v);

    assert_eq!(out, vec![1, 10, 100, 2, 20, 200, 3, 30, 300]);
}
```

---

## 第 16 章：名词速查表

| 名词 | 含义 |
|------|------|
| Observable | 被订阅的数据源，chain 上的中间产物 |
| Observer | 消费 next / error / complete 的下游 |
| Item<'a> | Observable 发射的值的类型（可以带 lifetime） |
| Err | Observable 的 error 类型 |
| CoreObservable | 纯逻辑 kernel，不关心 Local/Shared |
| Context | 执行环境：inner + scheduler |
| Local | 单线程执行，允许非 'static 借用 |
| Shared | 多线程，强制 Send + 'static |
| scan_map | 每步就地累积 + 每步发射 |
| flat_map | 把 Item 展平为多个 Item |
| collect | 汇聚为集合，complete 时发射一次 |
| last | 只发射最后一次产生的值 |
| subscribe | 执行边界，从这里开始真正消费数据 |
| Subject | Observable + Observer 双角色，手动推数据 |
| Unsub | subscribe 返回的取消订阅句柄 |

---

## 第 17 章：思维对照表（命令式 → 反应式）

| 命令式思维 | 反应式思维 |
|-----------|-----------|
| 遍历 Iterator | 声明 Observable + operator |
| `let mut v = vec![]` + `for` + `push` | `collect::<Vec<_>>().last()` |
| `if cond { do_a() } else { do_b() }` | `partition` / `branch` operator |
| `try { ... } catch { ... }` | `on_error` operator |
| `thread + channel` | `Shared` + `Subject` |
| `callback hell` | `flat_map` / `switch_map` |
| `poll` | `subscribe` → push 模型 |
| `break loop` | `take(n)` + `is_closed` |
| `return value` | `reduce` / `last` |

---

## 附录 A：源码定位表（遇到问题时参考）

| 问题 | 文件 | 要点 |
|------|------|------|
| 算子签名 | `src/observable.rs` | `fn scan_map<...>` 40 行附近 |
| Map 实现 | `src/ops/map.rs` | struct + MapObserver + impl |
| ScanMap 实现 | `src/ops/scan_map.rs` | acc 在 Observer 内部 |
| FlatMap 实现 | `src/ops/flat_map.rs` | `MergeAll<Map<S, F>>` |
| Collect 实现 | `src/ops/collect.rs` | complete 时发射集合 |
| FromIter | `src/observable/from_iter.rs` | 必须 is_closed 检查 |
| Create | `src/observable/create.rs` | Emitter 零成本分发 |
| Observer trait | `src/observer.rs` | error(self)/complete(self) 消费 self |
| Subject | `src/subject/` | re-entrancy 禁止 |
| Subscription | `src/subscription.rs` | unsubscribe(self) 消费 self |

源码路径：`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rxrust-1.0.0-rc.5/`

---

## 附录 B：rxrust 数据流生命周期

```
 fn compile(input: &str) -> String {         // input 的 lifetime: 'input
   │
   ├─ input.lines()                           // Iterator<Item = &'input str>
   │   ↓ line.to_string() → String (Shared 入口)
   │
   ├─ Shared::subject::<String, Infallible>()  // Observable<Item = String>
   │   ↓ scan_map(CompileState::new(), dispatch_pass)  ← 消费自身
   │   ├─ per-step: &mut CompileState
   │   └─ output: Vec<String>                 // dispatch_pass 创建
   │
   ├─ flat_map(|v| Shared::from_iter(v))      // Vec<String> → String
   │
   ├─ scan_map(CssBuilder::new(), feed)       ← 消费自身
   │   ├─ per-step: &mut CssBuilder
   │   └─ output: Vec<CssNode>
   │
   ├─ flat_map(|v| Shared::from_iter(v))      // Vec<CssNode> → CssNode
   │
   ├─ map(render_node)                        // &CssNode → String（首次 owner）
   │   └─ Observable<Item = String>
   │
   └─ collect::<Vec<String>>().last()         // 汇聚所有 owned String
       ↓
       subscribe(|v: Vec<String>| v.join("\n"))  // 终点产物：String
 }
```

**关键观察**：
- `&'input str` 从 `input.lines()` 一直到 `map(render)` 才终结
- 消费自身 `&mut State` 一直在累加
- 最小化 owned 数据：中间几乎全借用，只有终态才是 owned

---

## 附录 C：AI 行为六戒

1. 🚫 不要手写 for 循环处理流数据（用 scan_map / collect）
2. 🚫 不要在管线外部持有 mut state 共享给多个闭包
3. 🚫 不要在 collect 之前把 &str 转为 String
4. 🚫 不要在 flat_map 中返回非 Observable 类型
5. 🚫 不要在 subscribe 内再 subscribe
6. 🚫 不要用 Arc<Mutex> / Rc<RefCell> 管理状态
