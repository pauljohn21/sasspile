## Context

当前 `Env` 是一个扁平结构，包含 6 个 HashMap（`local_vars`、`local_mixins`、`local_functions`、`forwarded_vars`、`forwarded_mixins`、`forwarded_functions`）+ 全局字段（`global_writes`、`content`、`namespaces` 等）。作用域进出通过以下方式：

1. **进入 rule scope**：`eval_rule` clone 6 个 HashMap 作为快照（`rule.rs:229-234`）
2. **退出 rule scope**：`exit_scope` 用 `std::mem::take` 提取 rule 体内变更，恢复快照，再选择性传播 `!global` 写入、命名空间变量、新增 mixin/function
3. **flow control**（`@if`/`@for`/`@each`）：不创建新 scope，直接在当前 env 上操作 — **已符合 SCSS 规范**
4. **`@content` 快照**：`content_env` 用 `Rc<Env>` 保存整个环境
5. **mixin/function 调用**：`call_user_function` 中 clone env 创建新作用域

### SCSS 作用域规则（sass-spec 官方手册）

| 构造 | 创建新作用域 | 行为 |
|------|-------------|------|
| 规则体 `{...}` | 是 | 局部变量不传播到外层 |
| mixin 调用 | 是 | 参数和局部变量在新作用域 |
| function 调用 | 是 | 独立作用域，不访问调用方局部变量 |
| `@if`/`@else if`/`@else` | **否** | 在当前作用域内修改 |
| `@for` | **否** | 循环变量绑定到当前作用域 |
| `@each` | **否** | 同上 |
| `@while` | **否** | 同上 |
| `!global` | N/A | 强制写入全局作用域 |
| `!default` | N/A | 仅在变量未定义时赋值 |

### 约束

- `Env` 方法 `self -> Self`（move 语义，链式调用）
- 禁止 `env.clone()`（除 `@content` 快照）
- 禁止 `Rc::make_mut`、`RefCell`
- `eval_xxx` 接收 `Env`（move），返回 `(Vec<CssNode>, Env)`
- 只读方法保持 `&Env`

## Goals / Non-Goals

**Goals:**

- 消除 `eval_rule` 中的 6 次 HashMap clone（每次规则进入）
- 消除 `call_user_function` 中的 env clone
- `@content` 快照从 clone 整个 `Env` 降为 clone 一个 `Rc<Scope>` 指针
- 保持 `Env` 的 `self -> Self` move 语义 API 不变
- 保持所有 202 个核心测试通过
- 保持 sass-spec 通过率不降低

**Non-Goals:**

- 不改变 `ModuleExports` 结构（forwarded 表管理保持现状）
- 不改变 `eval_xxx` 方法签名（`Env` move 语义不变）
- 不改变 flow control 的作用域行为（已符合规范）
- 不改变 `Rc<Vec<(String, String, bool, Option<PathBuf>)>>` 的 extends 管理

## Decisions

### D1: `Scope` 结构体 + `Rc<Scope>` 父链

**选择**：引入 `Scope` 结构体，包含单层 `local_vars`/`local_mixins`/`local_functions`/`forwarded_*`，通过 `parent: Option<Rc<Scope>>` 链接父作用域。

**替代方案**：`Vec<Scope>` 栈结构 — 被否决，因为 `Env::clone`（`@content` 快照）需要复制整个 Vec，而 `Rc<Scope>` 链只需复制一个指针。

```rust
struct Scope {
    local_vars: HashMap<String, Value>,
    local_mixins: HashMap<String, MixinDef>,
    local_functions: HashMap<String, FunctionDef>,
    forwarded_vars: HashMap<String, Value>,
    forwarded_mixins: HashMap<String, MixinDef>,
    forwarded_functions: HashMap<String, FunctionDef>,
    parent: Option<Rc<Scope>>,
    // 全局写入在此层积累
    global_writes: HashMap<String, Value>,
}
```

### D2: `Env` 重构为 `Rc<Scope>` + 全局字段

**选择**：`Env` 持有 `current: Rc<Scope>`（当前活跃作用域），外加不参与作用域链的全局字段。

```rust
pub struct Env {
    current: Rc<Scope>,
    // 以下字段不参与作用域链
    content: Option<Rc<Vec<Node>>>,
    content_env: Option<Rc<Env>>,
    builtin_modules: Vec<String>,
    namespaces: HashMap<String, Rc<ModuleExports>>,
    base_path: Option<PathBuf>,
    depth: usize,
    extends: Rc<Vec<(String, String, bool, Option<PathBuf>)>>,
    current_selector: Option<String>,
    load_paths: Vec<PathBuf>,
    plain_css: bool,
    loaded_modules: Rc<HashSet<PathBuf>>,
    module_cache: Rc<HashMap<PathBuf, ModuleExports>>,
    pending_config: HashMap<String, Value>,
    consumed_config: HashSet<String>,
    star_members: HashMap<String, Vec<String>>,
    star_imported: HashSet<String>,
}
```

**关键**：`Env::clone` 只需 clone 一个 `Rc<Scope>`（原子计数器递增），而非 6 个 HashMap。

### D3: 变量查找 — 沿 scope 链向上搜索

**选择**：`lookup(name)` 从 `current` 开始，沿 `parent` 链向上搜索。

```rust
fn lookup(&self, name: &str) -> Option<&Value> {
    let mut scope = &*self.current;
    loop {
        if let Some(v) = scope.local_vars.get(name) {
            return Some(v);
        }
        scope = scope.parent.as_ref()?;
    }
}
```

**替代方案**：flat HashMap 查找 O(1) vs scope 链查找 O(depth) — 在 SCSS 中嵌套深度通常 < 20，性能差异可忽略。

### D4: 作用域进出 — 零 clone

**选择**：

```rust
// 进入 rule scope
fn enter_scope(self) -> Self {
    let new_scope = Rc::new(Scope {
        local_vars: HashMap::new(),
        local_mixins: HashMap::new(),
        local_functions: HashMap::new(),
        forwarded_vars: HashMap::new(),
        forwarded_mixins: HashMap::new(),
        forwarded_functions: HashMap::new(),
        parent: Some(Rc::clone(&self.current)),
        global_writes: HashMap::new(),
    });
    Self { current: new_scope, ..self }
}

// 退出 rule scope — 传播 !global 和新增 mixin/function
fn exit_scope(self) -> Self {
    // 取出 current 的 parent（恢复外层作用域）
    let parent = self.current.parent.clone();
    // 提取 current 的 global_writes 和 mixin/function 传播到 parent
    // ... 传播逻辑 ...
    Self { current: parent.unwrap(), ..self }
}
```

**关键**：无需 clone 任何 HashMap。`Rc::clone` 是原子计数器递增，开销极低。

### D5: `!global` 语义 — 沿链向上写入

**选择**：`bind_global(name, value)` 从 current 开始，沿 parent 链找到 root scope（`parent == None`），写入 root 的 `local_vars`。

**挑战**：`Rc<Scope>` 不可变，无法直接修改 root。

**方案**：用 `Rc::try_unwrap` 或 `Rc::make_mut` — 但 AGENTS.md 禁止 `Rc::make_mut`。

**替代方案**：在当前 scope 的 `global_writes` 表中记录，`exit_scope` 时传播到 parent。这与当前 `exit_scope` 逻辑一致，但 `global_writes` 从 `Env` 字段移到 `Scope` 字段。

### D6: `@content` 快照 — `Rc<Scope>` 指针

**选择**：`content_env` 保存 `Rc<Scope>`（当前作用域指针），而非整个 `Env` clone。

**关键**：`@content` 执行时从快照 scope 开始查找变量，符合 SCSS 闭包语义。

### D7: mixin/function 调用 — 新 scope 代替 clone

**选择**：`call_user_function` 用 `enter_scope()` 创建新作用域，而非 clone 整个 `Env`。

**关键**：mixin body 内的变量查找沿 scope 链向上到定义点，符合 SCSS 闭包语义。`captured_namespaces` 保持不变（模块导出仍然 clone 到 MixinDef/FunctionDef）。

## Risks / Trade-offs

### R1: scope 链查找 O(depth) vs flat HashMap O(1)

- **Risk**：深层嵌套规则查找变慢
- **Mitigation**：SCSS 嵌套深度通常 < 20，实际性能影响可忽略。如需优化，可加 flat cache（scope 退出时失效）

### R2: `!global` 无法直接修改 root scope

- **Risk**：`Rc<Scope>` 不可变，需通过 `global_writes` 中转
- **Mitigation**：`exit_scope` 时传播 `global_writes` 到 parent，逐层向上最终到达 root。与当前 `exit_scope` 逻辑一致

### R3: `exit_scope` 传播逻辑仍需 `std::mem::take`

- **Risk**：退出 scope 时仍需提取 current scope 的 `global_writes` 和 mixin/function
- **Mitigation**：`Rc::try_unwrap` 在引用计数为 1 时可直接获取所有权，避免 clone。通常 scope 退出时只有 `Env::current` 持有引用，`try_unwrap` 成功率高

### R4: API 兼容性

- **Risk**：大量 `eval_xxx` 方法内部访问 `env.local_vars` 等直接字段
- **Mitigation**：提供 `env.lookup(name)` 等方法替代直接字段访问。分阶段迁移，先添加新 API，再逐步替换直接访问

### R5: `with_namespace_var` 修改 namespace exports

- **Risk**：namespace exports 是 `Rc<ModuleExports>`，修改需 clone
- **Mitigation**：保持当前 `Rc::clone` + 重建模式不变，此操作不在 scope 链管理范围内

## Migration Plan

### Phase 1: 引入 `Scope` 结构体（不改变 `Env` 结构）

- 新增 `Scope` struct 到 `env.rs`
- 新增 `Scope` 的基本方法（`new`、`new_child`、`lookup` 等）
- 不修改 `Env` 结构，不影响现有代码

### Phase 2: `Env` 重构为 `Rc<Scope>` + 全局字段

- `Env.local_vars` 等字段移入 `Scope`
- `Env.current: Rc<Scope>` 替代 6 个 HashMap 字段
- 所有直接字段访问改为方法调用
- `enter_scope` / `exit_scope` 替代 `eval_rule` 中的 clone + `exit_scope`
- 全量测试验证

### Phase 3: `@content` 快照优化

- `content_env` 从 `Rc<Env>` 改为 `Rc<Scope>`
- `eval_content` 从 scope 快照恢复环境
- 测试验证

### Phase 4: mixin/function 调用优化

- `call_user_function` 用 `enter_scope` 替代 env clone
- 测试验证

### Rollback

- 每个 Phase 独立提交，可单独 revert
- Phase 1 可安全合入（不改变现有逻辑）
- Phase 2 是 breaking change，如出问题可 revert 到 Phase 1

## Open Questions

1. **`forwarded_*` 表是否应在 scope 链中传播？** — 当前 `exit_scope` 传播 forwarded 成员。如果 forwarded 表在 `Scope` 中，退出 scope 时需提取并传播到 parent
2. **`star_imported` / `star_members` 是否应在 scope 链中？** — 这些是模块系统字段，可能与 scope 链正交
3. **`pending_config` / `consumed_config` 的 scope 行为** — 这些是 `@use with` 配置，应在顶层 scope 管理
