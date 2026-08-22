## Context

sasspile 的内建函数分派当前分散在三个位置，每处都手工维护了一份几乎相同的函数名列表：

1. `module_dispatch.rs` — `module_builtin_name()`：130 条 match arm，将 `"math.abs"` 映射到 `"abs"`
2. `builtin.rs` — `call_builtin()`：130 条 match arm，将 `"abs"` 路由到 `math::call()` / `color::call()` 等子模块
3. `builtin.rs` — `is_known_builtin()`：130 条 match arm，检查函数名是否为已知内建函数

添加一个新内建函数需要同步修改三处。漏改任何一处会导致：模块限定调用失败、全局调用无法分派、或 CSS 透传行为错误。

## Goals / Non-Goals

**Goals:**
- 将三处重复的函数名注册合并为单一数据源
- 用 syn 3.0 + darling 0.24 派生宏实现，获得 IDE 补全、符号跳转、精确错误定位等长期维护收益
- 宏展开后生成与当前完全等价的 `match` 表达式，保持编译器跳表优化
- 所有现有测试零回归

**Non-Goals:**
- 不改变任何运行时行为或公共 API
- 不宏化 `call_builtin` 中的 meta 内联函数（type-of/inspect/if/keywords/calc-args/calc-name 等）
- 不宏化 CSS 原生函数透传逻辑（calc/env/var、is_css_function）
- 不改变 `call_module_function` 的调用流程

## Decisions

### 决策 1：Workspace 结构

sasspile 从单 crate 变为 workspace 双 crate：

```
sasspile/
├── Cargo.toml              ← [workspace] + 主 crate 配置
├── sasspile-macros/         ← proc-macro crate
│   ├── Cargo.toml           ← proc-macro = true
│   └── src/
│       └── lib.rs          ← #[proc_macro_derive(BuiltinRegistry)]
└── src/                     ← 主 crate（不变）
    └── eval/
        ├── module_dispatch.rs  ← 结构体定义 + #[derive]
        └── builtin.rs          ← 引用宏生成的函数
```

根 `Cargo.toml`：
```toml
[workspace]
members = [".", "sasspile-macros"]

[dependencies]
sasspile-macros = { path = "sasspile-macros" }
```

`sasspile-macros/Cargo.toml`：
```toml
[lib]
proc-macro = true

[dependencies]
syn = { version = "3", features = ["full"] }
darling = "0.24"
quote = "1"
proc-macro2 = "1"
```

### 决策 2：结构体 DSL 设计

每个内建模块用一个结构体声明，字段名 = 全局函数名，`#[aliases]` 属性声明模块限定名和别名：

```rust
use sasspile_macros::BuiltinRegistry;

#[derive(BuiltinRegistry)]
#[builtin(module = "math", dispatch = "math")]
struct MathBuiltins {
    abs: (),
    div: (),
    ceil: (),
    floor: (),
    round: (),
    max: (),
    min: (),
    percentage: (),
    random: (),
    pow: (),
    sqrt: (),
    sin: (),
    cos: (),
    tan: (),
    log: (),
    hypot: (),
    atan2: (),
    asin: (),
    acos: (),
    atan: (),
    clamp: (),
    unit: (),
    is_unitless: (),   // field name → "is-unitless" (auto kebab-case)
    compatible: (),
}

#[derive(BuiltinRegistry)]
#[builtin(module = "string", dispatch = "string")]
struct StringBuiltins {
    #[aliases = ["string.length"]]
    str_length: (),     // field name → "str-length" + alias "string.length"

    #[aliases = ["string.index"]]
    str_index: (),

    #[aliases = ["string.slice"]]
    str_slice: (),

    #[aliases = ["string.to-upper-case"]]
    to_upper_case: (),

    // ... 其余 string 函数
}
```

**字段名 → 函数名映射规则**：
- 默认：Rust `snake_case` → SCSS `kebab-case`（`is_unitless` → `is-unitless`）
- 字段名本身就是目标内建名（如 `str_length` → `str-length`）
- `#[aliases = [...]]` 声明模块限定名（`"string.length"`）和别名

**color 模块的一对多别名**：

```rust
#[derive(BuiltinRegistry)]
#[builtin(module = "color", dispatch = "color")]
struct ColorBuiltins {
    // color.adjust / color.adjust-color → "adjust-color"
    #[aliases = ["color.adjust", "color.adjust-color"]]
    adjust_color: (),    // → "adjust-color"

    #[aliases = ["color.change", "color.change-color"]]
    change_color: (),

    #[aliases = ["color.scale", "color.scale-color"]]
    scale_color: (),

    mix: (),             // 无别名，全局名 "mix" 直接匹配
    invert: (),
    // ...
}
```

### 决策 3：darling 属性解析

用 darling 的 `FromDeriveInput` 解析结构体级别属性，`FromField` 解析字段级别属性：

```rust
// sasspile-macros/src/lib.rs

#[derive(FromDeriveInput)]
#[darling(attributes(builtin))]
struct BuiltinRegistryInput {
    ident: Ident,
    module: String,       // "math"
    dispatch: String,     // "math" → 生成 math::call(...) 路由
    data: darling::ast::Data<darling::util::Ignored, BuiltinField>,
}

#[derive(FromField)]
#[darling(attributes(builtin))]
struct BuiltinField {
    ident: Option<Ident>,
    #[aliases(default = "[]")]
    aliases: Vec<String>,
}
```

### 决策 4：宏生成的三组代码

**A. `module_builtin_name`** — 模块限定名 → 内建名：

```rust
pub fn module_builtin_name(name: &str) -> &str {
    match name {
        "math.abs" => "abs",
        "math.div" => "div",
        // ... (只含 aliases，不含全局名)
        _ => name,
    }
}
```

从每个字段的 `#[aliases]` 和字段名（kebab-case 后作为内建名）生成。

**B. `is_known_builtin`** — 检查是否为已知函数：

```rust
pub fn is_known_builtin(name: &str) -> bool {
    matches!(name,
        "abs" | "math.abs" | "div" | "math.div" | ...
    )
}
```

包含每个字段的全局名（kebab-case）+ 所有 aliases。

**C. 模块分派 match arm** — 按模块路由到子模块 `call`：

在 `call_builtin` 中生成类似：
```rust
"abs" | "math.abs" | "div" | "math.div" | ... => {
    math::call(&name, pos_args, kw_args)?
        .ok_or_else(|| SassError::UndefinedFunction(name.clone()))
}
```

由于 `call_builtin` 是 `impl Evaluator` 的方法，宏不能直接生成到 `impl` 块内。方案是宏生成一个 **辅助函数**，在 `call_builtin` 中调用：

```rust
// 宏生成（module_dispatch.rs）
pub fn dispatch_builtin_module(
    name: &str,
    pos_args: &[Value],
    kw_args: &HashMap<String, Value>,
) -> Option<Result<Value>> {
    match name {
        "abs" | "math.abs" | ... => Some(
            math::call(name, pos_args, kw_args)
                .map(|opt| opt.unwrap_or_else(|| ...))
        ),
        "str-length" | "string.length" | ... => Some(
            string::call(name, pos_args, kw_args)...
        ),
        // ...
        _ => None,  // 未匹配，返回 None 让 call_builtin 继续手工分支
    }
}

// builtin.rs 中 call_builtin 改为
if let Some(result) = module_dispatch::dispatch_builtin_module(&name, pos_args, kw_args) {
    return result;
}
match name {
    // 仅保留 meta 内联函数和 CSS 透传
    "type-of" => ...,
    "inspect" => ...,
    // ...
    _ => Err(SassError::UndefinedFunction(name.clone())),
}
```

### 决策 5：不宏化的函数

保留在 `call_builtin` 中手工编写：`sass`、`type-of`、`inspect`、`if`、`content-exists`、`feature-exists`、`mixin-exists`、`function-exists`、`global-variable-exists`、`variable-exists`、`get-function`、`call`、`get-mixin`、`module-functions`、`module-mixins`、`module-variables`、`accepts-content`、`keywords`、`calc-args`、`calc-name`、`calc`、`env`、`var`、`rgba`、`rgb`、`darken`、`lighten`、`mix`、CSS 透传 fallback。

其中 `rgba`/`rgb`/`darken`/`lighten`/`mix` 虽属于 color 模块，但调用 `Self::builtin_*` 方法而非 `color::call`，签名不统一，不纳入宏。

## Risks / Trade-offs

- **[编译时间增加]** syn + darling + quote + proc-macro2 编译约增加 5-10 秒全量编译 → 长期维护收益（IDE 支持、错误定位）远超一次性编译开销
- **[双 crate 发布]** 需要同时发布 `sasspile` 和 `sasspile-macros` → `sasspile-macros` 仅 `sasspile` 使用，版本同步即可
- **[字段名 kebab-case 转换]** `is_unitless` → `is-unitless` 需要运行时转换 → 在宏编译期用 `quote!` 生成 `&'static str`，零运行时开销
- **[string 模块映射不一致]** `string.length` → `str-length`（字段名 `str_length`），但 `string.quote` → `quote`（字段名 `quote`）→ 字段名本身就是目标内建名，aliases 声明模块限定名，不需隐式推导
- **[dispatch_builtin_module 返回 Option]** 新增一层函数调用 → 编译器内联优化，且 `Option` 仅在未匹配时多一次 `None` 检查
