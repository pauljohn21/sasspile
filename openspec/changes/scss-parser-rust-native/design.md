## Context

sasspile 整体 RFP 架构要求每个编译阶段消费 self 返回新状态。parse 模块遵循相同模式，基于 `tokio-stream` crate 的 `Stream` trait。

### 设计：tokio-stream

**核心**: 使用 `tokio-stream::Stream` trait。不需要自研 Iterator/Type-State/NodeStream 等新抽象。

```rust
tokio_stream::Stream<Item = Result<Node>>
    │
    .try_collect::<Vec<_>>()?  →  Vec<Node>
    → Ast { nodes }
```

### 架构

```
Lexer → Vec<Token>
           │
      ParseStream::new(&[Token])   // 构建 Stream（消费 token 引用）
           │
      Stream::poll_next(Pin<&mut Self>, cx)
           │
           ├── tokens 耗尽 ──→ Poll::Ready(None)
           └── 否则 ──→ Poll::Ready(Some(Result<Node>))
                          │
                       内部 parse 方法消费 token，pos 自然推进
```

```rust
// 入口：parse tokens → AST
pub fn parse(tokens: &[Token]) -> Result<Ast> {
    use tokio_stream::StreamExt;
    let stream = ParseStream::new(tokens);
    // block_on 同步驱动 Stream 完成
    let nodes = futures::executor::block_on_stream(stream).try_collect::<Vec<_>>()?;
    Ok(Ast { nodes })
}
```

### 关键

**K1: ParseStream 包装 token 切片 + 位置**
```rust
struct ParseStream<'tok> {
    tokens: &'tok [Token],
    pos: usize,
}
```

**K2: 解析方法作为 `&mut self` 方法**
```rust
impl ParseStream {
    fn parse_node(&mut self) -> Result<Node>    // 消费 → 产出
    fn parse_rule(&mut self) -> Result<Node>
    fn parse_decl(&mut self) -> Result<Node>
    fn parse_expr(&mut self) -> Result<Value>
}
```

与 `tokio_stream::Stream::poll_next(Pin<&mut Self>, _)` 签名同态——`&mut self` 推进内部 pos，无需返回新切片。

**K3: 顶层 parse() 通过 block_on 驱动 Stream**
不需要手写循环。tokio-stream 提供 `StreamExt::try_collect`，收集所有 `Result<Item>` 到 `Result<Vec<Item>>`。

## 迁移

1. 添加 `tokio-stream` 依赖 ✅
2. ParseStream 实现 `Stream<Item = Result<Node>>`
3. sub-parser 全部作为 `&mut self` 方法
4. 顶层 `parse()` 用 `futures::executor::block_on_stream` 驱动
5. 验证 sass-spec

## 风险

- `futures::executor::block_on_stream` 可能引入 `futures` 依赖（tokio-stream 自带 re-export）
- sass-spec 兼容性：需 SPEC_STORE_CMD=run 验证

## 不变式

- `parse()` 公开 API 签名不变
- AST 类型（`Ast` / `Node` / `Value`）不变
- Reactor 的 `parse()` 签名不变
- 公开 `compile` / `compile_file` 不变
