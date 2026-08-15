# 解析器（待开发）

## 职责

将 Token 序列转换为类型安全的 AST，处理 Sass 的嵌套规则和 @-rules。

## 计划文件结构

```
parser/
├── mod.rs             # Parser 入口
├── ast.rs             # AST 节点定义
├── parser.rs          # 递归下降解析
├── interpolation.rs   # #{} 插值解析
└── recovery.rs        # 错误恢复
```

## AST 节点

```rust
pub struct Stylesheet {
    pub nodes: Vec<Node>,
}

pub enum Node {
    Rule(Rule),
    Declaration(Declaration),
    AtRule(AtRule),
    Comment(Comment),
}

pub struct Rule {
    pub selector: Selector,
    pub nodes: Vec<Node>,
}

pub struct Declaration {
    pub name: String,
    pub value: Expr,
    pub important: bool,
}

pub enum AtRule {
    Use(UseRule),
    Import(ImportRule),
    Forward(ForwardRule),
    Mixin(MixinDef),
    Include(Include),
    Function(FunctionDef),
    Return(Expr),
    If(IfStmt),
    Else(Vec<Node>),
    For(ForStmt),
    Each(EachStmt),
    While(WhileStmt),
    Extend(Selector),
    AtRoot(Vec<Node>),
    Media(MediaRule),
    Supports(SupportsRule),
    Content,
    Debug(Expr),
    Warn(Expr),
    Error(Expr),
    // ... 其他
}
```

## 解析策略

- **递归下降**：每种节点对应一个 parse 方法
- **嵌套支持**：Rule 内部可以包含规则和声明
- **@规则专用**：`@use`, `@mixin`, `@if` 等独立解析路径
- **错误恢复**：遇到非致命错误时同步到下一个完整语句

## 测试重点

- 基本选择器解析
- 属性/值解析
- 嵌套规则
- @规则变体
- 插值识别
- 错误恢复
