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
thiserror = "2"

[dev-dependencies]
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
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
| 全局可变状态 | `SharedSubject` 广播 / `scan` 累积 |
| 异常传播 | `Result<T, E>` + `?` 传播 |
| `None` 表示缺失 | `Option<T>` + `match`/`?` |

### 2. 禁止绕过 rxrust 管道

| ❌ 禁止 | ✅ 必须 |
|------|------|
| `for` 循环收集结果 | `.collect::<Vec<_>>()` |
| `if-else` 链分类 | `match` 表达式 |
| 手动 `Vec<Item>` + `push` | `.flat_map()` + `.collect()` |
| 递归函数处理树 | `.expand()` 响应式展开 |
| 手动栈维护嵌套 | `.scan()` 累积折叠 |
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

### 7. 单文件 ≤ 500 行

| 场景 | 推荐上限 |
|------|---------|
| 组件/模块文件 | 300 行 |
| 业务逻辑文件 | 400 行 |
| 类型定义文件 | 500 行 |

超过 **500 行**的文件必须先拆分再编写（源码和测试分别计算）。

### 8. 禁止 'static 滥用

理解实际生命周期关系，不要随意加 `'static`。

---

## 🏗️ 架构总览

### 响应式编译管道

```
Source(String)
    │
    ▼ scan() — Shared::create + box_it()
TokenStream (SharedBoxedObservable<'static, Token, Infallible>)
    │
    ▼ parse_stream()
AstStream (SharedBoxedObservable<'static, AstNode, Infallible>)
    │
    ▼ eval_stream() — expand + scan 模拟
CssStream (SharedBoxedObservable<'static, CssStmt, Infallible>)
    │
    ▼ collect + serialize
String (CSS 输出)
```

### 模块职责

| 模块 | 文件 | 职责 | 核心 rxrust 模式 |
|------|------|------|------------------|
| Lexer | `lexer/mod.rs` | 字符 → Token 流 | `Shared::create` + `box_it()` |
| Parser | `parser/mod.rs` | Token 流 → AST 流 | `Shared::from_iter` + `box_it()` |
| Evaluator | `eval/mod.rs` | AST 流 → CSS 流 | `expand` + `scan` 模拟 |
| Serializer | `serialize/mod.rs` | CSS 树 → 字符串 | 纯函数，无流 |
| Runtime | `runtime.rs` | 作用域 + 总线 | `Arc<EvalContext>` + `CompilerBus` |
| Bus | `bus.rs` | Mixin/Function/Var 注册表 | `SharedSubject` 广播 |

### 类型别名（核心）

```rust
// types.rs — 管道统一类型
pub type TokenStream = SharedBoxedObservable<'static, Token, Infallible>;
pub type AstStream   = SharedBoxedObservable<'static, AstNode, Infallible>;
pub type CssStream   = SharedBoxedObservable<'static, CssStmt, Infallible>;
pub type OutputStream = SharedBoxedObservable<'static, String, Infallible>;
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
// 从闭包创建（最常用的自定义源）
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

// Subject (hot observable — 手动 emit)
let subject: SharedSubject<'static, T, Infallible> = Shared::subject();
subject.next(value);      // emit
subject.complete();       // signal done
subject.box_it()          // convert to Observable for chaining
```

### 核心算子选择

| 场景 | 算子 | 说明 |
|------|------|------|
| 1:1 变换 | `.map(\|x\| ...)` | 值转换 |
| 1:N 展平 | `.flat_map(\|x\| Observable)` | 子节点注入流 |
| 递归展开 | `.expand(\|x\| Observable)` | 树 → 事件流（核心！） |
| 状态累积 | `.scan(init, \|s, x\| ...)` | 栈/累加器 |
| 过滤 | `.map(\|x\| Option).filter_map(...)` | 条件通过 |
| 终止收集 | `.collect::<Vec<_>>()` | 流 → Vec |
| 调试 | `.tap(\|x\| tracing::debug!(...))` | 副作用观察 |

### 类型擦除（box_it）

```rust
// 任何算子链的终点必须擦除类型
let stream: AstStream = Shared::create(|s| { ... })
    .flat_map(|x| Shared::from_iter(x))
    .box_it();  // 必须！
```

### Context: Local vs Shared

| Context | Use When | Internals | Item Bounds |
|---------|----------|-----------|-------------|
| `Local` | Single-thread (WASM, UI main) | `Rc<RefCell>` — zero lock | None |
| `Shared` | Multi-thread server | `Arc<Mutex>` — work-stealing | `Send + Sync` |

**rx-scss 默认使用 `Shared`**（静态生命周期，多线程能力）。

---

## 🧠 Evaluator 核心设计

### 响应式展开 + 扫描累积

evaluator 的核心是模拟两个 rxrust 算子：

```
AST 节点流
    │
    ▼ expand_nodes_to_events()  ← 模拟 expand
事件流 (Vec<EvalEvent>)
    EnterRule / LeaveRule / EnterMedia / ...
    │
    ▼ apply_event()  ← 模拟 scan
CSS 树流 (frame 栈累积)
```

### EvalEvent 枚举

```rust
enum EvalEvent {
    EnterRule(String),    // 进入嵌套规则
    LeaveRule,            // 退出规则 → pop frame
    EnterMedia(String),   // 进入 @media
    LeaveMedia,           // 退出 @media
    EnterSupports(String),// 进入 @supports
    LeaveSupports,        // 退出 @supports
    Terminal(CssStmt),    // 终端声明
}
```

### Frame 栈（scan 状态）

```rust
struct Frame {
    kind: FrameKind,         // Root / Rule / Media / Supports
    selector: Option<String>,
    query: Option<String>,
    stmts: Vec<CssStmt>,
}
```

Enter 事件 = push frame；Leave 事件 = pop frame 并组装 CssStmt 注入父 frame。

---

## 🚌 CompilerBus 设计

### 核心模式：SharedSubject 广播

```rust
pub struct CompilerBus {
    var_subject: SharedSubject<'static, VarEvent, Infallible>,
    module_subject: SharedSubject<'static, ModuleEvent, Infallible>,
    scope_subject: SharedSubject<'static, ScopeEvent, Infallible>,
    inner: Arc<Mutex<BusInner>>,  // 实际存储
}
```

### 为何用 Arc<Mutex> 而非 Rc<RefCell>

- `Send + Sync` 要求：跨线程安全
- 需要 `SharedSubject` 广播事件
- 闭包必须 `move` 进 rxrust 算子

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

## 🦀 Rust 所有权强制规则（函数式第一公民）

### 所有权：move 优先，禁止 clone 满天飞

| 禁止 | 替代 | 说明 |
|------|------|------|
| `env.clone()` | `env` move 进函数，返回 `(T, Env)` | 零拷贝传递 |
| `&mut Env` 参数 | `Env`（move）→ `self -> Self` 链式 | 不可变借用 + 返回新值 |
| `Rc<RefCell<T>>` | 按值传递 + 返回新值 | 避免 interior mutability |

### 迭代器：禁止显式 for 循环处理集合变换

| 禁止 | 替代 | 场景 |
|------|------|------|
| `for x in vec { result.push(f(x)) }` | `vec.into_iter().map(f).collect()` | map 变换 |
| `for x in &vec { if pred(x) { ... } }` | `vec.into_iter().filter(pred)...` | filter 筛选 |
| `for x in vec { match ... { Ok(v) => acc.push(v), Err(e) => return e } }` | `vec.into_iter().try_fold(acc, ...)` | 错误传播累积 |
| `for x in vec { if pred(x) { left.push(x) } else { right.push(x) } }` | `vec.into_iter().partition(pred)` | 分流 |
| 可变 `Vec` + push + extend | `flat_map` / `flatten` | 展平嵌套 |
| `for (i, x) in vec.iter().enumerate()` | `vec.into_iter().enumerate()` | 带索引 |

### 模式匹配：禁止 if-else 链处理枚举

```rust
// ❌ 禁止：if-else 链
if token == "{" { ... }
else if token == "}" { ... }
else if token == ";" { ... }
else { ... }

// ✅ 正确：match
match token {
    "{" => ...,
    "}" => ...,
    ";" => ...,
    _ => ...,
}
```

### 错误处理：禁止 match Err 分支

```rust
// ❌ 禁止：显式 match Err
let result = match parse(tokens) {
    Ok(ast) => ast,
    Err(e) => return Err(e),
};

// ✅ 正确：? 传播
let ast = parse(tokens)?;
```

### 函数签名的强制模式

| 场景 | 签名模板 | 说明 |
|------|----------|------|
| 数据变换 | `fn transform(input: Input) -> Output` | 消费输入，返回新值 |
| 带状态变换 | `fn step(state: State, input: Input) -> (Output, State)` | move 语义，返回新状态 |
| 管线阶段 | `fn next_stage(self) -> Result<NextStage>` | `self` 消费，类型状态机 |
| 链式构建 | `fn with_x(mut self, x: X) -> Self` | builder 模式 |
| 只读查询 | `fn query(&self, key: &str) -> Option<&Value>` | 纯函数，不可变借用 |

---

## 📋 代码风格强制

### match 优于 if-else

```rust
// ✅ 必须
match self.peek() {
    Token::AtMedia => self.parse_at_media(),
    Token::AtSupports => self.parse_at_supports(),
    _ => self.parse_rule_set(),
}
```

### ? 优于 match Err

```rust
// ✅ 必须
let ast = parse(tokens)?;
```

### partition 优于 for + if

```rust
// ✅ 必须
let (left, right): (Vec<_>, Vec<_>) = items.into_iter().partition(pred);
```

---

## 🔬 Tracing Span 强制规则

### 核心原则

跨函数/跨阶段的管道处理**必须**用 `tracing::span!`（或 `#[instrument]`），记录上下文与耗时。**禁止仅用 event! 单一日志**。

### Span 创建优先级

**默认首选：`#[instrument]` 宏** — 函数入口自动创建 span，参数自动记录为字段。

```rust
// ✅ 首选：函数入口用 #[instrument]
#[tracing::instrument(skip(large_param), fields(result = tracing::field::Empty))]
fn my_function(large_param: &BigType, input: &str) -> Result<...> {
    let result = do_work(large_param, input)?;
    tracing::Span::current().record("result", &result);
    Ok(result)
}
```

**备选 1: `.entered()` — 条件分支/内联代码块**

```rust
let _span = info_span!("parse_expr", expr = ?input, pos = self.pos).entered();
// ... logic ...
// _span drop 时自动退出
```

### tap 插桩（响应式管道内）

```rust
stream
    .tap(|x| tracing::debug!(value = ?x, "after filter"))
    .flat_map(|x| /* ... */)
```

### Span 字段 sigil

| Sigil | 含义 | 示例 |
|-------|------|------|
| `?` | Debug 格式化 | `field = ?value` |
| `%` | Display 格式化 | `field = %value` |
| 无 | 实现 Value trait | `field = value` |

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
| `debug_utility.rs` ~ `debug_utility6.rs` | 调试辅助 |
| `diag_custom_prop.rs` | 自定义属性诊断 |

### 测试命令

```bash
# 运行全部测试
cargo test

# 单个测试套件
cargo test --test compile_test
cargo test --test lexer_test
cargo test --test parser_test
cargo test --test eval_test
cargo test --test pipeline_test
cargo test --test integration_test
cargo test --test bootstrap_test

# tracing 调试
RUST_LOG=debug cargo test test_name 2>&1 | head -100
RUST_LOG=error cargo test --test integration_test -- --nocapture
```

---

## 🚫 反模式检测清单

出现以下任一情况，**立即停止**并通知用户：

- [ ] 同一行被反复修改 2 次以上
- [ ] 新增代码与现有枚举/trait 定义矛盾
- [ ] 生成代码超过目标文件行数 80%
- [ ] 连续 2 次在同一个函数中添加 `clone()`
- [ ] 把 `self -> Self` 链式改回 `&mut self`
- [ ] 用 `for + push` 替换已有的 `map/collect`
- [ ] 在纯函数中引入 `&mut` 参数
- [ ] 用 `if-else` 链替换已有的 `match`
- [ ] 修改波及 3 个以上不相关函数

---

## ✅ 自检清单

每次任务完成后：

- [ ] 未使用 Python
- [ ] 所有输出用 tracing 宏（无 println!/eprintln!）
- [ ] 测试在 tests/ 目录（无 inline #[cfg(test)]）
- [ ] 跨函数/管道使用 tracing span（或 `#[instrument]`）
- [ ] span 字段用 `?`/`%` sigil（非 `&format!(...)`）
- [ ] async 代码不用 `Span::enter`（用 `#[instrument]` 或 `.instrument()`）
- [ ] 无 `unwrap()`（用 `?`/`expect()`/`unwrap_or()`）
- [ ] 无 `clone()` 满天飞（先理解所有权设计）
- [ ] 无 `todo!()`/`unimplemented!()` 不标注 TODO
- [ ] 公开 API 有 `///` 文档注释
- [ ] 单文件 ≤ 500 行
- [ ] 集合变换用 `map/filter/collect` 而非 `for + push`
- [ ] 枚举分派用 `match` 而非 `if-else` 链
- [ ] 错误传播用 `?` 而非 `match ... Err(e) => return`
- [ ] 状态变更返回新值（`self -> Self`）而非 `&mut self`
- [ ] 管线阶段消费 `self`（类型状态机）而非 `&self` + clone
- [ ] 累积操作用 `try_fold` / `fold` 而非可变 `Vec` + push
- [ ] 分流用 `partition` 而非两个 `Vec` + for + if
- [ ] 调试遵循 4 步协议（如果是 bug 修复）
- [ ] OTel 追踪可用（如有配置）
- [ ] CodeGraph 用于代码查询（如有配置）
- [ ] 使用高效工具链
- [ ] Commit 等用户确认后再推送

---

## 📚 参考

- **rxrust 完整参考**：`.claude/skills/rxrust/SKILL.md`
- **OpenSpec 工作流**：`.claude/skills/openspec-*/SKILL.md`
