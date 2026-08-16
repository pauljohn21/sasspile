# 求值器 ✅ 已完成

## 职责

对 AST/表达式进行求值，处理变量查找、运算、函数调用、控制流。

## 文件结构（实际）

```
eval/
├── mod.rs             # 求值入口 + re-exports
├── evaluator.rs       # EvalContext (核心求值环境)
├── ops.rs             # 二元/一元运算符 (binary, unary)
├── functions.rs       # 用户函数调用分发
├── collections.rs     # 列表/Map 访问
└── error.rs           # EvalError
```

## 求值上下文

**文件: `sasspile/src/eval/evaluator.rs`**

```rust
pub struct EvalContext<'a> {
    env: &'a mut Env,          // 变量/函数环境
    symbol_table: &'a SymbolTable,
    // ...
}

pub struct Env {
    vars: Map<String, Value>,
    functions: Map<String, FunctionDef>,
    mixins: Map<String, MixinDef>,
}
```

## 运算符

**文件: `sasspile/src/eval/ops.rs`**

```rust
pub fn binary(op: BinOp, lhs: &Value, rhs: &Value) -> Result<Value, EvalError>;
pub fn unary(op: Unop, operand: &Value) -> Result<Value, EvalError>;
```

| 运算符 | 行为 |
|--------|------|
| `+` | 数值加法 / 字符串连接 |
| `-` | 数值减法 |
| `*` | 数值乘法 / 颜色乘法 |
| `/` | 除法（Sass 特殊处理） |
| `%` | 取模 |
| `==` / `!=` | 等值性比较 |
| `<` / `>` / `<=` / `>=` | 数值比较 |
| `and` / `or` / `not` | 布尔运算 |
| `+` (一元) | 正号 |
| `-` ( unary) | 负号 |

## 函数调用

**文件: `sasspile/src/eval/functions.rs`**

- 用户函数：查找符号表，创建新作用域，绑定参数
- 内置函数：通过 `builtin::dispatch` 分发
- 参数传递：按位置 + 关键字

## 控制流

- `@if` / `@else if` / `@else`
- `@for $i from X to Y` / `@for $i from X through Y`
- `@each $item in $list` / `@each $k, $v in $map`
- `@while $condition`

## 列表/Map 访问

**文件: `sasspile/src/eval/collections.rs`**

- `nth($list, $n)` — 索引从 1 开始
- `$map[$key]` — 键查找
- 边界越界错误

## 使用方式

```rust
use sasspile::eval::EvalContext;

let mut env = Env::new();
env.vars.insert("$color".into(), Value::Color(...));

let mut ctx = EvalContext::new(&mut env, &symbol_table);
let result = ctx.evaluate_expr(&expr)?;
```

## 测试

- `tests/eval_spec.rs`
