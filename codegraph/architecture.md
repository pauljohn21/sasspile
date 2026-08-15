# 架构设计

## 总体架构

sasslipe 采用**管道-过滤器**模式（Pipeline-Filter），每个编译阶段作为独立 Tokio 任务运行，通过异步 channel 通信。

### 架构图

```
�──────────────────────────────────────────────────────────────────────────�
│                     sasslipe Pipeline                                    │
├──────────────────────────────────────────────────────────────────────────�
│                                                                          │
│   ┌────────┐    ┌────────┐    ┌─────────�    �──────────�               │
│   │ Source │───▶│  Lex   │───▶│  Parse  │───▶            │               │
│   │ Loader │    │        │    │         │    │ Semantic │               │
│   └────────┘    └────────┘    └─────────┘    │ Analysis │               │
│       │                                       │          │               │
│       │         ┌──────────┐    ┌──────────┐  │          │               │
│       │         │  Module  │◀──│  Graph   │�─┤          │               │
│       │         │ Resolver │    │ Analysis │  └──────────�               │
│       │         └──────────┘    └──────────┘       │                     │
│       │              │                             ▼                     │
│       │              │         ┌──────────┐  ┌──────────┐               │
│       │              │         │Transform │◀─│Resolved  │               │
│       │              │         │  Pass    │  │   AST    │               │
│       │              │         └──────────�  └──────────┘               │
│       │              │              │                                    │
│       │              ▼              ▼                                    │
│       │         ┌──────────┐  ┌──────────┐                             │
│       │         │ Evaluate │─▶│   CSS    │                             │
│       │         │  Engine  │  │   Gen    │                             │
│       │         └──────────�  └──────────┘                             │
│       │                             │                                   │
│       │                             ▼                                   │
│       │         ┌──────────┐  ┌──────────┐                             │
│       └────────▶│  Watch   │─▶│  Format  │                             │
│                 │  Service │  │  Output  │                             │
│                 └──────────┘  └──────────�                             │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
```

## 核心设计模式

### 1. Pipeline Stage Trait

```rust
#[async_trait]
pub trait PipelineStage<Input, Output> {
    async fn process(&self, input: Input) -> Result<Output>;
}
```

每个阶段实现此 trait，通过 `then` 组合子串联。

### 2. Watch-based 响应式状态

```rust
pub struct ReactiveEnv {
    vars: watch::Sender<Map<String, Value>>,
}

// 订阅变量变更
pub fn subscribe(&self) -> watch::Receiver<Map<String, Value>>;
// 设置变量触发下游更新
pub fn set_var(&self, name: &str, value: Value);
```

### 3. Async Builtin Function

```rust
#[async_trait]
pub trait SassFn: Send + Sync {
    async fn call(&self, args: &[Value], env: &Env) -> Result<Value>;
}

// 注册到 HashMap
let mut registry: HashMap<String, Box<dyn SassFn>> = HashMap::new();
registry.insert("rgb".into(), Box::new(RgbFn));
```

## 数据流

```
Bytes → Tokens → AST → Resolved AST → Transformed → Evaluated → CSS AST → String
  │        │       │         │            │           │          │         │
  │        │       │         │            │           │          │         │
  ▼        ▼       ▼         ▼            ▼           ▼          ▼         ▼
 Source  Token  Node     ResolvedNode  Value      Value      CssRule   String
```

## 模块依赖关系

```
hrx ──（独立，HRX 解析器）
  │
  └── sasslipe（规划中）
       ├── value-system      # 值类型系统（被所有阶段依赖）
       ├── source            # SourceSpan, SourcePosition
       ├── diagnostics       # 错误报告
       ├── lexer             # 词法分析（依赖 source, diagnostics）
       ├── parser            # 语法分析（依赖 lexer, source）
       ├── semantic          # 语义分析（依赖 parser）
       ├── eval              # 求值器（依赖 value-system, semantic）
       ├── builtin-modules   # 内置模块（依赖 eval）
       ├── css-gen           # CSS 生成（依赖 value-system）
       ├── incremental       # 增量编译（依赖 eval, semantic）
       └── pipeline          # 管道编排（依赖所有上述模块）
```

## 类型系统概览

```
Value
├── Number { value: f64, unit: Unit }
├── String(String)
├── Boolean(bool)
├── Null
├── Color(Color)           // sRGB
├── List(Vec<Value>, Separator)
├── Map(Vec<(Value, Value)>)
├── ArgList(Vec<Value>)    // 带参数的列表
├── Function(name: String)
├── Calculation(String)    // calc() 表达式
└── Error(String)
```

## 错误处理策略

- **沿途收集**：错误不中断管道，收集后统一报告
- **源码位置**：每个 AST 节点附带 `SourceLocation`
- **诊断级别**：Error / Warn / Info
- **可恢复性**：Parser 实现错误恢复（synchronization points）

## 性能考量

| 问题 | 策略 |
|------|------|
| 不可变数据内存开销 | `Arc<T>` 共享大对象 |
| sass-spec 边缘 case | 每日 CI 运行 + 自动标记 |
| 异步管道调试 | `tracing` span + `--trace` 标志 |
| 编译速度 | `moka` 缓存、并行编译、流式处理 |
