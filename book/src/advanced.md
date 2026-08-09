# 进阶用法

## 媒体查询

sasspile 支持嵌套媒体查询，让你的响应式代码更清晰。

```scss
.responsive {
    width: 100%;
    font-size: 16px;

    @media (max-width: 576px) {
        width: 100%;
        font-size: 14px;
    }

    @media (min-width: 577px) and (max-width: 992px) {
        width: 80%;
    }

    @media (min-width: 993px) {
        width: 60%;
        margin: 0 auto;
    }
}
```

## 注释

### 多行注释

```scss
/* 这是一个多行注释
   会在生成的 CSS 中保留 */
.box {
    color: red;
}
```

### 单行注释

```scss
// 这是一个单行注释
// 不会出现在生成的 CSS 中
.box {
    color: red;
}
```

## 数学运算

sasspile 支持在 CSS 值中进行数学运算。

```scss
.container {
    // 加减乘除
    width: 100% - 20px;
    padding: 10px + 5px;
    font-size: 16px / 2;

    // 取模
    margin: 25px % 10px; // 5px

    // 复杂表达式
    width: (100% - 20px) / 2;
}
```

### 单位运算

```scss
.box {
    // 相同单位：运算结果保留单位
    width: 10px + 5px; // 15px

    // 不同单位：运算结果保留左边单位
    width: 10px + 5;   // 15px
}
```

## 通用选择器

sasspile 支持通用选择器 `*`：

```scss
* {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
}
```

## 伪类和伪元素

支持标准的 CSS 伪类和伪元素：

```scss
a {
    color: blue;

    &:hover {
        color: darkblue;
    }

    &:active {
        color: red;
    }

    &:focus {
        outline: 2px solid blue;
    }

    &::before {
        content: "→";
    }
}
```

## 错误处理

```rust
use sasspile::{compile_expanded, SassError};

fn compile_scss(scss: &str) -> Result<String, String> {
    match compile_expanded(scss) {
        Ok(css) => Ok(css),
        Err(SassError::UndefinedVariable(name)) => {
            Err(format!("变量 '{}' 未定义", name))
        }
        Err(SassError::LexError { message, position }) => {
            Err(format!("词法错误 (位置 {}): {}", position, message))
        }
        Err(SassError::ParseError { expected, found }) => {
            Err(format!("语法错误: 期望 '{}', 实际 '{}'", expected, found))
        }
        Err(e) => Err(format!("编译失败: {}", e)),
    }
}
```

## 自定义工具函数

虽然 sasspile 目前不支持自定义函数，但你可以通过 Rust 扩展功能：

```rust
use sasspile::compile_expanded;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let scss = std::fs::read_to_string("styles/main.scss")?;
    let css = compile_expanded(&scss)?;

    std::fs::write("dist/main.css", css)?;
    Ok(())
}
```

## 与构建系统集成

### 使用 build.rs

```rust
// build.rs
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=styles/main.scss");

    let scss = fs::read_to_string("styles/main.scss").unwrap();
    let css = sasspile::compile_expanded(&scss).unwrap();

    let out_dir = Path::new(env!("OUT_DIR"));
    fs::write(out_dir.join("main.css"), css).unwrap();
}
```

### 使用 npm scripts

```json
{
  "scripts": {
    "build:css": "cargo run --example build-css"
  }
}
```

## 性能优化

1. **使用压缩输出**：生产环境使用 `compile_compressed`

```rust
let css = compile_compressed(scss)?;
```

2. **分块编译**：大型项目可以分模块编译

```rust
let base_css = compile_expanded(read_file("base.scss"))?;
let module_css = compile_expanded(read_file("module.scss"))?;
```

## 最佳实践

1. **变量命名**：使用清晰的变量名

```scss
// 好
$primary-color: #3498db;

// 差
$pc: #3498db;
```

2. **组织结构**：按功能分组

```scss
// ===== 变量 =====
$primary: #3498db;

// ===== 基础样式 =====
* {
    box-sizing: border-box;
}

// ===== 组件 =====
.button {
    // ...
}
```

3. **使用注释**：说明复杂逻辑

```scss
/* 卡片组件
 * - 支持多种尺寸
 * - 响应式布局
 */
.card {
    // ...
}
```

## 限制与已知问题

当前版本（0.2）的限制：

1. 不支持 `@mixin` 和 `@include`
2. 不支持 `@extend`
3. 不支持 `@for`, `@each`, `@while` 循环
4. 不支持 `@if`, `@else` 条件语句

这些功能计划在未来版本中实现。