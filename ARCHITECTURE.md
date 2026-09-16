# sasspile-rx Architecture

> rxrust 四原语驱动的 SCSS 编译器 —— 每个 stage 都是 Observable 算子的纯组合。

---

## 1. 核心抽象: rxrust 四原语

```
┌───────────────────────────────────────────────────────────�
│                 rxrust 4 Primitives                        │
├───────────────────────────────────────────────────────────�
│ Observable<Item, Err>    — 惰性的流载体                     │
│ Observer<Item, Err>     — 编译器在 subscribe 被表达         │
│ Subscription            — 编译生命周期 (.unsubscribe())     │
│ Operator (.pipe())      — stage 的纯函数组合子              │
└───────────────────────────────────────────────────────────┘
```

sasspile-rx 的整个编译器是这四原素的组合,**没有任何命令式结构**。

---

## 2. 管道总览

```
   ┌──────────────────────────────────────────────────────────────────┐
   │                  sasspile_rx::compile(input)                       │
   └───────────────────────────────�──────────────────────────────────┘
                                   │
          ┌────────────────────────▼────────────────────────┐
          │               Stage 1: tokenize                  │
          │  scan_map(ScannerState::new(), \|s,c\| s.feed(c))│
          │  .flat_map(\|v\| v)   filter(¬Whitespace)        │
          │  .tap(debug)   .box_it()                         │
          └────────────────────────┬─────────────────────────�
                                   │ Token
          ┌────────────────────────▼────────────────────────┐
          │  Stage 2: parse                                  │
          │  scan_map(AstBuilder::new(), \|b,t\| b.feed(t))  │
          │  .flat_map(\|v\| v)   .tap(debug)   .box_it()    │
          └────────────────────────┬─────────────────────────┘
                                   │ Node
          �────────────────────────▼────────────────────────┐
          │  Stage 3: evaluate                               │
          │  .flat_map(expand_directive)                     │
          │  .publish(module_cache).ref_count()   [TODO]      │
          │  .distinct_until_changed() .tap(debug) .box_it() │
          └────────────────────────┬─────────────────────────┘
                                   │ Result<CssNode, CompileError>
          ┌────────────────────────▼────────────────────────�
          │  Stage 4: serialize                              │
          │  .map(CssNode::render // or COMPILE ERROR banner)│
          │  .flat_map(\|s\| s.chars())  .finalize(‖ elapsed)│
          │  .tap(trace)  .box_it()                          │
          └────────────────────────�─────────────────────────�
                                   │ char
                                   ▼
                             subscribe(collect)
                                   │
                                   ▼
                            Result<String, CompileError>
```

---

## 3. 阶段详解

### 3.1 tokenize_dst.rs — `Observable<char> → Observable<Token>`

| 算子 | 作用 |
|------|------|
| `Local::from_iter(chars)` | Creation:从字符串构建源 Observable |
| `scan_map(ScannerState, \|s,c\| s.feed(c))` | Transformation:状态累积产出 Token |
| `flat_map(\|tokens\| tokens)` | Transformation:Vec<Token> 展平 |
| `filter(\|t\| !t.is_whitespace())` | Filtering:去掉空白 |
| `tap(\|t\| debug!(?t, stage="tokenize"))` | Utility:副作用桥接 tracing |

### 3.2 parse_dst.rs — `Observable<Token> → Observable<Node>`

| 算子 | 作用 |
|------|------|
| `scan_map(AstBuilder::new(), \|b,t\| b.feed(t))` | Transformation:累积 Token 直到完整 rule |
| `flat_map(\|nodes\| nodes)` | Transformation:Vec<Node> 展平 |
| `tap(\|n\| debug!(?n, stage="parse"))` | Utility:tracing |
| `box_it()` | 类型擦除为 `LocalBoxedObservable<Node, Infallible>` |

`AstBuilder` 内部:
- `depth` 字段追踪嵌套
- 规则完成 (depth==0 ∧ 遇到 `RBrace`) 触发 `flush_rule()`
- `flush_rule()` 解析 selector + declarations → `Node::Rule`

### 3.3 evaluate (TODO)

计划支持:
- `flat_map(\|node\| expand_directive(node))` — @if/@for/@each/@while/@mixin/@include/@extend/@import/@use
- `publish(Local::subject()).ref_count()` — 模块缓存多订阅者共享
- 错误通过 `Result<CssNode, CompileError>` 值传播,不终止流

### 3.4 serialize_dst.rs (TODO)

- `map(\|item\| match item { Ok(css) => css.render(), Err(e) => err_banner(e) })`
- `flat_map(\|s\| s.chars())` — `String → char stream`
- `finalize(\|‖ info!(elapsed_us = ...))` — Utility 资源清理

---

## 4. 错误哲学: Infallible + Result<T,E>

```rust
// � 反模式: 错误终止管道,后续 stage 收不到任何 Item
Observable<CssNode, CompileError>

// ✓ 正确: 错误是数据,继续传播,在 subscribe 端点处理
Observable<Result<CssNode, CompileError>, Infallible>
```

`Observer::error(self, err)` 会消费 observer,流必须终止。sass-spec HRX 中**部分用例故意测试错误处理**——如果遇到错误就终止,这些用例 case 会变成"死流",永远拿不到 expected 输出。

---

## 5. 算子使用清单

| 类别 | 算子 | 使用位置 |
|------|------|---------|
| Creation | `Local::from_iter` | tokenize 入口 |
| Creation | `Local::of` | 单值包装 (待扩展) |
| Transformation | `scan_map` | tokenize + parse 状态累积 |
| Transformation | `flat_map` | @extend/@mixin 嵌套展开 |
| Transformation | `map` | serialize 渲染 |
| Filtering | `filter` | 去 Whitespace Token |
| Filtering | `distinct_until_changed` | evaluate 去重 (TODO) |
| Combination | `merge_all` | 多文件 parallel 编译 (TODO) |
| Utility | `tap` | 所有 stage 的 tracing 注入 |
| Utility | `finalize` | pipeline 尾部的 elapsed_us 上报 |
| Aggregation | `last()` | 管道末端收敛单值 |
| Connectable | `publish/ref_count` | 模块缓存多订阅者共享 (TODO) |

---

## 6. 副作用网关: tap

**约定**:所有副作用(.info/.error/.debug/.warn 写入 tracing + 缓存写入 + metric 递增 + SourceMap 记录)MUST 通过 `tap` 注入。`map`/`scan` 闭包 MUST 保持纯函数语义。

```rust
// ✓ 正确: 流变换是纯的,副作用在 tap 中
source.scan(Tokenizer::new(), pure_tokenize)
     .tap(\|t\| tracing::debug!(?t, stage = "tokenize"))
     .pipe(parse)
     .tap(\|n\| tracing::debug!(?n, stage = "parse"))

// ✗ 错误: 副作用与变换混合
source.scan(Tokenizer::new(), |s, c| {
    tracing::debug!(?s);   // ← side-effect in scan
    s.feed(c)
})
```

---

## 7. 类型擦除 + box_it

每个 `.pipe(stage)` 返回后 MUST 跟随 `.box_it()`:

```rust
let tok: LocalBoxedObservable<Token, Infallible> = source
    .scan(...)
    .box_it();   // ← 必要

// 如果没有 box_it,类型会是嵌套 10 层的泛型:
// FlatMap<Scan<Filter<FromIter<char>, ...>, ...>, ...>
```

`box_it()` 内部调用 `BoxedObservable`,把嵌套泛型擦除为 trait object 的统一签名。

---

## 8. Shared Context (rxrust 语义)

### 8.1 单订阅者内状态传递: scan

ASTBuilder、ScannerState、VariableScope 都通过 `scan(initial, reducer)` 的 `acc` 参数在流内传递。不需要 `Arc<Mutex>`/`Rc<RefCell>`,单线程内顺序执行保证因果性。

### 8.2 跨订阅者数据共享: publish + ref_count

```rust
let module_stream = file_requests
    .flat_map(|path| load_and_compile(path))
    .publish(Local::subject())   // multicast: 下游共享同一份
    .ref_count();                // 自动连/断 (只剩一个订阅者时 .unsubscribe)
```

sla 一个路径的 module,多个 @use 条目共享同一份 `EvaluatedModule`,第二次 @use 命中缓存(不重复 parse/evaluate/serialize)。

---

## 9. 订阅端点: compile()

```rust
pub fn compile(input: &str) -> Result<String, CompileError> {
    let _root = tracing::info_span!("sasspile.compile", input_len = input.len()).entered();

    let acc = Rc::new(RefCell::new(Vec::<char>::new()));
    let r = acc.clone();

    pipeline::build(input).subscribe(move |ch| {
        r.borrow_mut().push(ch);
    });

    let css: String = acc.borrow().iter().collect();
    if css.contains("COMPILE ERROR:") { Err(...) } else { Ok(css) }
}
```

subscribe closure 只做 1 件事:累积 char。这与 rxrust 的"不是在数组上循环,而是让流推数据给你"一致。

---

## 10. 失败模式 (Anti-patterns)

| 反模式 | 修正 |
|--------|------|
| `Observable<CssNode, IterateError>` | `Observable<Result<CssNode, CompileError>, Infallible>` |
| `map` 闭包内调用 tracing | 改用 `.tap(\|x\| debug!(?x))` |
| scan 闭包借用多个 RefCell | scan 外通过 `tab` 一次性取出,或拆分为两个 stage |
| stage 末尾不 `.box_it()` | 类型嵌套爆炸 |
| `unwrap()` / `expect()` 在 stage 内 | `Err(CompileError::...)`,通过 error-as-value 传播 |
| 全局共享 `mut State` | 用 `scan` 参数或 `publish/ref_count` |

---

## 11. 扩展点

新增 SCSS 特性时的操作步骤:

1. `sas-spec/spec/<category>/` 读取 HRX 文件,得到 input ↔ expected output
2. 确定算子类型:
   - **有状态累积(如变量)** → 加 `scan` 内部状态
   - **嵌套展开(@extend @mixin)** → `flat_map(directive, ...)`
   - **跨文件 sharing(@use)** → `publish/ref_count`
3. stage 内扩展,注意 `LocalBoxedObservable<T, Infallible>` 的签名统一
4. 测试:tests/ 新增 `#[test]`,subscribe 后断言
5. 验证:`cargo run --bin sasspile_tracker -- snapshot` → 再 snapshot → `diff`

---

## 12. 工程约定

- **单文件 ≤ 500 LOC**(源码和测试分别计算)
- **编译标志**:`--release` 跑 enterprise;`--debug` 跑单测
- **RUST_LOG=trace** 时 span 调用链全可见
- **clippy pedantic** 必须通过:`cargo clippy -- -W clippy::all -W clippy::pedantic`
- **推送**:SSH 方式;commit 后等用户确认

---

## 13. 企业基线验证管道 (Enterprise Gate Pipeline,P0)

这是 sasspile-rx 的 **首要验收目标**,优先于一切 sass-spec 用例。

### 验证入口

| 项目 | 入口文件 | 顶层 | 文件数 | 范式 |
|------|---------|------|--------|------|
| Bootstrap 5 | `bootstrap/scss/bootstrap.scss` | scss/ | ~99 | `@import` 全局共享 |
| Element Plus | `element-plus/packages/theme-chalk/src/index.scss` | theme-chalk/src/ | ~146 | `@use as *` + `!global` |

### 端到端流程

```
   �───────────────────────────────────────────────��
   │         sasspile_rx::compile_file(path)         │  ← 企业入口 (TODO)
   └────────────────────�──────────────────────────┘�
                        │ 读文件
                        ▼
   �───────────────────────────────────────────────��
   │ @import stack / @use stack                     │  ← directives 处理
   │ Bootstrap: 顺序 @import 单全局作用域             │
   │ Element Plus: @use as * + !global 多文件注入     │
   └────────────────────�──────────────────────────┘┘
                        ▼
   �───────────────────────────────────────────────┐�┐
   │  Stage 3: evaluate                             │
   │  .flat_map(expand_directive)                    │
   │  .publish(module_cache).ref_count()              │  ← 多订阅者共享
   └────────────────────┬──────────────────────────�┘┘
                        ▼
   �───────────────────────────────────────────────�┐�
   │  Stage 4: serialize → char stream              │
   └────────────────────┬──────────────────────────�┘┘
                        ▼
                 Result<String, CompileError>
```

### PASS / FAIL 判据

- **PASS**:编译完成,输出 CSS 不含 `COMPILE ERROR:` 标记,无 panic
- **FAIL**:任何阶段 panic 或返回 `Err(CompileError)`

### 验收命令

```bash
cargo run --bin sasspile_tracker -- enterprise
```

### 迭代顺序

1. 让 `bootstrap.scss` 端到端跑通 → 100%
2. 让 `index.scss` 端到端跑通 → 100%
3. 之后才扩展 sass-spec 用例数

### 工程影响

上述两个企业级入口涵盖:
- **变量/作用域**:Bootstrap 全局 / EP `!global`
- **函数**:`color.scale()` / `map.get()` / `mix()` 等内置
- **指令`@if/@else/@each/@for/@mixin/@include`**
- **嵌套规则 + `@at-root`**
- **`@media` 查询合并**
- **列表/映射数据结构**

企业基线 100% = 已实现 SCSS 语言 90% 核心特性。sass-spec 是边角情形补完。
