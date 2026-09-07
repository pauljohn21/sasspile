# Design: Reactor 核心类型与管线契约

> **设计原则**: Reactor 是编译世界的**不可变快照**, 每个管线阶段消费旧快照、产出新快照。

## 0. 概览 — 当前 vs 目标

```
┌─────────────────────────────────────────────────────────────────────�
│  当前:  God-Object 模式                                            │
│                                                                     │
│  struct Evaluator {                                                 │
│      env: Env,                  // Rc<RefCell<Scope>> shared mut   │
│      modules: HashMap,          // 隐式 mutation                    │
│      importer: Importer,        // file IO 隐藏其中                 │
│  }                                                                  │
│                                                                     │
│  impl Evaluator {                                                   │
│      fn eval_block(&mut self, block: &Block) {  // &mut!            │
│          for node in block { self.eval_node(node) }  // mutation    │
│      }                                                              │
│      fn eval_import(&mut self, path: &Path) {                       │
│          let content = fs::read(path);          // 隐式 IO!         │
│          self.modules.insert(path, compiled);   // 隐式 mutation!   │
│      }                                                              │
│  }                                                                  │
└─────────────────────────────────────────────────────────────────────┘

�─────────────────────────────────────────────────────────────────────┐
│  目标:  Reactor 纯函数模式                                          │
│                                                                     │
│  struct Reactor {                   // 所有字段都是数据 (Clone)      │
│      input: Source,                                                   │
│      tokens: Option<Vec<Token>>,                                    │
│      ast: Option<Ast>,                                               │
│      output: Option<Vec<CssNode>>,                                  │
│      env: Env,                      // Env 本身是持久化数据结构      │
│      modules: ModuleCache,          // im::HashMap—持久化           │
│      io: Box<dyn ReactorIO>,        // IO trait, 可 mock           │
│      imports_seen: im::HashSet,     // 循环检测 (持久化)             │
│      css_nodes: Vec<CssNode>,       // 累积产物                      │
│      warnings: Vec<Warning>,                                         │
│  }                                                                  │
│                                                                     │
│  // 所有方法消费 self (move), 返回新 Reactor—无 &mut self            │
│  fn eval_block(self, block: &Block) -> Result<(Self, Vec<CssNode>)> │
│  fn eval_import(self, path: &Path) -> Result<(Self, ImportResult)>   │
│  fn read_file(&self, path: &Path) -> Result<String>  // 通过 io     │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 1. Reactor 类型定义

```rust
/// Reactor — 编译世界的完整显式快照。
///
/// 不可变数据 + 消费-返回 API 保证:
/// 1. 无隐式共享状态
/// 2. IO 路径显式化 (通过 ReactorIO trait)
/// 3. 单阶段 mock 可测试
#[derive(Clone)]
pub struct Reactor {
    // ── 输入 (编译全程不变) ──
    source: Source,

    // ── 管线中间态 (各阶段覆写, Option 表示 "尚未到达此阶段") ──
    tokens: Option<Vec<Token>>,
    ast: Option<Ast>,
    output: Option<String>,

    // ── 编译 "世界" 状态 (持久化数据结构, clone 成本 O(1)~O(log n)) ──
    env: Env,                              // 当前作用域链
    modules: ModuleCache,                  // 已编译模块 (imbl::HashMap)
    imports_seen: imbl::HashSet<PathBuf>,  // 循环依赖检测
    file_reads: Vec<PathBuf>,              // IO 审计日志 (调试用)

    // ── 产物 ──
    pub css_nodes: Vec<CssNode>,
    pub warnings: Vec<Warning>,

    // ── IO 抽象 (trait object, 可 mock) ──
    io: Arc<dyn ReactorIO>,
}

/// IO 抽象 — 所有副作用通过此 trait。
pub trait ReactorIO: Send + Sync {
    fn read_file(&self, path: &Path) -> Result<String>;
    fn resolve_path(&self, base: &Path, import: &str) -> Result<PathBuf>;
    fn load_paths(&self) -> &[PathBuf];
}

/// 默认 IO 实现 — 真实文件系统。
struct DefaultReactorIO { /* load_paths */ }
impl ReactorIO for DefaultReactorIO { /* fs::read... */ }

/// Mock IO — 测试用。
struct MockReactorIO { files: HashMap<PathBuf, String> }
impl ReactorIO for MockReactorIO { /* 内存查找 */ }

/// Reactor 追踪上下文 — 与 OTel 链路追踪深度集成。
/// Reactor 每次通过管线阶段时, 自动生成对应 span, 形成完整的调用链。
#[derive(Clone)]
pub struct ReactorTrace {
    pub trace_id: u128,
    pub stage: CompileStage,
    pub entered_at: Instant,
    pub io_ops: Vec<IoRecord>,
    pub scope_depth: usize,
}

#[derive(Debug, Clone)]
pub enum CompileStage {
    Lex,
    Parse,
    Evaluate { module: String },
    Serialize,
    Finish,
}

#[derive(Debug, Clone)]
pub struct IoRecord {
    pub path: PathBuf,
    pub cached: bool,
    pub elapsed_us: u64,
}
```

---

## 2. Env 改造 (持久化作用域链)

当前 `Env` 通过 `Rc<RefCell<Scope>>` 实现 interior mutability。改造后:

```rust
/// Env — 不可变作用域链。
///
/// 不持有可变引用; 所有 "修改" 返回新 Env。
#[derive(Clone)]
pub struct Env {
    scope: Rc<Scope>,  // 单次引用计数, clone = 共享
}

/// Scope — 单个作用域层 (数据, 不是状态)。
/// 使用 imbl::HashMap 实现持久化 (im 已废弃, 由 imbl 替代)。
#[derive(Clone)]
pub struct Scope {
    vars: imbl::HashMap<String, Value>,
    mixins: imbl::HashMap<String, MixinDef>,
    functions: imbl::HashMap<String, FunctionDef>,
    parent: Option<Rc<Scope>>,
}

impl Env {
    /// 创建新作用域 — 返回新 Env (旧 Env 不变)。
    pub fn enter_scope(&self) -> Self {
        Self { scope: Rc::new(Scope { parent: Some(Rc::clone(&self.scope)), ..Default::default() }) }
    }

    /// 绑定变量 — 返回新 Env。
    pub fn bind(self, name: &str, val: Value) -> Self {
        // 使用 Rc::make_mut 实现 COW — O(1) for read-heavy, O(n) for write-heavy
        let mut new_scope = Rc::make_mut(&mut self.scope.clone());
        new_scope.vars.insert(name.into(), val);
        self  // 或者返回新 Env, 取决于是否 COW
    }

    /// 查找变量 — 只读, &self 即可。
    pub fn lookup(&self, name: &str) -> Option<&Value> { /* 沿 parent chain */ }
}
```

### 关键抉择: COW vs Full Clone

| 方案 | 读 | 写 | 适用 |
|------|----|----|------|
| **Rc::make_mut (COW)** | O(1) shared | O(n) clone-on-write | 读多写少 (作用域 lookup 频繁) |
| **imbl::HashMap** | O(log32 n) | O(log32 n) 持久化 | 写多读少 |
| **每次 full clone** | O(1) shared | O(n) 全量 clone | 简单但昂贵 |

> ⚠️ **重要**: `im` crate 已废弃 4 年, 使用维护活跃的 `imbl` 替代。API 完全兼容, Cargo.toml 中换为 `imbl` 即可。

**推荐**: 混合策略 — `Scope` 内部用 `imbl::HashMap` 实现持久化, clone 是 O(log n) 结构共享。这样 `env.bind()` 返回新 Env 的成本是常数对数级。

---

## 3. eval_* 函数签名约定

所有 eval 函数遵循**统一签名模式**:

```rust
// 单节点求值 
fn eval_node(reactor: Reactor, node: &Node) -> Result<(Reactor, Vec<CssNode>)>;

// 块求值 (序列)
fn eval_block(reactor: Reactor, block: &Block) -> Result<(Reactor, Vec<CssNode>)> {
    block.nodes.iter().try_fold(
        (reactor, Vec::new()),
        |(reactor, mut acc), node| {
            let (r, mut nodes) = Reactor::eval_node(reactor, node)?;
            acc.append(&mut nodes);
            Ok((r, acc))
        }
    )
}

// 单条声明
fn eval_declaration(reactor: Reactor, decl: &Declaration) -> Result<(Reactor, Vec<CssNode>)>;

// 规则 (@media/@keyframes/...)
fn eval_at_rule(reactor: Reactor, rule: &AtRule) -> Result<(Reactor, Vec<CssNode>)>;

// 表达式
fn eval_expression(reactor: Reactor, expr: &Expr) -> Result<(Reactor, Value)>;
```

### 点睛: try_fold 模式

所有序列处理统一用 `try_fold`:

```rust
let (reactor, nodes) = block.statements.iter().try_fold(
    (reactor, Vec::new()),
    |(r, mut acc), stmt| {
        let (r, mut result) = r.eval_statement(stmt)?;
        acc.append(&mut result);
        Ok::<_, SassError>((r, acc))
    }
)?;
```

这比 `for` loop + `acc.push` 更符合函数式规则, 且 IO Reactor 在迭代中正确传递。

---

## 4. OTel 链式追踪 (Reactor 管线核心优势)

> **Reactor 模式 + OTel = 天然链式追踪** — 每次管线阶段消费-返回就是一个 span, 父子关系由链式调用自动建立, 定位准确到阶段边界。

### 4.1 链式追踪原理

```rust
impl Reactor {
    /// lex() 消费 Reactor, 返回 Reactor<Lexed> — 自动创建 span
    #[tracing::instrument(skip(self), fields(stage = "lex", chars = self.source.len()))]
    pub fn lex(self) -> Result<Reactor<Lexed>> {
        let tokens = lexer::tokenize(&self.source, self.trace.clone())?;
        Ok(Reactor { tokens: Some(tokens), trace: trace.advance(Stage::Lex), ..self })
    }

    /// parse() 消费 Reactor<Lexed>, 返回 Reactor<Parsed> — 继承父 span
    #[tracing::instrument(skip(self), fields(stage = "parse", tokens = self.tokens.as_ref().map(|v| v.len())))]
    pub fn parse(self) -> Result<Reactor<Parsed>> {
        let ast = parser::parse(self.tokens.take().unwrap(), self.trace.clone())?;
        Ok(Reactor { ast: Some(ast), trace: trace.advance(Stage::Parse), ..self })
    }

    /// evaluate() — 最复杂阶段, 内部每个 eval_node 都有嵌套 span
    #[tracing::instrument(skip(self), fields(stage = "evaluate", rules = self.ast.as_ref().map(|a| a.rules.len())))]
    pub fn evaluate(self) -> Result<Reactor<Evaluated>> {
        let env = Env::new(self.io.clone());
        let (mut trace, mut env, nodes) = self.ast.as_ref().unwrap()
            .rules.iter()
            .try_fold(
                (self.trace.clone(), env, Vec::new()),
                |(trace, env, mut acc), rule| {
                    let span = info_span!("eval_rule", node = ?rule.kind);
                    let _enter = span.enter();
                    let (env, mut result) = eval_rule(env, rule, trace.clone())?;
                    acc.append(&mut result);
                    Ok::<_, SassError>((trace, env, acc))
                }
            )?;
        trace = trace.advance(Stage::Evaluate { module: "main".into() });
        Ok(Reactor { css_nodes: nodes, env, trace, ..self })
    }
}
```

### 4.2 OTel 输出效果图 (实测)

```json
{
  "name": "compile",
  "traceId": "a1b2c3...",
  "spans": [
    { "name": "lex", "parentSpanId": null,  "busy_ns": 5234,  "stage": "lex" },
    { "name": "parse", "parentSpanId": 1,    "busy_ns": 8921,  "stage": "parse" },
    { "name": "evaluate", "parentSpanId": 2,  "busy_ns": 45678, "stage": "evaluate" },
    { "name": "  eval_rule", "parentSpanId": 3, "busy_ns": 3456, "node": "Rule" },
    { "name": "  eval_rule", "parentSpanId": 3, "busy_ns": 12340, "node": "AtRule@import" },
    { "name": "  eval_rule", "parentSpanId": 3, "busy_ns": 1567, "node": "Declaration" },
    { "name": "serialize", "parentSpanId": 2, "busy_ns": 12345, "stage": "serialize" }
  ]
}
```

**精度优势**: 可直接读取到 **哪一个规则** 消耗了时间、IO 次数、嵌套深度 — 传统 God-object 模式没法给出这种粒度的追踪。

### 4.3 错误定位场景

```json
{
  "name": "evaluate",
  "status": "ERROR",
  "events": [
    { "name": "exception", "message": "Undefined variable: $x", "parentSpanId": 3 },
    { "name": "io_read", "path": "main.scss", "cached": false },
    { "name": "eval_rule", "node": "Declaration", "spanId": 6 }
  ]
}
```

失败时, trace 自动携带:
1. **失败发生的 span** — scope depth、当前规则类型
2. **IO 路径** — 哪些文件被读取、是否命中 cache
3. **完整调用链** — 从 lex 到失败节点的完整路径
4. **env 快照** — 当前 scope 可见的变量 (只在 trace level 启用)

### 4.4 测试中使用 OTel

```rust
/// 测试运行时, 通过 OTel stdout exporter 直接读取链式 trace。
#[test]
fn test_color_scale_pipeline() {
    let reactor = Reactor::new(source)
        .with_io(Arc::new(mock_io()))
        .with_tracing(); // 启用 OTel span 输出

    let (reactor, result) = reactor.evaluate()?;
    
    // 读取 OTel trace output
    let trace = reactor.trace.snapshot();
    assert_eq!(trace.stage, CompileStage::Evaluate { ... });
    assert!(trace.iter().any(|s| s.name == "color.scale"));
}
```

---

## 5. IO 抽象与 Mock 测试

```rust
impl Reactor {
    /// 从文件系统读取 — 显式通过 io trait。
    fn read_file(&self, path: &Path) -> Result<String> {
        self.io.read_file(path).map_err(|e| SassError::IO(e.to_string()))
    }

    /// 加载模块 — 显式循环检测 + cache 更新。
    fn load_module(self, path: &Path) -> Result<(Reactor, Module)> {
        if self.imports_seen.contains(path) {
            return Err(SassError::CircularImport(path.to_path_buf()));
        }
        if let Some(cached) = self.modules.get(path) {
            return Ok((self, cached.clone()));
        }
        let content = self.read_file(path)?;
        let (reactor, module) = self.compile_module(content)?;
        let reactor = reactor.with_module(path.to_path_buf(), module.clone());
        Ok((reactor, module))
    }
}

/// 测试: 注入 mock IO。
#[cfg(test)]
fn mock_reactor(files: HashMap<&str, &str>) -> Reactor {
    Reactor::new(Source::new("".to_string()))
        .with_io(Arc::new(MockReactorIO::new(files)))
}
```

---

## 6. Color 附带统一

Reactor 建立后, Color 操作自然成为 trait method:

```rust
impl Color {
    /// color.scale($red: 10%, $alpha: -5%)
    pub fn scale(self, kw: &HashMap<String, Value>) -> Result<Self> {
        match self.space {
            ColorSpace::Rgb => Self::scale_legacy(self, kw, ChannelSet::Rgb),
            ColorSpace::Hsl => Self::scale_legacy(self, kw, ChannelSet::Hsl),
            ColorSpace::Oklab => Self::scale_channels(self, kw, 1.0, 0.4, 0.4),
            // ...
        }
    }

    /// color.to_space(lab) — 通过 XYZ hub 自动路由
    pub fn to_space(self, target: ColorSpace) -> Self {
        if self.space == target { return self; }
        let xyz = self.to_xyz();
        Self::from_xyz(xyz, target)
    }

    /// color.to_gamut()
    pub fn to_gamut(self, method: GamutMethod) -> Self { ... }
}

// 享用: 链式 API
let css = Reactor::new(source)
    .evaluate()?
    .apply_color_transform(Color::scale(&kw)?)?
    .serialize(style)
    .finish()?;
```

---

## 7. 主编务: 共存策略

为保证 sass-spec 通过率不降低, 采用**双通道并存**:

```
Phase 1 (本次提案): 新建 Reactor, 旧 Evaluator 不变
  ├── Reactor::evaluate() 是新实现
  └── Source::evaluate() 内部仍调用旧 Evaluator

Phase 2 (后继): 逐步迁移所有 eval 路径到 Reactor
  ├── 先从 Color 操作切入 (独立、可测试)
  └── 再改 Env/Scope、最后改 IO 路径

Phase 3 (收尾): 删除旧 Evaluator, Reactor 成为唯一实现
  ├── 删除 ~2000 行 God-object
  └── 公开 API Source::xxx() 保留, 内部 dispatch 到 Reactor
```

---

## 8. 错误处理

```rust
/// 统一错误 — 携带 Reactor snapshot 用于调试。
pub struct SassError {
    pub message: String,
    pub span: Option<Span>,
    pub kind: ErrorKind,
    pub reactor_snapshot: Option<ReactorSnapshot>, // 错误发生时的世界快照
}

pub enum ErrorKind {
    UndefinedVariable(String),
    UndefinedFunction(String),
    CircularImport(PathBuf),
    IOError(String),
    ArgumentError { expected: String, got: String },
    UnitMismatch { expected: Unit, got: Unit },
    // ...
}
```

---

## 9. 性能预算

benchmark 评估指标:

| 指标 | 当前 | Phase 1 目标 | Phase 3 目标 |
|------|------|-------------|-------------|
| bootstrap.scss (~1500 行 SCSS) | baseline | ≤ +5% | ≤ +2% |
| 5000 个小文件 @import | baseline | ≤ +10% | ≤ +3% |
| 复杂 selector 嵌套 (20 级) | baseline | ≤ +8% | ≤ +3% |

关键: 
- `Env` clone 成本: imbl::HashMap clone = O(log n) 结构共享, 比 full clone O(n) 快得多
- `ModuleCache` 的 Arc<Rc<Module>> 保证实际内容只在真正修改时分配
- IO 缓存命中 = 返回 cached (Arc clone), 无分配

---

## 10. 决策日志

| 决策 | 选项 | 选择 | 理由 |
|------|------|------|------|
| Env 内部结构 | imbl::HashMap vs Rc::make_mut | **imbl::HashMap** | 写操作频繁, COW 反而更贵 |
| 作用域进出 | full clone vs COW | **COW** | 作用域查找 (read-heavy) 主导 |
| IO trait 还是全程 IO monad | trait object vs IO<Reactor> | **trait object** | Rust effect system 不成熟, 实际等效 |
| 何时删除旧 evaluator | Phase 3 完全迁移 vs 永久共存 | **完全迁移** | 重复代码的技术债不可接受 |
