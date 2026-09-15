---
name: rxrust-scss
description: Implement SCSS compiler features the rxrust-first way. Use when the user says "implement", "add feature", "fix spec failure", "implement directive", or mentions tokenize/parse/evaluate/serialize stages.PRIMARY GOAL: make Bootstrap + Element Plus compile 100% first. Enforces rxrust 4-primitives + 7 operator classes + sass-spec-only + tracing spans + no println + no dart-sass reference.
allowed-tools: Read, Write, Edit, MultiEdit, ListDir
license: MIT
metadata:
  author: sasspile-rx
  version: "1.1"
---

# rxrust-scss: rxrust-first SCSS 开发 skill

你是 sasspile-rx 的 rxrust-first SCSS 编译器开发代理。**所有行为 MUST 从 rxrust 四原语出发**——Observable / Observer / Operator / Subscription。**禁止参考 dart-sass,sass 官方实现,或任何非 rxrust 的思路**。sass-spec HRX 是语法行为契约。

## 🥇 首要任务:企业库 100% 全量通过 (优先级最高)

**在推进任何 sass-spec 用例之前**,必须先把 Bootstrap 和 Element Plus 跑通。这两个企业级项目代表真实世界的编译需求,比 sass-spec 海量用例更具实际价值。

### 验证入口

| 项目 | 入口文件 | 范式 |
|------|---------|------|
| Bootstrap 5 | `bootstrap/scss/bootstrap.scss` | `@import` 全局共享 |
| Element Plus | `element-plus/packages/theme-chalk/src/index.scss` | `@use as *` + `!global` |

### 验收命令

```bash
# 拍当前企业基线
cargo run --bin sasspile_tracker -- enterprise
# 实现 feature 后再跑对比
cargo run --bin sasspile_tracker -- diff <a.json> <b.json>
```

### 优先级排序

1. Bootstrap 100% (`bootstrap.scss` 端到端编译无错误)
2. Element Plus 100% (`index.scss` 端到端编译无错误)
3. sass-spec 增量提升 (在 1,2 基础上锦上添花)

### 何时使用 sass-spec

在 1,2 通过后,用 sass-spec 发现边角情形、修复 bug。**永远不要让 sass-spec 的个别用例阻塞企业目标**。

---

## 核心哲学 (必须遵守)

### 四原语优先级
1. **Observable<Item,Err>** — 流载体;编译的每一帧数据
2. **Observer<Item,Err>** — 编译器在 subscribe 端点被表达
3. **Subscription** — `subscribe()` 返回的编译生命周期
4. **Operator (.pipe())** — stage 的纯函数变换

### 七算子类,MUST 每类至少使用一次

| 类别 | 算子 | 在 sasspile-rx 中 |
|------|------|------------------|
| Creation | `Local::from_iter`, `Local::of` | 创建 char 源 |
| Transformation | `map`, `scan`, `scan_map`, `flat_map` | Token 转 AST、@extend 展开 |
| Filtering | `filter`, `distinct`, `distinct_until_changed` | 去重 Token、@use 同名过滤 |
| Combination | `merge`, `merge_all`, `zip` | 多文件并行 |
| Utility | `tap`, `finalize`, `delay` | tracing 注入、资源清理 |
| Aggregation | `reduce`, `last`, `collect` | 收敛为 `Result<String, _>` |
| Connectable | `multicast`, `publish`, `ref_count` | 模块缓存共享 |

### 错误管道:Infallible + Result<T,E>

```rust
// ✓ 正确
Observable<Result<CssNode, CompileError>, Infallible>

// ✗ 错误 — 违反 error-as-value
Observable<CssNode, CompileError>
```

### tap = 唯一副作用窗口

```rust
// ✓ 正确
nodes.tap(|n| tracing::debug!(?n, stage = "parse"))

// ✗ 错误 — 不要在 map 闭包里执行 side-effect
nodes.map(|n| { debug!(?n); eval(n) })
```

### box_it 类型擦除

每个 stage 的 `.pipe()` 返回后 MUST 紧接 `.box_it()`,擦除为 `LocalBoxedObservable<T, Infallible>`。

## 编码时刻表

### 实现一个 Stage 时
1. **Span 插桩**:在 stage 入口加 `let _span = tracing::info_span!(...).entered();`
2. **内部算子**:使用 `scan` 累积状态 / `flat_map` 展开嵌套 / `map` 转换
3. **副作用隔离**:所有 tracing 通过 `.tap()` 桥接
4. **类型返回**:`-> LocalBoxedObservable<Out, Infallible>`
5. **单文件 ≤ 500 LOC**

### 实现一个编译器指令 (@if/@for/@use/@include/@extend/...) 时
1. **spec 查找**:在 `sass-spec/spec/<category>/` 找到对应 HRX 文件,读取 input/output
2. **算子映射**:确定该指令是 scan (状态累积) 还是 flat_map (嵌套展开)
3. **实现 + 测试**:stage 内扩展,新增 `#[test]` 在 tests/ 目录
4. **snapshot**:`cargo run --bin sasspile_tracker -- snapshot` 后 diff 验证提升

## 调试三原则 (数据流优先)

### 🥇 原则一:看数据流，不要猜

rxrust 管道的每个 stage 边界已插入了 `.tap()` 观测点:
- `stage = "tokenize"` → Token 流出
- `stage = "parse"` → Node 流出
- `stage = "evaluate"` → 错误或结果
- `stage = "serialize"` → char 流出

**这就是证据链,不是 span。** 不要到处加 `debug_span!` 来猜测——先看 tap 输出找到精确的变形阶段。

```bash
RUST_LOG=trace cargo test test_name 2>&1 | grep -E '(node out|error|FAILED)'
```

```
示例证据链:
  node out node=MixinDef { name: "theme", params: ["$color"] }  ✅ parse 正确
  node out node=MixinCall { name: "theme", args: ["blue"] }    ✅ parse 正确  
  WARN evaluation failed error=undefined variable: $theme            ❌ evaluate 才是根因!
```

### 🔍 原则二:定位精确失败阶段

看 trace 时按数据流顺序扫描:
1. tokenize → 数字/符号是否被正确 token 化?
2. parse → Node 是否按语法结构正确产出? (不是被当成普通 rule?)
3. evaluate → 变量是否替换? mixin 是否展开?
4. serialize → CSS 字符串是否正确渲染?

找到**第一个数据变形的位置**,就是根因。

### ✅ 原则三:修复验证

修复后对比:
```bash
cargo test --test e2e_api 2>&1 | grep -E '(test test|FAILED|ok|panicked)'
cargo run --bin sasspile_tracker -- diff A B
```

## 单测约定

- 测试文件放 `tests/`,src/ 保持纯生产代码
- 每个测试 MUST 从 `Observable` 构造开始,不依赖全局状态
- use `tracing::info_span!` 封装测试入口
- 断言方式:`.last().subscribe(|r| assert!(r.is_ok()))` 或 subscribe 后 collect

## 项目约定

- 每文件 ≤ 500 行;超出必须拆分
- 所有代码(含 `src/`, `tests/`, `bins/`)MUST 使用 tracing 宏;**禁用 println!/eprintln!**
- 推送:SSH 方式 git push github main
- 提交后:必须等用户确认后再 push

## 验收标准 (按优先级)

- [ ] **Bootstrap 100%** — `bootstrap/scss/bootstrap.scss` 编译无错误 ⭐ 首要
- [ ] **Element Plus 100%** — `element-plus/packages/theme-chalk/src/index.scss` 编译无错误 ⭐ 首要
- [ ] 所有测试 `cargo test` 通过
- [ ] `cargo clippy -- -W clippy::all -W clippy::pedantic` 通过
- [ ] sass-spec HRX 匹配 (spec 在用例路径 —— 先锦上添花)

## Dashboard (外部对照输出格式)

```
enterprise baseline:
  bootstrap  v5.3  100% ✓
  element-plus v2.x 100% ✓
spec baseline:    28 / 10274 (0.27%)  ← 锦上添花,非阻塞
```

## 输出格式

每次实现时使用如下格式汇报:

```
## 实现: <feature> (schema: rxrust-scss)
rxrust 映射: <scan | flat_map | publish | ...>
stage: <tokenize | parse | evaluate | serialize>

Working on task N: <subtask desc>
  ✓ operator: <具体 rxrust 算子>
  ✓ test: <test fn name>
  ✓ snapshot: diff <a.json> b.json +X.X%
```

## Guardrails

- **禁止参考 dart-sass 源码**,sass-spec 为唯一契约
- **禁止在 map/scan 闭包内调用 tracing**,必须 `.tap()` 桥接
- **禁止绕过 scan 直接用可变局部变量代替**——违反 rxrust 状态传播
- **单测试用例必须先 subscribe 再断言**(不能只看 Observable 定义)
- **panic 反模式**:stage 内不得 unwrap/expect,错误 MUST 返回 `Result::Err`

## rxrust 操作符完整参考 (v1.0.0-rc.5)

### 创建类 (Creation)

| 算子 | 签名 | 用途 | sasspile-rx 场景 |
|------|------|------|-----------------|
| `Local::from_iter` | `impl IntoIterator -> Observable` | 从迭代器创建本地 Observable | char 源:`Local::from_iter(chars)` |
| `Local::of` | `T -> Observable` | 发射单个值 | `Local::of(Local::from_iter(...))` 嵌套源 |
| `Local::subject` | `-> Subject` | 手动推送值的 subject | 跨 stage 手动推送 |

### 变换类 (Transformation)

| 算子 | 签名 | 关键区别 | sasspile-rx 场景 |
|------|------|---------|-----------------|
| `map` | `FnMut(A)->B -> Observable<B>` | 1→1 静态映射 | CssNode→String 渲染 |
| `scan` | `init, FnMut(acc,A)->acc -> Observable<acc>` | **累积状态,每步都 emit 中间结果** | Token→AST 累积、parse 状态机 |
| `scan_map` | `init, FnMut(&mut acc,A)->B -> Observable<B>` | **累积 + 每步可 emit 不同类型** | 当前使用:parse.feed/eval 状态 |
| `flat_map` | `FnMut(A)->InnerObservable -> Observable<B>` | 1→N 展平,合并内部流 | @if/@for 多节点展开、@import |
| `filter_map` | `FnMut(A)->Option<B> -> Observable<B>` | map+filter 合二为一 | Token 过滤同时转类型 |
| `switch_map` | 同 flat_map | 取消前一个内部流,切换到新流 | @use 模块切换(未来) |
| `concat_map` | 同 flat_map | 顺序连接,不并发 | @import 串行加载 |
| `map_to` | `B -> Observable<B>` | 把所有输入映射为常数 | 占位/默认值 |
| `map_err` | `FnMut(E)->F -> Observable<T,F>` | 错误类型映射 | (当前不用,用 Result<T,E> 替代) |

### scan 家族详解 (状态传播核心)

```rust
// scan —— 累积同类型,emit 中间结果
Observable<In> -> scan(init, FnMut(acc, In) -> acc) -> Observable<acc>

// scan_map —— 累积任意状态,emit 任意类型
Observable<In> -> scan_map(init, FnMut(&mut state, In) -> Out) -> Observable<Out>

// 关键区别:
//   scan:     reducer 返回 acc 值本身,Observable 流出的就是 acc
//   scan_map: reducer 返回任意 Out 类型,Observable 流出的 Out 可与 acc 不同
```

**sasspile-rx 用法**:

```rust
// tokenize: char -> Token (ScannerState 累积 char,emit Token)
tokens = chars
  .scan_map(ScannerState::new(), |state, ch| state.feed(ch))
  .flat_map(Local::from_iter)      // Vec<Token> -> Token 展平

// parse: Token -> Node (AstBuilder 累积 Token,emit Node)
nodes = tokens
  .scan_map(AstBuilder::new(), |builder, token| builder.feed(token))
  .flat_map(Local::from_iter)      // Vec<Node> -> Node 展平

// evaluate: Node -> Result<Vec<CssNode>, _> (CompilerContext 累积,emit 评估结果)
outcomes = nodes
  .scan_map(CompilerContext::new(), |ctx, node| eval_and_accumulate(ctx, node))
  .flat_map(Local::from_iter)
```

### 过滤类 (Filtering)

| 算子 | 签名 | 行为 | sasspile-rx 场景 |
|------|------|------|-----------------|
| `filter` | `FnMut(&A)->bool` | 通过谓词才 emit | 过滤 Whitespace/Newline |
| `distinct` | `Eq+Hash+Clone` | 全局去重 | (一般不用于编译场景) |
| `distinct_until_changed` | `PartialEq` | 仅当与前值不同时 emit | 抑制重复错误 |
| `skip` | `usize` | 跳过前 N 个 | 跳过 shebang 等 |
| `take` | `usize` | 只取前 N 个 | 限制错误数量 |
| `skip_while` | `FnMut(&A)->bool` | 跳过直到谓词 false | 跳过前置空白 |
| `take_while` | `FnMut(&A)->bool` | 取到谓词 false 止 | 取特定 rule 段 |
| `skip_until` | 另一个 Observable | 跳过直到另一流 emit | (未来) |
| `take_until` | 另一个 Observable | 取到另一流 emit 止 | (未来) |

**sasspile-rx 用法**:

```rust
// Stage 1 tokenize 过滤 Whitespace+Newline
.filter(|t| !matches!(t, Token::Whitespace | Token::Newline))

// Stage 3 evaluate distinct_until_changed 抑制重复错误
.flat_map(|outcomes| Local::from_iter(outcomes))
.distinct_until_changed()
```

### 组合类 (Combination)

| 算子 | 行为 | sasspile-rx 场景 |
|------|------|-----------------|
| `merge` | 合并两个 Observable | 合并 @import 多源 |
| `merge_all` | 并发合并所有内部流 | @use 多模块并行加载 |
| `zip` | 两两配对 | 选择器与属性对齐 |
| `with_latest_from` | 从另一流取最新值配对 | (未来) |
| `combine_latest` | 任一 emit 时取所有流最新值配对 | (未来) |
| `pairwise` | 相邻两两配对 | 检测相邻字符模式 |
| `start_with` | 前置若干值 | 加 CSS 前缀 |
| `buffer` / `buffer_count` / `buffer_time` | 缓冲到 Vec | (未来:批量) |

**sasspile-rx 用法**:

```rust
// flat_map 内部用 concurrency=1 (顺序)
nodes.flat_map(|node| Local::from_iter(expand_node(node)))

// merge 合并
let merged = source_a.merge(source_b);
```

### 工具类 (Utility)

| 算子 | 行为 | sasspile-rx 场景 |
|------|------|-----------------|
| **`tap`** | 副作用,**不修改流**,每项都触发 | **唯一 tracing 窗口** |
| `finalize` | 在 complete/error 时执行清理 | 日志 elapsed_us |
| `delay` | 延迟所有通知 | (未来) |
| `observe_on` | 后续阶段切到指定 Scheduler | (本地用 Local:: 即可) |
| `subscribe_on` | 源阶段切到指定 Scheduler | (本地用 Local:: 即可) |
| `retry` | 出错自动重试 N 次 | @import 加载重试 |
| `debounce` / `throttle` | 时间维度过滤 | (一般不用) |

**⭐ tap 是 sasspile-rx 唯一 tracing 窗口**:

```rust
// ✅ 正确:side-effect 只在 tap 中
nodes
  .tap(|n| tracing::debug!(?n, stage = "parse", "node out"))
  .flat_map(|n| eval(n))

// ❌ 错误:不要在其他闭包里做 side-effect
nodes.flat_map(|n| { debug!(?n); eval(n) })
```

### 聚合类 (Aggregation)

| 算子 | 行为 | sasspile-rx 场景 |
|------|------|-----------------|
| `reduce` | 折叠为单个值(complete 时 emit) | 未来 CSS 合并 |
| `collect` | 收集为 Vec | 测试采集 |
| `last` | 取最后一个值 | 取最终 CSS |
| `into_future` | 转 Future (await) | (未来 async 集成) |
| `into_stream` | 转 async Stream | (未来) |
| `average` / `contains` | 辅助聚合 | 统计/检测 |

**sasspile-rx 用法**:

```rust
// 当前 subscribe 模式是收集为 Vec<char>
let result = Rc::new(RefCell::new(Vec::<char>::new()));
obs.subscribe(move |ch| { result.borrow_mut().push(ch); });

// 等价于 reduce 模式:
let result = obs.reduce(Vec::new(), |mut acc, ch| { acc.push(ch); acc });
```

### 连接类 (Connectable)

| 算子 | 行为 | sasspile-rx 场景 |
|------|------|-----------------|
| `publish` | 转为 ConnectableObservable | 模块缓存共享 |
| `ref_count` | 连接 + 引用计数 | 多订阅者共享同一 @use 模块 |
| `multicast` | 多播 (subject 包装) | (未来) |

**sasspile-rx 用法** (未来):

```rust
// @use 模块缓存命中: 多文件 @use 同一模块时共享缓存
let module_stream = source.publish().ref_count();
```

---

### 操作符决策树 — 该用哪个?

| 需求 | 选择 |
|------|------|
| 1→1 值转换 | `map` |
| 累积状态,每步 emit(同类型) | `scan` |
| 累积状态,每步 emit(不同类型) | `scan_map` |
| 1→N 展开 | `flat_map` |
| 1→N 顺序展开 | `flat_map` / `concat_map` |
| 1→N 切换(取消旧流) | `switch_map` |
| 按条件过滤 | `filter` |
| 去重相邻重复 | `distinct_until_changed` |
| 全局去重 | `distinct` |
| 副作用(日志/debug/trace) | **`tap`** ⭐ |
| complete/error 时清理 | `finalize` |
| 合并两源 | `merge` |
| 并发订阅所有子流 | `merge_all` |
| 收集为集合 | `collect` |
| 折叠为单个值(complete 时才 emit) | `reduce` |
| 取最后一个值 | `last` |
| 类型擦除 | `box_it` |

---

### box_it 与生命周期

```rust
// 每个 stage 边界后 box_it 擦除类型
let tokens: LocalBoxedObservable<Token, Infallible> = source
  .scan_map(...)
  .flat_map(...)
  .filter(...)
  .tap(...)
  .box_it();  // ← 类型擦除
```

**为什么需要 box_it?**

`scan_map<..., ScannerState, Vec<Token>>` 返回的类型极其复杂。`.pipe()` 串联多个算子后类型嵌套爆炸。`box_it()` 把它擦除为统一的 `LocalBoxedObservable<Item, Err>`,让 stage 间拼接可行。

---

## 快速参考

```
pipeline   → src/pipeline.rs              (编排)
tokenize   → src/tokenize_dst.rs          (scan_map + flat_map)
parse      → src/parse_dst.rs             (scan_map + flat_map + 行内指令)
evaluate   → src/evaluate_dst.rs          (scan_map + flat_map + tap)
serialize  → src/serialize_dst.rs         (map + flat_map + finalize)
lib API    → src/lib.rs compile()         (唯一公开入口)
tracker    → src/bin/sasspile_tracker.rs  (外部对照库: snapshot + enterprise)
snapshot   → .codegraph/snapshots/        (git-ignored)
spec       → sass-spec/spec/              (语法行为契约,非优先级)
enterprise → bootstrap/                  (Bootstrap submodule)
           → element-plus/               (Element Plus submodule)

tracker 命令:
  snapshot   — sass-spec HRX 全量 pass rate
  enterprise — Bootstrap + Element Plus 端到端编译
  diff a b  — 两版 snapshot 对比
  trend      — pass rate 时间序列
```
