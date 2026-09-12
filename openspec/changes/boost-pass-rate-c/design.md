## Context

sasspile 的 sass-spec 通过率因回归从 63.4% 降至 48.86%。直接根因（`fmt_color_fn` 格式错误）已在 previous phase 修复。

更深层的结构性问题：**CSS 和 SCSS 共用一套 AST（`Node` 枚举）和统一的 Evaluator**，导致：
- CSS 文件求值时散布 `if env.plain_css()` 检查，易遗漏
- CSS 新增特性（CSS Color 5、nesting level 2）必须修改共享枚举，SCSS 测试被拖累
- SCSS 同理，无法独立演化

**核心洞察**：CSS 和 SCSS 是两种独立语言，有各自版本线（CSS4→CSS5..., SCSS3→SCSS4...）。它们应该有独立的 AST、独立的 Evaluator、独立的语义边界。

## Goals / Non-Goals

**Goals:**
- 建立双 AST 架构：`ScssNode` / `CssNode` 独立枚举
- SCSS 和 CSS 各自独立演化，互不干扰
- 编译期保证 CSS 路径不会执行 Sass 逻辑
- 当前 sass-spec 通过率不降低（>48.86%）
- 为后续 CSS5/SCSS4 新特性预留独立扩展点

**Non-Goals:**
- 不在此变更中修复 sass-spec 具体 case bug（留作后续 Phases）
- 不改动 Serializer 核心逻辑（已能处理两种结构）
- 不引入新的外部依赖

## Decisions

### Decision 1: 双 AST 而非单 AST + mode 检查

**问题**: 单 AST + `match env.mode()` 方案中，mode 检查遗漏导致 bug
**决策**: 两套独立 AST 类型，编译期隔离
**依据**: Rust 类型系统可保证 `CssEvaluator` 代码不可能处理 `ScssNode::Variable`（不同类型）
**替代方案**: 保留 mode 检查 + 增加 lint — 被排除，运行时检查无法覆盖所有路径

### Decision 2: 扩展名驱动模式分派

**问题**: 如何确定一个文件走 SCSS 还是 CSS 管线
**决策**: 文件加载时检查扩展名 — `.scss`/`.sass` → SCSS, `.css` → CSS
**依据**: 这是 sass-spec 的约定，也是原生 Sass 的行为
**替代方案**: 文件内容嗅探 — 被排除，含糊且不可靠

### Decision 3: 共享 CssNode 输出类型

**问题**: 两套 Evaluator 产出什么类型
**决策**: 共享 `CssNode` 作为 Evaluator 输出、Serializer 输入
**依据**:
- SCSS 展平和 CSS 嵌套最终都需要表示为 `CssNode::Rule { declarations, children }`
- `children: []` = SCSS 产物, `children: [...]` = CSS 产物
- Serializer 按结构输出（无需 mode 参数）

### Decision 4: CSS @import 透传不展开

**问题**: CSS 文件中 `@import "x.css"` 应如何处理
**决策**: 输出 `CssNode::AtImport`，Serializer 序列化为 `@import "x.css";`
**依据**: CSS 原生 `@import` 是 at-rule，不应被嵌套组合
**对比**: SCSS 模式下 `@import` 展开内联 + 可能嵌套组合（现有行为不变）

### Decision 5: CSS 嵌套 @media 不提升

**问题**: CSS 文件中 `a { @media ... }` 的 @media 是否提升到外层
**决策**: 保留在 `a {}` 的 children 中（不提升）
**依据**: CSS Nesting 规范，@media 可嵌套在 style rule 内

## Architecture

```
                        ┌─────────────────────────────────────────────────┐
                        │              完全隔离的管线                      │
                        │                                                 │
   .scss / .sass         │    .css                                         │
       │                 │      │                                          │
       ▼                 │      ▼                                          │
   ScssLexer             │   CssLexer                                      │
       │                 │      │                                          │
       ▼                 │      ▼                                          │
   ScssParser            │   CssParser                                     │
       │                 │      │                                          │
       ▼                 │      ▼                                          │
   Ast<ScssNode>         │   Ast<CssNode>                                  │
       │                 │      │                                          │
       ▼                 │      ▼                                          │
   ScssEvaluator         │   CssEvaluator                                  │
       │                 │      │                                          │
       │  展平 + 组合     │      │  保留嵌套                                 │
       │  内建函数       │      │  禁止 Sass                                │
       │  模块系统       │      │  @import 透传                             │
       ▼                 │      ▼                                          │
   Vec<CssNode>          │   Vec<CssNode>                                  │
       │                 │      │                                          │
       └────────┬────────┴──────┘                                          │
                ▼                                                           │
         统一 Serializer（无 mode）                                         │
                │                                                           │
                ▼                                                           │
             CSS 输出                                                       │
                                                                       ────┘
```

## 类型定义概要

```rust
// ─── SCSS AST（完整 Sass 特性）───
enum ScssNode {
    Rule { selector: String, body: Vec<ScssNode> },
    Decl { property: String, value: ScssValue, important: bool },
    Variable { name: String, value: ScssValue, flags: VarFlags },
    If { branches: Vec<(ScssValue, Vec<ScssNode>)>, else_body: Vec<ScssNode> },
    For { ... },
    Each { ... },
    While { ... },
    MixinDef { ... },
    FunctionDef { ... },
    Include { ... }, Return(...), Content,
    Use { ... }, Forward { ... }, Import { ... },
    Extend { ... }, AtRoot { ... },
    AtRule { ... },
    Warn(...), Debug(...), Error(...),
    Comment(String, bool),
}

// ─── CSS AST（仅原生 CSS + Nesting）───
enum CssNode {
    Rule { selector: String, body: Vec<CssNode> },
    Decl { property: String, value: CssValue, important: bool },
    Comment(String),
    AtRule { name: String, params: Option<String>, body: Option<Vec<CssNode>> },
    Import { url: String, modifier: Option<String> },
    // 无 Variable、If、For、MixinDef、FunctionDef 等
}

// ─── 共享输出（序列化输入）───
enum CssNode {
    Rule { selector: String, declarations: Vec<CssNode>, children: Vec<CssNode> },
    Declaration { property: String, value: String, important: bool },
    Comment(String),
    AtRule { name: String, params: Option<String>, children: Vec<CssNode>, has_body: bool },
    AtImport { url: String, modifier: Option<String> },
    Raw(String),
}
```

## 文件结构变更

### 新增文件
| 文件 | 说明 |
|------|------|
| `src/parse/scss_ast.rs` | ScssAst + ScssNode 定义 |
| `src/parse/css_ast.rs` | CssAst + CssNode 定义 |
| `src/parse/css_parser.rs` | CssParser |
| `src/eval/scss_evaluator.rs` | ScssEvaluator（现有代码迁移） |
| `src/eval/scss_env.rs` | ScssEnv（现有 Env 重命名） |
| `src/eval/css_evaluator.rs` | CssEvaluator（新增 ~100 行） |

### 修改文件
| 文件 | 改动 |
|------|------|
| `src/parse/mod.rs` | 产出 Parsed 枚举 |
| `src/parse/scss_parser.rs` | 现有 Parser 迁移（重命名） |
| `src/css/node.rs` | CssNode 重构为共享输出类型 |
| `src/css/serialize.rs` | 适配新 CssNode |
| `src/eval/reactor.rs` | 管线分叉选择 Evaluator |

### 可删除代码
| 代码 | 原因 |
|------|------|
| `src/eval/plain_css.rs` | 类型系统已保证 |
| `Env.plain_css` 字段 | 不再需要 |
| 所有 `is_plain_css()` 调用 | 编译期替代运行时 |

## Migration Plan

### Phase 1: 类型定义 + 双 Parser（2 天）
- 新建 scss_ast.rs + css_ast.rs
- 新建 CssParser（在 ScssParser 基础上限制 Sass 语法）
- Parser 入口分派为 Parsed 枚举

### Phase 2: SCSS Evaluator 迁移（2 天）
- 现有 eval/ 模块迁移到 ScssEvaluator
- 重命名 Env → ScssEnv
- Reactor 在 Scss 路径调用 ScssEvaluator

### Phase 3: CSS Evaluator 新建（1 天）
- 新建 CssEvaluator（~100 行）
- eval_rule 保留嵌套
- eval_import 透传 at-rule
- Reactor 在 Css 路径调用 CssEvaluator

### Phase 4: 清理旧代码（1 天）
- 删除 plain_css.rs
- 删除所有 is_plain_css() 检查
- 删除 Env.plain_config 字段

## Risks / Trade-offs

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|------------|
| 两套 AST 初始代码量大 | 高 | 6 天工作量 | 分 Phase 渐进式迁移 |
| CSS Parser 遗漏 Sass 语法拒绝 | 中 | CSS 文件解析出 SCSS 节点 | reject_sass_syntax + 测试覆盖 |
| SCSS Evaluator 迁移遗漏 | 中 | 部分 SCSS 行为改变 | 全量测试 + spec 对比 |
| 修 CSS 期间新特性无法及时同步 | 低 | CSS5 支持延迟 | 类型独立 = 可稍后补 |

## Long-term Maintenance Benefits

| 场景 | 单 AST | 双 AST |
|------|--------|--------|
| CSS Color 5 新特性 | 改 Node → SCSS 测试重跑 | 改 CssNode → 不影响 SCSS |
| SCSS 4.0 新语法 | 改 Node → CSS 测试重跑 | 改 ScssNode → 不影响 CSS |
| 修 CSS nesting bug | 担心影响 SCSS 展平 | 改 CssEvaluator → SCSS 无风险 |
| 新人 onboarding | 理解两种语义 + mode 检查 | 选一个领域 → 读对应 enum |

## Open Questions

1. SCSS 文件中 `@use`/`@import` 一个 `.css` 文件时，子文件的 CSS 结果如何与 SCSS 上下文混合？
   → 初步方案：子文件独立求值，返回 CssNode 后由父 SCSS 上下文包装
2. 现有 `css/serialize.rs` 的 `flatten_nodes` 是否继续保留用于 SCSS 路径？
   → 是：SCSS 路径仍需展平（现有逻辑），CSS 路径直接输出

