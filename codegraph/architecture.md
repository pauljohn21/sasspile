# 架构设计

## 总体架构

sasslipe 采用**管道-过滤器**模式（Pipeline-Filter），每个编译阶段作为独立 Tokio 任务运行，通过异步 channel 通信。

### 架构图（当前状态）

```
sasslipe Pipeline

┌─────────────────────────────────────────────────────────┐
│                                                         │
│  ┌────────┐    ┌────────┐    ┌────────┐    ┌────────┐  │
│  │ Source │───▶│  Lex   │───▶│  Parse │───▶│Semantic│  │
│  │ Loader │    │   ✅   │    │   ✅   │    │   ✅   │  │
│  └────────┘    └────────┘    └────────┘    └────────┘  │
│      │                                          │       │
│      │         ┌──────────┐    ┌──────────┐    ▼       │
│      │         │  Module  │◀───│  Graph   ┌────────┐  │
│      │         │ Resolver │    │ Analysis │Resolved│  │
│      │         └──────────┘    └──────────┘  │  AST   │  │
│      │                                │      └────────┘  │
│      │              ┌──────────┐       │         │       │
│      │              │Transform │       ▼         ▼       │
│      │              │  (待定义) │   ┌────────┐          │
│      │              └──────────┘   │ Eval   │          │
│      │                    │        │   ✅   │          │
│      │                    ▼        └────────┘          │
│      │              ┌──────────┐       │                │
│      │              │   CSS    │      ▼                │
│      └────────────▶│   Gen    │   ┌────────┐          │
│                    │   ❌     │◀──│ Builtin│          │
│                    └──────────┘   │   ✅   │          │
│                         │         └────────┘          │
│                         ▼                              │
│                    ┌──────────┐                        │
│                    │  Format  │                        │
│                    │  Output  │                        │
│                    └──────────┘                        │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

## 核心设计模式

### 1. 不可变值 (Arc<Value>)

所有 `Value` 变体都是不可变的，通过 `Arc<Value>` 在线程间共享。
克隆 Value 是廉价的（Arc 引用计数递增）。

### 2. 符号表 (SymbolTable)

作用域栈结构，支持嵌套的变量、函数、混入、占位符查找。

### 3. 模块依赖图 (ModuleGraph)

有向图结构，Kahn 算法拓扑排序，检测循环依赖。

### 4. Dispatch 路由

统一入口 `builtin::dispatch(name, args, ctx)` 按 `module.function` 格式路由。

## 数据流

```
Bytes → Tokens → AST → ResolvedAST → Transformed → Evaluated → CSS AST → String
  │        │       │         │            │           │          │         │
  ▼        ▼       ▼         ▼            ▼           ▼          ▼         ▼
 ✅       ✅      ✅        ✅          (待定义)       ✅         ❌        ❌
```

## 模块依赖关系（实际）

```
hrx (独立)
│
sasspile
├── source     ← 独立基础
├── diagnostics← 独立基础
├── value      ← 独立基础 (被所有阶段依赖)
│
├── lexer      ← source, diagnostics, value
├── parser     ← lexer, source, value
│
├── semantic   ← parser, source, value
│   ├── symbol_table
│   ├── module (ModuleGraph, CycleCheck)
│   ├── extend (SelectorRegistry)
│   └── definitions (DefinitionRegistry)
│
├── eval       ← value, semantic, builtin
│   ├── evaluator
│   ├── ops
│   ├── functions (dispatch to builtin)
│   └── collections
│
├── builtin    ← value, eval
│   ├── sass_color
│   ├── sass_math
│   ├── sass_list
│   ├── sass_map
│   ├── sass_string
│   └── sass_meta
│
├── pipeline   ← 编排所有待完成
│
└── css_gen    ← ❌ 待开发
```

## 值类型层级

```
Value
├── Number { value: f64, unit: Unit }
├── String(String, Quoted)
├── Boolean(bool)
├── Null
├── Color(SassColor)         // RGBA
├── List(Vec<Value>, Separator)
├── Map(Vec<(Value, Value)>)
├── ArgList(Vec<Value>)
├── Function(String)
├── Calculation(String)      // calc() 延迟
└── Error(String)            // 哨兵值
```

## 错误处理策略

- **沿途收集**：错误不中断流程，收集到 Diagnostics 统一报告
- **源码位置**：每个 Token/AST 节点附带 `SourceSpan`
- **诊断级别**：Error / Warn / Info
- **可恢复性**：Parser 实现错误恢复（synchronization points）

## 性能考量

| 问题 | 策略 |
|------|------|
| 不可变数据内存开销 | `Arc<T>` 共享大对象 |
| sass-spec 边缘 case | 每日 CI 运行 + 自动标记 |
| 异步管道调试 | `tracing` span + `--trace` 标志 |
| 编译速度 | 并行编译（待实现 moka 缓存） |

## 线程安全约束

所有跨 Task 共享的类型必须实现 `Clone + Send + Sync + 'static`：
- `Value` ✅
- `Token` ✅  
- `Diagnostics` ✅
- `SymbolTable`（通过 Arc 共享）✅
