# Selector AST 设计文档

## 设计原则

- **函数式 move 语义**：所有转换消费输入返回新值，禁止 `&mut` 参数
- **纯数据类型**：AST 类型是 `#[derive(Debug, Clone, PartialEq)]` 的纯 enum/struct，无内部可变性
- **不侵入管线**：`CssNode::Rule.selector` 保持 `String`，选择器 AST 作为计算工具在需要时 parse/use/drop
- **零 clone 满天飞**：解析 → 计算 → 序列化全程 move
- **tracing span**：关键算法入口加 `#[instrument]` 或 `info_span!`
- **单文件 ≤ 500 行**：AST 类型 + parser + display 一个文件，算法一个文件

## 类型层级

```
┌─────────────────────────────────────────────────────────────────┐
│                      Selector 类型层级                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Selector                                                       │
│  │  Vec<ComplexSelector>  (逗号分隔列表)                        │
│  │                                                              │
│  ├─ ComplexSelector                                             │
│  │   Vec<(Option<Combinator>, CompoundSelector)>                │
│  │   组合器分隔的复合选择器序列                                   │
│  │   如: "a > b.c:hover" → [(None, a), (Child, b.c:hover)]      │
│  │                                                              │
│  ├─ CompoundSelector                                            │
│  │   Vec<SimpleSelector>  (无空格，同一复合体)                   │
│  │   如: "b.c:hover" → [Type(b), Class(c), Pseudo(:hover)]      │
│  │                                                              │
│  ├─ SimpleSelector (enum)                                       │
│  │   ├─ Universal                  → *                           │
│  │   ├─ Type(String)              → div, a, span                │
│  │   ├─ Class(String)             → .btn                        │
│  │   ├─ Id(String)                → #main                       │
│  │   ├─ Attribute { ... }         → [type="text"]              │
│  │   ├─ PseudoClass { name, arg } → :hover, :nth-child(2n+1)   │
│  │   ├─ PseudoElement { name }    → ::before                   │
│  │   └─ Placeholder(String)       → %button                    │
│  │                                                              │
│  └─ Combinator (enum)                                           │
│      Descendant (空格) | Child (>) | Adjacent (+) | Sibling (~)│
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## 类型定义

```rust
// src/css/selector_ast.rs

/// 顶层选择器——逗号分隔的复杂选择器列表。
#[derive(Debug, Clone, PartialEq)]
pub struct Selector(pub Vec<ComplexSelector>);

/// 复杂选择器——组合器分隔的复合选择器序列。
/// 第一个元素的 combinator 始终为 None（除非有前导组合器如 "> a"）。
#[derive(Debug, Clone, PartialEq)]
pub struct ComplexSelector {
    pub compounds: Vec<(Option<Combinator>, CompoundSelector)>,
}

/// 复合选择器——无空格的简单选择器序列。
#[derive(Debug, Clone, PartialEq)]
pub struct CompoundSelector(pub Vec<SimpleSelector>);

/// 简单选择器——最小不可分割的选择器单元。
#[derive(Debug, Clone, PartialEq)]
pub enum SimpleSelector {
    Universal,
    Type(String),
    Class(String),
    Id(String),
    Attribute {
        name: String,
        op: Option<String>,     // "=", "~=", "|=", "^=", "$=", "*="
        value: Option<String>,
        modifier: Option<String>, // "i", "s"
    },
    PseudoClass {
        name: String,
        arg: Option<String>,   // :nth-child(2n+1) → arg="2n+1"
    },
    PseudoElement {
        name: String,
        arg: Option<String>,   // ::highlight(name) → arg="name"
    },
    Placeholder(String),       // %button
}

/// 组合器。
#[derive(Debug, Clone, PartialEq)]
pub enum Combinator {
    Descendant,  // " "
    Child,       // ">"
    Adjacent,    // "+"
    Sibling,     // "~"
}
```

## 函数签名设计

```rust
// ── 解析 ────────────────────────────────────────────

/// 将选择器字符串解析为 Selector AST。
/// 解析失败时返回原始字符串包装（降级策略，不阻断编译）。
pub fn parse_selector(input: &str) -> Selector

// ── 序列化 ───────────────────────────────────────────

/// 将 Selector AST 序列化为规范字符串。
impl std::fmt::Display for Selector { ... }

// ── unify 算法 ──────────────────────────────────────

/// 统一两个复合选择器：合并简单选择器，冲突时返回 None。
/// 规则：
/// - Type 不同 → None（a + b → null）
/// - Id 不同 → None（#x + #y → null）
/// - PseudoElement 不同 → None（::before + ::after → null）
/// - Class/PseudoClass/Attribute → 并集去重
/// - Universal + Type → Type（Universal 被收窄）
pub fn unify_compound(a: &CompoundSelector, b: &CompoundSelector) -> Option<CompoundSelector>

/// 统一两个复杂选择器（从右端开始匹配复合选择器）。
/// 返回 None 表示不可统一。
pub fn unify_complex(a: &ComplexSelector, b: &ComplexSelector) -> Option<ComplexSelector>

/// 统一两个选择器列表（笛卡尔积，过滤 None）。
pub fn unify(a: &Selector, b: &Selector) -> Option<Selector>

// ── is_superselector 算法 ────────────────────────────

/// 判断 super 是否是 sub 的超选择器。
/// 规则：
/// - 复合选择器层：super 的 SimpleSelector 集合 ⊆ sub 的集合
///   - 但 PseudoElement 必须相同（或 super 无 PseudoElement）
/// - 复杂选择器层：super 的组合器序列是 sub 的子序列（LCS 匹配）
///   - 每个 super 的复合选择器 is_superselector 对应的 sub 复合选择器
pub fn is_superselector(super_sel: &Selector, sub_sel: &Selector) -> bool

/// 复合选择器级别的超选择器判断。
fn is_super_compound(super_c: &CompoundSelector, sub_c: &CompoundSelector) -> bool

/// 复杂选择器级别的超选择器判断（组合器序列匹配）。
fn is_super_complex(super_c: &ComplexSelector, sub_c: &ComplexSelector) -> bool

// ── extend 算法 ─────────────────────────────────────

/// 扩展选择器：在 selector 中找到匹配 extendee 的部分，
/// 用 extender 替换/追加。
/// 返回新的 Selector（原选择器 + 扩展后的选择器）。
pub fn extend_selector(
    selector: &Selector,
    extendee: &Selector,
    extender: &Selector,
) -> Selector

/// 替换选择器：在 selector 中找到匹配 original 的部分，
/// 用 replacement 替换。
pub fn replace_selector(
    selector: &Selector,
    original: &Selector,
    replacement: &Selector,
) -> Selector
```

## unify 算法详解

```
unify(".c.d", ".e.f")
  → parse → Compound([Class(c), Class(d)]) × Compound([Class(e), Class(f)])
  → 合并 → Compound([Class(c), Class(d), Class(e), Class(f)])
  → serialize → ".c.d.e.f" ✓

unify(".c.d", ".d.e")
  → parse → Compound([Class(c), Class(d)]) × Compound([Class(d), Class(e)])
  → 合并去重 → Compound([Class(c), Class(d), Class(e)])
  → serialize → ".c.d.e" ✓

unify("a", "b")
  → parse → Compound([Type(a)]) × Compound([Type(b)])
  → Type 冲突 → None → null ✓

unify("*", "c")
  → parse → Compound([Universal]) × Compound([Type(c)])
  → Universal 被 Type 收窄 → Compound([Type(c)])
  → serialize → "c" ✓

unify("a.c", "a.d")
  → parse → Compound([Type(a), Class(c)]) × Compound([Type(a), Class(d)])
  → Type 相同 → 合并 → Compound([Type(a), Class(c), Class(d)])
  → serialize → "a.c.d" ✓
```

## is_superselector 算法详解

```
is-superselector("c", "c.d")
  → super=Compound([Type(c)])  sub=Compound([Type(c), Class(d)])
  → {Type(c)} ⊆ {Type(c), Class(d)} → true ✓

is-superselector("c.e", "c:d.e")
  → super=Compound([Type(c), Class(e)])  sub=Compound([Type(c), PseudoClass(d), Class(e)])
  → {Type(c), Class(e)} ⊆ {Type(c), PseudoClass(d), Class(e)} → true ✓

is-superselector("c.d", "c")
  → super=Compound([Type(c), Class(d)])  sub=Compound([Type(c)])
  → {Type(c), Class(d)} ⊈ {Type(c)} → false ✓

is-superselector("::d", "c::d")
  → super=Compound([PseudoElement(d)])  sub=Compound([Type(c), PseudoElement(d)])
  → {PseudoElement(d)} ⊆ {Type(c), PseudoElement(d)} → true ✓

is-superselector("c", "c::d")
  → super=Compound([Type(c)])  sub=Compound([Type(c), PseudoElement(d)])
  → PseudoElement 不匹配 → false ✓
```

## 复杂选择器级别（带组合器）

```
unify("a.x", "b.y")
  → Complex([(None, a.x)]) × Complex([(None, b.y)])
  → 右端复合选择器 unify: unify_compound(a.x, b.y) = None
  → null ✓

unify(".a .b", ".c .d")
  → Complex([(None, .a), (Descendant, .b)]) × Complex([(None, .c), (Descendant, .d)])
  → 右端 unify_compound(.b, .d) = .b.d
  → 左端 unify_compound(.a, .c) = .a.c
  → 结果: .a.c .b.d ✓

is-superselector(".a .b", ".a .x .b")
  → super = [.a, (Desc) .b]
  → sub   = [.a, (Desc) .x, (Desc) .b]
  → super 的组合器序列是 sub 的子序列 → true ✓
```

## @extend 重写设计

```
现有 @extend 流程:
  eval_extend_node → env.add_extend(extender, target, optional, module)
  → 求值完成后 apply_extends(nodes, extends, module_selectors)
  → 字符串 replace

新 @extend 流程:
  eval_extend_node → env.add_extend(extender, target, optional, module)  [不变]
  → 求值完成后 apply_extends(nodes, extends, module_selectors)
  → 对每个 Rule:
    1. parse_selector(selector) → Selector AST
    2. 对每个 (extender, target):
       parse_selector(extender) → Selector
       parse_selector(target) → Selector
       extend_selector(sel, target, extender) → 新 Selector
    3. Selector.to_string() → 新 selector 字符串
```

## 模块文件结构

```
src/css/
├── mod.rs              # Serializer（不变）
├── node.rs             # CssNode（不变）
├── selector.rs         # 现有 sanitize_selector 等（保留）
├── selector_ast.rs     # 新增：AST 类型 + Parser + Display
└── selector_ops.rs     # 新增：unify + is_superselector + extend + replace
```

## 降级策略

- `parse_selector` 遇到无法解析的输入时，返回包含单个 `ComplexSelector` + 单个 `CompoundSelector` + 原始字符串的降级 AST
- `unify_compound` / `is_superselector` 在降级模式下回退到字符串操作（当前行为）
- 保证不引入回归

## Tracing 设计

```rust
#[tracing::instrument(level = "debug", fields(sel = %selector))]
pub fn parse_selector(selector: &str) -> Selector { ... }

#[tracing::instrument(level = "debug", fields(result = tracing::field::Empty))]
pub fn unify_compound(a: &CompoundSelector, b: &CompoundSelector) -> Option<CompoundSelector> {
    // ...
    tracing::Span::current().record("result", ?result);
    result
}

#[tracing::instrument(level = "debug")]
pub fn is_superselector(super_sel: &Selector, sub_sel: &Selector) -> bool { ... }

#[tracing::instrument(level = "info", skip(extends))]
pub fn apply_extends(nodes: Vec<CssNode>, extends: &[...]) -> Vec<CssNode> { ... }
```

## 测试策略

- 所有测试放在 `tests/` 目录（禁止内联测试）
- `tests/selector_ast_test.rs` — AST 解析 + 序列化 round-trip
- `tests/selector_unify_test.rs` — unify 算法（对照 sass-spec 预期值）
- `tests/selector_super_test.rs` — is_superselector 算法
- `tests/selector_extend_test.rs` — extend/replace 算法
- 最终验证：sass-spec `core_functions/selector/` + `directives/extend/`

---

# calc() 简化增强设计

## 失败分析

583 个 calc 失败分布：

| 子类 | 失败数 | 说明 |
|------|--------|------|
| calc/error/known_incompatible | 218 | 不兼容单位错误检测（`deg + s` 应报错） |
| calc/error/其他 | 53 | 语法错误、值错误、操作符错误、空格错误、复杂单位 |
| calc/no_operator | 23 | 无操作符场景：纯数字、括号去除、空白处理 |
| calc/operator | 24 | 运算符优先级、无空白操作符（`1px/2px`）、保留场景 |
| calc/constant | 16 | pi/e 常量在复杂表达式中的替换 |
| calc/simplify | 9 | 混合简化场景 |
| calc/parens | 5 | 括号简化精度 |
| calc/space | 6 | 空格规范化 |
| 其余 calculation 子函数 | 229 | round/rem/mod/sin/cos/tan/abs/sign/exp/pow/sqrt/log/clamp/min/max/hypot 等 |

## 核心问题

当前 `simplify_calc` 是**字符串模式匹配**——逐个尝试 `parse_simple_number` → `try_simplify_same_unit_arith` → `try_simplify_min_max`，无法处理：

1. **运算符优先级**：`calc(1px + 2px * 3)` 当前无法正确解析（先乘后加）
2. **单位兼容性转换**：`calc(1deg + 0.01745rad)` 应简化为 `calc(2deg)`（rad→deg 换算）
3. **不兼容单位错误**：`calc(1deg + 1s)` 应报错 `1deg and 1s are incompatible`
4. **嵌套 CSS 函数**：`calc(1px + min(2px, 3px))` 应简化为 `calc(1px + 2px)` → `3px`
5. **CSS 函数保留**：`calc(1px + var(--c))` 应保留 `calc(1px + var(--c))` 不简化

## calc 表达式 AST 设计

```rust
// src/eval/value/calc_ast.rs

/// calc 表达式 AST 节点
#[derive(Debug, Clone, PartialEq)]
pub enum CalcNode {
    /// 数字 + 单位
    Number(f64, Option<String>),
    /// CSS 常量：pi, e
    Constant(CalcConst),
    /// CSS 变量引用：var(--c) 或 var(--c, fallback)
    Var(String, Option<Box<CalcNode>>),
    /// 二元运算
    BinaryOp {
        op: CalcOp,
        left: Box<CalcNode>,
        right: Box<CalcNode>,
    },
    /// 嵌套 CSS 数学函数
    Func {
        name: String,       // "min", "max", "clamp", "round", ...
        args: Vec<CalcNode>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum CalcConst { Pi, E }

#[derive(Debug, Clone, PartialEq)]
pub enum CalcOp { Add, Sub, Mul, Div }
```

## 函数签名设计

```rust
// ── 解析 ────────────────────────────────────────────

/// 将 calc() 内部表达式字符串解析为 CalcNode AST。
/// 支持运算符优先级（先乘除后加减）、括号嵌套、CSS 函数。
pub fn parse_calc_expr(input: &str) -> Option<CalcNode>

// ── 单位兼容性 ──────────────────────────────────────

/// 检查两个单位是否兼容（可换算）。
/// "deg" vs "rad" → true (1rad = 57.2958deg)
/// "px" vs "cm" → true (1cm = 96px/2.54)
/// "deg" vs "s" → false
pub fn units_compatible(a: &str, b: &str) -> bool

/// 将值从 from_unit 转换为 to_unit。
/// 返回 None 表示不兼容。
pub fn convert_unit(value: f64, from_unit: &str, to_unit: &str) -> Option<f64>

// ── 简化 ────────────────────────────────────────────

/// 递归简化 CalcNode AST。
/// 策略：
/// 1. 递归简化子表达式
/// 2. 常量折叠：纯数字 + 同单位 → 计算
/// 3. 单位转换：兼容单位 → 换算后计算
/// 4. 不兼容单位 → 返回错误标记
/// 5. var()/Func 不可简化 → 保留
pub fn simplify_calc_node(node: &CalcNode) -> Result<CalcNode, CalcError>

// ── 序列化 ────────────────────────────────────────────

impl std::fmt::Display for CalcNode { ... }
```

## 单位兼容性表设计

```rust
// src/eval/value/calc_units.rs

/// CSS 单位兼容性分组——同组内可换算。
/// 组内第一个单位为基准单位。
const UNIT_GROUPS: &[&[&str]] = &[
    // 长度
    &["px", "cm", "mm", "in", "pt", "pc", "q"],
    // 角度
    &["deg", "rad", "grad", "turn"],
    // 时间
    &["s", "ms"],
    // 频率
    &["Hz", "kHz"],
    // 分辨率
    &["dpi", "dpcm", "dppx"],
    // 无单位
    &[""],
];

/// 单位换算因子（相对于组内基准单位）。
/// deg = 1.0, rad = 180/π, grad = 0.9, turn = 360.0
/// px = 1.0, cm = 96/2.54, mm = 96/25.4, in = 96.0, pt = 96/72, pc = 96/6
fn conversion_factor(unit: &str) -> Option<f64>
```

## 不兼容单位错误检测

```rust
/// calc 简化错误
#[derive(Debug, Clone)]
pub enum CalcError {
    /// 不兼容单位：`1deg and 1s are incompatible.`
    IncompatibleUnits(String, String),
    /// 除以零
    DivisionByZero,
    /// 语法错误
    SyntaxError(String),
}
```

`calc(1deg + 1s)` 的处理流程：
1. `parse_calc_expr("1deg + 1s")` → `BinaryOp { op: Add, left: Number(1, "deg"), right: Number(1, "s") }`
2. `simplify_calc_node` 检查 `units_compatible("deg", "s")` → false
3. 返回 `CalcError::IncompatibleUnits("1deg", "1s")`
4. 调用方将错误传播为 `SassError::Eval("1deg and 1s are incompatible.")`

## 简化规则详解

### 同单位加减法
```
calc(1px + 2px) → 3px ✓
calc(1px - 2px) → -1px ✓
```

### 兼容单位转换
```
calc(1deg + 0.017453rad) → calc(1deg + 0.999986deg) → calc(1.999986deg) → 2deg
calc(1cm + 10mm) → calc(1cm + 1cm) → 2cm  (或 10mm + 10mm → 20mm，取决于保留哪个单位)
```

### 不兼容单位保留
```
calc(1px + 2%) → calc(1px + 2%)  (保留，% 不是长度单位)
calc(1px + var(--c)) → calc(1px + var(--c))  (保留)
```

### 运算符优先级
```
calc(1px + 2px * 3) → calc(1px + 6px) → 7px  (先乘后加)
calc(2px * 3 + 1px) → calc(6px + 1px) → 7px
calc((1px + 2px) * 3) → calc(3px * 3) → 9px
```

### 嵌套函数简化
```
calc(1px + min(2px, 3px)) → calc(1px + 2px) → 3px
calc(max(1px, 2px) + min(3px, 4px)) → calc(2px + 3px) → 5px
calc(1px + clamp(0, 2px, 5px)) → calc(1px + 2px) → 3px
```

### 不可简化时保留
```
calc(1px + var(--c)) → calc(1px + var(--c))  (var 不可简化)
calc(1px + 2% * var(--c)) → calc(1px + 2% * var(--c))  (保留，但可去括号)
```

### CSS 数学函数简化
```
round(10px, 3px) → 9px  (round to nearest multiple)
rem(10px, 3px) → 1px
mod(10px, 3px) → 1px
abs(-5px) → 5px
sign(-5) → -1
```

## 模块文件结构

```
src/eval/value/
├── mod.rs              # 现有 eval_value（不变）
├── calc.rs             # 重写：入口 simplify_calc() 调用新 AST
├── calc_ast.rs         # 新增：CalcNode 类型 + parser + Display
├── calc_units.rs      # 新增：单位兼容性表 + 转换
├── display.rs          # 现有（不变）
├── ops.rs              # 现有（不变）
└── partial.rs          # 现有（不变）
```

## Tracing 设计

```rust
#[tracing::instrument(level = "debug", fields(expr = %input))]
pub fn parse_calc_expr(input: &str) -> Option<CalcNode> { ... }

#[tracing::instrument(level = "debug", fields(result = tracing::field::Empty))]
pub fn simplify_calc_node(node: &CalcNode) -> Result<CalcNode, CalcError> { ... }

#[tracing::instrument(level = "trace", fields(a = %a, b = %b))]
pub fn units_compatible(a: &str, b: &str) -> bool { ... }
```

## 测试策略

- `tests/calc_ast_test.rs` — calc 表达式 AST 解析 + 序列化 round-trip
- `tests/calc_units_test.rs` — 单位兼容性 + 转换
- `tests/calc_simplify_test.rs` — 简化规则（对照 sass-spec 预期值）
- 最终验证：sass-spec `values/calculation/` 全量
