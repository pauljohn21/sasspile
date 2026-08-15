# 管道编排（待开发）

## 职责

将 7 个编译阶段编排为 Tokio 异步管道，支持背压、缓存、并行。

## 计划文件结构

```
pipeline/
├── mod.rs           # 管道入口
├── backpressure.rs  # 背压与取消
├── concurrent.rs    # 并发编译
└── tracing.rs       # 进度跟踪
```

## 7 阶段

```
Source → Lex → Parse → Semantic → Transform → Evaluate → Codegen
```

## 管道实现

```rust
pub struct Pipeline {
    stages: Vec<Box<dyn PipelineStage>>,
}

impl Pipeline {
    pub async fn run(&self, input: impl Stream<Item = SourceInput>) -> impl Stream<Item = CssOutput> {
        // 每个 stage spawn 一个 tokio task
        // mpsc channel 连接各 stage
    }
}
```

## 背压与取消

- **背压**：使用 bounded channel，满时自动阻塞上游
- **取消**：`CancellationToken` 传播
- **错误传播**：错误沿管道逆向传播

## 并发策略

- 多入口文件并行编译
- 共享模块缓存（入度为 0 的模块并行）

## 进度跟踪

- 每个 stage 添加 `tracing::span!`
- 记录输入大小、处理时间
- 错误收集与报告

## 测试重点

- 管道顺序正确
- 背压生效
- 取消传播
- 错误处理
- 并发安全
