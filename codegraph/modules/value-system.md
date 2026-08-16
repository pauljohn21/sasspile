# 值系统 ✅ 已完成

## 职责

定义 Sass 中所有值类型及其操作，是整个编译器的基础数据类型层。

## 文件结构（实际）

```
value/
├── mod.rs       # Value 枚举定义 + SharedValue 类型别名
├── number.rs    # Number 结构与 Unit
├── color.rs     # SassColor (RGBA/HSLA)
├── ops.rs       # 等值/比较运算
├── error.rs     # ValueError 类型
```

## 核心类型

**文件: `sasspile/src/value/mod.rs`**

```rust
pub struct Value {
    Number(Number),       // 数值带单位
    String(String, Quoted), // 字符串（引号状态）
    Boolean(bool),        // 布尔
    Null,                 // Sass null
    Color(SassColor),     // sRGB 颜色
    List(Vec<Value>, Separator), // 列表
    Map(Vec<(Value, Value)>),    // 键值映射
    ArgList(Vec<Value>),  // 参数列表
    Function(String),     // 函数引用
    Calculation(String),  // calc() 延迟计算
    Error(String),        // 错误哨兵
}

pub enum Quoted { Quoted, Unquoted }

pub enum Separator { Comma, Space, Slash, Undecided }

pub type SharedValue = Arc<Value>;
```

## Number 类型

**文件: `sasspile/src/value/number.rs`**

```rust
pub struct Number {
    pub value: f64,
    pub unit: Unit,
}

pub enum Unit {
    None,
    Em, Rem, Px, Pt, Pc, In, Cm, Mm,
    Deg, Rad, Grad, Turn,
    S, Ms, Hz, Khz,
    Dpi, Dpcm, Dppx,
    Percent,
    Compound(Vec<Unit>),
}
```

## SassColor 类型

**文件: `sasspile/src/value/color.rs`**

```rust
pub struct SassColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: f64,  // 0.0 - 1.0
}
```

## 关键操作

- **等值性**（`PartialEq`）：包含单位、分隔符比较
- **比较**：数值直接比较
- **类型转换**：`coerce.rs` 处理数值→布尔等
- **线程安全**：`Value: Clone + Send + Sync`，跨 Task 共享 `Arc<Value>`

## 使用模式

```rust
use sasspile::value::{Value, Number, Unit, SassColor};

let num = Value::Number(Number { value: 42.0, unit: Unit::Px });
let color = Value::Color(SassColor::rgba(255, 0, 0, 1.0));
let shared: SharedValue = Arc::new(num);
```

## 测试

- `tests/lexer_spec.rs`（数值 token）
- 将来：`tests/value_spec.rs`
