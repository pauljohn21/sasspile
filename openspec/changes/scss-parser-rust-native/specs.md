# Delta Spec — SCSS Parser Rust Native Refactor

本文档定义 parse 模块从 cursor 模式迁移到 pure-function 模式的精确需求规格。
**任何实施必须在满足本文档所有 spec 的前提下进行。**

---

## SPEC-1: ParseContext 结构体

**需求**: 纯函数解析需要携带跨节点状态，而不是 `Parser` 结构体字段。

```rust
/// 解析上下文——跨节点传递的状态。
#[derive(Debug, Clone, Default)]
pub(crate) struct ParseContext {
    /// 是否已解析过非模块规则（用于 @forward/@use 验证）。
    pub(crate) saw_other_rule: bool,
    /// 是否在规则体内部（用于 @forward in_body 验证）。
    pub(crate) in_body: bool,
}
```

**不变式**:
- `saw_other_rule` 默认 `false`
- `in_body` 默认 `false`
- `parse_body` 进入时设 `in_body = true`，退出时恢复

---

## SPEC-2: 函数签名契约

**需求**: 所有子解析器统一采用 `(T, &[Token])` 返回模式。

**规则**:
| 输入 | 输出 | 语义 |
|------|------|------|
| `tokens: &[Token]` | `Result<(Node, &[Token])>` | 消费一个节点，返回剩余 |
| `tokens: &[Token]` | `Result<(Value, &[Token])>` | 消费一个值，返回剩余 |
| `tokens: &[Token]` | `Result<(String, &[Token])>` | 消费字符串属性名，返回剩余 |
| `tokens: &[Token]` | `&[Token]` | 纯辅助（skip_ws），返回剩余 |
| `tokens: &[Token]` | `bool` | 纯 lookahead（无消费） |

**禁止项**:
- 禁止 `&mut self`
- 禁止 `self.pos += 1` 或任何 `pos` 字段
- 禁止 `self.advance()`
- 禁止 `self.tokens.get(self.pos + n)` 直接索引

---

## SPEC-3: parse_node 分派分派逻辑

**需求**: `parse_node` 根据首个非 ws token 类型分派到具体解析器。

**分派规则**（按顺序，first match wins）:
1. `Token::AtRule(name)` → `parse_at_rule(tokens, ctx)` （消费 AtRule token 后解析）
2. `Token::Dollar(_)` → `parse_variable(tokens)` （消费 $name）
3. `Token::Comment(t, silent)` → `Node::Comment(t, silent)` （消费 Comment）
4. `Token::Semicolon` → 跳过，递归 parse_node（孤立 ;）
   - 如果 ; 后紧跟 `}` / `Eof` → 返回 `Node::Comment("", true)`（空 silent 注释）
5. `Token::Whitespace` → 跳过，递归 parse_node
6. 其他 →
   - `is_namespace_var(tokens)` → `parse_namespace_var(tokens)`
   - 否则 → `parse_rule_or_decl(tokens)`

**重要**: skip_ws 仅在 `parse_node` 开始时调用一次，子解析器应复用传入的 tokens。

---

## SPEC-4: 斜杠语义保留

**需求**: `/` 在不同上下文中行为不同。

| 上下文 | `/` 行为 |
|--------|----------|
| `parse_value` （变量赋值、函数参数） | 做除法 |
| `parse_decl_value` （CSS 声明值） | 斜杠分隔符 |
| 空格列表内 `1 2/3 4` | 斜杠分隔符（`2/3` 保留） |
| 括号表达式后 `(1)/2` | 做除法 |
| 变量引用后 `$x/2` | 做除法 |
| `/` 后紧跟 `+−*%` | 做除法 |

**实现约束**:
- `slash_followed_by_arith_op` 需要在纯函数版本中正确工作
- `/` 后的 min_bp=6 递归避免与斜杠列表分支交互

---

## SPEC-5: @规则上下文验证

**需求**: `@use` / `@forward` 必须在任何非模块规则之前声明。

**验证逻辑**（在 `parse_use` / `parse_forward` 入口处）:
```rust
if ctx.in_body {
    return Err(SassError::Eval("This at-rule is not allowed here.".into()));
}
if ctx.saw_other_rule {
    return Err(SassError::Eval(
        "@use rules must be written before any other rules.".into(),
    ));
}
```

**触发 `saw_other_rule = true` 的节点类型**:
- `Node::Rule { .. }`
- `Node::Decl { .. }`
- `Node::If { .. }`, `Node::For { .. }`, `Node::Each { .. }`, `Node::While { .. }`
- `Node::MixinDef { .. }`, `Node::FunctionDef { .. }`
- `Node::Include { .. }`, `Node::Extend { .. }`
- `Node::AtRoot { .. }`, `Node::AtRule { .. }`
- `Node::Error { .. }`, `Node::Warn { .. }`, `Node::Debug { .. }`
- `Node::Content`, `Node::Return { .. }`

**不触发 `saw_other_rule` 的节点类型**:
- `Node::Forward { .. }`
- `Node::Use { .. }`
- `Node::Import { .. }`
- `Node::Variable { .. }`
- `Node::Comment(_, _)`

---

## SPEC-6: 模块私有成员保护

**需求**: 以 `_` 开头的 mixin/function/variable 不能被命名空间外部访问。

**检查点**:
1. `parse_include`：如果 `@include namespace._mixin()` → 报错
2. `parse_ident_followup`：如果 `namespace._func()` → 报错
3. `parse_ident_followup`：如果 `$namespace._var` → 报错

**错误消息**:
- Mixin: `"Private members can't be accessed from outside their modules."`
- Function: `"Private members can't be accessed from outside their modules."`
- Variable: `"Private members can't be accessed from outside their modules."`

---

## SPEC-7: parse_selector 逻辑不变式

**需求**: 选择器解析器需保持与 cursor 版本完全相同的行为。

**关键子逻辑**:
1. **Bracket depth 跟踪**:
   - `[` → depth++
   - `]` → depth--
2. **Whitespace 处理**:
   - `bracket_depth > 0`：空白可能是属性操作符的一部分
     - 后面紧跟 `]` / `Assign` / `Tilde` / `Pipe` / `Caret` / `Star`：跳过空白
     - 否则（合法 modifier 检测）：消费 modifier 字符
   - `bracket_depth == 0`：空白 → 单空格拼接进 selector 字符串
3. **Modifier 检测**:
   - 单字符 ASCII 字母标识符
   - 紧跟 `]` 时有效
   - 否则错误 `"expected ], found modifier"`
4. **注释跳过**: 注释 token 在 selector 中跳过

---

## SPEC-8: parse_function_def 保留名检查

**需求**: 不允许使用保留函数名。

**检查规则（优先级顺序）**:
1. `type` — 大小写不敏感禁止（CSS Values and Units 5 保留）
2. 全小写检测：
   - `url`, `expression`, `element`, `and`, `or`, `not` — 全小写禁止
   - 大写/混合允许（Phase 2）
3. Vendor prefix：
   - `-prefix-element` 全小写仍禁止
   - `-prefix-url/-expression/-and/-or/-not` 已放宽

**错误消息**:
- `type` 系: `"This name is reserved for the plain-CSS function."`
- 其他: `"Invalid function name."`

---

## SPEC-9: peek_binding_power +/- 一元负号检测

**需求**: `+` 总是二元，`-` 一元需上下文判断。

**规则**:
- `+` → 总是 `Some((BinOpKind::Add, 4))`
- `-` → 需判断：
  - `has_ws_before + !has_ws_after + next_is_Number` → 一元负号 → `None`
  - 否则 → `Some((BinOpKind::Sub, 4))`

**实现**:
```rust
fn peek_binding_power(tokens: &[Token]) -> Option<(BinOpKind, u8)> {
    // tokens[0] 是当前 token（不 skip_ws，调用方负责）
    match tokens.first()? {
        Token::Plus => Some((BinOpKind::Add, 4)),
        Token::Minus => {
            let has_ws_before = /* 调用方传入或 tokens 前有 ws */;
            // 注意：在新方案中，调用前已经 skip_ws，所以需要传入上下文
            // 简化：在 skip_ws 调用后、check binding power 前 token 已全部确定
            Some((BinOpKind::Sub, 4))
        }
        // ...
    }
}
```

**实现注意**: 由于纯函数版本不在 Parser 内维护 pos，`-` 一元负号检测需要额外传入"前面是否有 ws"信息。建议：
- `parse_prefix` 中遇到 `-` 时直接处理（一元/二元）
- 不依赖 `peek_binding_power` 判断 `-`

---

## SPEC-10: parse_prefix 与 parse_expr 的递归关系

**需求**: Pratt 解析器的核心递归。

```
parse_expr(tokens, min_bp):
    (lhs, mut tokens) = parse_prefix(tokens)?
    loop:
        (op, bp) = peek_binding_power(tokens)?
        if bp < min_bp: break
        tokens = consume(op_token, tokens)
        (rhs, tokens) = parse_expr(tokens, bp + 1)?
        lhs = BinOp(op, lhs, rhs)
    Ok((lhs, tokens))

parse_prefix(tokens):
    skip_ws(tokens)
    match tokens.first():
        Minus → unary negation
        Literal → parse_literal(tokens)
        LParen → group / map / list → parse_paren_expr(tokens)
        LBracket → bracketed list
        Not → unary not
        Percent → "%" string
        _ → fallback
```

**不变式**:
- `parse_prefix` 必须先 `skip_ws` 再 peek
- `parse_expr` 内部必须先 `skip_ws` 再 peek binding power
- 二元运算符消费后必须 `skip_ws` 再 parse rhs

---

## SPEC-11: parse_interp_adjacent 逻辑

**需求**: 拼接相邻的 ident/number/interp/hash 为插值片段。

**消费规则**（按顺序）:
- `Token::Ident(t)` 且非关键字 → push Text(t)
- `Token::Number(n)` → push Text(n)
- `Token::Interp(e)` → push Expr(e)
- `Token::Hash(h)` → push Text("#{h}")
- 其他 → break

**后缀行为**:
- 无后续 → `Value::Interp(segments)`
- 后续 `(` → 作为函数名：`Value::Call(joined_name, args)`
- segments.len() == 1 + Expr + `(` → `Value::Call(expr, args)`

---

## SPEC-12: CSS 函数 raw 内容保留

**需求**: `calc()` / `clamp()` / `env()` / `var()` / `url()` / `css()` / `attr()` 不解析内部参数，保留 raw。

**例外**: `url("string")` 且参数是字符串 → 走正常 parse_args。

**raw 处理**: 从 `(` 到匹配 `)` 的所有内容原样输出为 `Value::Calc("calc(...)")`。

---

## SPEC-13: CSS Level 4 rgb/hsl/hwb 空格分隔语法

**需求**: `rgb(R G B / A)` / `hsl(H S L / A)` / `hwb(H W B / A)` 空格分隔支持。

**检测逻辑**:
- 函数名匹配 + `(` 后
- 第一个参数是 Number
- 后面紧跟 `is_value_start()` 或 `Slash`（不是逗号）
- → 空格分隔语法

**回退**: 不是空格分隔 → 标准 parse_args（回退 pos 语义 = 重起 parse_args）。

**注意**: 在纯函数方案中"回退"是通过重新从保存点 tokens 调用 parse_args 实现。

---

## SPEC-14: parse_config 重复检测

**需求**: `@use with($x: 1, $x: 2)` → 错误（同一变量只配置一次）。

**正规化规则**: 变量名中的 `-` → `_` 后再比较。
- `my-var` 和 `my_var` 视为重复 → 报错

---

## SPEC-15: 错误消息稳定性

**需求**: 重构前后对相同输入产生完全相同的错误消息。

**关键错误消息（不可更改）**:
| 场景 | 消息 |
|------|------|
| `$x: y !global` 在 namespace | `"!global isn't allowed for variables in other modules."` |
| `@forward` 在 body 内 | `"This at-rule is not allowed here."` |
| `@forward` 在其他规则后 | `"@forward rules must be written before any other rules."` |
| `@use` 在 body 内 | `"This at-rule is not allowed here."` |
| `@use` 在其他规则后 | `"@use rules must be written before any other rules."` |
| `type` 函数名 | `"This name is reserved for the plain-CSS function."` |
| 全小写保留函数名 | `"Invalid function name."` |
| `namespace._private` 访问 | `"Private members can't be accessed from outside their modules."` |
| 重复配置变量 | `"The same variable may only be configured once."` |
| mixin 命名空间标识符 | `"."` 的 expected = `"{"` 或 `"("` |

---

## SPEC-16: Reactor 兼容

**需求**: `Reactor::parse()` 内部必须能继续调用 `crate::parse::Parser::parse(tokens)`。

```rust
// 必须保留的关联函数
impl Parser {
    pub fn parse(tokens: &[Token]) -> Result<Ast> {
        parse_fn(tokens)  // 委托给自由函数
    }
}
```

**或**: `pub fn parse(tokens: &[Token]) -> Result<Ast>` 自由函数直接对外暴露。

---

## SPEC-17: 公开 API 不变

**以下函数签名必须保持不变**:
```rust
pub fn compile(input: &str, style: OutputStyle) -> Result<String>
pub fn compile_expanded(input: &str) -> Result<String>
pub fn compile_compressed(input: &str) -> Result<String>
pub fn compile_file(path: &PathBuf, style: OutputStyle) -> Result<String>
pub fn compile_file_with_load_paths(path, style, load_paths) -> Result<String>
pub fn parse_scss(input: &str) -> Result<Ast>  // 实际通过公开 use 暴露
```

---

## SPEC-18: 性能要求

**需求**: 重构后不引入性能回退。

| 指标 | 上限 |
|------|------|
| Bootstrap 5.3.8 编译时间 | 不超过 baseline +5% |
| 大文件 (100K loc) | 解析时间不超过 baseline +10% |

**避免**:
- 不在热路径 clone `Vec<Token>`
- 不在内层循环做 O(n²) 切片复制
- `&[Token]` 切片操作是 O(1) 必须利用

---

## SPEC-19: 模块可见性

**需求**:
- `ParseContext` 只对 `parse` 模块可见：`pub(crate)`
- 各 `parse_xxx` 函数也 `pub(crate)`
- 公开入口保持不变：`parse()`, `parse_scss`

---

## SPEC-20: 禁止清单

**以下模式禁止出现在新 parse 代码中**:

```rust
// ❌ 禁止
struct Parser { pos: usize, .. }  // cursor 状态
fn advance(&mut self) -> Option<&Token>
self.pos += 1
self.tokens.get(self.pos)
match self.peek() { .. }        // &mut self 方法
while !self.at_end() { .. }     // 循环末端检测
let saved = self.pos; self.pos = saved;  // 回滚模式
```

**唯一允许的"循环"**: `parse_all` / `parse_body` 中的顶层 while，
但使用 `first_non_ws(rest)` 而非 `at_end()`。

---

## 验收标准

实施完成后，必须通过以下验证：

```bash
cargo check                      # 零错误
cargo clippy --all-targets       # 零新增 warning
cargo test --test compile_test   # 57/57 通过
cargo test --test stage_test     # 10/10 通过
cargo test --test interp_test    # 15/15 通过
cargo test --test bs_spec        # 15/15 通过
cargo test --test common_test    # 5/5 通过
cargo test --test ast_test       # 8/8 通过

# sass-spec 不退化（全量运行后对比）
SPEC_STORE_CMD=run cargo test --test spec_store -- --nocapture
SPEC_STORE_CMD=stats cargo test --test spec_store -- --nocapture
```

**sass-spec 判定**: 新 snapshot 通过数 ≥ 旧 snapshot 通过数 (7698/12131 = 63.4%)
