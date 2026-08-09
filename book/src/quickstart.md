# 快速开始

## 安装

将 sasspile 添加到你的 `Cargo.toml`：

```toml
[dependencies]
sasspile = "0.2"
```

## 基础使用

### 编译 SCSS 为 CSS

```rust
use sasspile::{compile_expanded, compile_compressed};

// 展开式输出
let css = compile_expanded("a { color: red; }")?;
println!("{}", css);
// 输出:
// a {
//   color: red;
// }

// 压缩式输出
let css = compile_compressed("a { color: red; }")?;
println!("{}", css);
// 输出: a{color:red;}
```

### 使用变量

```rust
use sasspile::compile_expanded;

let scss = r#"
$primary: #3498db;

.button {
    background: $primary;
    color: white;
}
"#;

let css = compile_expanded(scss)?;
println!("{}", css);
```

## 完整示例

```rust
use sasspile::compile_expanded;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let scss = r#"
$primary: #3498db;
$spacing: 16px;

.card {
    padding: $spacing;
    background: $primary;
    color: white;

    &:hover {
        background: darken($primary, 10%);
    }
}
"#;

    let css = compile_expanded(scss)?;
    println!("{}", css);
    Ok(())
}
```

## CLI 使用

sasspile 提供了命令行工具，可以从标准输入读取 SCSS：

```bash
echo "a { color: red; }" | cargo run
echo "a { color: red; }" | cargo run -- --compressed
```

## 输出风格

sasspile 支持两种输出风格：

| 风格 | 函数 | 特点 |
|------|------|------|
| 展开式 | `compile_expanded` | 带缩进和换行，便于阅读 |
| 压缩式 | `compile_compressed` | 无空白，最小化输出 |

## 错误处理

```rust
use sasspile::{compile_expanded, SassError};

match compile_expanded("a { color: $undefined; }") {
    Ok(css) => println!("{}", css),
    Err(SassError::UndefinedVariable(name)) => {
        eprintln!("错误: 变量 '{}' 未定义", name);
    }
    Err(e) => eprintln!("编译失败: {}", e),
}
```