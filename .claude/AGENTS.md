# AGENTS.md — rx-scss 项目开发规范

> **核心哲学**：rx-scss 是以 **rxrust 响应式流** 为管道的 SCSS 编译器。所有跨阶段数据处理必须用 Observable 算子表达，禁止命令式可变状态。Rust 所有权是第一公民，禁止 GC 思维。

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

### 3. 其他禁止

- **禁止 `println!`/`eprintln!`**：全部用 `tracing` 宏（`info!`/`debug!`/`warn!`/`error!`/`trace!`）
- **禁止 `#[cfg(test)]` 内联测试**：测试统一放 `tests/` 目录
- **禁止 `unwrap()`**：用 `?` / `expect()` / `unwrap_or()`
- **禁止 Python 脚本**：数据处理一律用 Rust test 或 `rust-script`
- **禁止 `clone()` 满天飞**：理解借用和所有权，只在必要时 clone

---

## 🏗️ 架构总览

### 响应式编译管道

```
Source(String)
    │
    ▼ scan()
TokenStream (SharedBoxedObservable<'static, Token, Infallible>)
    │
    ▼ parse_stream()
AstStream (SharedBoxedObservable<'static, AstNode, Infallible>)
    │
    ▼ eval_stream()
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

---

## 🦀 Rust 所有权强制规则

### 函数签名模板

| 场景 | 签名 | 说明 |
|------|------|------|
| 纯变换 | `fn transform(input: Input) -> Output` | 消费，返回 |
| 带状态 | `fn step(state: State, input: Input) -> (Output, State)` | move 语义 |
| 链式构建 | `fn with_x(mut self, x: X) -> Self` | builder 模式 |
| 只读查询 | `fn query(&self, key: &str) -> Option<&Value>` | 不可变借用 |
| 类型状态机 | fn next_stage(self) -> Result<NextStage> | 消费 self |

### 数据流原则

```rust
// ❌ GC 思维：修改传入的集合
fn process(items: &mut Vec<Item>) {
    items.push(new_item);
}

// ✅ Rust 所有权：消费 + 返回
fn process(items: Vec<Item>) -> Vec<Item> {
    items.into_iter()
        .map(transform)
        .collect()
}

// ✅ 或者不可变借用只读
fn process(items: &[Item]) -> Vec<Item> {
    items.iter()
        .map(transform)
        .collect()
}
```

### 迭代器优于 for 循环

```rust
// ❌ 禁止
let mut result = Vec::new();
for node in nodes {
    if node.is_css() {
        result.push(transform(node));
    }
}

// ✅ 必须
nodes.into_iter()
    .filter(|n| n.is_css())
    .map(transform)
    .collect::<Vec<_>>()

// ✅ 带错误传播
nodes.into_iter()
    .try_fold(Vec::new(), |mut acc, node| {
        acc.push(transform(&node)?);
        Ok::<_, CompileError>(acc)
    })
```

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

## 📋 代码风格强制

### match 优于 if-else

```rust
// ❌ 禁止
if token == "{" { ... }
else if token == "}" { ... }
else if token == ";" { ... }

// ✅ 必须
match token {
    "{" => ...,
    "}" => ...,
    ";" => ...,
    _ => ...,
}
```

### ? 优于 match Err

```rust
// ❌ 禁止
let result = match parse(tokens) {
    Ok(ast) => ast,
    Err(e) => return Err(e),
};

// ✅ 必须
let ast = parse(tokens)?;
```

### partition 优于 for + if

```rust
// ❌ 禁止
let mut left = Vec::new();
let mut right = Vec::new();
for x in items {
    if pred(x) { left.push(x) } else { right.push(x) }
}

// ✅ 必须
let (left, right): (Vec<_>, Vec<_>) = items.into_iter().partition(pred);
```

---

## 🔬 Tracing 规范

### 首选 `#[instrument]`

```rust
#[tracing::instrument(skip(large_param), fields(result = tracing::field::Empty))]
fn my_function(large_param: &BigType, input: &str) -> Result<...> {
    let result = do_work(large_param, input)?;
    tracing::Span::current().record("result", &result);
    Ok(result)
}
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
- `src/serialize/mod.rs` 中的内联测试是历史遗留，新测试不添加

### 测试命名

```rust
#[test]
fn test_token_stream_emits_expected_semicolon() { ... }

#[test]
fn test_expand_handles_nested_rule_correctly() { ... }
```

### 诊断工具

```bash
# 基础 tracing
RUST_LOG=debug cargo test test_name 2>&1 | head -100

# 极简输出
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

## 📁 文件结构规范

| 文件 | 行数上限 | 说明 |
|------|---------|------|
| 管道阶段文件 | 300 | lexer/parser/serialize |
| 业务逻辑文件 | 400 | eval/mod.rs, expr.rs |
| 类型定义文件 | 500 | types.rs |
| 测试文件 | 500 | tests/ 目录 |

---

## 📚 参考

- **rxrust 完整参考**：`.claude/AGENT.md`（旧版，包含算子速查）
- **rxrust Skill**：`.claude/skills/rxrust/SKILL.md`
- **OpenSpec 工作流**：`.claude/skills/openspec-*/SKILL.md`
