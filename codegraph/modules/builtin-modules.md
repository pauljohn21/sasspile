# 内置模块 ✅ 已完成

## 职责

实现 Sass 规范中的 6 个内置模块：`sass:color`, `sass:math`, `sass:list`, `sass:map`, `sass:string`, `sass:meta`。

## 文件结构（实际）

```
builtin/
├── mod.rs          # dispatch 统一分发入口
├── sass_color.rs   # sass:color 函数
├── sass_math.rs    # sass:math 函数
├── sass_list.rs    # sass:list 函数
├── sass_map.rs     # sass:map 函数
├── sass_string.rs  # sass:string 函数
└── sass_meta.rs    # sass:meta 函数
```

## 分发机制

**文件: `sasspile/src/builtin/mod.rs`**

```rust
pub fn dispatch(
    name: &str,          // e.g., "color.adjust-hue"
    args: &[Expr],
    ctx: &mut EvalContext,
) -> Result<Option<Value>, EvalError>
```

解析 `module.function` 格式，路由到对应模块的 `call` 函数。

## 模块函数索引

### sass-color
- `rgb`, `rgba`, `red`, `green`, `blue`
- `hsl`, `hsla`, `hue`, `saturation`, `lightness`
- `alpha`, `opacity`, `adjust-hue`, `lighten`, `darken`
- `saturate`, `desaturate`, `grayscale`, `invert`
- `mix`, `scale-color`, `change-color`, `ie-hex-str`

### sass-math
- `pi`, `e`, `tau`
- `abs`, `ceil`, `floor`, `round`
- `max`, `min`, `clamp`
- `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `atan2`
- `pow`, `sqrt`, `log`, `exp`
- `percentage`, `unit`, `comparable`

### sass-list
- `length`, `nth`, `set-nth`
- `append`, `join`, `zip`
- `index`, `separator`
- `slash` (构建 slash 分隔列表)

### sass-map
- `get` (map-get), `keys`, `values`, `has-key`
- `merge`, `remove`
- `set` (map-set)

### sass-string
- `index` (str-index), `length` (str-length)
- `slice` (str-slice), `insert` (str-insert)
- `to-upper-case`, `to-lower-case`
- `unique-id`, `unquote`, `quote`

### sass-meta
- `inspect`, `type-of`, `unit`
- `feature-exists`, `variable-exists`, `function-exists`, `mixin-exists`
- `call`, `get-function`, `content-exists`, `content-args`

## MODULE_NAMES

```rust
pub const MODULE_NAMES: &[&str] = &[
    "color", "math", "list", "map", "string", "meta",
];
```

## 使用方式

```rust
// 通过 EvalContext
let result = builtin::dispatch("math.sin", args, ctx)?;

// 直接调用
use sasspile::builtin::sass_math;
sass_math::call("sin", args, ctx);
```

## 测试

- `tests/builtin_spec.rs`（各模块函数覆盖）
