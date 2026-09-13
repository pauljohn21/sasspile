## Context

sasspile 是纯 Rust 函数式 SCSS 编译器。当前架构是同步管线，但模块加载层存在 IO 串行瓶颈。同时 Reactor 携带大量未使用字段，代码风格有命令式残留。

本次设计目标：**内部 async（tokio 模块加载）+ 外部同步 API + 死代码清理 + 函数式风格强化**。

## Goals / Non-Goals

### Goals

- 模块加载使用 `tokio::fs` 异步读文件
- 内部 async 不暴露给调用方（公开 API 仍返回 `Result<String>`）
- 删除所有已识别的死代码（结构体、字段、trait、impl）
- 强化函数式风格（消除 for+push、if-else 链、match-Err-return）

### Non-Goals

- 不做全管线 async（lex/parse/serialize 仍同步，无收益）
- 不做模块间并行编译（仅 IO 层 async）
- 不改变任何公开 API 签名

## Architecture

### Async 边界设计

```
┌───────────────────────────────────────────────────────────────┐
│ 公开 API (sync)                                                │
│  compile(input, style) -> Result<String>                       │
│       │                                                        │
│       ▼                                                        │
│  Global tokio Runtime                                   │
│  RT.get_or_init() → OnceLock<tokio::runtime::Runtime>  │
│       │                                                        │
│       ▼                                                        │
│  runtime().block_on(async_pipeline)                             │
│       │                                                        │
│       ├── .lex()?          sync (Iterator, 内存操作)            │
│       ├── .parse()?        sync (Parser::parse, 内存操作)      │
│       ├── .evaluate().await?  ← ASYNC 入口                    │
│       │     │                                                  │
│       │     ├── eval_nodes()    sync (AST 递归求值)            │
│       │     ├── eval_use()      sync dispatch                  │
│       │     │     └── load_module().await  ← tokio::fs IO     │
│       │     └── eval_import()   sync dispatch                  │
│       │           └── load_import().await  ← tokio::fs IO     │
│       ├── .serialize(style) sync (字符串构建)                  │
│       └── .finish()        sync                                │
└───────────────────────────────────────────────────────────────┘
```

### 全局 Runtime 初始化

```rust
// lib.rs
use std::sync::OnceLock;
use tokio::runtime::Runtime;

static RT: OnceLock<Runtime> = OnceLock::new();

fn runtime() -> &'static Runtime {
    RT.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .thread_name("sasspile-worker")
            .worker_threads(2)
            .build()
            .expect("failed to initialize tokio runtime")
    })
}
```

### 模块加载 async 化

```rust
// module.rs — load_module
async fn load_module(
    path: &Path,
    config: &[(String, Value)],
    caller_env: &Env,
    validate_config: bool,
) -> Result<ModuleExports> {
    // 缓存检查 (sync)
    if caller_env.loaded_modules.contains(path) { ... }

    // 异步读文件
    let source = tokio::fs::read_to_string(path).await
        .map_err(|e| SassError::Module(format!("Cannot read {}: {e}", path.display())))?;

    // lex+parse 是 CPU 密集但快速, 直接同步执行
    // (spawn_blocking 仅在大规模模块编译时有收益, 暂不用)
    let tokens: Vec<Token> = Lexer::new(&source)
        .filter(|t| !matches!(t.as_ref(), Ok(Token::Eof)))
        .collect::<Result<Vec<_>>>()?;
    let ast = crate::parse::parse(&tokens)?;

    // eval (sync, 内部递归有 depth 限制)
    let (module_css, final_env) = Self::eval_nodes(&ast.nodes, env)?;
    // ... 后续处理不变 ...
}
```

### Reactor 瘦身

删除字段前后对比：

```
BEFORE (14 fields):              AFTER (8 fields):
─────────────────────            ─────────────────────
text: String                     (moved into lex, not stored)
base_path: Option<PathBuf>       base_path: Option<PathBuf>
load_paths: Vec<PathBuf>         load_paths: Vec<PathBuf>
tokens: Option<Vec<Token>>       tokens: Option<Vec<Token>>
ast: Option<Ast>                 ast: Option<Ast>
serialized: Option<String>       serialized: Option<String>
env: Option<Env>              ✗  (removed)
modules: HashMap<...>         ✗  (removed - ModuleCacheEntry)
imports_seen: Vec<PathBuf>       imports_seen: Vec<PathBuf>
io_log: Vec<IoRecord>         ✗  (removed)
css_nodes: Vec<CssNode>          css_nodes: Vec<CssNode>
warnings: Vec<Warning>        ✗  (removed)
io: Arc<dyn ReactorIO>        ✗  (removed)
trace: ReactorTrace              trace: ReactorTrace
_state: PhantomData<S>           _state: PhantomData<S>
```

### ReactorTrace 修复

```rust
// BEFORE: advance 丢失 entered_at
pub fn advance(&self, stage: CompileStage) -> Self {
    Self {
        trace_id: self.trace_id,
        stage,
        entered_at: std::time::Instant::now(),  // BUG: 覆盖了初始时间
    }
}

// AFTER: 保留原始 entered_at
pub fn advance(&self, stage: CompileStage) -> Self {
    Self {
        trace_id: self.trace_id,
        stage,
        entered_at: self.entered_at,  // 保留管线入口时间
    }
}
```

### Parser 回归 Iterator

```rust
// parse/mod.rs — 删除 Stream/FusedStream impl
impl<'tok> Iterator for ParseStream<'tok> {
    type Item = Result<Node>;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_ws();
        if self.at_end() {
            return None;
        }
        Some(self.parse_node())
    }
}

// Parser::parse 同步
pub fn parse(tokens: &[Token]) -> Result<Ast> {
    ParseStream::new(tokens)
        .collect::<Result<Vec<_>>>()
        .map(|nodes| Ast { nodes })
}
```

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| 全局 runtime 首次调用延迟 | `OnceLock` 只初始化一次, 后续调用零开销 |
| 嵌套 `block_on` (如测试中内嵌调用) | 使用 `tokio::runtime::Handle::try_current()` 检测并 fallback |
| async 递归函数栈深度 | 模块加载已有 `depth > 50` 限制, async 不增加额外栈消耗 |
| Send/Sync bound (Env 含 Rc) | Env/ModuleExports 含 `Rc`, 不能跨 await 点。模块加载函数内部完成所有 Env 操作, 不将 Rc 传入 spawn_blocking |

## Decision: Env 的 Rc 与 async 的兼容性

`Env` 和 `ModuleExports` 广泛使用 `Rc<Scope>` 和 `Rc<HashMap>` — 它们不是 `Send`。这意味着：

- 不能将 `Env` 传入 `spawn_blocking`（需要 `Send`）
- 不能在 `.await` 点持有跨越的 `Env` 引用

**决策**：`load_module` / `load_import` 的 async 仅用于文件 IO（`tokio::fs::read_to_string`），eval 阶段保持同步且不跨越 `.await` 点。这样 `Env` 的 `Rc` 不触碰 async 边界，无需改为 `Arc`。

```rust
// 正确: 读文件是 await, 但 Env 操作在 .await 之前
async fn load_module(...) -> Result<ModuleExports> {
    let source = tokio::fs::read_to_string(path).await?;  // IO async
    // ... 后续全部 sync, Env 的 Rc 不跨 await ...

    // 如果后续需要 spawn_blocking, 必须 clone 所需数据:
    // let module_cache = caller_env.get_module_cache().clone();
    // tokio::task::spawn_blocking(move || { ... }).await?
}
```
