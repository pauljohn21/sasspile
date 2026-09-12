# Direction A: Dual AST 完全隔离架构设计

> **SCSS 和 CSS 是两种独立语言，有各自的版本线和演化节奏。它们应该有各自独立的 AST、独立的 Evaluator、独立的语义边界。**

## 0. 设计哲学：为什么必须完全隔离

### 0.1 版本独立演化

```
时间线 ──────────────────────────────────────────────────→

CSS 项目:     CSS3    CSS4 (Color Level 4)    CSS5 (??)    CSS6 (??)
                                    ↓                ↓
                              nesting lvl1     nesting lvl2
                              color-mix()      color-level5
                              @scope           @scope lvl2
                                   ↓                ↓
SCSS 项目:    SCSS 3.x         SCSS 3.x++        SCSS 4.x (??)
                                    ↓                ↓
                              module system      new control flow
                  @use/@forward    meta.*              ...
```

**关键事实**：
- CSS 由 W3C/CSSWG 维护，版本节奏独立
- SCSS 由 Sass 团队维护，版本节奏独立
- 两者的新特性互不依赖，**强耦合 = 互相拖累**

### 0.2 单 AST 的根本缺陷

```rust
// 单 AST 方案 — 每次新增 CSS 特性都要碰 SCSS 代码
match node {
    Node::CssNewFeature(...) => { /* CSS 5 新功能 */ }
    Node::ScssIf(...) => { /* SCSS 控制流 */ }
    // 问题：两个领域混在一个 enum 里，reviewer 必须同时理解两者
}
```

**症状**：
- CSS 出新特性 → 必须修改共享 `Node` 枚举 → SCSS 测试被迫重跑
- SCSS 加新语法 → CSS 分支（即使不相关）受影响
- `match env.mode()` 散落各处 — 漏一个就是 bug

### 0.3 双 AST 的解决思路

```rust
// SCSS 管 SCSS，CSS 管 CSS — 编译期保证不串门
enum ScssNode { /* 完整 Sass 特性 */ }
enum CssNode { /* 仅原生 CSS + nesting */ }

// Cr5 新功能只改 CssNode
// SCSS 4.0 新语法只改 ScssNode
// 两者完全独立演化，互不干扰
```

---

## 1. 架构总览

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

---

## 2. 两套 AST 定义

### 2.1 共享输出类型

```rust
/// 共享 CssNode — Evaluator 的输出，Serializer 的输入。
/// 
/// SCSS 和 CSS Evaluator 都产出这个类型，因此 Serializer 无需知道来源。
#[derive(Debug, Clone, PartialEq)]
pub enum CssNode {
    /// CSS 规则：`selector { declarations...  nested... }`
    Rule {
        selector: String,
        declarations: Vec<CssNode>,
        children: Vec<CssNode>,
    },
    /// CSS 声明：`property: value;`
    Declaration {
        property: String,
        value: String,
        important: bool,
    },
    /// 注释
    Comment(String),
    /// @at-rule with body
    AtRule {
        name: String,
        params: Option<String>,
        children: Vec<CssNode>,
        has_body: bool,
    },
    /// @import（无 body）
    AtImport {
        url: String,
        modifier: Option<String>,
    },
    /// 原始 CSS 文本
    Raw(String),
}
```

### 2.2 SCSS AST（完整 Sass 特性）

```rust
/// SCSS 语法树——.scss / .sass 文件解析产出。
/// 
/// 包含完整 Sass 特性：变量、控制流、mixin、函数、模块系统等。
pub struct ScssAst {
    pub nodes: Vec<ScssNode>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScssNode {
    // ── CSS 共有结构 ──
    Rule {
        selector: String,
        body: Vec<ScssNode>,
    },
    Decl {
        property: String,
        value: ScssValue,
        important: bool,
    },
    Comment(String, bool),  // (text, is_silent)

    // ── SCSS 特有：变量与表达式 ──
    Variable {
        name: String,
        value: ScssValue,
        flags: VarFlags,
    },

    // ── SCSS 特有：控制流 ──
    If {
        branches: Vec<(ScssValue, Vec<ScssNode>)>,
        else_body: Vec<ScssNode>,
    },
    For {
        var: String,
        from: ScssValue,
        to: ScssValue,
        inclusive: bool,
        body: Vec<ScssNode>,
    },
    Each {
        vars: Vec<String>,
        list: ScssValue,
        body: Vec<ScssNode>,
    },
    While {
        cond: ScssValue,
        body: Vec<ScssNode>,
    },

    // ── SCSS 特有：Mixin & Function ──
    MixinDef {
        name: String,
        params: Vec<ScssParam>,
        body: Vec<ScssNode>,
    },
    FunctionDef {
        name: String,
        params: Vec<ScssParam>,
        body: Vec<ScssNode>,
    },
    Include {
        name: String,
        args: Vec<ScssArg>,
        content: Option<Vec<ScssNode>>,
    },
    Return(ScssValue),
    Content,

    // ── SCSS 特有：模块系统 ──
    Use {
        url: String,
        namespace: String,
        star: bool,
        config: Vec<ScssConfigVar>,
    },
    Forward {
        url: String,
        show: Vec<String>,
        hide: Vec<String>,
        prefix: String,
        config: Vec<ScssConfigVar>,
    },
    Import {
        url: String,
        modifier: String,
    },

    // ── SCSS 特有：@extend & @at-root ──
    Extend {
        selector: String,
        optional: bool,
    },
    AtRoot {
        query: Option<String>,
        body: Vec<ScssNode>,
    },

    // ── at-rules（语义化） ──
    AtRule {
        name: String,
        params: Option<String>,
        body: Option<Vec<ScssNode>>,
    },

    // ── SCSS 特有：调试 ──
    Warn(ScssValue),
    Debug(ScssValue),
    Error(ScssValue),
}
```

### 2.3 CSS AST（仅原生 CSS）

```rust
/// CSS 语法树——.css 文件解析产出。
/// 
/// 仅包含原生 CSS 特性 + CSS Nesting。
/// 无变量、无控制流、无 mixin、无函数、无 @extend、无 @at-root。
pub struct CssAst {
    pub nodes: Vec<CssNode>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CssNode {
    // ── CSS 原生结构 ──
    Rule {
        selector: String,
        body: Vec<CssNode>,
    },
    Decl {
        property: String,
        value: CssValue,
        important: bool,
    },
    Comment(String),

    // ── CSS at-rules ──
    AtRule {
        name: String,
        params: Option<String>,
        body: Option<Vec<CssNode>>,
    },

    // ── CSS 原生 import（保持 at-rule 形式，不展开） ──
    Import {
        url: String,
        modifier: Option<String>,
    },

    // 注意：故意不包含以下变体：
    //   - Variable（CSS 用自定义属性 --xxx，不参与 Sass 求值）
    //   - If/For/Each/While（无控制流）
    //   - MixinDef/FunctionDef（无抽象机制）
    //   - Extend/AtRoot（CSS 原生不支持）
    //   - Warn/Debug/Error（无调试语句）
}
```

### 2.4 类型差异对照

| 变体 | ScssNode | CssNode | 说明 |
|------|----------|---------|------|
| Rule | ✅ | ✅ | 共有 |
| Decl | ✅ | ✅ | 共有 |
| Comment | ✅ (含 silent) | ✅ (仅 block) | CSS 无 `//` 注释 |
| AtRule | ✅ | ✅ | 共有 |
| **Variable** | ✅ | ❌ | CSS 无 Sass 变量 |
| **If/For/Each/While** | ✅ | ❌ | CSS 无控制流 |
| **MixinDef/FunctionDef** | ✅ | ❌ | CSS 无抽象 |
| **Include/Content** | ✅ | ❌ | CSS 无 mixin 调用 |
| **Use/Forward** | ✅ | ❌ | CSS 无模块系统 |
| **Import** | ✅ (展开) | ✅ (透传) | 语义不同 |
| **Extend** | ✅ | ❌ | CSS 无 @extend |
| **AtRoot** | ✅ | ❌ | CSS 无 @at-root |
| **Warn/Debug/Error** | ✅ | ❌ | CSS 无调试语句 |

---

## 3. 两套 Evaluator

### 3.1 SCSS Evaluator（展平语义）

```rust
/// SCSS 求值器——消费 ScssAst，产出 CssNode 树（展平）。
/// 
/// - 嵌套 Rule → 展平并组合选择器（`a .b`）
/// - `@media` in rule → 提升 + 合并选择器
/// - `@import` → 内联展开
/// - 内建函数 → 求值替换
pub struct ScssEvaluator;

impl ScssEvaluator {
    pub fn evaluate(ast: &ScssAst, env: ScssEnv) -> Result<Vec<CssNode>> {
        Self::eval_nodes(&ast.nodes, env)
    }

    fn eval_node(node: &ScssNode, env: ScssEnv) -> Result<(Vec<CssNode>, ScssEnv)> {
        match node {
            ScssNode::Rule { selector, body } => Self::eval_rule(selector, body, env),
            ScssNode::Decl { property, value, important } => 
                Self::eval_decl(property, value, *important, env),
            ScssNode::Variable { name, value, flags } => 
                Self::eval_variable(name, value, flags, env),
            ScssNode::If { branches, else_body } => Self::eval_if(branches, else_body, env),
            ScssNode::For { var, from, to, inclusive, body } => 
                Self::eval_for(var, from, to, *inclusive, body, env),
            ScssNode::Each { vars, list, body } => Self::eval_each(vars, list, body, env),
            ScssNode::While { cond, body } => Self::eval_while(cond, body, env),
            ScssNode::MixinDef { name, params, body } => Self::eval_mixin_def(name, params, body, env),
            ScssNode::FunctionDef { name, params, body } => Self::eval_func_def(name, params, body, env),
            ScssNode::Include { name, args, content } => Self::eval_include(name, args, content, env),
            ScssNode::Return(v) => Self::eval_return(v, env),
            ScssNode::Use { url, namespace, star, config } => 
                Self::eval_use(url, namespace, *star, config, env),
            ScssNode::Forward { url, show, hide, prefix, config } => 
                Self::eval_forward(url, prefix, config, env, show, hide),
            ScssNode::Import { url, modifier } => Self::eval_import(url, modifier, env),
            ScssNode::Extend { selector, optional } => Self::eval_extend(selector, *optional, env),
            ScssNode::AtRoot { query, body } => Self::eval_at_root(query, body, env),
            ScssNode::AtRule { name, params, body } => Self::eval_at_rule(name, params, body, env),
            ScssNode::Warn(v) => Self::eval_warn(v, env),
            ScssNode::Debug(v) => Self::eval_debug(v, env),
            ScssNode::Error(v) => Self::eval_error(v, env),
            ScssNode::Content => Self::eval_content(env),
            ScssNode::Comment(text, silent) => Self::eval_comment(text, *silent, env),
        }
    }

    fn eval_rule(
        selector: &str,
        body: &[ScssNode],
        env: ScssEnv,
    ) -> Result<(Vec<CssNode>, ScssEnv)> {
        // SCSS 展平逻辑（现有 RuleBuilder 模式）
        let env = env.enter_scope().with_selector(selector.clone());
        let (css, new_env) = Self::eval_nodes(body, env)?;
        let result = css.into_iter()
            .fold(RuleBuilder::new(selector), RuleBuilder::push)
            .build();
        Ok((result, new_env.exit_scope()))
    }
}
```

### 3.2 CSS Evaluator（嵌套保留语义）

```rust
/// CSS 求值器——消费 CssAst，产出 CssNode 树（保留嵌套）。
/// 
/// - 嵌套 Rule → 保留 children 结构（不组合选择器）
/// - `@media` in rule → 保留在嵌套内（不提升）
/// - `@import` → 直接输出 at-rule（不展开）
/// - 无变量、无控制流、无 mixin
pub struct CssEvaluator;

impl CssEvaluator {
    pub fn evaluate(ast: &CssAst) -> Result<Vec<CssNode>> {
        Self::eval_nodes(&ast.nodes)
    }

    fn eval_node(node: &CssNode) -> Result<Vec<CssNode>> {
        match node {
            CssNode::Rule { selector, body } => Self::eval_rule(selector, body),
            CssNode::Decl { property, value, important } => 
                Self::eval_decl(property, value, *important),
            CssNode::Comment(text) => Ok(vec![CssNode::Comment(text.clone())]),
            CssNode::AtRule { name, params, body } => Self::eval_at_rule(name, params, body),
            CssNode::Import { url, modifier } => {
                // CSS 模式：直接输出 @import at-rule，不展开
                Ok(vec![CssNode::AtImport {
                    url: url.clone(),
                    modifier: modifier.clone(),
                }])
            }
        }
    }

    fn eval_rule(selector: &str, body: &[CssNode]) -> Result<Vec<CssNode>> {
        // CSS 模式：求值 body，保留嵌套结构
        let css = Self::eval_nodes(body)?;
        
        let (declarations, children) = css.into_iter().partition(|n| matches!(n, CssNode::Declaration { .. }));
        
        Ok(vec![CssNode::Rule {
            selector: selector.to_string(),
            declarations,
            children,  // 子规则保持嵌套！
        }])
    }
    
    fn eval_decl(property: &str, value: &CssValue, important: bool) -> Result<Vec<CssNode>> {
        // CSS 模式：简单值求值（无变量替换）
        let val = value.to_string();
        Ok(vec![CssNode::Declaration {
            property: property.to_string(),
            value: val,
            important,
        }])
    }
}
```

### 3.3 两套 Evaluator 的核心差异

| 维度 | ScssEvaluator | CssEvaluator |
|------|--------------|-------------|
| **输入** | `&ScssAst` | `&CssAst` |
| **环境** | `ScssEnv`（复杂：scope chain、module cache） | 无（或极简） |
| **嵌套处理** | `RuleBuilder` 展平 + 选择器组合 | 直接保留 `children` |
| **@import** | 内联展开 + 可能嵌套组合 | 输出 `CssNode::AtImport` |
| **@media in rule** | 提升 + 合并选择器 | 保留在 children 中 |
| **@extend** | 收集 + 应用 | 不存在 |
| **内建函数** | 求值替换 | 不存在 |
| **控制流** | 求值条件/循环 | 不存在 |

---

## 4. Parser 双入口

### 4.1 统一入口

```rust
/// 语法分析器入口——根据文件扩展名分派。
pub struct Parser;

impl Parser {
    /// 解析文件——返回对应模式的 AST。
    pub fn parse_file(path: &Path, source: &str) -> Result<Parsed> {
        let mode = CompileMode::from_path(path);
        Self::parse_with_mode(mode, source)
    }

    fn parse_with_mode(mode: CompileMode, source: &str) -> Result<Parsed> {
        let tokens = Lexer::new(source).collect::<Result<Vec<_>>>()?;
        match mode {
            CompileMode::Scss => {
                let ast = ScssParser::parse(&tokens)?;
                Ok(Parsed::Scss(ast))
            }
            CompileMode::Css => {
                let ast = CssParser::parse(&tokens)?;
                Ok(Parsed::Css(ast))
            }
        }
    }
}
```

### 4.2 SCSS Parser（现有 Parser 迁移）

```rust
/// SCSS 语法分析器——完整 Sass 语法。
/// 
/// 现有 `parse::Parser` 迁移到此，不变。
pub struct ScssParser;

impl ScssParser {
    pub fn parse(tokens: &[Token]) -> Result<ScssAst> {
        // 现有 parse 逻辑，产出 ScssNode 而非 Node
        ...
    }
}
```

### 4.3 CSS Parser（简化版）

```rust
/// CSS 语法分析器——仅原生 CSS + nesting。
/// 
/// 共享大部分 SCSS token 识别逻辑，但拒绝 Sass 特有语法。
pub struct CssParser;

impl CssParser {
    pub fn parse(tokens: &[Token]) -> Result<CssAst> {
        // 复用 SCSS parser 的基础设施（parse_rule, parse_decl 等）
        // 但遇到 Sass 特有语法（$var, @if 等）时报错
        ...
    }

    /// 检查 token 是否为 Sass 特有语法——是则报错。
    fn reject_sass_syntax(token: &Token) -> Result<()> {
        match token {
            Token::Dollar(_) => Err(SassError::Parse {
                expected: "CSS value".into(),
                found: "Sass variable".into(),
            }),
            // ... 其他 Sass 特有 token
            _ => Ok(()),
        }
    }
}
```

---

## 5. Serializer 统一（无 mode）

```rust
/// CSS 序列化器——消费 CssNode 树，输出 CSS 字符串。
/// 
/// 不需要知道来源是 SCSS 还是 CSS——CssNode 已编码正确结构。
pub struct Serializer;

impl Serializer {
    pub fn serialize(nodes: &[CssNode], style: OutputStyle) -> String {
        match style {
            OutputStyle::Expanded => Self::serialize_expanded(nodes, 0),
            OutputStyle::Compressed => Self::serialize_compressed(nodes),
        }
    }
}

// 序列化规则（不变）：
// - Rule { declarations, children: [...] } → 递归输出 children（CSS 产物）
// - Rule { declarations, children: [] }   → 只输出声明（SCSS 产物）
// - AtImport { url }                      → `@import "url";`
```

---

## 6. Reactor 管线

```rust
/// 编译管线——根据入口文件类型选择对应 Evaluator。
impl Reactor<StateParsed> {
    pub fn evaluate(self) -> Result<Reactor<StateEvaluated>> {
        let css_nodes = match self.parsed {
            Parsed::Scss(ast) => {
                let env = ScssEnv::default();
                ScssEvaluator::evaluate(&ast, env)?
            }
            Parsed::Css(ast) => {
                CssEvaluator::evaluate(&ast)?
            }
        };

        Ok(Reactor {
            state: StateEvaluated,
            css_nodes,
            ..self
        })
    }
}

impl Reactor<StateEvaluated> {
    pub fn serialize(self, style: OutputStyle) -> Reactor<StateSerialized> {
        // Serializer 无需知道来源
        let css = Serializer::serialize(&self.css_nodes, style);
        Reactor {
            state: StateSerialized,
            serialized: css,
            ..self
        }
    }
}
```

---

## 7. 影响范围

### 7.1 新增类型/模块

| 新文件 | 说明 | 估计行数 |
|--------|------|----------|
| `src/parse/scss_ast.rs` | ScssAst + ScssNode 定义 | ~150 行 |
| `src/parse/css_ast.rs` | CssAst + CssNode 定义 | ~80 行 |
| `src/parse/css_parser.rs` | CssParser（简化 parser） | ~120 行 |
| `src/eval/scss_evaluator.rs` | ScssEvaluator（现有 eval 迁移） | ~400 行 |
| `src/eval/css_evaluator.rs` | CssEvaluator（新增） | ~100 行 |
| `src/eval/scss_env.rs` | ScssEnv（现有 Env 重命名） | ~200 行 |

### 7.2 改动文件

| 文件 | 改动类型 | 估计行数 |
|------|----------|----------|
| `src/parse/mod.rs` | 改产出为 `Parsed` 枚举 | ~30 行 |
| `src/css/node.rs` | 重构 `CssNode`（拆分为输出类型） | ~50 行 |
| `src/css/serialize.rs` | 适配新 `CssNode` | ~30 行 |
| `src/eval/reactor.rs` | 管线分叉选择 Evaluator | ~20 行 |

### 7.3 可删除的代码

| 代码 | 原因 |
|------|------|
| `src/eval/plain_css.rs` | 不再需要——CssNode 类型系统已保证 |
| `src/eval/env_impl.rs` 的 `plain_css` 字段 | 不再需要 |
| 所有 `is_plain_css()` 调用 | 类型系统替代运行时检查 |

---

## 8. 渐进式迁移路径

### Phase 1：类型定义 + 双 Parser

1. 新建 `scss_ast.rs` + `css_ast.rs` 定义两套 AST
2. 新建 `CssParser`（可在 ScssParser 基础上 fork + 限制）
3. `Parser` 入口分派：`parse_file` → `Parsed::Scss` / `Parsed::Css`

**产出**：类型系统建立，可解析两种文件但 eval 暂未分叉

### Phase 2：SCSS Evaluator 迁移

1. 现有 `eval/` 子模块迁移到 `ScssEvaluator`
2. 重命名 `Env` → `ScssEnv`
3. Reactor 在 Scss 路径调用 `ScssEvaluator::evaluate`

**产出**：SCSS 行为完全不变，但已独立到 `scss_evaluator.rs`

### Phase 3：CSS Evaluator 新建

1. 新建 `CssEvaluator`（~100 行）
2. 实现 `eval_rule`（保留嵌套）
3. 实现 `eval_import`（透传 at-rule）
4. Reactor 在 Css 路径调用 `CssEvaluator::evaluate`

**产出**：CSS 文件走独立路径，嵌套保留

### Phase 4：清理旧代码

1. 删除 `plain_css.rs`
2. 删除所有 `is_plain_css()` 检查
3. 删除 `Node::Variable`、`Node::If` 等在 CSS 路径的"幽灵分支"

**产出**：代码更简洁，编译期安全保证

---

## 9. 长期维护收益

| 场景 | 单 AST 方案 | 双 AST 方案 |
|------|------------|-------------|
| CSS 出新功能（CSS Color 5） | 改 `Node` → 重跑 SCSS 测试 | 改 `CssNode` → 不影响 SCSS |
| SCSS 出新语法（@some-new-rule） | 改 `Node` → 重跑 CSS 测试 | 改 `ScssNode` → 不影响 CSS |
| 修 CSS nesting bug | 担心影响 SCSS 展平 | 改 CssEvaluator → SCSS 无风险 |
| 加 CSS 新 at-rule 支持 | 加 variant + 确认 SCSS 分支 | 加 `CssNode` variant → 完成 |
| 新人 onboarding | 理解两种语义 + 5 处 mode 检查 | 选一个领域 → 读对应的 enum |

---

## 10. 风险评估与缓解

| 风险 | 概率 | 影响 | 缓解 |
|------|------|------|------|
| 两套 AST 初始代码量大 | 高 | 5-7 天工作量 | 分 Phase 渐进式迁移 |
| CSS Parser 遗漏 Sass 语法拒绝 | 中 | CSS 文件解析出 SCSS 节点 | `reject_sass_syntax` + 测试覆盖 |
| SCSS Evaluator 迁移遗漏 | 中 | 部分 SCSS 行为改变 | 全量测试 + spec 对比 |
| 两套求值结果不一致 | 低 | 相同选择器输出不同 | 共享 CssNode + Serializer |
| 子文件 @import 跨界 | 中 | SCSS @import .css 时的语义 | 子文件独立判断 mode |

---

## 11. 工作量估计

| Phase | 工作量 | 产出 |
|-------|--------|------|
| Phase 1: 类型 + Parser | 2 天 | 可解析两种文件 |
| Phase 2: SCSS Eval 迁移 | 2 天 | SCSS 行为不变但已隔离 |
| Phase 3: CSS Eval 新建 | 1 天 | CSS 嵌套保留 |
| Phase 4: 清理 | 1 天 | 删除旧代码 |
| **总计** | **6 天** | **双 AST 完全隔离** |

---

## 12. 总结

**核心理念**：CSS 和 SCSS 是两种语言。用两种类型（`CssNode` / `ScssNode`）表达两种语言，用两个 Evaluator 实现两种语义，用一套 Serializer 统一输出。

**关键收益**：
- 编译期保证 CSS 路径不会执行 Sass 逻辑
- 版本独立演化，互不影响
- 代码审查各自独立，降低认知负担
- 新人可以只学一个领域
