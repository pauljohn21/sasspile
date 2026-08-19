---
name: sasspile-dev
description: |
  CRITICAL: Master skill for sasspile — pure functional SCSS compiler in Rust.
  Use for ANY work on the codebase: compilation pipeline, debugging, built-in functions,
  CSS serialization, and selector operations.
  Triggers: sasspile, SCSS, Sass, 编译, pipeline, lexer, parser, evaluator, serializer,
  debugging, tracing, bug, 内建函数, color, selector, CSS, 词法, 语法, 求值, 序列化
globs: ["src/**/*.rs", "tests/**/*.rs", "**/Cargo.toml"]
---

# sasspile 开发全技能指南

> sasspile 是纯 Rust 函数式 SCSS 编译器。本文档是开发单一入口，覆盖编译管线、调试、内建函数、CSS 序列化、选择器操作。

---

## 1. 编译管线架构

```
Source::new(input)     → Source     (源码包装)
  .lex()               → Lexed      (词法分析 → Token 流)
  .parse()             → Parsed     (语法分析 → AST)
  .evaluate()          → Evaluated  (求值 → CssNode 树)
  .serialize(style)    → Serialized (序列化 → CSS 字符串)
```

### 入口函数 (`src/lib.rs`)

| 函数 | 输入 | 输出 | 用途 |
|------|------|------|------|
| `compile(input, style)` | `&str` | `Result<String>` | 通用入口 |
| `compile_expanded(input)` | `&str` | `Result<String>` | 展开式输出 |
| `compile_compressed(input)` | `&str` | `Result<String>` | 压缩输出 |
| `compile_file(path, style)` | `&PathBuf` | `Result<String>` | 文件编译 |
| `compile_file_with_load_paths(path, style, paths)` | `&PathBuf` | `Result<String>` | 带加载路径 |

---

## 2. 阶段详解

### 阶段 1：词法分析 (`src/lex/`)

```
src/lex/
├── mod.rs      # Lexer 结构体 + tokenize 逻辑
└── token.rs    # Token 枚举定义
```

| Token | 说明 |
|-------|------|
| `Ident(String)` | 标识符 |
| `Number(f64, Option<String>)` | 数字（可选单位） |
| `String(String, bool)` | 字符串 |
| `Color(u32)` | 十六进制颜色 |
| `AtKeyword(String)` | `@use`、`@mixin` 等 |
| `LParen`/`RParen`/`LBrace`/`RBrace` | 括号 |
| `Comma`/`Semicolon`/`Colon` | 分隔符 |
| `Amp`/`Slash`/`Star`/`Dot` | 运算符 |
| `Eof` | 文件结束 |

**追踪命令**：
```bash
RUST_LOG="sasspile::lex=trace" cargo test --test lex_test -- --nocapture
```

---

### 阶段 2：语法分析 (`src/parse/`)

```
src/parse/
├── mod.rs          # Parser 入口
├── ast/
│   ├── mod.rs      # Node 枚举 + Value 枚举定义 + ColorFormat + 颜色辅助函数
│   └── display.rs  # Display for Value（ColorFormat 分派序列化）
├── ast_impl.rs     # AST 实现（to_scss 等）
├── at_rules.rs     # @use/@mixin/@include/@if/@for 解析
├── expr/
│   ├── mod.rs      # 表达式解析入口
│   └── prefix.rs   # Pratt 前缀解析 + parse_number/parse_hash_color
└── nodes.rs        # 节点解析辅助
```

**核心 AST 类型**：

```rust
pub enum Node {
    Rule { selector: String, body: Vec<Node> },
    Decl { property: String, value: Value, important: bool },
    Variable { name: String, value: Value, flags: VarFlags },
    If { branches: Vec<(Value, Vec<Node>)>, else_body: Option<Vec<Node>> },
    For { var: String, from: Value, to: Value, inclusive: bool, body: Vec<Node> },
    Each { vars: Vec<String>, list: Value, body: Vec<Node> },
    While { condition: Value, body: Vec<Node> },
    Mixin { name: String, params: Vec<Param>, body: Vec<Node> },
    Include { name: String, args: Vec<Arg>, content: Option<Vec<Node>> },
    Content, Return(Value),
    AtRule { name: String, params: String, body: Vec<Node> },
    Use { url: String, namespace: Option<String>, star: bool, config: Vec<(String, Value)> },
    Forward { ... }, Extend { selector: String },
    Comment(String, bool),
}

pub enum Value {
    Number(f64, Option<String>), String(String, bool),
    Color(Color), List(Vec<Value>, Separator, bool),
    Map(Vec<(Value, Value)>), Bool(bool), Null,
    Call(String, Vec<Arg>),
    BinOp { op: BinOp, left: Box<Value>, right: Box<Value> },
    UnaryOp(UnaryOp, Box<Value>), Interp(String),
    If { ... }, Identifier(String),
}
```

**追踪命令**：
```bash
RUST_LOG="sasspile::parse=debug" cargo test --test parse_test -- --nocapture
```

---

### 阶段 3：求值 (`src/eval/`)

```
src/eval/
├── mod.rs              # Evaluator + eval_nodes
├── value/
│   ├── mod.rs          # eval_value + eval_binop + units_compatible + eval_interp_str
│   ├── ops.rs          # 值运算实现（add/sub/mul/div/modulo/compare 细节）
│   └── display.rs      # inspect_value + 值显示格式化
├── rule.rs             # Rule 求值
├── control_flow.rs     # @if/@for/@each/@while
├── mixin.rs            # @mixin/@include
├── extend.rs           # @extend 后处理
├── at_params.rs        # @media/@supports 参数插值和表达式求值
├── module.rs           # @use/@forward + call_module_function
├── builtin.rs          # call_builtin 分派入口 + meta 函数
├── builtin/
│   ├── math.rs         # 数学函数（abs/ceil/floor/round/div/pow/clamp/...）+ 命名参数合并 + 参数验证
│   ├── color.rs        # 颜色函数（invert/hsl/hwb/adjust-color/...）
│   ├── list.rs         # 列表函数（length/nth/append/join/...）
│   ├── map.rs          # 映射函数（map-get/map-merge/...）
│   ├── string.rs       # 字符串函数（str-length/str-slice/...）+ 参数验证
│   └── selector.rs     # 选择器函数（selector-nest/selector-parse/...）
└── color.rs            # 颜色辅助（rgb_to_hsl/hwb_to_rgb/hsl_to_rgb）
```

**求值入口**：
```rust
Evaluator::evaluate(ast) -> Result<Vec<CssNode>>
Evaluator::evaluate_with_path(ast, base_path) -> Result<Vec<CssNode>>
Evaluator::evaluate_with_path_and_load_paths(ast, base_path, load_paths) -> Result<Vec<CssNode>>
```

**内建函数分派链**：
```
eval_value(Call)
  → call_function(name, pos_args, kw_args, env)
    → call_user_function(name, ...)     // 用户 @function
    → call_module_function(name, ...)   // color.adjust() / math.round()
    → call_builtin(name, ...)           // 旧版全局 adjust-color()
```

**sass:color → 全局名映射**（`src/eval/module.rs`）：

```rust
"color.adjust" => "adjust-color"
"color.change" => "change-color"
"color.scale" => "scale-color"
"color.mix" => "mix"
"color.channel" => "color-channel"
"color.whiteness" => "whiteness"
"color.blackness" => "blackness"
"color.to-space" => "to-space"
"color.to-gamut" => "to-gamut"
```

**追踪命令**：
```bash
RUST_LOG="sasspile::eval=info" cargo test --test compile_test <name> -- --nocapture
RUST_LOG="sasspile::eval::builtin::color=trace" cargo test --test compile_test -- --nocapture
```

---

### 阶段 4：CSS 序列化 (`src/css/`)

```
src/css/
├── mod.rs          # Serializer::serialize + serialize_nodes
├── node.rs         # CssNode 枚举
└── selector.rs     # sanitize_selector + normalize_attr_selectors + has_bogus_combinators
```

**CssNode 类型**：

```rust
pub enum CssNode {
    Rule { selector: String, declarations: Vec<CssNode>, children: Vec<CssNode> },
    Declaration { property: String, value: String, important: bool },
    AtRule { name: String, params: Option<String>, children: Vec<CssNode>, has_body: bool },
    Comment(String),
    AtRoot(Vec<CssNode>),
    Raw(String),
    Return(Value),  // 不序列化，仅内部传播
}
```

**输出风格**：

| 风格 | 说明 |
|------|------|
| `Expanded` | 展开格式（默认，可读性好） |
| `Compressed` | 压缩格式（无空白） |

**序列化关键逻辑**：
- `flatten_nodes` — 展平嵌套规则
- `merge_at_rules` — 合并相邻同名的 @media/@supports
- `sanitize_selector` — 净化占位符 + 组合器验证 + 属性选择器规范化
- 非 ASCII 内容自动添加 `@charset "UTF-8"` 前缀

**追踪命令**：
```bash
RUST_LOG="sasspile::css=debug" cargo test --test compile_test -- --nocapture
```

---

## 3. 内建函数完整参考

### 3.1 Math 函数

| 函数 | 参数 | 说明 |
|------|------|------|
| `abs(n)` | 1 number | 绝对值 |
| `ceil(n)` | 1 number | 向上取整 |
| `floor(n)` | 1 number | 向下取整 |
| `round(n)` | 1 number | 四舍五入 |
| `min(...)` | N numbers | 最小值 |
| `max(...)` | N numbers | 最大值 |
| `percentage(n)` | 1 number | 转百分比（×100） |
| `div(a, b)` | 2 numbers | 除法 |
| `pow(base, exp)` | 2 numbers | 幂运算 |
| `sqrt(n)` | 1 number | 平方根 |
| `sin(n)` | 1 number | 正弦（弧度） |
| `cos(n)` | 1 number | 余弦（弧度） |
| `tan(n)` | 1 number | 正切（弧度） |
| `asin(n)` | 1 number | 反正弦（返回 deg） |
| `acos(n)` | 1 number | 反余弦（返回 deg） |
| `atan(n)` | 1 number | 反正切（返回 deg） |
| `atan2(y, x)` | 2 numbers | 四象限反正切（返回 deg） |
| `hypot(...)` | N numbers | 欧几里得范数 |
| `log(n, base?)` | 1-2 numbers | 对数 |
| `random(max?)` | 0-1 numbers | 随机数 |
| `clamp(min, val, max)` | 3 numbers | 值钳制 |
| `unit(n)` | 1 number | 获取单位字符串 |
| `is-unitless(n)` | 1 number | 是否无单位 |
| `compatible(a, b)` | 2 numbers | 单位是否兼容 |
| `comparable(a, b)` | 2 numbers | compatible 别名（仅全局，math.comparable 未定义） |

**命名参数支持**：所有 math 函数支持命名参数调用（如 `math.abs($number: 3)`、`math.clamp($min: 0, $number: 1, $max: 2)`、`math.pow($base: 2, $exponent: 3)`、`math.div($number1: 6, $number2: 3)`）。`merge_math_args()` 按参数名合并 pos_args 和 kw_args。

**添加新 Math 函数**：在 `src/eval/builtin/math.rs` 的 `call()` 中添加分支，并在 `math_param_names()` 中注册参数名。

---

### 3.2 Color 函数

**构造函数**：

| 函数 | 说明 | 文件 |
|------|------|------|
| `rgb(r, g, b)` / `rgba(r, g, b, a)` | RGB 构造 | `builtin.rs` → `builtin_rgba` |
| `hsl(h, s%, l%)` / `hsla(...)` | HSL 构造 | `builtin/color.rs` |
| `hwb(h, w%, b%)` / `hwb(h, w%, b%, a)` | HWB 构造 | `builtin/color.rs` |

**操作函数**：

| 函数 | 说明 | 文件 |
|------|------|------|
| `adjust-color($color, $red?, $green?, $blue?, $hue?, $saturation?, $lightness?, $whiteness?, $blackness?, $alpha?)` | 增量调整 | `builtin/color.rs` |
| `change-color($color, $red?, $green?, ...)` | 绝对替换 | `builtin/color.rs` |
| `scale-color($color, $red?, $green?, $blue?, $saturation?, $lightness?, $alpha?)` | 比例缩放 | `builtin/color.rs` |
| `mix($color1, $color2, $weight)` | 颜色混合 | `builtin.rs` → `builtin_mix` |
| `darken($color, $amount)` | 变暗 | `builtin.rs` → `builtin_darken` |
| `lighten($color, $amount)` | 变亮 | `builtin.rs` → `builtin_lighten` |
| `adjust-hue($color, $degrees)` | 调整色相 | `builtin/color.rs` |
| `saturate($color, $amount?)` | 增加饱和度（CSS 滤镜透传） | `builtin/color.rs` |
| `desaturate($color, $amount)` | 减少饱和度 | `builtin/color.rs` |
| `grayscale($color)` | 灰度化 | `builtin/color.rs` |
| `complement($color)` | 补色 | `builtin/color.rs` |
| `invert($color)` | 反相 | `builtin/color.rs` |
| `opacify($color, $amount)` / `fade-in(...)` | 增加不透明度 | `builtin/color.rs` |
| `transparentize($color, $amount)` / `fade-out(...)` | 减少不透明度 | `builtin/color.rs` |

**通道读取函数**：

| 函数 | 返回值 | 文件 |
|------|--------|------|
| `red(color)` | number (0-255) | `builtin/color.rs` |
| `green(color)` | number (0-255) | `builtin/color.rs` |
| `blue(color)` | number (0-255) | `builtin/color.rs` |
| `alpha(color)` / `opacity(color)` | number (0-1) | `builtin/color.rs` |
| `hue(color)` | number (deg) | `builtin/color.rs` |
| `saturation(color)` | number (%) | `builtin/color.rs` |
| `lightness(color)` | number (%) | `builtin/color.rs` |
| `whiteness(color)` | number (%) | `builtin/color.rs` |
| `blackness(color)` | number (%) | `builtin/color.rs` |
| `color-channel($color, $channel)` | number | `builtin/color.rs` |

**Level 4 颜色空间函数**：

| 函数 | 说明 | 文件 |
|------|------|------|
| `is-powerless($color, $channel)` | 通道是否无效 | `builtin/color.rs` |
| `is-in-gamut($color)` | 是否在色域内 | `builtin/color.rs` |
| `is-legacy($color)` | 是否 legacy 空间 | `builtin/color.rs` |

**颜色辅助函数**（`src/eval/color.rs`）：
- `rgb_to_hsl(r, g, b) → (h, s, l)`
- `hsl_to_rgb(h, s, l) → Color`
- `hwb_to_rgb(h, w, b, a) → Color`

**颜色序列化辅助函数**（`src/parse/ast.rs`）：
- `hsl_to_rgb_percent(h, s, l) → (r%, g%, b%)` — 从 HSL 精确计算 RGB 百分比（避免 u8 精度丢失）
- `format_pct_val(v) → String` — 格式化百分比值（0-100，10 位小数截断）
- `format_hue(h) → String` — 格式化 hue 值（整数无小数点）
- `format_pct(v) → String` — 格式化百分比值（0-1 → 0%-100%）
- `format_alpha(a) → String` — 格式化 alpha 值

**ColorFormat 枚举**（`src/parse/ast.rs`）：

| 格式 | 用途 | 序列化示例 |
|------|------|------------|
| `Auto` | hex / 命名颜色 / rgba | `#ff0000`, `red`, `rgba(0,0,0,0.5)` |
| `Rgb` | rgb(r,g,b) 固定格式 | `rgb(255, 0, 0)` |
| `RgbPercent(h,s,l)` | HSL 操作结果的百分比输出 | `rgb(72%, 0%, 0%)` |
| `Hsl(h,s,l)` | hsl() 创建的颜色保留格式 | `hsl(120, 50%, 50%)` |
| `Hwb(h,w,b)` | hwb() 创建的颜色保留格式 | `hwb(0 30% 40%)` |

**颜色算法说明**：
- `darken`/`lighten`：通过 HSL lightness 增减实现（非 RGB 倍数）
- `saturate`/`desaturate`：通过 HSL saturation 增减实现
- `adjust-hue`/`complement`/`invert`：通过 HSL hue 旋转实现（非 RGB 反色）
- `grayscale`：通过 HSL saturation=0 实现（非 RGB 平均值）
- 所有 HSL 操作结果用 `RgbPercent` 格式输出（匹配 sass-spec）
- 依赖 `color` crate v0.3 提供色彩空间转换参考

**添加新 Color 函数**：在 `src/eval/builtin/color.rs` 的 `call()` 中添加 match 分支，返回 `Ok(Some(Value::...))`。

---

### 3.3 List 函数

| 函数 | 说明 | 文件 |
|------|------|------|
| `length($list)` / `list-length($list)` | 列表长度 | `builtin/list.rs` |
| `nth($list, $n)` | 取第 n 个 | `builtin/list.rs` |
| `append($list, $val, $separator?)` | 追加元素 | `builtin/list.rs` |
| `join($list1, $list2, $separator?)` | 连接列表 | `builtin/list.rs` |
| `index($list, $value)` | 查找元素索引 | `builtin/list.rs` |
| `list-separator($list)` / `separator($list)` | 获取分隔符 | `builtin/list.rs` |
| `set-nth($list, $n, $value)` | 设置第 n 个 | `builtin/list.rs` |
| `is-bracketed($list)` | 是否有方括号 | `builtin/list.rs` |
| `zip(...)` | 多列表拉链合并 | `builtin/list.rs` |

**添加新 List 函数**：在 `src/eval/builtin/list.rs` 的 `call()` 中添加分支。

---

### 3.4 Map 函数

| 函数 | 说明 | 文件 |
|------|------|------|
| `map-get($map, $key)` | 获取值 | `builtin/map.rs` |
| `map-keys($map)` | 所有键 | `builtin/map.rs` |
| `map-values($map)` | 所有值 | `builtin/map.rs` |
| `map-has-key($map, $key)` | 是否包含键 | `builtin/map.rs` |
| `map-merge($map1, $map2)` | 合并映射 | `builtin/map.rs` |
| `map-remove($map, ...$keys)` | 移除键 | `builtin/map.rs` |
| `map-set($map, $key, $value)` | 设置键值 | `builtin/map.rs` |
| `map-deep-merge(...)` | 深层合并 | `builtin/map.rs` |
| `map-deep-remove(...)` | 深层移除 | `builtin/map.rs` |

**添加新 Map 函数**：在 `src/eval/builtin/map.rs` 的 `call_map_builtin()` 中添加分支。

---

### 3.5 String 函数

| 函数 | 说明 | 文件 |
|------|------|------|
| `str-length($string)` | 字符串长度 | `builtin/string.rs` |
| `str-slice($string, $start, $end?)` | 截取子串 | `builtin/string.rs` |
| `str-index($string, $substring)` | 查找子串位置 | `builtin/string.rs` |
| `str-insert($string, $insert, $index)` | 插入子串 | `builtin/string.rs` |
| `to-upper-case($string)` | 转大写 | `builtin/string.rs` |
| `to-lower-case($string)` | 转小写 | `builtin/string.rs` |
| `unquote($string)` | 去引号 | `builtin/string.rs` |
| `quote($string)` | 加引号 | `builtin/string.rs` |
| `unique-id()` | 生成唯一 ID | `builtin/string.rs` |
| `str-split($string, $separator)` | 分割字符串 | `builtin/string.rs` |

**添加新 String 函数**：在 `src/eval/builtin/string.rs` 的 `call()` 中添加分支。

---

### 3.6 Selector 函数

| 函数 | 说明 | 文件 |
|------|------|------|
| `selector-append(...$selectors)` | 拼接选择器 | `builtin/selector.rs` |
| `selector-nest(...$selectors)` | 嵌套选择器（空格分隔） | `builtin/selector.rs` |
| `selector-is-super($super, $sub)` | 超集检查 | `builtin/selector.rs` |
| `selector-parse($selector)` | 解析为列表 | `builtin/selector.rs` |
| `selector-simple-selectors($selector)` | 拆分为简单选择器 | `builtin/selector.rs` |
| `selector-unify($selector1, $selector2)` | 统一选择器 | `builtin/selector.rs` |
| `selector-extend($selector, $target, $extender)` | 扩展选择器 | `builtin/selector.rs` |
| `selector-replace($selector, $original, $replacement)` | 替换子串 | `builtin/selector.rs` |

**添加新 Selector 函数**：在 `src/eval/builtin/selector.rs` 的 `call()` 中添加分支。

---

### 3.7 Meta 函数

| 函数 | 说明 |
|------|------|
| `type-of($value)` | 返回类型名 |
| `inspect($value)` | 值的 Sass 表示 |
| `if($condition, $if-true, $if-false)` | 条件表达式 |
| `call($function-name, ...$args)` | 动态调用 |
| `keywords($args)` | 获取关键字参数映射 |
| `feature-exists($feature)` | 检查 Sass 特性 |
| `function-exists($name)` | 检查函数是否存在 |
| `mixin-exists($name)` | 检查 mixin 是否存在 |
| `variable-exists($name)` / `global-variable-exists($name)` | 检查变量是否存在 |
| `get-function($name)` | 获取函数引用 |
| `content-exists()` | 检查是否有 @content |
| `is-known-builtin($name)` | 检查是否为已知内建函数 |
| `calc(...)` / `env(...)` / `var(...)` | CSS 原生函数原样保留 |

---

## 4. CSS 序列化详解

### 4.1 选择器净化 (`sanitize_selector`)

处理流程：
1. `normalize_attr_selectors` — 属性选择器规范化（去引号、修饰符空格）
2. `normalize_adjacent_compounds` — 相邻复合选择器规范化（`[a]b` → `[a] b`）
3. `has_bogus_combinators` — 组合器验证（无效返回空字符串）
4. 占位符移除 — 顶层移除纯占位符部分，伪类内清理

### 4.2 组合器验证规则

| 上下文 | 前导组合器 | 连续组合器 | 尾部组合器 |
|--------|-----------|-----------|-----------|
| 顶层 | 允许单个 | 禁止 | 禁止 |
| `:has()` | 允许单个 | 禁止 | 禁止 |
| `:is/:where/:not/matches` | 禁止 | 禁止 | 禁止 |

### 4.3 @charset 处理

输出含非 ASCII 字符时自动添加 `@charset "UTF-8";` 前缀：
- `Expanded` 模式：`@charset "UTF-8";\n{css}`
- `Compressed` 模式：`@charset"UTF-8";{css}`

### 4.4 @规则合并

`merge_at_rules` 合并相邻的同名 `@media`/`@supports` 块。

---

## 5. 调试追踪系统

### 5.1 Span 层级树

```
eval_nodes                    # 节点列表求值
├── eval_node_item            # try_fold 每个节点
│   └── eval_node             # 单节点求值
│       ├── eval_rule         # 规则求值
│       ├── eval_for          # @for 循环
│       ├── eval_each         # @each 循环
│       ├── eval_include      # @include mixin
│       └── eval_value        # 值表达式
│           ├── call_function     # 函数调用入口
│           │   ├── call_builtin      # 内建函数
│           │   ├── call_module_function # 模块函数
│           │   └── call_user_function   # 用户函数
│           └── eval_interp_str  # 插值求值
├── load_module               # 文件模块加载
└── apply_extends             # @extend 后处理
```

### 5.2 调试命令

```bash
# Level 1: 只看错误
RUST_LOG=error cargo test --test compile_test <name> -- --nocapture

# Level 2: 看函数调用链
RUST_LOG=info cargo test --test compile_test <name> -- --nocapture

# Level 3: 完整 span 嵌套
RUST_LOG=debug cargo test --test compile_test <name> -- --nocapture

# Level 4: 含值表达式
RUST_LOG=trace cargo test --test compile_test <name> -- --nocapture

# Per-target 过滤
RUST_LOG="sasspile::color=trace" cargo test --test compile_test -- --nocapture
RUST_LOG="sasspile::eval::extend=debug" cargo test --test compile_test -- --nocapture

# CSS Diff 详情
RUST_LOG="cssdiff=debug" cargo test --test cf_diag diag_<subdir> -- --nocapture

# 最小化失败用例
RUST_LOG="minimize=info" cargo test --test minimize minimize_color_error -- --nocapture
```

### 5.3 Event Targets 值快照

| Target | Level | 场景 |
|--------|-------|------|
| `sasspile::color` | trace | 颜色转换输入/输出 |
| `sasspile::extend` | info | @extend 匹配成功 |
| `cssdiff` | info | CSS diff 摘要 |
| `minimize` | info | 最小化轮次 |

### 5.4 错误模式速查

| 错误信息 | 根因 | 修复方向 |
|----------|------|---------|
| `Undefined function: xxx` | 函数未注册或映射缺失 | 检查 `call_module_function` 映射表 |
| `Undefined variable: $xxx` | 变量作用域问题 | 检查 `env.lookup` 和 `@use` 命名空间 |
| `Missing argument $xxx` | 参数数量不匹配 | 检查 `builtin.rs` / `builtin/math.rs` / `builtin/string.rs` 对应函数 |
| `Only N arguments allowed` | 参数过多 | 检查 math/string 函数参数验证 |
| `$number: X is not a number` | math 函数收到非数字参数 | 检查 `merge_math_args` 合并后的值类型 |
| `$string: X is not a string` | string 函数收到非字符串参数 | 检查 `call_string_builtin` 参数验证 |
| `$base: Expected Xpx to have no units` | math.pow 带单位参数 | 检查 `pow` 单位验证逻辑 |
| `Parse error: expected X` | 解析器不支持该语法 | 检查 `parse_value` / `parse_args` |
| `Cannot load module: ...` | 文件路径解析失败 | 检查 `resolve_file` |
| `content_diff` | 输出内容差异 | 用 `RUST_LOG="cssdiff=debug"` 看逐行 diff |
| `missing_output` | 实际输出缺少行 | 检查 CSS 序列化 |

### 5.5 添加新 Span

```rust
// 属性 span（函数入口）
#[instrument(skip(large_param), fields(context_field = value))]
fn my_function(large_param: &BigType, context_field: &str) -> Result<...> { ... }

// 手动 span（条件分支）
let span = tracing::info_span!("my_function", key_field = value);
let _enter = span.enter();

// 错误记录
result.map_err(|e| {
    tracing::error!(error = %e, "my_function failed");
    e
})?;
```

**Span 级别规范**：

| 级别 | 使用场景 |
|------|---------|
| `error` | 错误发生时 |
| `warn` | 可恢复的异常 |
| `info` | 关键路径入口 |
| `debug` | 次要路径 |
| `trace` | 高频调用 |

---

## 6. 关键类型速查

| 类型 | 文件 | 说明 |
|------|------|------|
| `Token` | `lex/token.rs` | 词法单元 |
| `Node` | `parse/ast.rs` | AST 节点 |
| `Value` | `parse/ast.rs` | 求值结果 |
| `Color` | `parse/ast.rs` | RGBA 颜色（r/g/b: u8, a: f64） |
| `ColorFormat` | `parse/ast.rs` | 颜色格式追踪（Auto/Rgb/RgbPercent/Hsl/Hwb） |
| `CssNode` | `css/node.rs` | CSS 输出节点 |
| `Env` | `eval/mod.rs` | 求值环境（变量/函数/mixin 作用域） |
| `Arg` | `parse/ast.rs` | 函数调用参数 |
| `OutputStyle` | `lib.rs` | 输出风格枚举 |
| `SassError` | `error.rs` | 错误类型 |

---

## 7. 添加内建函数标准流程

1. **确定类别**：判断函数属于 color/list/map/string/selector/math/meta
2. **编辑分派入口**：在 `src/eval/builtin.rs` 的 `call_builtin` match 中添加分支
3. **实现函数逻辑**：
   - 简单函数：直接写在 `builtin.rs` 中
   - 复杂函数组：在 `builtin/<cat>.rs` 中添加 match 分支
4. **注册为已知函数**：在 `is_known_builtin()` 中添加名称
5. **CSS 透传处理**：如果是 CSS 原生函数，在 `is_css_function()` 中添加
6. **命名参数**：如需支持命名参数（math/string/list 函数），在对应子模块中注册参数名
7. **验证**：`cargo test --test compile_test`

### 示例：添加 `color.lightness` 的新通道读取

```rust
// 1. 在 builtin/color.rs 的 call() 中添加分支
"lightness" => {
    let color_arg = args.first().or_else(|| kw_args.get("$color"));
    match color_arg {
        Some(Value::Color(c)) => {
            let (_, _, l) = Evaluator::rgb_to_hsl(c.r, c.g, c.b);
            Ok(Some(Value::Number(l * 100.0, Some("%".into()))))
        }
        _ => Err(SassError::Eval("lightness 需要 1 个颜色参数".into())),
    }
}

// 2. 在 builtin.rs 的 is_known_builtin 中添加 "lightness"
// 3. 在 call_builtin 的 match 中已包含 "lightness" → color::call
```

---

## 8. 测试命令

```bash
# 核心测试
cargo test --test compile_test    # 41 个
cargo test --test stage_test      # 10 个
cargo test --test ast_test        # 8 个
cargo test --test common_test     # 5 个

# 兼容性测试
cargo test --test bs_spec -- --nocapture    # 15 个（Bootstrap）
cargo test --test ep_full -- --nocapture    # 121 个（Element Plus，约 28 秒）

# sass-spec 全量统计（约 70 秒）
RUST_LOG="sass_spec_full=info,sasspile=warn" cargo test --test sass_spec_full -- --nocapture
# 基线：3478/11775 = 29%（全量统计，只跳过 libsass/non_conformant 弃用目录）

# sass-spec 诊断
cargo test --test cf_diag diag_<subdir> -- --nocapture

# 全量统计
RUST_LOG=info cargo test --test sass_spec_full test_sass_spec_full_stats -- --nocapture
```

---

## 禁止事项

- **禁止未追踪就修复** — 必须先用 `RUST_LOG` 看错误链路
- **禁止猜测根因** — 用 span 链路定位，而非猜测
- **禁止批量修改** — 每次只改一个变量，用 tracing 验证
- **禁止跳过验证** — 修复后必须确认错误消失且无回归

## 9. 错误消息规范

所有错误消息必须使用英文，以匹配 sass-spec 的期望输出：

### 9.1 错误消息格式

| 函数类别 | 格式示例 |
|----------|----------|
| Math 参数缺失 | `Missing argument $number` |
| Math 参数过多 | `Only 1 argument is allowed but 2 were passed` |
| Math 类型错误 | `$number: X is not a number` |
| Math 单位错误 | `$base: Expected Xpx to have no units` |
| String 参数缺失 | `Missing argument $string` |
| String 类型错误 | `$string: X is not a string` |
| 未定义函数 | `Undefined function: xxx` |
| 未定义变量 | `Undefined variable: $xxx` |

### 9.2 SassError 定义

`src/error.rs` 中的 `SassError` 枚举使用 `thiserror` 派生，所有变体的 `Display` 实现输出纯英文消息（无中文前缀）。

### 9.3 内联参数验证

Math 和 String 函数在各自模块（`builtin/math.rs`、`builtin/string.rs`）中直接内联参数验证，返回格式化的英文错误消息。
