# sasspile

[![Crates.io](https://img.shields.io/crates/v/sasspile)](https://crates.io/crates/sasspile)
[![Docs.rs](https://docs.rs/sasspile/badge.svg)](https://docs.rs/sasspile)
[![License](https://img.shields.io/crates/l/sasspile)](LICENSE)
[![CI](https://github.com/pauljohn21/sasspile/workflows/CI/badge.svg)](https://github.com/pauljohn21/sasspile/actions)

纯 Rust 函数式 SCSS 编译器，使用 Rust 1.97 + Edition 2024 构建。

> **v0.4.0** — sass-spec 通过率 53%（2566/4848），Bootstrap 5 和 Element Plus 全量通过。

sasspile 是一个从零实现的 SCSS 编译器，采用纯函数式风格。通过类型状态机（Type-State Pattern）确保编译阶段类型安全，使用 Iterator + fold + 不可变数据结构实现零副作用的编译流程。

## 特性

- **类型状态机管线**: `Source → Lexed → Parsed → Evaluated → Serialized`
- **纯函数式风格**: Iterator + fold + 不可变数据
- **零依赖**: 纯 Rust 实现，无外部 C 库
- **sass-spec 兼容**: 2566/4848 (53%) 通过
- **Bootstrap 5.3.8**: 全量编译通过 ✅
- **Element Plus**: 121/121 (100%) 全量通过 ✅
- **tracing 调试**: 内建 span + event 追踪链路
- **AI 开发技能**: 内置 `skill.md` 综合开发指南

## 快速开始

添加依赖：

```toml
[dependencies]
sasspile = "0.4"
```

最小示例：

```rust
use sasspile::{compile_expanded, compile_compressed};

fn main() -> Result<(), sasspile::SassError> {
    // 展开式输出
    let css = compile_expanded("a { color: red; }")?;
    println!("{}", css);
    // => "a {\n  color: red;\n}\n"

    // 压缩式输出
    let css = compile_compressed("a { color: red; }")?;
    println!("{}", css);
    // => "a{color:red;}"

    Ok(())
}
```

## 使用示例

```rust
use sasspile::compile_expanded;

let scss = r#"
$primary: #3498db;

.btn {
    background: $primary;
    color: white;

    &:hover {
        background: darken($primary, 10%);
    }

    @media (max-width: 768px) {
        width: 100%;
    }
}
"#;

let css = compile_expanded(scss)?;
println!("{}", css);
```

## 支持的功能

### 变量与作用域

- 全局变量与局部变量
- 变量插值 `#{$var}`

### 嵌套规则

- 选择器嵌套
- 属性嵌套（`font: { size: 14px; weight: bold; }`）
- 父选择器引用 `&`

### 运算

- 数学运算（`+`, `-`, `*`, `/`, `%`）
- 颜色运算
- 字符串拼接

### 颜色函数（完整）

构造：`rgb`/`rgba`/`hsl`/`hsla`/`hwb`

操作：`adjust-color`/`change-color`/`scale-color`/`mix`/`darken`/`lighten`/`adjust-hue`/`saturate`/`desaturate`/`grayscale`/`complement`/`invert`/`opacify`/`fade-in`/`transparentize`/`fade-out`

通道：`red`/`green`/`blue`/`alpha`/`opacity`/`hue`/`saturation`/`lightness`/`whiteness`/`blackness`/`color-channel`

Level 4：`is-powerless`/`is-in-gamut`/`is-legacy`

### 字符串函数

`str-length`/`str-index`/`str-slice`/`str-insert`/`str-split`/`to-upper-case`/`to-lower-case`/`unquote`/`quote`/`unique-id`

### 列表函数

`length`/`nth`/`append`/`join`/`index`/`separator`/`set-nth`/`is-bracketed`/`list-slash`/`zip`

### Map 函数

`map-get`/`map-keys`/`map-values`/`map-has-key`/`map-merge`/`map-remove`/`map-set`/`map-deep-merge`/`map-deep-remove`

### 数学函数

`abs`/`ceil`/`floor`/`round`/`min`/`max`/`percentage`/`div`/`pow`/`sqrt`/`sin`/`cos`/`tan`/`asin`/`acos`/`atan`/`atan2`/`hypot`/`log`/`random`/`clamp`/`unit`/`compatible`

### 选择器函数

`selector-append`/`selector-nest`/`selector-parse`/`selector-simple-selectors`/`selector-unify`/`selector-extend`/`selector-replace`/`selector-is-super`

### Mixin 与函数

- `@mixin` / `@include` / `@content`
- `@function` / `@return`
- `@if` / `@else` / `@for` / `@each` / `@while`
- `@import` / `@use` / `@forward`
- `@extend` / `@at-root` / `@warn` / `@debug` / `@error`

### 指令

- `@media` 媒体查询
- `//` 单行注释
- `/* */` 多行注释
- `!default` / `!important` 标记

### 选择器

- 通用选择器 `*`
- 伪类 `:hover`, `:focus`, `:nth-child(n)`
- 伪元素 `::before`, `::after`

## 测试

```bash
# 核心测试
cargo test --test compile_test      # 41 个
cargo test --test stage_test        # 10 个
cargo test --test ast_test          # 8 个
cargo test --test common_test       # 5 个

# 兼容性测试
cargo test --test bs_spec           # 15 Bootstrap 测试
cargo test --test ep_full           # 121 Element Plus 测试（约 28 秒）
```

全部通过：**compile 41/41 + stage 10/10 + ast 8/8 + common 5/5 + BS 15/15 + EP 121/121**

> 详见根目录 `skill.md` 获取完整开发指南。

## 架构设计

sasspile 使用类型状态机模式构建编译管线：

```
Source { content }  ──lex()──►  Lexed { tokens }
                                  │
                                  ▼
                             Parsed { ast }
                                  │
                                  ▼
                            Evaluated { css_tree }
                                  │
                                  ▼
                           Serialized { css_string }
```

每个阶段通过类型转换确保：
- 必须先解析后求值
- 必须先求值后序列化
- 编译错误在类型层面被阻止

## 许可证

MIT
