# 求值器（待开发）

## 职责

对 AST 进行求值，处理表达式、函数调用、控制流。

## 计划文件结构

```
eval/
├── mod.rs             # 求值器入口
├── evaluator.rs       # 求值上下文
├── ops.rs             # 运算符实现
├── functions.rs       # 函数调用
└── collections.rs     # 列表/Map 访问
```

## 求值上下文

```rust
pub struct EvalContext {
    env: Arc<Env>,
    symbol_table: Arc<SemanticInfo>,
}

pub struct Env {
    vars: Map<String, Value>,
    functions: Map<String, FunctionDef>,
    mixins: Map<String, MixinDef>,
}
```

## 运算符实现

| 运算符 | 行为 |
|--------|------|
| `+` | 数值加法 / 字符串连接 |
| `-` | 数值减法 |
| `*` | 数值乘法 / 颜色乘法 |
| `/` | 除法（Sass 特殊处理）|
| `%` | 取模 |
| `==` / `!=` | 等值性比较 |
| `<` / `>` / `<=` / `>=` | 数值比较 |
| `and` / `or` / `not` | 布尔运算 |
| `+` (一元) | 正号 |
| `-` (一元) | 负号 |

## 函数调用

- 用户函数：查找符号表，创建新作用域，绑定参数
- 内置函数：通过 registry 分发
- 参数传递：按位置 + 关键字

## 控制流

- `@if` / `@else if` / `@else`
- `@for $i from X to Y` / `@for $i from X through Y`
- `@each $item in $list` / `@each $k, $v in $map`
- `@while $condition`

## 列表/Map 访问

- `nth($list, $n)`
- `$map[$key]`
- 索引越界错误

## 测试重点

- 算术运算（含单位转换）
- 字符串操作
- 列表/Map 访问
- 内置函数分发
- 用户函数调用
- 控制流求值
