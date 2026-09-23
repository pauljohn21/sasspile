## Design: main → rx 能力迁移架构

### 0. 核心原则（不可违反）

> rx 不是 main 的移植，是以 rxrust 响应式约束重新实现。

```
铁律:
1. scan_map (CompileState, dispatch_pass)      ← 唯一状态修改窗口
2. flat_map(Shared::from_iter(...))            ← 1→N 展开的唯一手段
3. collect::<T>() + last() + subscribe         ← 唯一收集出口
4. 纯函数 helper                               ← 离开 scan_map 闭包后禁止副作用
5. 单文件 ≤ 500 行                             ← 强制模块化
```

---

### 1. 当前 rx 管线（v0.1 — 已完成）

```
Shared::from_stream(futures::stream::iter(input.lines()))
  .scan_map(CompileState::new(), dispatch_pass)
      dispatch_pass 内:
        ── 状态机 (Collecting 4 态):
           None ─→ (@for/$each/$if/$mixin 开启) ─→ For/Each/If/MixinDef
           For/Each/If/MixinDef ─→ (遇到 }) ─→ None (finalize_collecting)
        ── 非收集态分发:
           @each (含 { 不含 }) ─→ 多行收集
           @each (含 { 含 })  ─→ 单行立即展开
           @for   同 @each
           @mixin (含 { })   ─→ 单行立即 / 多行收集
           @include           ─→ expand_mixin (fold 参数)
           @if/@use/@forward   ─→ 消费
           selector {         ─→ selector_stack 拼接, emit 完整选择器
           }                  ─→ selector_stack.pop, emit }
           _                  ─→ 透传 (vec![token])
  .flat_map(|v: Vec<String>| Shared::from_stream(futures::stream::iter(v)))
  .collect::<Vec<String>>()
  .last()
  .subscribe(|css| tx_opt.take().map(|s| s.send(css.join("\n"))))
→ format_css(&rx.await.unwrap_or_default())
→ String
```

---

### 2. 目标 rx 管线（v1.0 — 迁移后）

新增 **CSS AST 构建阶段** + **extend/hoisting 阶段** + **函数求值阶段**：

```
Phase 1: StyleLine stream
═══════════════════════════════════════════════════════════════════
input.lines()
  → Shared::from_stream
  .scan_map(CompileState::new(), line_dispatch)
      ─ 同 v0.1 嵌套选择器展开 + selector_stack
      ─ + CSS 指令 (@Media/@Keyframes/@supports) 状态收集
      ─ + @extend 暂存到 extends_queued 列表
      ─ + 字面量 token 透传 (declaration/rule-header/rule-closing)
  .flat_map(from_iter)
  .collect::<Vec<String>>().last()

Phase 2: Post-process directives
═══════════════════════════════════════════════════════════════════
(apply_extends  — 非必须,可 inline 进 dispatch 或作为独立算子)
(var resolution — 已完成 variable lookup in state.scope.variables)

Phase 3: CSS AST Construction (新增)
═══════════════════════════════════════════════════════════════════
StyleLine stream → scan_map(CssBuilder::new(), css_builder_feed)
  CssBuilder.feed(line):
    跟踪 current_selector, depth
    "{":  push RuleNode { selector, children: vec![] }, current_selector = None
    "}":  pop RuleNode, 附加到 parent's children 或 emit top-level
    "color: red":  push DeclarationNode { prop, value }
    "@media ...":  push AtRuleNode { query, children: vec![] }
  emits: Vec<CssNode>

Phase 4: Serialize (改造 formatCss 为 AST→String)
═══════════════════════════════════════════════════════════════════
scan_map + flat_map 后转换为 CssNode stream
  .flat_map(|css_node: CssNode| from_iter(render_node(css_node)))
  .collect::<String>()
  .last()
  → String (formatted CSS)
```

---

### 3. 能力 #1: css-ast-construction

#### 数据结构

```rust
// src/css/node.rs
#[derive(Clone, Debug)]
pub enum CssNode {
    Rule {
        selector: String,           // 已展平的完全选择器
        children: Vec<CssNode>,     // body 子节点
    },
    Declaration {
        property: String,
        value: String,
    },
    AtRoot {
        // @at-root 包装: 其 children 提升到顶层
        children: Vec<CssNode>,
    },
    AtRule {
        query: String,              // @media / @keyframes / ...
        children: Vec<CssNode>,
    },
    Comment(String),
}
```

#### scan_map builder

```rust
// src/css/builder.rs
pub struct CssBuilder {
    pub stack: Vec<CssNode>,        // Rule 嵌套栈
    pub output: Vec<CssNode>,       // top-level 累积
}

impl CssBuilder {
    pub fn feed(&mut self, line: &str) -> Vec<CssNode> {
        // 处理 depth 不匹配的 Rule 弹出 + 新 Rule/Declaration/Media 创建
    }
}
```

#### 测试：builder 独立可测

```rust
let style_lines = vec![".parent {", "  color: red;", "}"].into_iter();
let nodes: Vec<CssNode> = style_lines
    .scan_map(CssBuilder::new(), |b, line| b.feed(line))
    .flat_map(from_iter)
    .collect::<Vec<_>>().last().subscribe(...)
// assert nodes == vec![Rule { selector: ".parent", children: [Declaration("color", "red")] }]
```

---

### 4. 能力 #2: selector-extend

#### 数据结构

```rust
// src/css/selector.rs
pub struct ExtendTarget {
    pub selector: String,       // 被扩展的选择器
    pub extenders: Vec<String>, // 添加此 selector 的源选择器
}
```

#### 状态机集成

CompileState 新增字段：

```rust
pub struct CompileState {
    // ... existing ...
    pub extends_queued: Vec<(String, String)>, // (extender, target)
    pub placeholders_extending: Vec<(String, Vec<String>)>, // (placeholder, extenders)
}
```

rx 响应式处理策略：

1. **收集阶段** (`Collecting::ExtendDef`): 遇到 `%placeholder { ... }`，收集其 body
2. **暂存阶段**: `@extend %placeholder` 解析后暂存到 `extends_queued`
3. **应用阶段**: `finalize_collecting`（规则闭合）时或最终 resolve pass 合并规则

关键：**extend 不立即消费规则，等 top-level rules 收集完毕后再统一应用**。这与 `CssBuilder` 的输出交互。

---

### 5. 能力 #3: function-call-eval

#### 架构

```rust
// src/eval/builtin.rs
use rxrust::prelude::*;
// 纯函数 map：调用参数 → 返回值
type BuiltinFn = fn(&[String]) -> Option<String>;

// 注册
pub const BUILTINS: &[(&str, BuiltinFn)] = &[
    ("lighten", color::lighten),
    ("darken", color::darken),
    ("rgba", color::rgba),
    ("round", math::round),
    ("nth", list::nth),
    ...
];
```

#### 集成到 dispatch_pass

```rust
t if is_function_call(t) => {
    // 例: "background: lighten($color, 10%);"
    match try_eval_builtin(t, state) {
        Some(resolved) => vec![resolved],      // "background: #fff;"
        None => vec![token],                    // 无法求值则原样透传
    }
}
```

`try_eval_builtin` 闭包参数来自 `CompileState` 的 `scope.variables` 查询。

---

### 6. 能力 #7: at-root-hoisting

#### CssNode::AtRoot 特殊处理

```rust
// CssBuilder.feed 遇到 CssNode::AtRoot 时:
// 不推入 stack，直接展开其 children 到 output
let at_root_hoist = |node: &CssNode| {
    match node {
        CssNode::AtRoot { children } => children.clone(),
        _ => vec![node.clone()],
    }
};
```

---

### 7. 文件拆分（避免单文件 > 500）

```
src/directive/
├── pipeline.rs (约 150 行)
│   use super::{state, finalize, selector_stack, format};
│   pub fn compile_pipeline(input: &str) -> String { ... }
│   fn dispatch_pass(state: &mut CompileState, token: String) -> Vec<String> { ... }
│
├── state.rs (约 170 行) ✅ 已有
│   pub struct CompileState { ... }
│
├── finalize.rs (约 130 行)
│   pub fn finalize_collecting(state: &mut CompileState) -> Vec<String> { ... }
│
├── selector_stack.rs (约 80 行)
│   pub fn resolve_nested_selector(parent: Option<&str>, selector: &str) -> String { ... }
│
└── format.rs (约 80 行)
    pub fn format_css(raw: &str) -> String { ... }
```

---

### 8. 整合点

| 整合点 | 主文件 | 辅助函数 |
|--------|--------|----------|
| 指令展开 | `dispatch_pass` | `expand_mixin`, `parse_*_sig` |
| 嵌套选择器 | `dispatch_pass` | `resolve_nested_selector` |
| 函数求值 | `dispatch_pass`（或 pre-process） | `try_eval_builtin` |
| Extend 暂存 | `dispatch_pass` (ExtendsDef 收集态) | `apply_extends` |
| CSS 输出 | `format_css` (或 CssNode→String) | `render_node` |
| Module 加载 | `Collection::Use` 阶段 | `load_module` |
