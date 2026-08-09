# sasspile

纯 Rust 函数式 SCSS 编译器。

## 特性

- **类型状态机管线**: `Source → Lexed → Parsed → Evaluated → Serialized`
- **纯函数式风格**: Iterator + fold + 不可变数据
- **零依赖**: 纯 Rust 实现，无外部 C 库
- **sass-spec 兼容**: 通过官方测试套件验证
- **Bootstrap 5.3.8 验证**: 核心功能测试通过

## 使用

```rust
use sasspile::{compile_expanded, compile_compressed};

// 展开式输出
let css = compile_expanded("a { color: red; }")?;
// => "a {\n  color: red;\n}\n"

// 压缩式输出
let css = compile_compressed("a { color: red; }")?;
// => "a{color:red;}"
```

## 支持的功能

- 变量 (`$var`)
- 嵌套规则 (`a { b { ... } }`)
- 数学运算 (`+`, `-`, `*`, `/`, `%`)
- 颜色函数 (`rgba`, `darken`, `lighten`, `mix`, `invert`, `grayscale`)
- 字符串函数 (`str-length`, `str-index`, `str-slice`, `to-upper-case`, `to-lower-case`)
- 列表函数 (`list-length`, `nth`, `append`, `join`)
- Map 函数 (`map-get`, `map-keys`, `map-values`, `map-merge`)
- 数学函数 (`abs`, `ceil`, `floor`, `round`, `min`, `max`, `percentage`, `sqrt`, `sin`, `cos`, `tan`, `pow`)
- 媒体查询 (`@media`)
- 注释 (`//` 和 `/* */`)
- 通用选择器 (`*`)
- 伪类/伪元素 (`:hover`, `::before`)

## 测试

```bash
cargo test
```

110+ 测试全部通过，包括：
- 75 单元测试
- 21 sass-spec 合规测试
- 13 Bootstrap 5.3.8 验证测试

## 许可证

MIT
