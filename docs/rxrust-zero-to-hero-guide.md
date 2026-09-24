# rxrust 从零到一：零基础 AI 响应式编程完全指南

> **目标读者**：零 Rust / 零响应式框架经验的 AI Agent
> **目标**：理解每一行代码为什么这样写，能独立构建正确的 rxrust 管线
> **原则**：零简化，每个概念都配完整源码 + 错误示例 + 逐步推理

---

## 目录

0. [心智模型转换：从命令式到响应式](#0)
1. [rxrust 是什么？它解决什么问题？](#1)
2. [核心三件套 Stream / Observer / Subscription 的深度理解](#2)
3. [什么是 Observable？它是数据源还是数据管道？](#3)
4. [创建 Observable 的方式：Subject / from_iter / of / range](#4)
5. [算子（Operators）完整分类与选择决策树](#5)
6. [map：1:1 类型转换的语义与闭包签名](#6)
7. [filter：过滤不等式——FnMut(&T) -> bool](#7)
8. [scan_map：响应式的状态机——&mut 就地修改的秘密](#8)
9. [flat_map：1:N 展平——为什么要返回 Observable](#9)
10. [collect / last / subscribe：终端三部曲](#10)
11. [Local vs Shared——你必须知道的线程模型差异](#11)
12. [Shared 中的 'static 约束——为什么入口必须是 String](#12)
13. [完整管线构建：从 Subject 到 subscribe 的逐步推演](#13)
14. [四大支柱：借引用、消费自身、响应式思想、终点收集](#14)
15. [错误示例 100 例与修复](#15)
16. [调试协议：tracing span 插桩与证据链](#16)
17. [sasspile 实际编译管线深度剖析](#17)
18. [rxrust 源码导读：必须读哪些文件](#18)
19. [提交前自查清单](#19)
20. [常见 AI 行为偏差与纠正](#20)

---

<a id="0"></a>
## 0. 心智模型转换：从命令式到响应式

### 0.1 命令式编程的思维习惯

命令式编程的核心是 **"怎么做"（How）**：

```rust
// 命令式思维：一步步告诉计算机"怎么做"
let mut result = vec![];
for line in input.lines() {
    let expanded = expand_vars(line);
    let tokens = tokenize(expanded);
    result.push(tokens);
}
// result 现在包含了所有数据
```

命令式特点：
1. **立即执行**：代码从上到下立即执行
2. **可变状态反复读写**：`result` 反复 `push`
3. **控制流主导**：`for`、`if`、`match` 控制执行顺序
4. **副作用驱动**：`push` 就是向共享状态写入

### 0.2 响应式编程的思维习惯

响应式编程的核心是 **"是什么关系"（What）**：

```rust
// 响应式思维：声明数据之间的变换关系
let pipeline = source
    .map(expand_vars)      // "map: 每行做变量展开"
    .flat_map(tokenize);    // "flat_map: 每行展开为多个 token"
// 注意：此时还没有执行！ pipeline 只是声明了关系
// 必须 subscribe 才能执行
```

响应式特点：
1. **惰性求值**：定义管道时不执行，subscribe 时才执行
2. **关系声明**：每个算子声明"输入→输出"的关系
3. **数据流驱动**：数据从上游流向下游，每个节点只做一件事
4. **终端触发**：只有 `subscribe`、`collect` 等终端算子才触发实际计算

### 0.3 为什么 AI 容易在响应式代码上犯错

AI 的训练数据 99% 是命令式代码，导致三个典型偏差：

| 偏差 | 表现 | 后果 |
|------|------|------|
| 立即化 | 写了 `stream.map(f)` 就以为 `f` 已经执行了 | 忘记 subscribe，什么也没发生 |
| 共享化 | 用外部 `let mut state` 收集数据 | 编译错误（borrow 检查失败） |
| String 化 | 每一步都 `to_string()` clone | 性能损失，违反"终点收集"原则 |

### 0.4 核心心智转变

**转变前**（命令式）：
```
我要遍历输入 → 对每个元素做 A → 做 B → 收集到 result → 返回 result
```

**转变后**（响应式）：
```
Subject（数据入口）
  → map(A)          // A 变换
  → scan_map(state, B)  // + 状态
  → flat_map(展开)   // 1:N
  → collect/<Vec<_>> // 汇聚
  → subscribe(消费)  // 触发 + 消费
```

**记住**：管线 = 声明关系，subscribe = 触发执行。

---

<a id="1"></a>
## 1. rxrust 是什么？它解决什么问题？

### 1.1 简介

rxrust 是 Rust 生态的 **反应式扩展（Reactive Extensions）** 库。它提供了一套类型安全的、零成本的抽象来构建异步数据流处理管道。

rxrust 1.0.0-rc.5 源码位置：
```
~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rxrust-1.0.0-rc.5/
```

### 1.2 它解决了什么问题？

**问题 1：嵌套回调**
```rust
// 无 rxrust：回调地狱
process_line(line, |tokens| {
    for token in tokens {
        if token.is_valid() {
            render(token, |output| {
                save(output, |success| {
                    // ... 无限嵌套
                });
            });
        }
    }
});
```

**问题 2：手动状态管理**
```rust
// 无 rxrust：手动维护状态 + 边界检查
let mut state = CompileState::default();
let mut outputs = vec![];
for line in input {
    for token in state.process(line) {
        outputs.push(token);
    }
}
// 手动处理 complete 通知、错误传播...
```

**解决**：rxrust 提供 **声明式管道**，自动处理：
- 错误传播
- 完成通知取消
- 背压（backpressure）
- 线程调度

### 1.3 rxrust 的架构分层

```
┌────────────────────────────────────────────┐
│  用户代码（管线）：source.map(f).scan_map(s, g)..  │
├────────────────────────────────────────────┤
│  算子层（src/ops/）：map.rs, scan_map.rs, flat_map.rs │
├────────────────────────────────────────────┤
│  观察者层（src/observer.rs）：Observer trait + 各种包装 │
├────────────────────────────────────────────┤
│  上下文层（src/context.rs）：Local vs Shared + Scheduler │
├────────────────────────────────────────────┤
│  原语层（Subject, Subscription）            │
└────────────────────────────────────────────┘
```

---

<a id="2"></a>
## 2. Core Triad：Stream / Observer / Subscription

### 2.1 Observer trait——rxrust 的"原子单元"

Observer 是 rxrust 中最基础的 trait：

```rust
// src/observer.rs（简化版）
pub trait Observer {
    type Item;   // 观察的元素类型
    type Error;  // 错误类型

    fn next(&mut self, value: Item);      // 接收下一个值
    fn error(self, err: Error);           // 错误通知（self = move 终结）
    fn complete(self);                    // 完成通知（self = move 终结）
}
```

**关键洞察**：
- `next` 是 `&mut self` → 可以被多次调用
- `error` 和 `complete` 是 `self`（按值接收）→ **调用后 Observer 被消费掉，不能再使用**

这就是"终结语义"在类型层面的保证。

### 2.2 Observable——Observer 的对偶

Observer 是"推"的接收端，Observable 是"推"的发送端。

```rust
// src/observable.rs（简化版）
pub trait Observable {
    type Item;
    type Error;

    fn observe<O>(self, observer: O) -> Subscription
    where O: Observer<Item=Item, Error=Error> + 'a;  // 'a 是 Observable 的生命周期
}
```

**等式关系**：
```
Observable<Item=T>  ⟺  存在一种方式，使得observer.next(T) 被调用
```

### 2.3 Subscription——取消令牌

Subscription 代表一个活跃订阅，可以被取消：

```rust
pub struct SubscriptionInner<Mutex> {  // Mutex = MutRc(Shared) 或 MutRc(Local)
    subscribers: Vec<Subscriber<M>>,     // 订阅者列表
    // ...
}

pub type Subscription = Box<dyn SubscriptionLike>;
pub trait SubscriptionLike {
    fn unsubscribe(&mut self);  // 取消订阅
    fn is_closed(&self) -> bool;
}
```

### 2.4 三者关系图

```
    Observable (数据源 + 管道)
         │
         │ .observe(observer)
         ▼
    ┌─────────┐  Self  ┌──────────────┐
    │Observer  │───────►│ Subscription  │  ← 保持存活直到 complete
    │(消费端)  │        │ (取消令牌)    │
    └─────────┘        └──────────────┘
```

---

<a id="3"></a>
## 3. 什么是 Observable？它是数据源还是数据管道？

### 3.1 双重身份

Observable **既可以是数据源，也可以是管道**。这是初学者最容易困惑的地方。

### 3.2 Observable 作为数据源

```rust
// Subject 是一个可手动推送数据的数据源
let subject = Local::subject::<String, Infallible>();
subject.next("hello".to_string());   // 手动推送
subject.next("world".to_string());
subject.complete();                  // 手动完成
```

### 3.3 Observable 作为管道

```rust
// 一个 map 返回的新 Observable 也是 Observable
let source = Local::subject::<i32, Infallible>();
let doubled = source.map(|x: i32| x * 2);  // doubled 是一个新的 Observable
                                           // 但它不是数据源！
                                           // 它的数据来自 source

// 这就是"管道"：doubled 订阅 source，对每个元素做 *2
```

### 3.4 管道链是"声明"，不是"执行"

```rust
let pipeline = source
    .map(|x: i32| x * 2)      // 创建一个新的 Observable，但没有执行
    .filter(|x: &i32| x > &10) // 又创建一个新的 Observable
    .scan_map(vec![], |acc: &mut Vec<i32>, x: i32| {  // 再创建
        acc.push(x);
        acc.clone()
    });

// 此时还没有任何计算发生！pipeline 只是描述了一个计算图

// 只有 subscribe 才触发实际执行
pipeline.subscribe(|x: Vec<i32>| {
    tracing::info!(?x, "收到值");
});
```

### 3.5 数据流动方向

```
Subject (手动推送)
  │
  ▼ next()
map 包装的 Observer  →  ×2 后调用 downstream.next()
  │
  ▼ next()
filter 包装的 Observer → 如果 >10 则调 downstream.next()，否则丢弃
  │
  ▼ next()
scan_map 包装的 Observer → 累积到 Vec，调 downstream.next(acc.clone())
  │
  ▼ subscribe 注册的终端闭包
terminal consumer → 最终消费
```

### 3.6 核心等式

```
source.map(f).subscribe(g)

等价于：

subject.observe(MapObserver { f, downstream: Box::new(terminal) })
其中 MapObserver::next 实现为：
    fn next(&mut self, value: T) {
        let mapped = (self.f)(value);
        self.downstream.next(mapped);
    }
```

这就是 rxrust 的本质：**每个算子返回一个包装了下游 Observer 的新 Observer**。

---

<a id="4"></a>
## 4. 创建 Observable 的方式

### 4.1 Subject：手动推送的数据源

Subject 是最常用的数据源，用于将非 rxrust 代码接入管线：

```rust
use rxrust::prelude::*;
use rxrust::subject::Subject;

// Shared 版本（多线程，需要 'static + Send）
let subject: SharedSubject<String, Infallible> = Shared::subject::<String, Infallible>();

// SharedSubject 是 Subject<Arc<Mutex<Vec<SharedSubscriber<T, E>>>>
// 需要 clone 后使用
subject.next("line 1".to_string());   // clonenext
subject.next("line 2".to_string());
subject.complete();                   // complete
```

### 4.2 of / from_iter：惰性迭代器源

```rust
// of：创建发射固定值的 Observable
let source = Shared::of(42);  // 发射单个值 42 然后 complete

// from_iter：将迭代器转为 Observable
let source = Shared::from_iter(vec![1, 2, 3, 4, 5]);  // 发射 1,2,3,4,5 后 complete
```

### 4.3 range：数值范围源

```rust
// 发射 0..5 然后 complete
let source = Shared::range(0..5);
```

### 4.4 empty / never / error

```rust
let empty = Shared::empty::<i32, Infallible>();  // 立即 complete，不发射值
let never = Shared::never::<i32, Infallible>();  // 永远不 complete
let err = Shared::error::<i32, MyError>(MyError::new());  // 立即 error
```

### 4.5 为什么 sasspile 用 Subject 而不用 from_iter？

```rust
// sasspile 的驱动模式：
let subject = Shared::subject::<String, Infallible>();
let (tx, rx) = std::sync::mpsc::channel::<String>();

// ... 构建管线 ...

// 驱动阶段：逐行注入
input.lines().for_each(|line| subject.clone().next(line.to_string()));
subject.clone().complete();

// 同步等待终端结果
let result = rx.recv().unwrap_or_default();
```

原因：
1. **同步获取结果**：管线实际是同步推入的，mpsc 的 `recv()` 可以阻塞等待
2. **控制推送时机**：可以一行一行地推送（虽然这里是一口气推完）
3. **complete 语义**：Subject.complete() 会触发 collect 的汇聚发射

---

<a id="5"></a>
## 5. 算子完整分类与选择决策树

### 5.1 算子决策树

```
需要转换类型吗？
├── 是，1:1 转换 → map
├── 是，1:N 转换 → flat_map
├── 需要累积状态？
│   ├── 是，累积 + 每步发射 → scan_map
│   └── 否，过滤掉某些值 → filter
└── 是终端操作？
    ├── 汇聚为集合 → collect
    ├── 只取最后一个 → last
    ├── 开始消费 → subscribe
    └── 取前 N 个 → take
```

### 5.2 算子完整签名速查

```rust
// 1:1 转换
fn map<B, F>(self, f: F) -> Map<Self, F>
where F: FnMut(Item) -> B  // 每来一个 Item，产生一个 B

// 过滤（不改变类型）
fn filter<F>(self, f: F) -> Filter<Self, F>
where F: FnMut(&Item) -> bool  // 返回 true 则通过，false 则丢弃

// 带状态的累积输出
fn scan_map<Acc, F, Output>(self, initial: Acc, f: F) -> ScanMap<Self, Acc, F>
where F: FnMut(&mut Acc, Item) -> Output  // &mut Acc 就地修改！

// 1:N 展平
fn flat_map<B, F, Inner>(self, f: F) -> FlatMap<Self, F, Inner>
where F: FnMut(Item) -> Inner, Inner: Observable  // 来一个 Item，产生一个 Inner Observable

// 终端：汇聚
fn collect<C>(self) -> Collect<Self, C>
where C: Extend<Item>  // C = Vec<T> 或 HashSet<T> 等

// 终端：只保留最后一个
fn last(self) -> Last<Self>

// 终端：消费
fn subscribe<F>(self, f: F) -> Subscription
where F: FnMut(Item)  // 每来一个值执行一次 f
```

### 5.3 关键类型约束 Summary

| 算子 | 闭包签名 | 闭包约束 | 返回 |
|------|---------|---------|------|
| map | `FnMut(Item) -> B` | Item 可以是引用或值 | impl Observable |
| filter | `FnMut(&Item) -> bool` | 必须借用 | impl Observable |
| scan_map | `FnMut(&mut Acc, Item) -> Output` | Acc 就地修改 | impl Observable |
| flat_map | `FnMut(Item) -> Inner: Observable` | Inner 必须是 Observable | impl Observable |
| collect | 无闭包 | 终端 | impl Observable |
| last | 无闭包 | 终端 | impl Observable |
| subscribe | `FnMut(Item)` | 终端 | Subscription |

### 5.4 算子的"冷"特性

**重要**：rxrust 的 Observable 是 **冷（cold）的**——每次 subscribe 都会重新从头执行整个管线：

```rust
let pipeline = source.map(|x| x * 2).filter(|x| x > &10);

// 第一次 subscribe
pipeline.subscribe(|x| tracing::info!("A: {}", x));

// 第二次 subscribe -> 重新执行！
pipeline.subscribe(|x| tracing::info!("B: {}", x));
```

这与"热"Observable（如 broadcast Subject）不同——热 Observable 的所有订阅者共享同一份数据流。

---

<a id="6"></a>
## 6. map：1:1 类型转换

### 6.1 签名解读

```rust
fn map<B, F>(self, f: F) -> impl Observable<Item=B, Error=Self::Error>
 where F: FnMut(Self::Item) -> B
```

三个要点：
1. `FnMut`：闭包可以捕获 &mut 环境（但不是必须的）
2. `Item -> B`：输入一个旧 Item，输出一个新类型 B
3. `impl Observable<Item=B>`：返回的新 Observable 的 Item 类型是 B

### 6.2 完整示例

```rust
use rxrust::prelude::*;

let subject = Shared::subject::<i32, Infallible>();
let (tx, rx) = std::sync::mpsc::channel::<String>();

subject.clone()
    .map(|x: i32| x * 2)              // i32 -> i32
    .map(|x: i32| format!("val={}", x))  // i32 -> String
    .collect::<Vec<String>>()
    .last()
    .subscribe(move |v: Vec<String>| {
        let _ = tx.send(v.join(","));
    });

subject.next(1);
subject.next(2);
subject.next(3);
subject.complete();

let result = rx.recv().unwrap_or_default();
assert_eq!(result, "val=2,val=4,val=6");
```

### 6.3 map 中的借用规则

```rust
// ✅ 正确：map 闭包接收 owned Item，可以从中借用
.map(|s: String| s.len())  // s 是 owned String，s.len() 借用后返回 usize

// ✅ 正确：map 闭包接收 owned Item，可以返回引用（如果生命周期允许）
.map(|s: String| s.as_str())  // ❌ 不行！返回引用会悬垂

// ✅ 正确：map 闭包接收 owned Item，返回 owned 值
.map(|s: String| s.to_uppercase())  // 返回新的 owned String

// ❌ 错误：试图在 map 闭包内修改外部状态
let mut count = 0;
.map(|s: String| { count += 1; s })  // 编译错误！FnMut 闭包不能捕获 &mut 外部变量
                                     // 因为 map 可能被多线程调用
```

### 6.4 map vs scan_map 的选择

```
需要累积状态吗？
├── 否 → map（无状态，纯函数式转换）
└── 是 → scan_map（有状态，&mut Acc 就地修改）
```

---

<a id="7"></a>
## 7. filter：过滤不等式

### 7.1 签名解读

```rust
fn filter<F>(self, f: F) -> impl Observable<Item=Self::Item, Error=Self::Error>
 where F: FnMut(&Self::Item) -> bool
```

关键：`&Self::Item` —— filter 闭包**只借用**，不消费 Item。

### 7.2 完整示例

```rust
let subject = Shared::subject::<i32, Infallible>();
let (tx, rx) = std::sync::mpsc::channel::<Vec<i32>>();

subject.clone()
    .filter(|x: &i32| x > &10)  // 借用 &i32，返回 bool
    .filter(|x: &i32| x % 2 == 0)  // 链式过滤
    .collect::<Vec<i32>>()
    .last()
    .subscribe(move |v| { let _ = tx.send(v); });

subject.next(5);   // 被第一个 filter 过滤
subject.next(12);  // 通过
subject.next(15);  // 被第二个 filter 过滤
subject.next(20);  // 通过
subject.complete();

let result = rx.recv().unwrap_or_default();
assert_eq!(result, vec![12, 20]);
```

### 7.3 filter 不改变类型

filter 的输入和输出类型相同：
```rust
// filter 前: Observable<Item=i32>
// filter 后: Observable<Item=i32>  （类型不变！）
```

### 7.4 filter 与 map 的链式组合

```rust
// 先过滤再转换（更高效：只对通过过滤的元素做转换）
.filter(|x: &i32| x > &0)
.map(|x: i32| x * 2)

// 先转换再过滤（可能浪费计算：对负数也做了 *2）
.map(|x: i32| x * 2)
.filter(|x: &i32| x > &0)
```

---

<a id="8"></a>
## 8. scan_map：响应式的状态机

### 8.1 签名解读

```rust
fn scan_map<Acc, F, Output>(
    self,
    initial: Acc,
    f: F
) -> impl Observable<Item=Output, Error=Self::Error>
 where F: FnMut(&mut Acc, Self::Item) -> Output
```

三个要点：
1. `initial: Acc`：初始状态值
2. `&mut Acc`：闭包内可以**就地修改**状态
3. `-> Output`：每步可以发射一个 Output 值

### 8.2 为什么 scan_map 是"消费自身"的体现

```rust
// scan_map 的 &mut Acc 模式：
// - Acc 的所有权在 rxrust 框架内部
// - 闭包每次被调用时，拿到同一个 Acc 的 &mut 引用
// - 闭包可以就地修改 Acc，无需 clone，无需外部共享

.scan_map(CompileState::default(), |state: &mut CompileState, token: String| -> Vec<String> {
    state.line_count += 1;      // ✅ 就地修改，零 clone
    state.dispatch(token)       // 返回 Vec<String>
})
```

### 8.3 scan_map 的内部实现原理

```rust
// src/ops/scan_map.rs（简化版）
pub struct ScanMapObserver<Acc, F, Downstream> {
    acc: Acc,           // 状态存储在 Observer 内部
    f: F,               // 闭包
    downstream: Downstream,  // 下游 Observer
}

impl<Acc, F, Output, Downstream> Observer for ScanMapObserver<Acc, F, Downstream> {
    type Item = Input;
    type Error = Downstream::Error;

    fn next(&mut self, value: Input) {
        let output = (self.f)(&mut self.acc, value);  // 调用闭包，传入 &mut Acc
        self.downstream.next(output);  // 将 Output 推给下游
    }
}
```

**关键**：`acc` 存储在 Observer 结构体内部，每次 `next` 调用时通过 `&mut self.acc` 传给闭包。这就是"消费自身"——状态在 Observer 内部演化，外部无法访问。

### 8.4 完整示例：行号计数器

```rust
use rxrust::prelude::*;

struct LineCounter {
    count: usize,
    indent: usize,
}

let subject = Shared::subject::<String, Infallible>();
let (tx, rx) = std::sync::mpsc::channel::<Vec<String>>();

subject.clone()
    .scan_map(LineCounter { count: 0, indent: 0 },
        |state: &mut LineCounter, line: String| -> Vec<String> {
            state.count += 1;
            let indent = "  ".repeat(state.indent);
            let result = format!("{}L{}: {}", indent, state.count, line);

            // 跟踪缩进
            if line.contains('{') { state.indent += 1; }
            if line.contains('}') { state.indent = state.indent.saturating_sub(1); }

            vec![result]  // 每行产生一个输出
        })
    .flat_map(|v: Vec<String>| Shared::from_iter(v))
    .collect::<Vec<String>>()
    .last()
    .subscribe(move |v| { let _ = tx.send(v); });

subject.next(".foo {".to_string());
subject.next("  color: red;".to_string());
subject.next("}".to_string());
subject.complete();

let result = rx.recv().unwrap_or_default();
assert_eq!(result, vec![
    "L1: .foo {",
    "  L2:   color: red;",
    "L3: }",
]);
```

### 8.5 scan_map 与 fold 的区别

```
scan_map: 每步都发射 Output → 流中每个元素都可能产生输出
fold:     只在 complete 时发射最终状态 → 只有一个输出
```

### 8.6 scan_map 的 Acc 类型选择

```rust
// 简单状态：用基本类型
.scan_map(0usize, |count: &mut usize, x: i32| -> i32 {
    *count += 1;
    x + *count as i32
})

// 复杂状态：用 struct
.scan_map(CompileState::default(), dispatch_pass)

// 集合状态：用 Vec/HashMap
.scan_map(HashMap::new(), |map: &mut HashMap<String, i32>, kv: (String, i32)| {
    map.insert(kv.0, kv.1);
    map.clone()  // 注意：如果 Acc 是集合，Output 通常需要 clone
})
```

---

<a id="9"></a>
## 9. flat_map：1:N 展平

### 9.1 签名解读

```rust
fn flat_map<B, F, Inner>(
    self,
    f: F
) -> impl Observable<Item=B, Error=Self::Error>
 where F: FnMut(Self::Item) -> Inner,
       Inner: Observable<Item=B, Error=Self::Error>
```

关键：
1. 闭包返回的不是 B，而是 `Inner: Observable`
2. Inner Observable 会被"订阅"并"合并"到主管线中
3. 这就是"1:N 展平"——来一个 Item，产生多个值

### 9.2 flat_map 的内部实现

```rust
// src/ops/flat_map.rs（简化版）
// flat_map = map + merge_all
// 1. map: 将 Item 转为 Inner Observable
// 2. merge_all: 将所有 Inner Observable 合并为一个

pub struct FlatMapObserver<F, Downstream> {
    f: F,
    downstream: Downstream,
    // 内部维护一个"活跃 Inner Observable"列表
}

impl<F, Inner, Downstream> Observer for FlatMapObserver<F, Downstream> {
    fn next(&mut self, value: Input) {
        let inner = (self.f)(value);  // 产生 Inner Observable
        // 订阅 Inner，将其值转发给 downstream
        inner.observe(MergeAllObserver { downstream: self.downstream });
    }
}
```

### 9.3 完整示例：一行展开为多行

```rust
use rxrust::prelude::*;

let subject = Shared::subject::<String, Infallible>();
let (tx, rx) = std::sync::mpsc::channel::<Vec<String>>();

subject.clone()
    // 输入: "a,b,c"  输出: ["a", "b", "c"]
    .flat_map(|line: String| {
        let parts: Vec<String> = line.split(',')
            .map(|s| s.trim().to_string())
            .collect();
        Shared::from_iter(parts)  // Vec<String> → Observable
    })
    .collect::<Vec<String>>()
    .last()
    .subscribe(move |v| { let _ = tx.send(v); });

subject.next("foo, bar, baz".to_string());
subject.next("hello, world".to_string());
subject.complete();

let result = rx.recv().unwrap_or_default();
assert_eq!(result, vec!["foo", "bar", "baz", "hello", "world"]);
```

### 9.4 flat_map 的顺序保证

**重要**：在 Shared 上下文中，flat_map 使用 `MergeAll` 策略：
- 第一个 Inner Observable 完全完成后，才开始第二个
- 因此**顺序是有保证的**

```
输入: [A, B]
A → flat_map → [A1, A2, A3]
B → flat_map → [B1, B2]

输出顺序: A1, A2, A3, B1, B2  （顺序保证）
```

### 9.5 flat_map 与 map 的选择

```
闭包返回什么？
├── 单个值（T） → map
└── 集合/迭代器/另一个 Observable → flat_map
```

### 9.6 常见错误：flat_map 闭包返回非 Observable

```rust
// ❌ 错误：flat_map 闭包必须返回 Observable
.flat_map(|x: String| x.split(',').collect::<Vec<_>>())  // 返回 Vec，不是 Observable

// ✅ 正确：用 Shared::from_iter 包装
.flat_map(|x: String| Shared::from_iter(x.split(',').map(|s| s.to_string()).collect::<Vec<_>>()))

// ✅ 正确：分两步写
.flat_map(|x: String| {
    let parts: Vec<String> = x.split(',').map(|s| s.trim().to_string()).collect();
    Shared::from_iter(parts)
})
```

---

<a id="10"></a>
## 10. collect / last / subscribe：终端三部曲

### 10.1 collect：汇聚为集合

```rust
fn collect<C>(self) -> impl Observable<Item=C, Error=Self::Error>
 where C: Extend<Self::Item> + Default
```

**行为**：
- 内部维护一个 `C` 类型的集合
- 每来一个 Item，调用 `collection.extend_one(item)`
- **只在 complete 时**发射最终的集合

```rust
let subject = Shared::subject::<i32, Infallible>();

subject.clone()
    .filter(|x: &i32| x > &5)
    .collect::<Vec<i32>>()  // 只在 complete 时发射 Vec<i32>
    .subscribe(|v: Vec<i32>| {
        tracing!(?v, "收到汇聚结果");
    });

subject.next(3);   // 被过滤
subject.next(7);   // 收集到 Vec
subject.next(10);  // 收集到 Vec
subject.complete();  // 此时才发射 vec![7, 10]
```

### 10.2 last：只保留最后一个值

```rust
fn last(self) -> impl Observable<Item=Self::Item, Error=Self::Error>
```

**行为**：
- 记住最后一个收到的值
- complete 时发射那个值
- 如果没有收到任何值，不发射

```rust
let subject = Shared::subject::<Vec<String>, Infallible>();

subject.clone()
    .collect::<Vec<String>>()  // 发射 Vec<String>
    .last()                     // 只保留最后一个 Vec<String>
    .subscribe(|v: Vec<String>| {
        // 这里只会收到一次：最后一次 collect 的结果
    });
```

### 10.3 subscribe：执行边界

```rust
fn subscribe<F>(self, f: F) -> Subscription
 where F: FnMut(Self::Item)
```

**行为**：
- 这是**终端操作**——从这里开始，整个管线被"激活"
- 返回 Subscription handle（必须保持存活）
- 闭包每收到一个 Item 就执行一次

### 10.4 三者的组合模式

```rust
// 模式 1：collect + subscribe（收集所有值后一次性消费）
.collect::<Vec<T>>()
.subscribe(|v: Vec<T>| { /* 处理所有值 */ });

// 模式 2：collect + last + subscribe（多次汇聚，只取最后一次）
.collect::<Vec<T>>()
.last()
.subscribe(|v: Vec<T>| { /* 只处理最后一次汇聚 */ });

// 模式 3：直接 subscribe（逐值消费）
.subscribe(|x: T| { /* 每来一个值处理一次 */ });
```

### 10.5 subscribe 的 move 语义

```rust
let (tx, rx) = std::sync::mpsc::channel::<String>();

// subscribe 闭包必须是 move 的，因为它可能比当前函数活得更久
.subscribe(move |v: Vec<String>| {
    let _ = tx.send(v.join("\n"));  // tx 被 move 进闭包
});

// 这里 tx 已经被 move，不能再使用
// 只能通过 rx.recv() 获取结果
```

---

<a id="11"></a>
## 11. Local vs Shared——线程模型差异

### 11.1 核心区别

| 特性 | Local | Shared |
|------|-------|--------|
| 内部可变性 | `Rc<RefCell<T>>` | `Arc<Mutex<T>>` |
| 线程安全 | ❌ 单线程 | ✅ 多线程 |
| 性能 | 更快（无锁） | 稍慢（有锁） |
| 类型约束 | 无 'static 要求 | 需要 'static + Send |
| 适用场景 | 单线程应用 | 多线程/跨线程传递 |

### 11.2 为什么 sasspile 必须用 Shared

```rust
// sasspile 的管线需要在多线程环境中运行：
// 1. Subject 可能在主线程推送数据
// 2. 管线内部算子可能在另一个线程执行
// 3. 终端 subscribe 闭包可能在另一个线程执行

// 如果用 Local：
let subject = Local::subject::<String, Infallible>();
// ❌ 编译错误：Local 的 Subject 不能跨线程传递
// 因为 Rc 不是 Send
```

### 11.3 Shared 的代价

```rust
// Shared 需要 'static + Send
// 所以不能传 &str（除非是 &'static str）
let subject = Shared::subject::<String, Infallible>();  // ✅ String: 'static + Send
let subject = Shared::subject::<&'static str, Infallible>();  // ✅ 'static 引用
// let subject = Shared::subject::<&str, Infallible>();  // ❌ 非 'static 引用
```

### 11.4 Shared 的内部流程

```
SharedSubject<Arc<Mutex<Vec<SharedSubscriber<T, E>>>>> {
    subscribers: Arc<Mutex<Vec<SharedSubscriber>>>
}

next(value):
    1. 锁定 Arc<Mutex>
    2. 遍历所有 SharedSubscriber
    3. 对每个 subscriber 调用 raw_next(value.clone())
    4. 解锁

observe(subscriber):
    1. 锁定 Arc<Mutex>
    2. 将 subscriber 加入列表
    3. 解锁
    4. 返回 Subscription（用于后续移除）
```

---

<a id="12"></a>
## 12. Shared 中的 'static 约束

### 12.1 为什么入口必须是 String

```rust
// Shared 的 Subject 定义：
pub type SharedSubject<T, E> = Subject<T, E, MutArc<SharedCtx>>;
// MutArc = Arc<Mutex<...>>
// Arc<T> 要求 T: Send + 'static

// 所以 SharedSubject<T> 要求 T: Send + 'static
// String: 'static + Send ✅
// &str: 不是 'static ❌
```

### 12.2 中间步骤的借用

虽然入口必须是 String，但中间步骤可以借用：

```rust
.scan_map(CssBuilder::new(), |builder: &mut CssBuilder, line: String| -> Vec<CssNode> {
    builder.feed(&line)  // &line 借用 String 作为 &str
})
```

这里 `line` 是 `String`（因为 Shared 要求），但 `feed` 接收 `&str`，所以 `&line` 自动解引用为 `&str`。

### 12.3 终点收集时才产生 owned String

```rust
// 中间步骤：借用 &CssNode
.map(|node: CssNode| render_node(&node))  // render_node(&CssNode) -> String

// 终点：collect 汇聚为 Vec<String>
.collect::<Vec<String>>()
.last()
.subscribe(move |css_vec: Vec<String>| {
    let _ = tx.send(css_vec.join("\n"));  // 唯一的 owned String 产生点
});
```

---

<a id="13"></a>
## 13. 完整管线构建：从 Subject 到 subscribe

### 13.1 逐步推演 sasspile 管线

```rust
// 步骤 1：创建入口 Subject
let subject = Shared::subject::<String, Infallible>();
// SharedSubject<String, Infallible> — 可以手动推送 String

// 步骤 2：创建结果通道
let (tx, rx) = std::sync::mpsc::channel::<String>();
// tx 将被 move 进 subscribe 闭包
// rx 将在主线程阻塞等待结果

// 步骤 3：构建管线（此时不执行！）
let subscription = subject.clone()
    // Phase 1: 状态机处理
    .scan_map(CompileState::new(), dispatch_pass)
    // 展平 Vec<String> 为逐个 String
    .flat_map(|v: Vec<String>| Shared::from_iter(v))
    // Phase 2: CSS AST 构建
    .scan_map(CssBuilder::new(), |builder: &mut CssBuilder, line: String| -> Vec<CssNode> {
        builder.feed(&line)
    })
    // 展平 Vec<CssNode> 为逐个 CssNode
    .flat_map(|v: Vec<CssNode>| Shared::from_iter(v))
    // 汇聚所有 CssNode
    .collect::<Vec<CssNode>>()
    .last()
    // @media 合并
    .map(|nodes| merge_media_nodes(nodes))
    // 再展平
    .flat_map(|nodes| Shared::from_iter(nodes))
    // Phase 3: 借用渲染
    .map(|node: CssNode| render_node(&node))
    // 汇聚所有 String
    .collect::<Vec<String>>()
    .last()
    // 终端：触发执行
    .subscribe(move |css_vec: Vec<String>| {
        let _ = tx.send(css_vec.join("\n"));
    });

// 步骤 4：驱动管线
input.lines().for_each(|line| subject.clone().next(line.to_string()));
subject.clone().complete();

// 步骤 5：等待结果
let result = rx.recv().unwrap_or_default();
```

### 13.2 管线执行时间线

```
时间 ──────────────────────────────────────────────────────►

主线程:    [构建管线] [推送所有行] [complete] [阻塞等待 rx.recv()]
                          │           │
                          ▼           ▼
管线内部:              [Phase1处理] [Phase2处理] [Phase3处理] [tx.send]
                                                        │
                                                        ▼
主线程:                                            [收到结果，继续]
```

### 13.3 为什么管线是"同步"的

虽然 Shared 支持多线程，但 sasspile 的管线实际是同步的：
1. 所有 `next()` 调用在主线程顺序执行
2. 每个 `next()` 触发整个管线链的同步求值
3. `complete()` 触发 collect 的汇聚发射
4. `subscribe` 闭包在 `complete()` 的调用栈中被调用
5. `tx.send()` 将结果放入通道
6. 主线程的 `rx.recv()` 拿到结果

---

<a id="14"></a>
## 14. 四大支柱

### 14.1 支柱 1：借引用（&T / &mut Acc）

**原则**：中间步骤尽量借用，不 clone。

```rust
// ✅ 借引用
.scan_map(CssBuilder::new(), |builder: &mut CssBuilder, line: String| -> Vec<CssNode> {
    builder.feed(&line)  // &line 借用 String 作为 &str
})
.map(|node: CssNode| render_node(&node))  // render_node 借用 &CssNode

// ❌ 不必要的 clone
.map(|node: CssNode| render_node(&node.clone()))  // 无意义的 clone
```

### 14.2 支柱 2：消费自身（&mut Acc）

**原则**：状态演化用 `scan_map` 的 `&mut Acc` 就地修改。

```rust
// ✅ 消费自身：状态在 Observer 内部演化
.scan_map(CompileState::new(), |state: &mut CompileState, token: String| -> Vec<String> {
    state.line_count += 1;      // 就地修改
    state.dispatch(token)
})

// ❌ 外部可变（编译错误）
let mut state = CompileState::new();
stream.map(|x| { state.update(x); ... })  // borrow 冲突
```

### 14.3 支柱 3：响应式思想（chain=声明，subscribe=执行）

**原则**：管线是声明式关系，subscribe 是执行边界。

```rust
// ✅ 先声明关系
let pipeline = source
    .scan_map(CompileState::new(), dispatch_pass)
    .flat_map(|v: Vec<String>| Shared::from_iter(v))
    .scan_map(CssBuilder::new(), |b: &mut CssBuilder, line: String| -> Vec<CssNode> {
        b.feed(&line)
    })
    .flat_map(|v: Vec<CssNode>| Shared::from_iter(v))
    .collect::<Vec<CssNode>>()
    .last()
    .map(|nodes| merge_media_nodes(nodes))
    .flat_map(|nodes| Shared::from_iter(nodes))
    .map(|node: CssNode| render_node(&node))
    .collect::<Vec<String>>()
    .last();

// 后执行
pipeline.subscribe(move |css_vec: Vec<String>| {
    let _ = tx.send(css_vec.join("\n"));
});
```

### 14.4 支柱 4：终点收集（→ String）

**原则**：中间步骤借用，最终结果必须是 owned String。

```rust
// ✅ 终点收集
.collect::<Vec<String>>()
.last()
.subscribe(move |css_vec: Vec<String>| {
    let _ = tx.send(css_vec.join("\n"));  // 唯一的 owned String 产生点
});

// ❌ 中间步骤产生 owned String
.map(|node: CssNode| {
    let owned = format!("{:?}", node);  // 不必要的中间 String
    owned
})
```

---

<a id="15"></a>
## 15. 错误示例 100 例与修复

### 15.1 错误类别 1：忘记 subscribe

```rust
// ❌ 错误：写了管线但没有 subscribe
let pipeline = source.map(|x: i32| x * 2);
// 什么也没发生！pipeline 只是声明，没有执行

// ✅ 修复
source.map(|x: i32| x * 2).subscribe(|x| tracing::info!("{}", x));
```

### 15.2 错误类别 2：外部可变状态

```rust
// ❌ 错误：试图在闭包内修改外部状态
let mut results = vec![];
source.map(|x: i32| { results.push(x * 2); x }).subscribe(|_| {});
// 编译错误：FnMut 闭包不能捕获 &mut results（多线程不安全）

// ✅ 修复：用 scan_map
source.scan_map(vec![], |acc: &mut Vec<i32>, x: i32| {
    acc.push(x * 2);
    acc.clone()
}).subscribe(|v| tracing::info!(?v));
```

### 15.3 错误类别 3：flat_map 返回非 Observable

```rust
// ❌ 错误
.flat_map(|x: String| x.split(',').collect::<Vec<_>>())

// ✅ 修复
.flat_map(|x: String| Shared::from_iter(x.split(',').map(|s| s.to_string()).collect::<Vec<_>>()))
```

### 15.4 错误类别 4：在 map 中返回引用

```rust
// ❌ 错误：返回悬垂引用
.map(|s: String| s.as_str())  // s 在闭包结束时被 drop，返回的 &str 悬垂

// ✅ 修复：返回 owned 值
.map(|s: String| s.to_uppercase())
```

### 15.5 错误类别 5：忘记 move 捕获

```rust
// ❌ 错误：subscribe 闭包可能比 tx 活得更久
let (tx, rx) = std::sync::mpsc::channel::<String>();
source.subscribe(|x: String| {
    let _ = tx.send(x);  // tx 被借用，但 subscribe 要求 'static
});

// ✅ 修复
source.subscribe(move |x: String| {
    let _ = tx.send(x);  // tx 被 move 进闭包
});
```

### 15.6 错误类别 6：Shared 用 &str

```rust
// ❌ 错误：Shared 需要 'static
let subject = Shared::subject::<&str, Infallible>();

// ✅ 修复
let subject = Shared::subject::<String, Infallible>();
```

### 15.7 错误类别 7：from_iter 无 is_closed 检查

```rust
// ❌ 错误：from_iter 内部循环不检查 is_closed
// 如果下游取消订阅，循环仍在继续

// ✅ 修复：rxrust 的 from_iter 内部已处理
// 但如果你自己实现 Observable，必须检查
fn observe<O>(self, observer: O) -> Subscription
where O: Observer {
    for item in self.iter {
        if observer.is_closed() { break; }  // 必须检查！
        observer.next(item);
    }
    observer.complete();
}
```

### 15.8 错误类别 8：scan_map 闭包签名错误

```rust
// ❌ 错误：scan_map 闭包第一个参数是 &mut Acc，不是 Acc
.scan_map(CompileState::new(), |state: CompileState, token: String| -> Vec<String> {
    // 编译错误：需要 &mut CompileState
})

// ✅ 修复
.scan_map(CompileState::new(), |state: &mut CompileState, token: String| -> Vec<String> {
    state.line_count += 1;
    state.dispatch(token)
})
```

### 15.9 错误类别 9：管线类型不匹配

```rust
// ❌ 错误：flat_map 返回的 Inner 的 Item 类型与下游不匹配
.scan_map(CompileState::new(), dispatch_pass)  // 输出 Vec<String>
.flat_map(|v: Vec<String>| Shared::from_iter(v))  // 输出 String
.scan_map(CssBuilder::new(), |b: &mut CssBuilder, line: String| -> Vec<CssNode> {
    // ✅ 这里 line 是 String，匹配
    b.feed(&line)
})
```

### 15.10 错误类别 10：忘记 complete

```rust
// ❌ 错误：推送了数据但忘记 complete
subject.next("line 1".to_string());
subject.next("line 2".to_string());
// 没有 complete！collect 永远不会发射！

// ✅ 修复
subject.next("line 1".to_string());
subject.next("line 2".to_string());
subject.complete();  // 必须 complete 才能触发 collect 发射
```

---

<a id="16"></a>
## 16. 调试协议：tracing span 插桩与证据链

### 16.1 强制规则

1. **禁止凭直觉猜测根因**：所有 bug 修复必须基于 tracing trace 证据链
2. **SPAN 插桩**：疑似路径每个入口/出口加 span
3. **TRACE 采集**：`RUST_LOG=trace cargo test -- --nocapture` 收集证据
4. **根因定位**：必须引用具体 span + 字段值
5. **修复验证**：修复后清理临时 span

### 16.2 span 插桩模式

```rust
use tracing::info_span;

fn dispatch_pass(state: &mut CompileState, token: String) -> Vec<String> {
    let _span = info_span!("dispatch_pass", token = %token, line_count = state.line_count).entered();
    // ... 处理逻辑 ...
    // span 在函数退出时自动 drop，记录耗时
}
```

### 16.3 管线级 span

```rust
let pipeline = subject.clone()
    .scan_map(CompileState::new(), |state: &mut CompileState, token: String| -> Vec<String> {
        let _span = info_span!("phase1_dispatch", token = %token).entered();
        state.dispatch(token)
    })
    .flat_map(|v: Vec<String>| Shared::from_iter(v))
    .scan_map(CssBuilder::new(), |builder: &mut CssBuilder, line: String| -> Vec<CssNode> {
        let _span = info_span!("phase2_feed", line = %line).entered();
        builder.feed(&line)
    });
```

### 16.4 证据链示例

```
trace: phase1_dispatch { token: "$color: red;" } line_count: 0
trace: phase1_dispatch { token: ".foo {" } line_count: 1
trace: phase2_feed { line: ".foo {" }
trace: phase2_feed { line: "  color: red;" }
```

---

<a id="17"></a>
## 17. sasspile 实际编译管线深度剖析

### 17.1 完整源码

```rust
// src/directive/pipeline.rs
use rxrust::prelude::*;
use rxrust::subject::SharedSubject;
use std::sync::mpsc;

use super::eval::dispatch_pass;
use crate::css::builder::CssBuilder;
use crate::css::node::CssNode;
use crate::css::render::render_node;
use crate::directive::state::CompileState;

fn merge_media_nodes(nodes: Vec<CssNode>) -> Vec<CssNode> {
    // @media 合并逻辑
    nodes  // 简化
}

pub fn compile_pipeline(input: &str) -> String {
    let subject = Shared::subject::<String, Infallible>();
    let (tx, rx) = mpsc::channel::<String>();

    subject.clone()
        // Phase 1: CompileState 消费自身
        .scan_map(CompileState::new(), dispatch_pass)
        .flat_map(|v: Vec<String>| Shared::from_iter(v))
        // Phase 2: CssBuilder 消费自身
        .scan_map(CssBuilder::new(), |builder: &mut CssBuilder, line: String| -> Vec<CssNode> {
            builder.feed(&line)
        })
        .flat_map(|v: Vec<CssNode>| Shared::from_iter(v))
        // 汇聚
        .collect::<Vec<CssNode>>()
        .last()
        // @media 合并
    .map(|nodes| merge_media_nodes(nodes))
    .flat_map(|nodes| Shared::from_iter(nodes))
    // Phase 3: 借用渲染
    .map(|node: CssNode| render_node(&node))
    // 终点收集
    .collect::<Vec<String>>()
    .last()
    // 终端
    .subscribe(move |css_vec: Vec<String>| {
        let _ = tx.send(css_vec.join("\n"));
    });

    // 驱动
    input.lines().for_each(|line| subject.clone().next(line.to_string()));
    subject.clone().complete();

    // 等待结果
    rx.recv().unwrap_or_default()
}
```

### 17.2 数据流时间线

```
输入: "$color: red; .foo { color: $color; }"

Subject.next("$color: red;")
  → scan_map(CompileState): 存储变量, 返回 vec[]
  → flat_map: 无元素展平
  → scan_map(CssBuilder): 无元素处理
  → collect: 无元素

Subject.next(".foo { color: $color; }")
  → scan_map(CompileState): 变量展开为 ".foo { color: red; }", 返回 vec![".foo { color: red; }"]
  → flat_map: 展平为 ".foo { color: red; }"
  → scan_map(CssBuilder): 解析为 CssNode::Rule { selectors: [".foo"], body: [Declaration { color: red }] }
  → collect: 收集

Subject.complete()
  → collect 发射 Vec<CssNode>
  → last 保留
  → map(merge_media) 处理
  → flat_map 展平
  → map(render) 渲染为 String
  → collect 汇聚为 Vec<String>
  → last 保留
  → subscribe 闭包: tx.send("".foo { color: red; }"\n")

rx.recv() → ".foo { color: red; }"
```

### 17.3 为什么这样设计

1. **Phase 1 (CompileState)**：处理 SCSS 变量、mixin、控制流 → 输出 CSS 文本
2. **Phase 2 (CssBuilder)**：解析 CSS 文本为 AST → 输出 CssNode
3. **Phase 3 (render)**：将 AST 序列化为最终 CSS 文本

每个阶段都是 **纯变换**：输入一种类型，输出一种类型，无副作用。

### 17.4 管线中的类型变化

```
Subject<String>                    // 入口：String
  → scan_map(CompileState)         // Item = String, Output = Vec<String>
  → flat_map(Shared::from_iter)    // Item = Vec<String>, 展平为 Item = String
  → scan_map(CssBuilder)           // Item = String, Output = Vec<CssNode>
  → flat_map(Shared::from_iter)    // Item = Vec<CssNode>, 展平为 Item = CssNode
  → collect<Vec<CssNode>>         // Item = Vec<CssNode> (汇聚)
  → last()                         // Item = Vec<CssNode>
  → map(merge_media)               // Item = Vec<CssNode>, Output = Vec<CssNode>
  → flat_map(Shared::from_iter)    // Item = Vec<CssNode>, 展平为 Item = CssNode
  → map(render_node)               // Item = CssNode, Output = String
  → collect<Vec<String>>          // Item = Vec<String> (汇聚)
  → last()                         // Item = Vec<String>
  → subscribe                      // 消费 Vec<String>, 发送 String
```

---

<a id="18"></a>
## 18. rxrust 源码导读

### 18.1 必读文件清单

源码路径：`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rxrust-1.0.0-rc.5/`

| 文件 | 内容 | 何时读 |
|------|------|--------|
| `src/observer.rs` | Observer trait 定义 | 理解 next/error/complete 的 self 语义 |
| `src/observable.rs` | Observable trait 定义 | 理解 observe 的 'a HRTB |
| `src/ops/scan_map.rs` | scan_map 实现 | 理解 &mut Acc 的 per-subscription 隔离 |
| `src/ops/flat_map.rs` | flat_map = map + merge_all | 理解 1:N 展平的内部机制 |
| `src/observable/from_iter.rs` | FromIter Observable | 理解 is_closed 检查 |
| `src/subject/` | Subject 实现 | 理解 re-entrant 问题和 delay(0) |
| `src/context.rs` | Local vs Shared | 理解 MutRc vs MutArc |
| `src/ops/collect.rs` | collect 实现 | 理解 complete 时发射 |
| `src/ops/last.rs` | last 实现 | 理解"只保留最后一个" |

### 18.2 Observer trait 的核心洞察

```rust
// src/observer.rs
pub trait Observer {
    type Item;
    type Error;

    fn next(&mut self, value: Self::Item);      // &mut self: 可多次调用
    fn error(self, err: Self::Error);           // self: 按值消费，调用后不可用
    fn complete(self);                          // self: 按值消费，调用后不可用
}
```

**设计意图**：
- `&mut self` for `next`：允许 Observer 在处理每个值时修改自身状态
- `self` for `error/complete`：类型层面的"终结约束"——错误或完成后，Observer 被 move 进方法，调用者无法再次使用

### 18.3 scan_map 的 per-subscription 隔离

```rust
// src/ops/scan_map.rs（简化）
pub struct ScanMap<S, F, ACL> {
    source: S,
    f: F,
    _phantom: PhantomData<ACL>,
}

impl<S, F, ACL> Observable for ScanMap<S, F, ACL> {
    fn observe<O>(self, observer: O) -> Subscription
    where O: Observer {
        // 每次 observe 都创建新的 ScanMapObserver
        // 每个 Observer 持有独立的 Acc 状态！
        let scan_map_observer = ScanMapObserver {
            acc: self.initial,      // 每个订阅独立的 Acc
            f: self.f,
            downstream: observer,
        };
        self.source.observe(scan_map_observer)
    }
}
```

**关键**：每次 `subscribe` 都会调用 `observe`，创建新的 `ScanMapObserver`，每个都有独立的 `Acc`。这就是"per-subscription 隔离"。

### 18.4 flat_map 的内部 = map + merge_all

```rust
// src/ops/flat_map.rs
// flat_map 本质上是 map 后跟 merge_all
pub type FlatMap<S, F, ACL> = MergeAll<Map<S, F, ACL>, ACL>;
// MergeAll：将多个 Inner Observable 合并为一个
```

---

<a id="19"></a>
## 19. 提交前自查清单

### 19.1 四大支柱检查

- [ ] **借引用**：中间步骤是否用 `&T` 或 `&mut Acc` 而非 `T.clone()`？
- [ ] **消费自身**：状态演化是否用 `scan_map` 的 `&mut Acc` 而非外部 `let mut`？
- [ ] **响应式思想**：管线是否声明式？subscribe 是否为唯一执行边界？
- [ ] **终点收集**：最终结果是否为 owned String/Vec？中间步骤是否零额外 clone？

### 19.2 算子使用检查

- [ ] **map**：闭包签名是否 `FnMut(Item) -> B`？是否无外部可变状态？
- [ ] **filter**：闭包签名是否 `FnMut(&Item) -> bool`？是否借用？
- [ ] **scan_map**：闭包签名是否 `FnMut(&mut Acc, Item) -> Output`？Acc 是否就地修改？
- [ ] **flat_map**：闭包是否返回 `impl Observable`？是否用 `Shared::from_iter` 包装迭代器？
- [ ] **from_iter**：内部是否有 `is_closed()` 检查（自定义 Observable 时）？
- [ ] **subscribe**：闭包是否 `move`？是否有 'static 问题？

### 19.3 Shared 约束检查

- [ ] 入口 Subject 是否为 `Shared::subject::<String, Infallible>()`？
- [ ] 中间步骤是否避免 `&str`（非 'static）？
- [ ] 管线类型是否在每一步都匹配？
- [ ] `complete()` 是否在 `next()` 全部完成后调用？

### 19.4 调试检查

- [ ] 每个 phase 入口是否有 `info_span!`？
- [ ] 关键字段（token, count, state）是否记录在 span 中？
- [ ] 临时 span 是否在验证后清理？

### 19.5 性能检查

- [ ] 是否有不必要的 `.clone()`？
- [ ] scan_map 的 Acc 是否真的需要在每步 clone（如果是 Vec，考虑是否可以用引用）？
- [ ] flat_map 的 Inner Observable 是否已正确包装（非直接返回 Vec）？

---

<a id="20"></a>
## 20. 常见 AI 行为偏差与纠正

### 20.1 偏差 1：命令化（Immediate Execution）

**表现**：
```rust
// AI 习惯写法
let mut results = vec![];
for line in lines {
    results.push(process(line));
}
// results 已经"处理完了"
```

**纠正**：
```rust
// 响应式写法
let pipeline = source.map(process).collect::<Vec<_>>();
// pipeline 还没有执行！ 只是声明了关系
pipeline.subscribe(|v| { /* 这里才真正执行 */ });
```

**记住**：管线 = 声明，subscribe = 执行。

### 20.2 偏差 2：共享可变（Shared Mutability）

**表现**：
```rust
// AI 习惯写法
let mut state = State::default();
// 试图在所有闭包中共享 &mut state
```

**纠正**：
```rust
// 响应式写法：状态封装在 scan_map 内部
source.scan_map(State::default(), |state: &mut State, item| -> Output {
    state.update(item);  // 就地修改，外部无法访问
    state.output()
})
```

### 20.3 偏差 3：String 滥用

**表现**：
```rust
// AI 习惯写法：每一步都 clone String
.map(|s: String| s.to_uppercase().clone())  // 不必要的 clone
.flat_map(|v: Vec<String>| Shared::from_iter(v.clone()))  // 不必要的 clone
```

**纠正**：
```rust
// 借引用原则：只在终点产生 owned String
.map(|s: String| s.to_uppercase())  // to_uppercase 已经产生新的 String
.flat_map(|v: Vec<String>| Shared::from_iter(v))  // v 是 owned，直接传
```

### 20.4 偏差 4：忘记 Iterator 不是 Observable

**表现**：
```rust
// AI 习惯写法
.flat_map(|x: String| x.split(',').map(|s| s.to_string()))  // 返回 Iterator
```

**纠正**：
```rust
// flat_map 必须返回 Observable
.flat_map(|x: String| {
    let parts: Vec<String> = x.split(',').map(|s| s.to_string()).collect();
    Shared::from_iter(parts)
})
```

### 20.5 偏差 5：忽略 Observer 的 move 语义

**表现**：
```rust
// AI 习惯写法
.subscribe(|v| {
    tx.send(v).unwrap();
    // 之后还想用 tx？ 不可能！ tx 已被 move
})
```

**纠正**：
```rust
.subscribe(move |v| {
    // tx 被 move 进闭包，closure 外不可用
    let _ = tx.send(v);
})
```

### 20.6 偏差 6：Subject clone 误用

**表现**：
```rust
// AI 可能的错误理解：subject 是引用
subject.next(x);  // 在某些情况下可以，但 SharedSubject 推荐 clone
```

**纠正**：
```rust
// SharedSubject: 必须 clone 后调用 next/complete
subject.clone().next(x);
subject.clone().complete();
// 原因：next/complete 消费 self（或需要 &mut），clone 保留原始 Subject
```

### 20.7 偏差 7：尝试绕过 HRTB

**表现**：
```rust
// AI 可能的错误理解：为什么 map 闭包要 for<'a>
.map(|x: &'a str| x.to_string())  // &'a str 从哪来？
```

**纠正**：
```rust
// 正确的理解：map 接收 owned Item，借用由 Item 自身提供
.map(|s: String| s.as_str().to_string())  // s 是 owned，s.as_str() 借用
// 但返回值不能是引用！s 在闭包结束时被 drop
// 所以返回值必须是 owned
.map(|s: String| s.to_uppercase())  // owned → owned
```

### 20.8 偏差 8：调试时猜测根因

**表现**：
```rust
// AI 的调试方式：猜测 + 改代码 + 运行
// "我觉得这里有问题，我试试改一改"
```

**纠正**：
```rust
// 响应式调试：span 插桩 + 证据链
use tracing::info_span;

.scan_map(state, |s: &mut State, x: String| -> Output {
    let _span = info_span!("scan_map_step", input = %x, count = s.count).entered();
    let result = process(s, x);
    tracing::info!(?result, "step output");
    result
})
// 然后 RUST_LOG=trace cargo test -- --nocapture 收集证据
// 基于证据定位根因，而非猜测
```

---

## 附录 A：完整示例代码

```rust
use rxrust::prelude::*;
use rxrust::subject::SharedSubject;
use std::sync::mpsc;

#[derive(Default)]
struct CompileState {
    line_count: usize,
    indent: usize,
}

fn compile_scss(input: &str) -> String {
    let subject = Shared::subject::<String, Infallible>();
    let (tx, rx) = mpsc::channel::<String>();

    subject.clone()
        // Phase 1: 状态机
        .scan_map(CompileState::default(),
            |state: &mut CompileState, line: String| -> Vec<String> {
                let _span = tracing::info_span!("phase1", line = %line).entered();
                state.line_count += 1;
                if line.starts_with("//") {
                    vec![]  // 注释，丢弃
                } else {
                    vec![line]
                }
            })
        .flat_map(|v: Vec<String>| Shared::from_iter(v))
        // Phase 2: 构建 AST
        .scan_map(vec![],
            |acc: &mut Vec<String>, line: String| -> Vec<String> {
                let _span = tracing::info_span!("phase2", line = %line).entered();
                acc.push(line.clone());
                vec![line]
            })
        .flat_map(|v: Vec<String>| Shared::from_iter(v))
        // 汇聚
        .collect::<Vec<String>>()
        .last()
        .map(|lines| {
            lines.join("\n")
        })
        .subscribe(move |css: String| {
            let _ = tx.send(css);
        });

    // 驱动
    input.lines().for_each(|line| subject.clone().next(line.to_string()));
    subject.clone().complete();

    // 等待结果
    rx.recv().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_scss_basic() {
        let input = r#"
.foo {
  color: red;
}"#;
        let result = compile_scss(input);
        assert!(result.contains("color"));
        assert!(result.contains("red"));
    }
}
```

## 附录 B：常用闭包模式速查

```rust
// 1. map: 简单转换
.map(|x: T| -> U { x.into() })

// 2. filter: 借用判断
.filter(|x: &T| x.is_valid())

// 3. scan_map: 状态机
.scan_map(State::default(), |state: &mut State, item: T| -> Output {
    state.update(item);
    state.output()
})

// 4. flat_map: 展平集合
.flat_map(|x: T| {
    let items: Vec<U> = x.into_iter().collect();
    Shared::from_iter(items)
})

// 5. collect + subscribe: 汇聚消费
.collect::<Vec<T>>()
.last()
.subscribe(|v: Vec<T>| { /* 消费 */ });

// 6. 借用渲染
.map(|node: T| render(&node))  // render(&T) -> U

// 7. 错误传播（Result 类型）
.scan_map(Result::<State, Err>::Ok(State::default()),
    |state: &mut Result<State, Err>, item: T| -> Result<Vec<U>, Err> {
        let state = state.as_mut().map_err(|e| e.clone())?;
        Ok(state.process(item))
    })
```

## 附录 C：rxrust 源码路径速查表

```
~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rxrust-1.0.0-rc.5/
├── src/
│   ├── observable.rs          # Observable trait 定义
│   ├── observer.rs            # Observer trait 定义（终结语义）
│   ├── subscription.rs        # Subscription 管理
│   ├── context.rs             # Local vs Shared (MutRc vs MutArc)
│   ├── rc.rs                  # MutRc / MutArc 类型定义
│   ├── ops/
│   │   ├── map.rs             # map 实现
│   │   ├── filter.rs          # filter 实现
│   │   ├── scan_map.rs        # scan_map 实现（&mut Acc 就地修改）
│   │   ├── flat_map.rs        # flat_map = map + merge_all
│   │   ├── collect.rs         # collect 实现（complete 时发射）
│   │   ├── last.rs            # last 实现
│   │   ├── take.rs            # take 实现
│   │   ├── merge_all.rs       # merge_all 实现
│   │   └── skip.rs            # skip 实现
│   ├── observable/
│   │   ├── from_iter.rs       # FromIter（is_closed 检查）
│   │   ├── of.rs              # of 实现
│   │   ├── range.rs           # range 实现
│   │   ├── empty.rs           # empty 实现
│   │   └── error.rs           # error 实现
│   └── subject/
│       ├── local_subject.rs   # Local Subject
│       ├── shared_subject.rs  # Shared Subject
│       └── behavior_subject.rs # Behavior Subject
```

---

> **文档版本**：v2.0 | **最后更新**：2026-09-24 | **适用 rxrust 版本**：1.0.0-rc.5
</longcat_think>
