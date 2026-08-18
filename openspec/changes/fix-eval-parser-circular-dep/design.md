## Context

sasspile 当前的编译管道为 Lexer → Parser → Evaluator → Serializer。但 eval 层在 `@use`、`@import` 和插值求值时直接回调 parser/lexer，形成循环依赖：

```
Parser → AST → Evaluator → (遇到 @use/@import/interpolation) → Parser → AST → ...
```

CodeGraph 确认 `eval_use_rule`（`src/eval/mod.rs:310`）和 `eval_import_rule`（`src/eval/atrule.rs:277`）直接调用 `crate::parser::parse`。同时 `eval_interpolation_expr`（`src/eval/interp.rs:92-100`）也调用 `crate::lexer::tokenize` + `crate::parser::Parser`。

此外，`Env` 的 `std::mem::replace(env, Env::new_global())` + `*env = *func_env.parent.take().unwrap()` 模式在 panic 时会破坏环境链不变量。

## Goals / Non-Goals

**Goals:**

- 消除 eval → parser 的循环依赖
- 实现 `@use with ($var: value)` 配置注入
- 用闭包模式替换 `mem::replace` 作用域管理
- 所有跨函数管道用 tracing span 记录上下文

**Non-Goals:**

- 不重写整个编译管道架构
- 不修改 `@extend` 的字符串匹配逻辑
- 不修改 `color.rs` 行数超限

## Decisions

### 决策 1: 引入 `ModuleResolver` trait

**选择：** 在 `src/resolver/mod.rs` 中定义 trait，eval 层通过泛型参数或 `&dyn` 引用调用。

```rust
pub trait ModuleResolver {
    fn resolve(&mut self, url: &str, base_dir: &Path) -> Result<ResolvedModule, SassError>;
}

pub struct ResolvedModule {
    pub ast: Vec<Stmt>,
    pub is_css: bool,
    pub raw_content: Option<String>,
    pub source_path: PathBuf,
}
```

**替代方案 A（否决）：** 在 eval 层直接传入函数指针 `fn(&str) -> Result<Vec<Stmt>, SassError>`。否决原因：无法携带模块缓存状态。

**替代方案 B（否决）：** 将 `@use`/`@import` 处理提升到 compile 层。否决原因：`@use` 需要在求值过程中动态执行（变量决定加载哪个模块），无法在编译前静态决定。

### 决策 2: 默认实现 `FileResolver`

```rust
pub struct FileResolver {
    cache: HashMap<PathBuf, Vec<Stmt>>,
    loading: HashSet<PathBuf>,  // 循环引用检测
}
```

`FileResolver` 实现 `ModuleResolver` trait，封装 tokenize → parse 流程。缓存已解析的 AST 避免重复解析。

### 决策 3: `@use with` 配置注入流程

```
1. 解析 config 表达式在调用者环境中求值 → 得到 (name, Value) 对列表
2. 通过 ModuleResolver 解析模块文件 → 得到 AST
3. 创建子环境 Env::with_child_scope
4. 在子环境中预设配置变量（不带 !default，直接覆盖）
5. 在子环境中求值模块 AST
6. 收集模块公开成员（非 `-` 前缀的变量/函数/mixin）到 ModuleEnv
```

### 决策 4: `Env::with_child_scope` 闭包模式

```rust
impl Env {
    pub fn with_child_scope<R>(&mut self, f: impl FnOnce(&mut Env) -> R) -> R {
        let parent = std::mem::replace(self, Env::new_global());
        let mut child = Env::new_child(parent);
        let result = f(&mut child);
        // 恢复：从 child 中取回 parent，放回 self
        *self = *child.parent.take().unwrap();
        result
    }
}
```

即使 `f` panic，Rust 的 unwind 会跳过恢复代码，但 `self` 仍指向空全局环境——这在 panic 场景下可接受（进程即将终止或上层 catch_unwind 处理）。比当前的 `take().unwrap()` 更安全，因为不会在非 panic 场景下失败。

### 决策 5: 插值表达式预解析

对于 `eval_interpolation_expr` 中的复杂表达式（非纯变量引用），改为将插值内容在 parser 阶段就解析为 `Expr`，附加到 `InterpPart::Expr(Expr)` 中，消除 eval 层重新 tokenize+parse 的需要。

### 决策 6: 模块求值结果缓存（ModuleCache）

**背景：** 当前 `FileResolver.cache` 只缓存 AST（`Vec<Stmt>`），每次 `@use` 同一个模块时仍然重复执行 `eval_stmts`。这违反了 Sass spec 的核心语义——**每个模块只应求值一次**，后续 `@use` 复用第一次的求值结果。Element Plus 编译 bug #4 的根因正是 `common/var.scss` 被多次 `@use`，第二次求值时 `$box-shadow` 变量值被 `var()` CSS 函数调用覆盖。

**选择：** 在 eval 层引入 `ModuleCache`，缓存模块求值后的公开成员和 CSS 输出。

```rust
/// 已求值模块的缓存 — 保证每个模块只求值一次（Sass spec 要求）
pub struct ModuleCache {
    /// path → 求值后的公开成员 + CSS 输出
    evaluated: HashMap<PathBuf, EvaluatedModule>,
}

/// 模块求值结果 — 从第一次 @use 时收集
pub struct EvaluatedModule {
    /// 公开变量（非 `-` 前缀）
    pub vars: HashMap<String, Value>,
    /// 公开函数
    pub funcs: HashMap<String, UserFunction>,
    /// 公开 mixin
    pub mixins: HashMap<String, Mixin>,
    /// 模块产生的 CSS 输出（只输出一次）
    pub css_output: Vec<CssRule>,
}
```

**eval_use_rule 逻辑变更：**

```
1. resolver.resolve(url) → 拿到 AST（AST 层面有 FileResolver.cache 缓存，不重复 parse）
2. 检查 ModuleCache：
   a. 有缓存且无 with config → 直接取 EvaluatedModule，跳过 eval_stmts
   b. 有缓存但有 with config → 报错（Sass spec: with 只能在第一次 @use 时使用）
   c. 无缓存 → eval_stmts → 收集公开成员 → 存入 ModuleCache
3. 注册 ModuleEnv 到 env.modules
4. CSS 输出只追加第一次的 css_output（后续 @use 不重复输出 CSS）
```

**关键约束：**
- `@use "module" with (...)` 只能在模块**第一次**被加载时使用，后续重复 `@use` 同一模块带 `with` 是错误
- `@use` 不带 `with` 的后续加载不重复输出 CSS（Sass spec: `@use` 的 CSS 输出只在第一次加载时产生）
- `ModuleCache` 的 key 是 `PathBuf`（规范化的绝对路径），与 `FileResolver.cache` 的 key 一致
- 模块求值过程中产生的 `@extend` 请求也需要在第一次求值时收集

**替代方案 A（否决）：** 把求值缓存放入 `FileResolver`。否决原因：`FileResolver` 的职责是文件加载+parse，求值结果依赖 `Env`/`eval_stmts`，属于 eval 层职责，放入 resolver 会破坏分层。

**替代方案 B（否决）：** 不缓存，每次 `@use` 都重新求值。否决原因：违反 Sass spec 语义，且导致 Element Plus bug #4 无法修复。

### 决策 7: 实现 `@forward` 指令

**背景：** 当前 `@forward` 在 `eval_stmt` 中是 no-op（只打了一条 trace）。Sass spec 要求 `@forward` 将目标模块的公开成员（变量、函数、mixin）转发给当前模块的使用者，相当于在当前模块的公开接口中重新暴露被转发模块的成员。

**选择：** 实现 `eval_forward_rule`，复用 `ModuleResolver` 加载模块，复用 `ModuleCache` 缓存求值结果。

**`@forward` 语义流程：**

```
1. resolver.resolve(url) → 拿到 AST（或从缓存取）
2. 检查 ModuleCache：
   a. 有缓存 → 直接取 EvaluatedModule
   b. 无缓存 → eval_stmts → 收集成员 → 存入 ModuleCache
3. 将 EvaluatedModule 的成员注入当前模块的公开成员集合
   - 如果有 show 列表，只注入列表中的成员
   - 如果有 hide 列表，排除列表中的成员
   - 否则注入所有非 `-` 前缀的成员
4. @forward 不产生 CSS 输出（与 @use 不同）
```

**关键约束：**
- `@forward` 不产生 CSS 输出，只转发成员
- `@forward` 转发的成员在当前模块被 `@use` 时，对使用者可见
- `show`/`hide` 过滤只按成员名匹配，不区分类型（变量/函数/mixin）

## Risks / Trade-offs

| 风险 | 缓解措施 |
|------|---------|
| trait 引入动态分发开销 | 用泛型单态化代替 `&dyn`，零成本抽象 |
| 模块缓存增加内存占用 | `FileResolver.cache` 只存 AST；`ModuleCache` 只存公开成员+CSS输出；大项目可加 LRU |
| `with_child_scope` 仍依赖 `mem::replace` | panic 场景影响可接受；非 panic 路径保证不变量 |
| 插值预解析需要修改 parser 和 AST | `InterpPart::Expr` 已存在，只需 parser 阶段填充 |
| `@use with` 配置变量求值顺序与 Sass spec 差异 | 参照 sass-spec HRX 测试验证顺序 |
| `ModuleCache` 缓存后变量值被冻结，后续修改不生效 | Sass spec 要求模块只求值一次，这是正确行为 |
| `@forward` + `@use` 交互产生成员覆盖顺序问题 | 参照 sass-spec `@forward` 测试用例验证覆盖优先级 |
| `@forward` 的 show/hide 与 `@use` 的 with 组合语义复杂 | 先实现基础 `@forward`，`@forward ... as` 后续迭代 |

## Tracing Span 设计

跨函数管道必须用 tracing span 记录：

| Span 名称 | Stage | 关键字段 |
|-----------|-------|---------|
| `module_resolve` | eval | `url`, `resolved_path`, `is_css` |
| `module_cache_hit` | eval | `path`, `module` |
| `module_cache_store` | eval | `path`, `module`, `var_count`, `func_count` |
| `use_config_inject` | eval | `config_count`, `module` |
| `env_child_scope` | eval | `scope_depth`, `caller` |
| `interp_eval` | eval | `expr`, `result` |
| `eval_forward` | eval | `url`, `show_count`, `hide_count` |
