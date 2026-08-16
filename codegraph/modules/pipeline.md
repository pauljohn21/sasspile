# 管道编排 🔨 骨架存在

## 职责

将 7 个编译阶段编排为 Tokio 异步管道，支持背压、缓存、并行。

## 当前状态

**文件: `sasspile/src/pipeline.rs`**

目前只有 Compiler 骨架：

```rust
pub struct Compiler;

impl Compiler {
    pub fn new() -> Self { ... }
    pub async fn compile(&self, source: &str) -> Result<String> { ... }
    pub async fn compile_file(&self, path: &str) -> Result<String> { ... }
}
```

核心逻辑 (`todo!()`) 待实现：内部 channel 连接、阶段 spawn 尚未完成。

## 待实现文件结构

```
pipeline.rs              # 当前：Compiler 骨架
# 待添加：
# pipeline/backpressure.rs  # bounded channel 背压
# pipeline/concurrent.rs    # 多入口并行编译
# pipeline/tracing.rs       # 进度跟踪 span
```

## 计划架构

```rust
pub struct Pipeline {
    stages: Vec<Box<dyn PipelineStage>>,
}

impl Pipeline {
    pub async fn run(&self, input: SourceInput) -> CssOutput {
        // 每个 stage spawn 一个 tokio task
        // mpsc channel 连接各 stage
        // watch channel 用于增量更新
    }
}
```

## 7 阶段（当前遮通状态）

| 阶段 | 模块 | 状态 |
|------|------|------|
| Source | `source/` | ✅ |
| Lex | `lexer/` | ✅ |
| Parse | `parser/` | ✅ |
| Semantic | `semantic/` | ✅ |
| Transform | 待定义 | ❌ |
| Evaluate | `eval/` + `builtin/` | ✅ |
| Codegen | CSS 生成 | ❌ |

## 背压与取消（计划）

- **背压**：使用 bounded channel，满时自动阻塞上游
- **取消**：`CancellationToken` 传播
- **错误传播**：错误沿管道逆向传播

## 进度跟踪

- 每个 stage 添加 `tracing::span!`
- 记录输入大小、处理时间
- 错误收集与报告
