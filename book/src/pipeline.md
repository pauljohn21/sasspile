# 编译管线

sasspile 的编译管线分为五个阶段，每个阶段通过类型状态机模式确保正确转换。

## 管线流程图

```
Source { content: String }
    |
    | .lex()
    v
Lexed { tokens: Vec<Token> }
    |
    | .parse()
    v
Parsed { ast: Ast }
    |
    | .evaluate()
    v
Evaluated { nodes: Vec<CssNode> }
    |
    | .serialize(style)
    v
Serialized { css: String }
```

## 阶段详解

### 1. Source（源码）

编译管线的起点，封装待编译的 SCSS 源码文本。

```rust
let source = Source::new("a { color: red; }".to_string());
```

### 2. Lexed（词法分析）

使用手写扫描器将源码转换为 Token 序列。

- 支持 Unicode 字符（包括中文）
- 惰性求值（Iterator 实现）
- O(n) 时间复杂度

```rust
let lexed = source.lex()?;
```

### 3. Parsed（语法分析）

使用递归下降解析器将 Token 序列转换为抽象语法树（AST）。

- 纯函数式风格
- Result 组合子模式
- 支持嵌套规则、变量、@规则

```rust
let parsed = lexed.parse()?;
```

### 4. Evaluated（求值）

求值器遍历 AST，展开变量、计算表达式、调用内建函数。

- 使用 `try_fold` 替代可变状态
- 不可变环境（`im::HashMap`）
- 支持作用域链

```rust
let evaluated = parsed.evaluate()?;
```

### 5. Serialized（序列化）

将求值后的 CSS 节点树序列化为最终 CSS 字符串。

- 展平嵌套规则
- 支持展开式和压缩式输出
- 格式化输出

```rust
let serialized = evaluated.serialize(OutputStyle::Expanded);
```

## 完整示例

```rust
use sasspile::stage::source::Source;
use sasspile::OutputStyle;

let serialized = Source::new("a { color: red; }".to_string())
    .lex()?
    .parse()?
    .evaluate()?
    .serialize(OutputStyle::Expanded);

println!("{}", serialized.css);
```

## 类型状态机优势

1. **编译期检查**：无法跳过阶段（如直接从 Source 到 Evaluated）
2. **清晰的数据流**：每个阶段的输入输出都是明确的
3. **易于测试**：可以单独测试每个阶段
4. **错误隔离**：错误发生在哪个阶段一目了然