## Why

sasspile 整体是 **RFP（Reactive Functional Pipeline）** 架构：

```rust
Reactor::new(source)     // StateRaw
    .lex()?              // → StateLexed
    .parse()?            // → StateParsed
    .evaluate()?         // → StateEvaluated
    .serialize(style)    // → StateSerialized
    .finish()?           // → String
```

每步：消费 `self`，返回 `Reactor<NextState>`。类型状态机保证顺序。

**问题**：parse 模块是 Dart 翻译风格 —— `Parser { tokens, pos }` + `advance()` + `skip_ws()` + `while !at_end()` 循环。这是 **命令式 cursor** 模式，与外部 RFP 链风格完全冲突：

| 外部（Reactor） | 内部（当前 Parser） |
|-----------------|---------------------|
| `self.method()?` 消费并推进 | `self.advance()` + `self.pos += 1` |
| 类型状态编译期保证 | `pos` 手动维护，越界即 panic |
| 每个阶段独立、可组合 | 所有状态通过 `&mut self` 纠缠 |
| 链式调用 | while 循环 + 条件 break |

**结果**：调用者看到的是优雅的链式 APIs，但底层实现是指针推进 + 手动状态维护。认知失调。

**设计约束**：不变 AST 类型（`ast::Ast` / `ast::Node` / `ast::Value`）。

## What Changes

### 目标：parse 内部也统一为"消费 self 返回 self"的 Builder 链

```
Parser::new(tokens)      // 构建初始状态
    .node()              // 解析一个节点，返回 (Node, Parser)'
    .parse_all()         // 解析全部，返回 Parser<Parsed>
    .finish()            // 消费 Parser<Parsed>，返回 Ast
```

```rust
// 入口（与 Reactor 链式调用同构）
pub fn parse(tokens: &[Token]) -> Ast {
    Parser::new(tokens)
        .parse_all()
        .finish()
}
```

### 设计原则

1. **Parser 是 Builder，不是 Cursor**：方法消费 self、返回 `(Output, Parser)` 或新 Parser
2. **显式状态 token，无 `pos` 索引**：Parser 持有 `tokens: &[Token]`，子解析器返回剩余 tokens
3. **解析函数分层**：`parse_node(tokens) -> (Node, tokens)` 在 Parser 上被调用，但逻辑是纯函数
4. **类型状态标记**（可选）：用泛型 `Parser<S>` 区分 Parsing/Parsed 状态（类似 Reactor 的 `StateRaw/StateLexed/...`）

### 简化示例

```rust
struct Parser<'t> {
    tokens: &'t [Token],
}

impl<'t> Parser<'t> {
    pub fn new(tokens: &'t [Token]) -> Self {
        Self { tokens }
    }

    /// 解析全部节点。
    pub fn parse_all(mut self) -> (Vec<Node>, &'t [Token]) {
        let mut nodes = Vec::new();
        let mut toks = skip_ws(self.tokens);
        while let Some(tok) = first_non_ws(toks) {
            let (node, rest) = parse_node(toks);
            nodes.push(node);
            toks = skip_ws(rest);
        }
        (nodes, toks)
    }
}

pub fn parse(tokens: &[Token]) -> Ast {
    let parser = Parser::new(tokens);
    let (nodes, _) = parser.parse_all();
    Ast { nodes }
}
```

每个子解析（`parse_rule`, `parse_decl`, `parse_expr`）同样采用 `(Node, &[Token])` 返回模式，内部无 pos 回滚。

### 删除冗余

- `src/parse/css_ast.rs` （已删除）
- `src/parse/css_parser.rs` （已删除）
- `src/eval/css_evaluator.rs` （已删除）
- `src/parse/scss_ast.rs` （type alias，已删除）
- `CompileMode` 枚举（不再需要分派）
- `Parsed::Css` 变体（不再需要）

### 关键文件

- `src/parse/mod.rs` — `parse()`/Parser Builder
- `src/parse/nodes.rs` — statement 解析
- `src/parse/at_rules*.rs` — @规则
- `src/parse/expr.rs` — Pratt 表达式
- `src/parse/params.rs` — 参数解析

## Capabilities

### New Capabilities

- **风格统一**：parse 内部也是"消费 → 返回 → 链式"，与 Reactor 一致
- **编译期保证**：不使用 `pos += 1` 手动推进，消除越界 panic
- **可测试性**：每个 parse 函数纯 `(Input, State) → (Output, State)`，可独立断言

### Non-Goals

- 不改变 AST 类型结构
- 不引入 nom / combine 等外部解析框架
- 不影响 Evaluator / Serializer
- 不使 sass-spec 测试退化
