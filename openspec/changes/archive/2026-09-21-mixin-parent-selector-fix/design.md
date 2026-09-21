## Context

### 问题本质

5 次局部修复尝试全部失败，根因是 `Env.current_selector: Option<String>` 作为**扁平选择器字符串**，无法表达 SCSS 中 `&` 的词法作用域语义：

```
.el-popper {                      // frame[0]: ".el-popper"
  $arrow-selector: #{& + '__arrow'};  // & 应展开为 ".el-popper"
  @include when(dark) {           // frame[1]: ".el-popper.is-dark"
    > #{$arrow-selector}::before  // $arrow-select器 = ".el-popper__arrow"（定义时展开）
  }                               // 期望: ".el-popper.is-dark > .el-popper__arrow::before"
}
```

当前架构下 `$arrow-selector` 存储为字面 `"&__arrow"`（因为 `&` 不在 `Value::Interp` 求值时展开）。当 `@at-root { &.is-dark { > &__arrow::before } }` 处理时，`&` 被展开为 frame[1] 的 `".el-popper.is-dark"`，得到 `> .el-popper.is-dark__arrow::before`——**错误！**

### 扁平字符串 vs 帧栈对比

| 场景 | 扁平字符串（当前） | 帧栈（重构后） |
|------|-------------------|---------------|
| `$sel: &` 在 `.a{}` 内 | current_selector=".a" → 存字面"&" | frame top=".a" → 存字面"&" |
| `#{$sel}` 在 `.b{}` 内使用 | current_selector=".b" → &展开为".b" ❌ | frame top=".b" 但 $sel 捕获了 frame[".a"] → &展开为".a" ✓ |
| mixin `@include` | mixin 内 current_selector 不变 | mixin 内 push 新 frame，@content 继承调用者 frame |
| `@at-root` | current_selector 不变（hoc） | 标记 is_at_root，& 展开跳过 @at-root 帧 |

### BEM Mixin 模式分析

EP 的 BEM mixin 链：
```
b(avatar) → .el-avatar { @content }
  @content 包含: > img {...}, @include m(circle){...}, ...
    m(circle):
      $selector = &           // 期望捕获 ".el-avatar"
      @each: $currentSelector = & + "--" + "circle" + ","
      @at-root { #{$currentSelector} { @content } }
```

关键要求：
1. `$selector: &` 赋值时**不展开**（存字面 `"&"`）
2. `#{...}` 表达式求值时，`&` 按**当前帧**展开
3. mixin body 内 `@each` 产生的 `Value::Interp` 中的 `&` 展开为**调用者帧**（而非 mixin 内部帧）
4. `@at-root` 内的 `&` 展开为 `@at-root` **调用者**帧

## Goals / Non-Goals

**Goals:**
- 引入 SelectorFrameStack 替代 `current_selector: Option<String>`，支持 `&` 的词法作用域展开
- mixin `m()`/`e()`/`when()` 模式的 `&` 展开与 EP dist 100% 一致
- `@at-root` 内的 `&` 正确展开为调用者上下文
- `> img` 等 child combinator 在 mixin 调用链中不丢失父选择器

**Non-Goals:**
- 不改变 CSS selector AST（`SimpleSelector::ParentReference` 不变）
- 不改变 `combine_selectors` 函数（仍用字符串替换 `&`）
- 不修改 SCSS parser（`&` 的解析逻辑不变）
- 不引入新的 CLI 或 public API

## Decisions

### Decision 1：SelectorFrameStack 数据结构

```rust
// src/eval/env.rs
#[derive(Debug, Clone)]
pub(crate) struct SelectorFrame {
    /// 此帧的完整 flat selector（combined parent + raw）
    pub rendered: String,
    /// 原始选择器（未与 parent 组合）
    pub raw: String,
    /// 父帧在栈中的索引
    pub parent_idx: Option<usize>,
    /// 是否由 @at-root 创建
    pub at_root: bool,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct SelectorFrameStack {
    frames: Vec<SelectorFrame>,
}

impl SelectorFrameStack {
    /// 创建空栈（顶层，无选择器）
    pub fn new() -> Self { Self { frames: vec![] } }
    
    /// 推入新帧（规则嵌套时调用）
    pub fn push(&mut self, raw: String) -> usize { ... }
    
    /// 弹出栈顶
    pub fn pop(&mut self) { ... }
    
    /// 获取栈顶帧的 rendered selector
    pub fn current(&self) -> Option<&str> { ... }
    
    /// 查找最近的非 @at-root 帧（用于 @at-root 内 & 展开）
    pub fn nearest_non_at_root(&self) -> Option<&str> { ... }
    
    /// 渲染完整栈路径（用于调试）
    pub fn full_path(&self) -> String { ... }
}
```

**字段说明**：
- `rendered`: 组合后的选择器，如 `".el-popper.is-dark > .el-popper__arrow"`
- `raw`: 当前层级的未组合选择器，如 `"&.is-dark"` 或 `"> &__arrow"`
- `parent_idx`: 栈中父帧索引（避免 Rc 开销）
- `at_root`: 标记调用者是否在 `@at-root` 内

### Decision 2：变量中的 `&` 捕获机制

```rust
// src/eval/value/mod.rs — Value::Interp 求值
Value::Interp(segments) => {
    let s = eval_interp_segments(segments, env);
    // EP FIX: Value::Interp 结果中的 & 按定义时帧展开
    let s = if s.contains('&') {
        // 查找 & 对应的定义帧
        // 如果 & 来自变量（如 $selector），展开为变量定义时的帧
        // 如果 & 直接出现在 #{} 中，展开为当前帧
        s  // 见 Decision 3
    } else { s };
    Ok(Value::String(s, false))
}
```

**延迟展开方案**：不立即展开，而是将 `&` 标记为 `Value::ParentRef(frame_idx)`，存储在变量值中。

```rust
// 新增 Value 变体
pub enum Value {
    ...
    /// 表达式中的 & 父选择器引用（携带定义帧索引）
    ParentRef(usize),  // 帧栈索引
    ...
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Value::ParentRef(_) => write!(f, "&"),
            ...
        }
    }
}
```

**变量赋值时的处理**：
```rust
// src/eval/value/mod.rs
pub(crate) fn expand_value_parent_refs(val: Value, stack: &SelectorFrameStack) -> Value {
    match val {
        Value::ParentRef(frame_idx) => {
            stack.get(frame_idx)
                .map(|f| Value::String(f.rendered.clone(), false))
                .unwrap_or(val)
        }
        Value::String(s, q) if s.contains('&') => {
            // 混合字符串："{prefix}&{suffix}" → 展开 & 部分
            if let Some(current) = stack.current() {
                Value::String(s.replace('&', current), q)
            } else {
                Value::String(s, q)
            }
        }
        Value::Interp(segments) => {
            // 递归展开 segments 中的 ParentRef
            let expanded: Vec<InterpSegment> = segments.into_iter().map(|seg| {
                match seg {
                    InterpSegment::Expr(expr) => InterpSegment::Expr(expr),  // Expr 需要 re-eval
                    InterpSegment::Text(t) => InterpSegment::Text(t),
                }
            }).collect();
            Value::Interp(expanded)
        }
        other => other,
    }
}
```

### Decision 3：mixin 帧传播规则

```
Mixin Call Stack:
  content_env (caller)  ← 捕获调用者帧栈
  mixin_env (callee)    ← 继承 content_env 的帧栈

Mixin Body Evaluation:
  $selector: &;         ← ParentRef(current_top_frame_idx)
  
  @each $unit in $mod:
    $currentSelector: #{$selector + ...}  
    ← 求值时 ParentRef 展开为帧栈对应 rendered
    
  @at-root {            ← 推入新帧 (at_root=true)
    #{$currentSelector}  ← 使用当前帧栈渲染
  }
```

### Decision 4：@at-root 语义

```rust
// eval_at_root 实现
pub(crate) fn eval_at_root(body: &[Node], env: Env) -> Result<...> {
    // 推入 @at-root 帧
    let mut stack = env.selector_frames.clone();
    stack.push(SelectorFrame {
        rendered: String::new(),  // @at-root 无自身选择器
        raw: String::new(),
        parent_idx: stack.top_idx(),
        at_root: true,
    });
    let env = env.with_selector_frames(stack);
    let (css, new_env) = eval_nodes(body, env)?;
    Ok((vec![CssNode::AtRoot(css, query)], new_env))
}
```

## Risks / Trade-offs

| 风险 | 缓解措施 |
|------|---------|
| SelectorFrameStack 性能开销 | 使用 `usize` 索引而非 `Rc`，栈帧复用 |
| 约 30+ 处 `get_selector()`/`with_selector()` 调用点需要迁移 | 提供兼容 API：`get_selector()` → `stack.current().map(String::from)` |
| mixin 帧传播规则复杂 | 通过 EP trace test 逐文件验证 |
| 帧索引失效（栈 pop 后引用悬挂） | 使用 Weak 模式：ParentRef 存储 (frame_id, generation) |
| 混合字符串展开错误 | 仅展开 `&` 字符（CSS 中不可能在 identifier 中出现裸 `&`） |

## Migration Plan

**Step 1**: 新增 `SelectorFrameStack` 结构和 API（不影响现有行为）
**Step 2**: Env 增加 `selector_frames` 字段，`current_selector` 改为 deprecated alias
**Step 3**: 逐步迁移调用点：eval_rule → push/pop frame, eval_mixin → clone frame stack
**Step 4**: 新增 `Value::ParentRef`，parser/literals.rs Amp → ParentRef
**Step 5**: `eval_value::Value::Interp` 内部使用 ParentRef-aware 展开
**Step 6**: `eval_variable` 中使用 `expand_value_parent_refs`
**Step 7**: 通过 core tests + ep_full + EP 对比验证
