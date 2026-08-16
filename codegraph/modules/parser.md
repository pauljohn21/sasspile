# 解析器 ✅ 已完成

## 职责

将 Token 序列转换为类型安全的 AST，处理 Sass 的嵌套规则和 @-rules。

## 文件结构（实际）

```
parser/
├── mod.rs             # Parser 入口
├── ast.rs             # AST 节点定义 (Stylesheet/Node/Rule/Declaration/AtRule)
├── core.rs            # 递归下降解析器主体
├── expr.rs            # 表达式解析
├── interpolation.rs   # #{} 插值解析
├── at_rules.rs        # @规则 (@use/@if/@mixin/@for/@each/@while/@content 等)
├── selector.rs        # 选择器解析
└── recovery.rs        # 错误恢复
```

## AST 层级

```rust
Stylesheet
└── Vec<Node>
    ├── Rule { selector, nodes[] }
    ├── Declaration { name, value: Expr, important }
    ├── AtRule(Use|Import|Forward|Mixin|Include|Function|Return|If|Else|For|Each|While|Extend|AtRoot|Media|Supports|Content|Debug|Warn|Error)
    └── Comment(String)
```

## 使用方式

```rust
use sasspile::lexer::tokenize;
use sasspile::parser::Parser;

let (tokens, _) = tokenize(source);
let mut parser = Parser::new(tokens);
let stylesheet = parser.parse_stylesheet()?;
```

## 控制流支持

- `@if expr { ... }`
- `@else if expr { ... }`
- `@else { ... }`
- `@for $i from X [to|through] Y`
- `@each $item in $list` / `@each $k, $v in $map`
- `@while $condition`

## @规则支持

- `@use "module" [as namespace] [with (key: value)]`
- `@forward "module" [as prefix-*] [hide/show list]`
- `@import "url"`
- `@mixin name($params) { ... }`
- `@include name($args) [using ($block)]`
- `@function name($params) { ... @return expr }`
- `@extend selector [!optional]`
- `@media (...)`, `@supports (...)`
- `@at-root { ... }`
- `@debug expr`, `@warn expr`, `@error expr`
- `@content [using ($args)]`

## 已知失败模式

1. **`and`/`or` 逻辑运算符缺失** — Token 存在但解析失败
2. `@else if` 链式解析边界
3. `@if` 条件含 `and` 组合时短路
4. 大括号深层追踪
5. `@extend` 多行选择器

## 测试

- `tests/parser_spec.rs`
- `tests/sass_spec_parse.rs`（集成：475/1306 通过）
