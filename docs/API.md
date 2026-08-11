# sasspile API 参考文档

> 本文件记录 `cargo doc` 中 `missing-docs` 警告覆盖的所有公开类型、变体和字段的语义说明。
> 源码中已有 `///` 注释的项不再重复，仅补充缺失项。

## 目录

- [Token 枚举](#token-枚举)
- [SassError 枚举](#sasserror-枚举)
- [Node 枚举](#node-枚举)
- [Value 枚举](#value-枚举)
- [BinOp / BinOpKind / UnaryOp](#binop--binopkind--unaryop)
- [Color 结构体](#color-结构体)
- [Separator 枚举](#separator-枚举)
- [Ast 结构体](#ast-结构体)
- [Env 方法](#env-方法)

---

## Token 枚举

`src/lex/token.rs` — 词法分析器产出的 token 类型。

### 关键字变体

| 变体 | 语义 |
|------|------|
| `True` | 布尔字面量 `true` |
| `False` | 布尔字面量 `false` |
| `Null` | 空值字面量 `null` |
| `And` | 逻辑与关键字 `and` |
| `Or` | 逻辑或关键字 `or` |
| `Not` | 逻辑非关键字 `not` |

### 符号变体

| 变体 | 字符 | 语义 |
|------|------|------|
| `LParen` | `(` | 左圆括号 |
| `RParen` | `)` | 右圆括号 |
| `LBrace` | `{` | 左花括号 |
| `RBrace` | `}` | 右花括号 |
| `LBracket` | `[` | 左方括号 |
| `RBracket` | `]` | 右方括号 |
| `Colon` | `:` | 冒号（声明分隔符） |
| `Semicolon` | `;` | 分号（语句终止符） |
| `Comma` | `,` | 逗号（列表/参数分隔符） |
| `Dot` | `.` | 点（小数点 / 类选择器） |
| `Plus` | `+` | 加号（加法 / 一元正号） |
| `Minus` | `-` | 减号（减法 / 一元负号） |
| `Star` | `*` | 星号（乘法 / 通配选择器） |
| `Slash` | `/` | 斜杠（除法 / 路径分隔符） |
| `Percent` | `%` | 百分号（取模 / 百分比单位） |
| `Amp` | `&` | & 符号（父选择器引用） |
| `Caret` | `^` | 脱字符（XOR / 属性选择器） |
| `Tilde` | `~` | 波浪号（同级选择器） |
| `Bang` | `!` | 感叹号（`!important` / `!default` / `!global`） |
| `Assign` | `=` | 赋值号 |
| `Eq` | `==` | 等于比较 |
| `NotEq` | `!=` | 不等于比较 |
| `Less` | `<` | 小于比较 |
| `Greater` | `>` | 大于比较 |
| `LessEq` | `<=` | 小于等于比较 |
| `GreaterEq` | `>=` | 大于等于比较 |
| `DotDotDot` | `...` | 剩余参数展开符 |
| `Pipe` | `\|` | 竖线（`@supports selector(\|...)` 语法） |

---

## SassError 枚举

`src/error.rs` — 统一错误类型。

### `Lex` 变体字段

| 字段 | 类型 | 语义 |
|------|------|------|
| `message` | `String` | 词法错误描述信息 |
| `pos` | `usize` | 错误发生的源码字节偏移位置 |

### `Parse` 变体字段

| 字段 | 类型 | 语义 |
|------|------|------|
| `expected` | `String` | 解析器期望的 token 或结构描述 |
| `found` | `String` | 实际遇到的 token 或结构描述 |

### `Type` 变体字段

| 字段 | 类型 | 语义 |
|------|------|------|
| `expected` | `String` | 期望的类型描述 |
| `actual` | `String` | 实际的类型描述 |

---

## Node 枚举

`src/parse/ast.rs` — 语法树节点。

### `Rule` 变体

| 字段 | 类型 | 语义 |
|------|------|------|
| `selector` | `String` | 选择器文本（如 `"div.main"`） |
| `body` | `Vec<Node>` | 规则体内的子节点列表 |

### `Decl` 变体

| 字段 | 类型 | 语义 |
|------|------|------|
| `property` | `String` | CSS 属性名（如 `"color"`） |
| `value` | `Value` | 属性值表达式 |
| `important` | `bool` | 是否标记 `!important` |

### `Variable` 变体

| 字段 | 类型 | 语义 |
|------|------|------|
| `name` | `String` | 变量名（不含 `$` 前缀） |
| `value` | `Value` | 变量值表达式 |
| `flags` | `VarFlags` | `!default` / `!global` 标志 |

### `If` 变体

| 字段 | 类型 | 语义 |
|------|------|------|
| `branches` | `Vec<(Value, Vec<Node>)>` | 条件分支列表，每项为 `(条件, 体)` |
| `else_body` | `Option<Vec<Node>>` | `@else` 体（无条件分支） |

### `For` 变体

| 字段 | 类型 | 语义 |
|------|------|------|
| `var` | `String` | 循环变量名 |
| `from` | `Value` | 起始值表达式 |
| `to` | `Value` | 结束值表达式 |
| `inclusive` | `bool` | `through` = true（含上界），`to` = false（不含） |
| `body` | `Vec<Node>` | 循环体节点列表 |

### `Each` 变体

| 字段 | 类型 | 语义 |
|------|------|------|
| `vars` | `Vec<String>` | 解构变量名列表 |
| `list` | `Value` | 待遍历的列表/Map 表达式 |
| `body` | `Vec<Node>` | 循环体节点列表 |

### `While` 变体

| 字段 | 类型 | 语义 |
|------|------|------|
| `cond` | `Value` | 循环条件表达式 |
| `body` | `Vec<Node>` | 循环体节点列表 |

### `MixinDef` 变体

| 字段 | 类型 | 语义 |
|------|------|------|
| `name` | `String` | mixin 名称 |
| `params` | `Vec<Param>` | 参数定义列表 |
| `body` | `Vec<Node>` | mixin 体节点列表 |

### `Include` 变体

| 字段 | 类型 | 语义 |
|------|------|------|
| `name` | `String` | 要包含的 mixin 名称 |
| `args` | `Vec<Arg>` | 调用参数列表 |
| `content` | `Option<Vec<Node>>` | `@content` 块内容 |

### `Content` 变体

无字段。表示 `@content` 占位——在 mixin 体中标记 mixin 调用者传入的内容块插入位置。

### `FunctionDef` 变体

| 字段 | 类型 | 语义 |
|------|------|------|
| `name` | `String` | 函数名称 |
| `params` | `Vec<Param>` | 参数定义列表 |
| `body` | `Vec<Node>` | 函数体节点列表 |

### `Return` 变体

| 字段 | 类型 | 语义 |
|------|------|------|
| (0) | `Value` | 返回值表达式 |

### `Use` 变体

| 字段 | 类型 | 语义 |
|------|------|------|
| `url` | `String` | 模块 URL（如 `"sass:math"` 或文件路径） |
| `namespace` | `Option<String>` | `as` 指定的命名空间 |
| `star` | `bool` | `as *` 通配导入标志 |
| `config` | `Vec<(String, Value)>` | `with ($x: val)` 配置参数列表 |

### `Forward` 变体

| 字段 | 类型 | 语义 |
|------|------|------|
| `url` | `String` | 要转发的模块 URL |
| `show` | `Vec<String>` | `show` 白名单成员 |
| `hide` | `Vec<String>` | `hide` 黑名单成员 |
| `prefix` | `Option<String>` | `as prefix-*` 前缀重映射 |

### `Import` 变体

| 字段 | 类型 | 语义 |
|------|------|------|
| `url` | `String` | 要导入的文件 URL |

### `Extend` 变体

| 字段 | 类型 | 语义 |
|------|------|------|
| `selector` | `String` | 要继承的选择器 |
| `optional` | `bool` | `!optional` 标志——不存在匹配时不报错 |

### `AtRoot` 变体

| 字段 | 类型 | 语义 |
|------|------|------|
| `query` | `Option<String>` | `@at-root` 查询条件（如 `(without: media)`） |
| `body` | `Vec<Node>` | `@at-root` 体节点列表 |

### `AtRule` 变体

| 字段 | 类型 | 语义 |
|------|------|------|
| `name` | `String` | @规则名称（如 `"media"`, `"keyframes"`） |
| `params` | `Option<String>` | @规则参数文本 |
| `body` | `Option<Vec<Node>>` | @规则体（`None` 表示无 body） |

### `Warn` / `Debug` / `Error` 变体

| 变体 | 字段 | 语义 |
|------|------|------|
| `Warn` | `Value` | `@warn` 指令的警告消息表达式 |
| `Debug` | `Value` | `@debug` 指令的调试消息表达式 |
| `Error` | `Value` | `@error` 指令的错误消息表达式 |

---

## Value 枚举

`src/parse/ast.rs` — 值表达式。

### 变体一览

| 变体 | 语义 |
|------|------|
| `Number(f64, Option<String>)` | 数值 + 可选单位（如 `16px`, `3.14`, `50%`） |
| `String(String, bool)` | 字符串内容 + 是否有引号 |
| `Color(Color)` | 颜色值 |
| `List(Vec<Value>, Separator, bool)` | 列表元素 + 分隔符 + 是否含括号 |
| `Map(Vec<(Value, Value)>)` | Map 键值对列表 |
| `Variable(String)` | 变量引用（不含 `$`） |
| `Bool(bool)` | 布尔值 |
| `Null` | null 值 |
| `Call(String, Vec<Arg>)` | 函数调用——`name(args)` |
| `Interp(String)` | 插值表达式 `#{...}` |
| `BinOp(Box<BinOp>)` | 二元运算 |
| `UnaryOp(UnaryOp, Box<Value>)` | 一元运算 |
| `Calc(String)` | `calc()` 原样保留表达式 |
| `Spread(Box<Value>)` | 剩余参数展开 `...` |

---

## BinOp / BinOpKind / UnaryOp

### `BinOp` 结构体字段

| 字段 | 类型 | 语义 |
|------|------|------|
| `op` | `BinOpKind` | 运算符类型 |
| `left` | `Value` | 左操作数 |
| `right` | `Value` | 右操作数 |

### `BinOpKind` 枚举变体

| 变体 | 运算符 | 语义 |
|------|--------|------|
| `Add` | `+` | 加法 |
| `Sub` | `-` | 减法 |
| `Mul` | `*` | 乘法 |
| `Div` | `/` | 除法 |
| `Mod` | `%` | 取模 |
| `Eq` | `==` | 等于比较 |
| `NotEq` | `!=` | 不等于比较 |
| `Lt` | `<` | 小于比较 |
| `Gt` | `>` | 大于比较 |
| `LtEq` | `<=` | 小于等于比较 |
| `GtEq` | `>=` | 大于等于比较 |
| `And` | `and` | 逻辑与（短路求值） |
| `Or` | `or` | 逻辑或（短路求值） |

### `UnaryOp` 枚举变体

| 变体 | 运算符 | 语义 |
|------|--------|------|
| `Neg` | `-` | 一元负号 |
| `Not` | `not` | 逻辑非 |

---

## Color 结构体

`src/parse/ast.rs` — RGB(A) 颜色表示。

| 字段 | 类型 | 语义 |
|------|------|------|
| `r` | `u8` | 红色通道（0-255） |
| `g` | `u8` | 绿色通道（0-255） |
| `b` | `u8` | 蓝色通道（0-255） |
| `a` | `f32` | Alpha 通道（0.0-1.0） |

---

## Separator 枚举

`src/parse/ast.rs` — 列表分隔符类型。

| 变体 | 语义 |
|------|------|
| `Comma` | 逗号分隔——`(a, b, c)` |
| `Space` | 空格分隔——`(a b c)` |
| `Slash` | 斜杠分隔——`(a / b / c)` |
| `Undecided` | 未确定——单元素或待推断 |

---

## Ast 结构体

`src/parse/ast.rs` — AST 根容器。

| 字段 | 类型 | 语义 |
|------|------|------|
| `nodes` | `Vec<Node>` | 顶层语法树节点列表 |

---

## Env 方法

`src/eval/mod.rs` — 不可变求值环境。

| 方法 | 签名 | 语义 |
|------|------|------|
| `new_env` | `() -> Self` | 创建空环境（等价于 `Default::default()`） |
| `bind` | `(name: String, value: Value) -> Self` | 不可变插入变量绑定，返回新环境 |
| `lookup` | `(name: &str) -> Option<&Value>` | 按名查找变量引用 |
| `has_var` | `(name: &str) -> bool` | 判断变量是否已定义 |

---

## 补充说明

本文件由 `RUSTDOCFLAGS="-W missing-docs" cargo doc` 驱动生成，覆盖以下源文件的缺失项：

- `src/lex/token.rs` — 34 个 Token 符号/关键字变体
- `src/error.rs` — 6 个 SassError 变体字段
- `src/parse/ast.rs` — 90+ 个 Node/Value/BinOp/Color/Separator/Ast 变体及字段
- `src/eval/mod.rs` — 4 个 Env 公开方法

共计 135 项 `missing-docs` 警告，全部在此文档中补充语义说明。
