# AGENTS.md — rx-scss 项目开发规范

> **核心哲学**：rx-scss 是以 **rxrust 响应式流** 为管道的 SCSS 编译器。所有跨阶段数据处理必须用 Observable 算子表达，禁止命令式可变状态。Rust 所有权是第一公民，禁止 GC 思维。

---

## 📦 Rust 工具链

| Item | Specification |
|------|---------------|
| Edition | 2024 |
| Toolchain | 1.99 |

Cargo.toml 必须有 `edition = "2024"`。

## 📋 依赖

```toml
[dependencies]
rxrust = "1.0.0-rc.5"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
thiserror = "2"

[dev-dependencies]
similar = "2"
```

---

## ⛔ 绝对禁止项（违反 = 任务失败）

### 1. 禁止 GC 思维回潮

| ❌ GC 思维（Python/JS/Java） | ✅ Rust 所有权思维 |
|------|------|
| `list.append(x)` 修改 | `vec.into_iter().map(f).collect()` 消费+返回 |
| `dict["key"] = value` 修改 | 返回新值或共享不可变引用 |
| `obj.copy()` 显式拷贝 | `move` 语义，无需 clone |
| `for x in items: result.append(f(x))` | `items.into_iter().map(f).collect()` |
| `if (x) { ... } else if (y) { ... }` | `match { x => ..., y => ..., _ => ... }` |
| 全局可变状态 | `SharedSubject` 广播 / `scan_map(&mut acc, ...)` 累积 |
| 异常传播 | `Result<T, E>` + `?` 传播 |
| `None` 表示缺失 | `Option<T>` + `match`/`?` |

### 2. 禁止绕过 rxrust 管道

| ❌ 禁止 | ✅ 必须 |
|------|------|
| `for` 循环收集结果 | `.collect::<Vec<_>>()` |
| `if-else` 链分类 | `match` 表达式 |
| 手动 `Vec<Item>` + `push` | `.flat_map()` + `.collect()` |
| 递归函数处理树 | `.expand()` 响应式递归展开（核心！） |
| 手动栈维护嵌套 | `.scan_map(&mut acc, ...)` 累积折叠（&mut 零 clone） |
| `Arc<Mutex<Vec>>` 做缓冲 | `SharedSubject` 广播 |
| `subscribe` 里手动收集 | 管线末端一次性 `collect` |

### 3. 禁止 Python

| 禁止 | 替代 |
|------|------|
| `python3 xxx.py` | `rust-script xxx.rs` |
| `pip install xxx` | 添加到 Cargo.toml |
| 创建 `.py` 文件 | 使用 `rust-script -e` |

### 4. 禁止 println! / eprintln!

**所有代码**（含 `src/` 和 `tests/`）一律禁止：

```rust
// ❌ 禁止
println!("...");
eprintln!("...");

// ✅ 必须
info!("...");
warn!("...");
error!("...");
debug!("...");
trace!("...");
```

### 5. 禁止 #[cfg(test)] 内联测试

```rust
// ❌ 禁止
#[cfg(test)]
mod tests { ... }

// ✅ 所有测试放在 tests/ 目录，src/ 保持纯生产代码
```

### 6. 禁止 unwrap()

- 生产代码用 `?` / `expect()` / `unwrap_or()` / `unwrap_or_else()`
- 禁止 `clone()` 满天飞 — 先理解所有权设计
- 禁止 `todo!()` / `unimplemented!()` 不标注 `// TODO:` 并说明计划

### 7. 禁止 rxrust 反模式回潮（2026-10 源码审计修正）

| ❌ 反模式（已犯过） | ✅ 正确做法 | 历史位置 |
|------|------|------|
| `Arc<Mutex<Vec<T>>>` + `.subscribe(\|x\| vec.lock().unwrap().push(x))` | `.collect::<Vec<_>>().into_iter().next().unwrap_or_default()` | eval/mod.rs, parser/mod.rs, pipeline.rs |
| 手动 `Vec<Work>` queue + `while pop` 模拟 `expand` / `flat_map` | 使用 `.expand(emit_events)` + `.scan_map(fold_state)` 算子链 | eval/mod.rs emit_node_events |
| 命令式 `for event { match { ... } }` 模拟 `.scan()` | 使用 `.scan_map(initial, \|acc, x\| ...)` 有状态折叠管线 | eval/mod.rs fold_frames |
| 在 subscribe 闭包内对每个 item 手动收集结果 | 管线末端一次性 `.collect::<Vec<_>>()` | 所有阶段 |
| `'static` 滥用——为满足生命周期强制 clone 一切 | 理解实际生命周期关系；优先设计 owned 数据流 | 整个管线 |
| 命令式模拟 rxrust 算子后声称"响应式" | 必须使用真正的 `flat_map`/`scan_map`/`expand`/`filter_map` 算子 | eval/mod.rs |
| `#[cfg(test)]` 测试代码写 src/ 模块内 | 所有测试放在 `tests/` 目录 | serialize/mod.rs, builder.rs |
| `.scan(FrameStack, fold)` 每帧 clone 整帧栈 | `.scan_map(\|state, ev\| ...)` 用 `&mut FrameStack` 零 clone | eval/mod.rs |
| `.box_it()` 在每个节点/每个算子后调用 | `.box_it()` **只在管线最终返回边界调用一次** | eval/emit.rs (89 处滥用) |
| flat_map 闭包内 `Vec::new() + push + Shared::from_iter(vec)` | flat_map/expand 闭包直接返回 `impl Observable<Item=T>`（惰性） | emit.rs |
| `sync_collect_vec = Arc<Mutex<Vec>>` + subscribe | `collect::<Vec<_>>().into_iter().next()` | observable_ext.rs |

### 8. 单文件 ≤ 500 行

| 场景 | 推荐上限 |
|------|---------|
| 组件/模块文件 | 300 行 |
| 业务逻辑文件 | 400 行 |
| 类型定义文件 | 500 行 |

超过 **500 行**的文件必须先拆分再编写（源码和测试分别计算）。

### 9. 禁止 'static 滥用

理解实际生命周期关系，不要随意加 `'static`。

### 10. 禁止 `Infallible` 残留（2026-10 新增）

全链路响应式重构后，错误类型已从 `Infallible` 迁移到 `CompileError`。**新增代码 MUST NOT 使用 `Infallible`**：

```rust
// ❌ 禁止
use std::convert::Infallible;
pub type AstStream = SharedBoxedObservable<'static, AstNode, Infallible>;

// ✅ 必须
pub type AstStream = SharedBoxedObservable<'static, AstNode, CompileError>;
```

✅ **2026-10-10 已清除** — `Infallible` 已从 `src/` 和 `tests/` 全量清除，错误统一经 `CompileError` 传播。

### 11. 强制使用 OpenTelemetry Tracing（禁止 eprintln!/println!）

**所有**调试输出、测试打印、诊断信息 **必须** 使用 `tracing` 宏，统一接入 OpenTelemetry：

| ❌ 禁止 | ✅ 必须 |
|------|------|
| `eprintln!("...")` | `tracing::debug!("...")` |
| `println!("...")` | `tracing::info!("...")` |
| 任何直接 stdout/stderr 输出 | 通过 `tracing-subscriber` + `EnvFilter` |

**测试中调试**：
```rust
// ✅ 正确
tracing::debug!(output = %result, "rest arg parsed");
// 运行: RUST_LOG=debug cargo test test_name -- --nocapture

// ❌ 禁止
eprintln!("output: {}", result);
```

### 12. 行为基准：仅参考 @sass-spec（禁止参照 dart-sass）

SCSS 语法行为的**唯一权威参考**是官方 [sass/sass-spec](https://github.com/sass/sass-spec) 测试套件：

| ❌ 禁止 | ✅ 必须 |
|------|------|
| 以 dart-sass 输出为正确性标准 | 以 @sass-spec input/output 为正确性标准 |
| 参考 dart-sass 源码实现细节 | 参考 @sass-spec 测试用例 + Sass 官方文档 |
| "dart-sass does it this way" 作为设计理由 | "sass-spec expects this output" 作为设计理由 |

---

## 🏗️ 架构总览

### 目标：全链路响应式编译管道（规划中）

```
Source(String)
    │
    ▼ Shared::create (lexer)
TokenStream (SharedBoxedObservable<'static, Token, CompileError>)
    │
    ▼ Shared::create + subscribe_all 桥接 (parser: parser_feed 增量解析)
AstStream (SharedBoxedObservable<'static, AstNode, CompileError>)
    │
    ▼ expand + scan_map(&mut) + filter_map (eval — 递归展开 + 帧栈累积)
CssStream (SharedBoxedObservable<'static, CssStmt, CompileError>)
    │
    ▼ fold + map (serialize — 累积渲染)
OutputStream (SharedBoxedObservable<'static, String, CompileError>)
    │
    ▼ collect_boxed（终端阻塞收集）
String (CSS 输出)
```

**关键差异（vs 旧架构）**:
- ❌ 旧: Parser/Eval 中间 `collect_boxed` → Vec → `from_iter`（断裂点）
- ✅ 新: 流式直连，零中间收集
- ❌ 旧: `Infallible` 错误类型
- ✅ 新: `CompileError` 统一错误通道
- ❌ 旧: `flat_map` 做 AST 递归
- ✅ 新: `expand` 深度优先递归展开

**✅ 2026-10-10 全链路响应式重构完成** — `Infallible` 已从全链路清除，统一使用 `CompileError`。openspec `full-reactive-pipeline` 已实施并测试通过（244 tests）。

### 模块职责

| 模块 | 文件 | 职责 | 核心 rxrust 模式 |
|------|------|------|------------------|
| Lexer | `lexer/mod.rs` | 字符 → Token 流 | `Shared::create` + `box_it()` |
| Parser | `parser/mod.rs` | Token 流 → AST 流 | `Shared::create` + `subscribe_all` 桥接 |
| Evaluator | `eval/mod.rs` | AST 流 → CSS 流 | `expand` + `scan_map(&mut)` + `filter_map` |
| Serializer | `serialize/mod.rs` | CSS 树 → 字符串 | `fold` 累积 + `map(render)` |
| Runtime | `runtime.rs` | 作用域 + 总线 | `Arc<EvalContext>` + `CompilerBus` |
| Bus | `bus.rs` | Mixin/Function/Var 注册表 | `SharedSubject` 广播 |
| Telemetry | `telemetry.rs` | Tracing 初始化 | `tracing-subscriber` + `EnvFilter` |

### 类型别名（核心 — 目标状态）

```rust
// types.rs — 管道统一类型
pub type TokenStream = SharedBoxedObservable<'static, Token, CompileError>;
pub type AstStream   = SharedBoxedObservable<'static, AstNode, CompileError>;
pub type CssStream   = SharedBoxedObservable<'static, CssStmt, CompileError>;
pub type OutputStream = SharedBoxedObservable<'static, String, CompileError>;
```

### 入口函数 (`src/lib.rs`/`src/pipeline.rs`)

| 函数 | 输入 | 输出 | 用途 |
|------|------|------|------|
| `from_string(source, options)` | `&str`, `&Options` | `Result<String, CompileError>` | 字符串编译 |
| `from_path(path, options)` | `&Path`, `&Options` | `Result<String, CompileError>` | 文件编译 |
| `from_string_with_paths(source, options, paths)` | `&str`, `&Options`, `Vec<PathBuf>` | `Result<String, CompileError>` | 带加载路径编译 |

---

## 🔄 RxRust 核心用法

### 创建 Observable

```rust
// 从闭包创建（自定义源）
Shared::create(move |subscriber| {
    subscriber.next(value);
    subscriber.complete();
}).box_it()

// 从迭代器（同步有界数据）
Shared::from_iter(vec_of_items).box_it()

// 单个值
Shared::of(single_value).box_it()

// 空流
Shared::empty::<ItemType>().box_it()

// Subject
let subject: SharedSubject<'static, T, CompileError> = Shared::subject();
subject.next(value);
subject.complete();
subject.box_it()
```

### 核心算子选择

| 场景 | 算子 | 说明 |
|------|------|------|
| 1:1 变换 | `.map(\|x\| ...)` | 值转换 |
| 1:N 展平 | `.flat_map(\|x\| Observable)` | 子节点注入流 |
| 递归展开 | `.expand(\|x\| Observable)` | 树 → 事件流（深度优先！） |
| 状态累积 | `.scan_map(\|state, x\| ...)` | &mut 零 clone 累积 |
| 终止收集 | `.collect::<Vec<_>>()` | 流 → Vec（仅最终消费端） |
| 调试 | `.tap(\|x\| tracing::debug!(...))` | 副作用观察 |

### 类型擦除（box_it）

```rust
let stream: AstStream = Shared::create(|s| { ... })
    .flat_map(|x| Shared::from_iter(x))
    .box_it();  // 只在阶段边界调用！
```

### Context: Local vs Shared

| Context | Use When | Internals | Item Bounds |
|---------|----------|-----------|-------------|
| `Local` | Single-thread (WASM, UI main) | `Rc<RefCell>` — zero lock | None |
| `Shared` | Multi-thread server | `Arc<Mutex>` — work-stealing | `Send + Sync` |

**rx-scss 默认使用 `Shared`**。

---

## 🧠 Evaluator 核心设计

### 响应式展开 + 扫描累积

```
AST 节点流
    │
    ▼ expand(emit_events)          ← 递归展开 AST 树
事件流 (EvalEvent)
    EnterRule / LeaveRule / EnterMedia / ...
    │
    ▼ scan_map(EvalState, fold_frame)  ← &mut 有状态折叠累积
中间状态流 (EvalState)
    │
    ▼ filter_map(emit_completed)   ← frame 关闭时发射 CssStmt
CSS 树流
```

### EvalEvent 枚举

```rust
enum EvalEvent {
    EnterRule(String),
    LeaveRule,
    EnterMedia(String),
    LeaveMedia,
    EnterSupports(String),
    LeaveSupports,
    Terminal(CssStmt),
}
```

### Frame 栈（scan_map 状态）

```rust
struct Frame {
    kind: FrameKind,         // Root / Rule / Media / Supports
    selector: Option<String>,
    query: Option<String>,
    stmts: Vec<CssStmt>,
}
```

---

## 🚌 CompilerBus 设计

### 核心模式：SharedSubject 广播

```rust
pub struct CompilerBus {
    var_subject: SharedSubject<'static, VarEvent, CompileError>,
    module_subject: SharedSubject<'static, ModuleEvent, CompileError>,
    scope_subject: SharedSubject<'static, ScopeEvent, CompileError>,
    inner: Arc<Mutex<BusInner>>,
}
```

### 变量查找 = 沿 parent 链向上

```rust
fn get_var(&self, ctx: &EvalContext, name: &str) -> Option<Value> {
    let mut current_id = Some(ctx.scope_id());
    while let Some(s) = current_id {
        if let Some(v) = variables.get(&(s, name)) { return Some(v.clone()); }
        current_id = parent_map.get(&s).copied();
    }
    None
}
```

---

## 🦀 Rust 所有权强制规则

### 所有权：move 优先，禁止 clone 满天飞

### 迭代器：禁止显式 for 循环处理集合变换

| 禁止 | 替代 |
|------|------|
| `for x in vec { result.push(f(x)) }` | `vec.into_iter().map(f).collect()` |
| `for x in &vec { if pred(x) { ... } }` | `vec.into_iter().filter(pred)...` |
| `for x in vec { match ... { Ok(v) => acc.push(v) } }` | `vec.into_iter().try_fold(acc, ...)` |
| `for x in vec { if pred(x) { left.push(x) } else { right.push(x) } }` | `vec.into_iter().partition(pred)` |

### 模式匹配：禁止 if-else 链处理枚举

```rust
// ✅ 必须
match token {
    "{" => ...,
    "}" => ...,
    ";" => ...,
    _ => ...,
}
```

### 错误处理：禁止 match Err 分支

```rust
// ✅ 必须
let ast = parse(tokens)?;
```

### 函数签名的强制模式

| 场景 | 签名模板 |
|------|----------|
| 数据变换 | `fn transform(input: Input) -> Output` |
| 带状态变换 | `fn step(state: State, input: Input) -> (Output, State)` |
| 管线阶段 | `fn next_stage(self) -> Result<NextStage>` |
| 只读查询 | `fn query(&self, key: &str) -> Option<&Value>` |

---

## 📋 代码风格强制

### match 优于 if-else
### ? 优于 match Err
### partition 优于 for + if

---

## 🔬 Tracing Span 强制规则

### 核心原则

跨函数/跨阶段的管道处理**必须**用 `tracing::span!`（或 `#[instrument]`），记录上下文与耗时。

### Span 创建优先级

**首选：`#[instrument]` 宏**

```rust
#[tracing::instrument(skip(large_param), fields(result = tracing::field::Empty))]
fn my_function(large_param: &BigType, input: &str) -> Result<...> {
    let result = do_work(large_param, input)?;
    tracing::Span::current().record("result", &result);
    Ok(result)
}
```

**备选: `.entered()`**

```rust
let _span = info_span!("parse_expr", expr = ?input, pos = self.pos).entered();
```

### Telemetry 初始化

```rust
rx_scss::telemetry::init_tracing();      // 生产
rx_scss::telemetry::init_test_tracing(); // 测试
```

### Span 字段 sigil

| Sigil | 含义 |
|-------|------|
| `?` | Debug 格式化 |
| `%` | Display 格式化 |
| 无 | 实现 Value trait |

---

## 🧪 测试规范

### 测试位置

- `src/` 内：**禁止** `#[cfg(test)]` 模块
- `tests/` 目录：所有集成测试

### 测试文件清单

| 测试文件 | 覆盖内容 |
|----------|----------|
| `compile_test.rs` | 端到端编译流程 |
| `lexer_test.rs` | 词法分析器 |
| `parser_test.rs` | 语法分析器 |
| `eval_test.rs` | 求值器 |
| `pipeline_test.rs` | 管线集成 |
| `bus_test.rs` | CompilerBus |
| `builder_test.rs` | CompileBuilder |
| `types_test.rs` | 类型定义 |
| `value_test.rs` | 值运算 |
| `runtime_test.rs` | 运行时上下文 |
| `integration_test.rs` | 集成测试 |
| `bootstrap_test.rs` | Bootstrap 兼容性 |
| `telemetry_test.rs` | Tracing/Telemetry 集成 |
| `serialize_test.rs` | 序列化器 |
| `debug_utility.rs` ~ `debug_utility6.rs` | 调试辅助 |
| `diag_*.rs` | 诊断辅助测试 |
| `while_loop_test.rs` | while 循环 |
| `rfs_function_test.rs` | RFS 函数 |

### 测试命令

```bash
cargo test
cargo test --test compile_test
RUST_LOG=debug cargo test test_name -- --nocapture
RUST_LOG=error cargo test --test integration_test -- --nocapture
```

---

## 🚫 反模式检测清单

出现以下任一情况，**立即停止**：

- [ ] 同一行被反复修改 2 次以上
- [ ] 新增代码与现有枚举/trait 定义矛盾
- [ ] 生成代码超过目标文件行数 80%
- [ ] 连续 2 次在同一个函数中添加 `clone()`
- [ ] 把 `self -> Self` 链式改回 `&mut self`
- [ ] 用 `for + push` 替换已有的 `map/collect`
- [ ] 在纯函数中引入 `&mut` 参数
- [ ] 用 `if-else` 链替换已有的 `match`
- [ ] 修改波及 3 个以上不相关函数
- [ ] 出现 subscribe-collect GC 模式
- [ ] 命令式 queue/while pop 模拟 flat_map/expand
- [ ] 用命令式 for+match 模拟 scan 算子
- [ ] 新增 #[cfg(test)] 内联测试模块
- [ ] 新代码仍使用 `Infallible` 类型

---

## ✅ 自检清单

每次任务完成后：

- [ ] 未使用 Python
- [ ] 所有输出用 tracing 宏（无 println!/eprintln!）
- [ ] 测试在 tests/ 目录
- [ ] 跨函数/管道使用 tracing span
- [ ] span 字段用 `?`/`%` sigil
- [ ] 无 `unwrap()`
- [ ] 无 `clone()` 满天飞
- [ ] 无 `todo!()`/`unimplemented!()` 不标注 TODO
- [ ] 无 subscribe-collect GC 模式
- [ ] 算子链使用真正的 rxrust 算子
- [ ] 公开 API 有 `///` 文档注释
- [ ] 单文件 ≤ 500 行
- [ ] 枚举分派用 `match`
- [ ] 错误传播用 `?`
- [ ] 调试遵循 4 步协议（如果是 bug 修复）
- [ ] Commit 等用户确认后再推送

---

## 📚 参考

- **rxrust 完整参考（含 OTel）**：`.claude/skills/rxrust/SKILL.md`
- **OpenSpec 工作流**：`.claude/skills/openspec-*/SKILL.md`
- **telemetry 模块**：`src/telemetry.rs`
- **全链路响应式重构规划**：`openspec/changes/full-reactive-pipeline/`
