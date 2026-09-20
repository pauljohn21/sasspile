## Context

sasspile 当前对 CSS 原生数学函数的处理路径：

```
Source → Lexer → Parser → Evaluator (builtin/math.rs / calc_ast.rs) → Serializer (display.rs)
```

**问题**：CSS 函数（`round()`, `clamp()`, `rem()`, `mod()`, `min()`, `max()`, `var()`）进入 eval 后被 Sass 数学模块处理，导致：
1. `var(--c, 1 + 2)` 的 fallback `1 + 2` 被简化为 `3` → 违反 CSS 规范（fallback 是懒求值）
2. `var(--c,)` 序列化丢失尾部逗号后空格
3. vendor prefix `-A-CALC` 未被规范化为 `-a-calc`
4. CSS `round(117, 25)` — 2-参数 round 被当 Sass math.round 而非 CSS 策略取整
5. `calc(//\n c)` 注释处理不符合 CSS 规范

**约束**：
- 纯 Rust 所有权模型，无 GC
- 函数式迭代器链风格
- 无 `clone()` 满天飞，无 `&mut self`

## Goals / Non-Goals

**Goals:**
- CSS 原生 math 函数在 eval 阶段保持原样传递到 serializer（不简化）
- CSS `round(strategy, number, step)` 走策略取整算法
- CSS `rem()`/`mod()` 实现 + 无穷/NaN 行为
- `var()` 符合 CSS 规范的序列化
- vendor prefix 函数名小写规范化

**Non-Goals:**
- 不改变 Sass `math.*` 模块行为（仅限 CSS 函数路径）
- 不实现 `calc-size()`（仅 6 个 case、新属性）
- 不实现 `mix-blend-mode` 等非 math 函数

## Decisions

### Decision 1: CSS 函数识别时机

**选择**：在 eval 分派阶段识别 CSS 原生函数名，标记为 `CssNativeFunc`

**替代方案**：
- A) 在 parse 阶段标记 — 但 parse 不知道调用上下文
- B) 在 serialize 阶段回溯 — 但 AST 已简化，无法还原
- C) **eval 阶段标记**（选中）— 已有 callee 信息，可精准判断

```rust
// 新增 CSS 原生函数集合
const CSS_MATH_FUNCTIONS: &[&str] = &[
    "round", "clamp", "rem", "mod", "min", "max", "var",
    "calc", "element", "expression",
];
```

### Decision 2: round() 策略取整

**选择**：独立 `css_round(strategy_name, value, step)` 函数，严格按 CSS 规范

```rust
fn css_round(strategy: &str, value: f64, step: f64) -> f64 {
    match strategy {
        "nearest" => (value / step).round() * step,
        "up" => (value / step).ceil() * step,       // toward +infinity
        "down" => (value / step).floor() * step,    // toward -infinity
        "to-zero" => (value / step).trunc() * step, // toward zero
        _ => value,
    }
}
```

**验证**：`round("up", -5.1)` = `-5`（up 向 +inf = 取 ceil(-5.1/1) * 1 = -5）✓

### Decision 3: var() 序列化

**选择**：在 `CalcNode` AST 中 Var 节点增加 `trailing_comma_normalized` 标记

```rust
CalcNode::Var {
    name: String,
    fallback: Option<Box<CalcNode>>,
    /// 原始 var() 是否有尾部逗号
    had_trailing_comma: bool,
}
```

序列化时：`var(--c, )` + 空格（如果有尾部逗号但无 fallback）

### Decision 4: vendor prefix 规范化

**选择**：CSS 函数名 lowering 在 parse → AST 构建阶段完成

```rust
fn normalize_css_fn_name(name: &str) -> String {
    // -A-CALC → -a-calc, -C-ELEMENT → -c-element
    // 保留 vendor prefix 连字符，仅 lowering 函数名部分
    match name.rfind('-') {
        Some(idx) if idx > 0 => {
            let (prefix, func) = name.split_at(idx);
            format!("{}{}", prefix, func.to_lowercase())
        }
        None => name.to_lowercase(),
    }
}
```

### Decision 5: rem/mod CSS 语义

- `rem(a, b)` = `a - b * trunc(a/b)`（向零取余，sign 同 a）
- `mod(a, b)` = `a - b * floor(a/b)`（向负无穷取余，sign 同 b）
- 无穷行为：`rem(5, infinity) = 5`, `rem(5, -infinity) = 5`

## Risks / Trade-offs

- [过度 lowering] → 仅在 CSS 上下文 lowering，不碰 Sass 函数名
- [NaN 传播] → `round(NaN, NaN)` 应序列化为 `calc(NaN)` 而非数字
- [性能] → 新分支对非 math 函数无开销（early return）
