---
name: rxrust-source-first
description: 写任何 rxrust 代码前, 强制先读框架源码理解内部机制.当需要新增算子、修改管线、排查编译错误、或不确定 API 用法时激活。核心原则:不猜 API、不造轮子、不绕过框架。先 read_file 读 rxrust 源码, 从 src/ 内部找答案。
allowed-tools: Read, Write, Edit, MultiEdit, ListDir, run_terminal_cmd
license: MIT
metadata:
  author: sasspile-rx
  version: "1.0"
---

# rxrust-source-first: 读源码优先开发模式

## 触发条件

当遇到以下情况时**必须**先读源码, 不直接写代码:

1. **不确定算子签名** — scan_map/flat_map/collect/last 的闭包签名、返回类型
2. **不确定创建方式** — from_iter/from_stream/of 的约束和边界
3. **类型不满足 trait bound** — `the trait bound ... is not satisfied`
4. **Subscription 类型不匹配** — `SourceWithDynamicSubs is not a Future`
5. **不确定调度器行为** — Shared vs Local 的选择
6. **不确定 subscribe 返回值** — 如何驱动 TaskHandle

---

## 必读源码清单

### 路径前缀

```
~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rxrust-1.0.0-rc.5/
```

### 按问题类型查找

| 问题类型 | 文件 | 关键行 |
|---------|------|--------|
| 所有算子签名 | `src/observable.rs` | 全文 |
| Observable 创建 | `src/factory.rs` | 全文 |
| scan_map 实现 | `src/ops/scan_map.rs` | 全文 |
| flat_map / merge_all | `src/ops/flat_map.rs`, `src/ops/merge_all.rs` | 全文 |
| collect | `src/ops/collect.rs` | 全文 |
| last | `src/ops/last.rs` | 全文 |
| subscribe 终点 | `src/observable.rs` | 141 行 (subscribe 签名) |
| Context / Shared / Local | `src/rc.rs`, `src/context.rs` | 全文 |
| Subscription 包装 | `src/subscription/source_with_dynamic.rs` | 全文 |
| box_it 类型擦除 | `src/observable.rs` | 1944 行 |
| 调度器 / TaskHandle | `src/scheduler.rs` | 全文 |
| Observer trait | `src/observer.rs` | 全文 |
| 类型别名 | `src/observable/boxed.rs` | LocalBoxedObservable |
| from_stream 实现 | `src/observable/from_stream.rs` | 全文 |

---

## 标准开发流程

### Step 1: 先定位源码

```
读 → 理解 → 写
```

**禁止**: 没读源码就写代码
**要求**: 用 read_file 找到具体行号, 引用行号说明

### Step 2: 基于源码写代码

```rust
// 示例: 发现了 scan_map 签名 (observable.rs 402 行)
// F: for<'a> FnMut(&mut Acc, Self::Item<'a>) -> Output
// 所以 reducer 是 &mut 就地修改, 返回 Output
.scan_map(CssBuilder::new(), |builder, line: String| {
    builder.feed(&line)  // &mut builder 就地修改
                          // 返回 Vec<CssNode> 是 Output
})
```

### Step 3: 基于源码排错

如果编译错误是 `the trait bound ... is not satisfied`:
1. 定位到具体算子源码
2. 检查泛型参数约束
3. 检查 `Send + 'static` 边界
4. 检查 `Err` 类型是否匹配

如果运行时问题是管线不产出:
1. 检查 `flat_map` 子流是否正确返回 Observable
2. 检查 `collect/last` 是否正确链接
3. 检查 `subscribe` 返回的 handle 是否被 block_on

---

## 源码快查

### scan_map 签名和语义

```rust
// observable.rs 402 行
fn scan_map<Acc, Output, F>(self, initial: Acc, f: F) -> Self::With<ScanMap<Self::Inner, F, Acc>>
where
    F: for<'a> FnMut(&mut Acc, Self::Item<'a>) -> Output,
```

- `initial: Acc` — 初始 Acc 值 (所有权 move 进算子)
- `f: F` — 闭包, 每次调用的签名是 `FnMut(&mut Acc, Self::Item<'a>) -> Output`
- 返回 `Self::With<ScanMap<...>>` — 新的 Observable

### flat_map 签名

```rust
// observable.rs 2400 行
fn flat_map<F, Inner>(self, f: F) -> Self::With<FlatMap<Self::Inner, F, Inner>>
where
    F: for<'a> FnMut(Self::Item<'a>) -> Inner,
    Inner: Context<Inner: ObservableType<Err = Self::Err>>,
```

- `Inner` 必须是一个 Observable Context (如 `Shared<FromIter<S>>`)
- `Err` 类型必须匹配

### subscribe + TaskHandle

```rust
// observable.rs 141 行
fn subscribe<F, U>(self, f: F) -> U
where
    F: for<'a> FnMut(Self::Item<'a>),
    Self::Inner: CoreObservable<Self::With<FnMutObserver<F>>, Unsub = U>,
```

- 返回类型 `U` = 算子的 `Unsub` 关联类型
- `from_stream` + `Shared` → `U = TaskHandle`
- 经 `collect/last` 后 → `U = SourceWithDynamicSubs<SourceWithDynamicSubs<TaskHandle, ...>, ...>`

---

## 操作守则

### ✅ 必须

1. **写代码前 read_file 源码** — 找到具体行号
2. **理解 trait bound 含义** — 不是复制粘贴, 是理解为什么
3. **先读 signature 再写闭包** — 闭包签名由框架定, 不是自己定
4. **遇到错误读源码定位** — 算子内部实现会告诉你约束

### ❌ 禁止

1. **没读源码就猜 API** — `subscribe_boxed` 不存在, `Local::new(boxed)` 是 nested
2. **自己实现框架已有的功能** — 已经有 scan_map 就别手写 fold
3. **绕过框架约束** — 强制 transmute 生命周期, 绕过 Send + 'static
4. **盲目试错编译** — 改动 10 次靠运气, 不如读 1 次源码

---

## 输出格式

当写 rxrust 代码时, 必须注明源码参考:

```markdown
### Source Reference
- `scan_map` 签名: `src/observable.rs` 402 行
- `flat_map` 签名: `src/observable.rs` 2400 行
- `subscribe` 签名: `src/observable.rs` 141 行
- `SourceWithDynamicSubs`: `src/subscription/source_with_dynamic.rs`
```

---

## 调试 Protocol

当遇到编译/运行时错误:

1. **读错误** — 确定是哪个算子报错
2. **读源码** — 对应算子的 trait bound 和 subscribe 实现
3. **查返回类型** — SourceWithDynamicSubs 需要 `.source.source`
4. **查闭包签名** — `FnMut(&mut Acc, Item)` 不是 `FnMut(Acc, Item) -> Acc`
5. **写最小复现** — 独立 test 验证单个算子
6. **集成回管线** — 确认编译通过
