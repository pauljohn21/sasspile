# 值系统（待开发）

## 职责

定义 Sass 中所有值类型及其操作，是整个编译器的基础数据类型层。

## 计划文件结构

```
value_system/
├── mod.rs       # Value 枚举定义
├── number.rs    # 数值与单位
├── color.rs     # sRGB 颜色
├── ops.rs       # 算术/比较/运算（≤80行）
├── coerce.rs    # 类型转换（≤60行）
└── ser.rs       # CSS 序列化
```

## 类型定义

```rust
pub enum Value {
    Number { value: f64, unit: Unit },
    String(String),
    Boolean(bool),
    Null,
    Color(Color),
    List(Vec<Value>, Separator),
    Map(Vec<(Value, Value)>),
    ArgList(Vec<Value>),
    Function(name: String),
    Calculation(String),
    Error(String),
}

pub enum Separator {
    Comma,
    Space,
    Slash,   // 列表 slash 分隔
    Undecided,
}

pub enum Unit {
    None,
    Em, Rem, Px, Pt, Pc, In, Cm, Mm,
    Deg, Rad, Grad, Turn,
    S, Ms,
    Hz, Khz,
    Dpi, Dpcm, Dppx,
    Percent,
    // 复合单位
    Compound(Vec<Unit>),
}
```

## 关键操作

- **等值性**：`==` / `!=`（Sass 语义）
- **比较**：`<` / `>` / `<=` / `>=`（仅 Number / 同单位）
- **算术**：`+` / `-` / `*` / `%`
- **除法**：`/` 的特殊处理（Sass 除法）
- **字符串连接**：`+`
- **布尔运算**：`and` / `or` / `not`
- **类型转换**：`coerce.rs`

## 测试重点

- 等值性（包含 unit 比较）
- CSS 序列化输出格式
- 单位复合运算
- slash 分隔列表识别
