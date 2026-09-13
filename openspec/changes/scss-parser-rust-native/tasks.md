## Phase 0: 清理 CSS 残留 — 已完成 ✅

### Task 0.1: 清理 Parse 模块的 CSS/双模式引用
- [x] `parse/mod.rs`: 移除 Parsed::Css 变体 → Parsed = Ast（type alias）
- [x] `parse/mod.rs`: 移除 `CompileMode` 枚举 → 始终 SCSS
- [x] `parse/mod.rs`: 移除 `parse_dispatch(mode)` → `parse(tokens) -> Parsed`
- [x] `parse/mod.rs`: 移除 `compile_mode` 相关逻辑
- [x] 清理 `css_ast.rs` / `css_parser.rs` / `scss_ast.rs` 的导入引用

### Task 0.2: 清理 eval 模块
- [x] `eval/mod.rs`: 确认 `css_evaluator` 已删除
- [x] `eval/reactor.rs`: 移除 `css_ast` 字段
- [x] `eval/reactor.rs`: 移除 `mode: CompileMode` 字段
- [x] `eval/reactor.rs`: evaluate() 简化为只有 ScssEvaluator 分支

### Task 0.3: 清理其他引用
- [x] 搜索 `CssAst` / `CssNode` / `CssEval` / `CompileMode` / `parse_dispatch` → 全部移除

## Phase 1: 添加 ParseContext + 基础纯函数

### Task 1.1: ParseContext 结构体
- [ ] 在 `parse/mod.rs` 添加 `ParseContext` 结构体（`saw_other_rule: bool`, `in_body: bool`）
- [ ] 添加 `ParseContext::default()` impl
- [ ] 添加 `ParseContext::with_body(self, body: bool) -> Self` builder 方法（用于 `parse_body` 切换）

### Task 1.2: 基础辅助纯函数
- [ ] `skip_ws(tokens: &[Token]) -> &[Token]` — 跳过开头 Whitespace
- [ ] `skip_ws_and_comments(tokens: &[Token]) -> &[Token]` — 跳过开头 Whitespace + Comment
- [ ] `first_non_ws(tokens: &[Token]) -> Option<&Token>` — 首个非 Whitespace token

### Task 1.3: expect 纯函数
- [ ] `expect(tokens: &[Token], tok: &Token) -> Result<&[Token]>` — 消费匹配 token 或报错
- [ ] 包含原有 context 信息（pos 附近 5 个 token）

### Task 1.4: is_lookahead 纯函数
- [ ] `is_rule(tokens: &[Token]) -> bool` — lookahead 检测 `{` 先出现 → 规则（从 nodes.rs 移植）
- [ ] `is_namespace_var(tokens: &[Token]) -> bool` — 检测 ident . dollar 模式

### Task 1.5: 验证
- [ ] cargo check 编译通过
- [ ] 编译期无新增 warning（除已有 allow 外）

## Phase 2: 顶层 parse 入口重构

### Task 2.1: parse() 函数化
- [ ] 将 `Parser::parse` 从 `&mut self` 方法重构为 `parse(tokens: &[Token]) -> Result<Ast>` 自由函数
- [ ] 使用 ParseContext 跟踪 saw_other_rule 状态
- [ ] 在循环中根据 node 类型更新 ctx

### Task 2.2: parse_body 改造
- [ ] `parse_body(tokens: &[Token], ctx: &ParseContext) -> Result<(Vec<Node>, &[Token])>`
- [ ] 通过 ParseContext.with_body(true) 传递 in_body 状态
- [ ] parse_node 调用时传入 ctx 引用

### Task 2.3: 验证
- [ ] cargo test --test compile_test 核心编译测试通过
- [ ] cargo test --test parse_test 通过（如果存在）

## Phase 3: 节点解析重构（nodes.rs → 自由函数）

### Task 3.1: parse_node 纯函数化
- [ ] `parse_node(tokens: &[Token], ctx: &ParseContext) -> Result<(Node, &[Token])>`
- [ ] 保留完整的 match 分支逻辑
- [ ] @forward/@use 验证通过 ctx 而非 self.in_body/self.saw_other_rule

### Task 3.2: parse_rule / parse_decl 纯函数化
- [ ] `parse_rule(tokens: &[Token]) -> Result<(Node, &[Token])>` — 选择器 + body
- [ ] `parse_decl(tokens: &[Token]) -> Result<(Node, &[Token])>` — 属性:值 + important
- [ ] `parse_property(tokens: &[Token]) -> Result<(String, &[Token])>`
- [ ] `check_important(tokens: &[Token]) -> Result<(bool, &[Token])>`

### Task 3.3: parse_selector 纯函数化
- [ ] `parse_selector(tokens: &[Token]) -> Result<(String, &[Token])>`
- 完整保留原有 logic：bracket_depth、modifier 检测、attr 操作符识别

### Task 3.4: parse_variable 纯函数化
- [ ] `parse_variable(tokens: &[Token]) -> Result<(Node, &[Token])>`
- [ ] 含 !default / !global flags

### Task 3.5: parse_namespace_var 纯函数化
- [ ] `parse_namespace_var(tokens: &[Token]) -> Result<(Node, &[Token])>`
- [ ] 保留 private member 检查
- [ ] 保留 !global 禁止

### Task 3.6: 验证
- [ ] cargo test --test compile_test 全通过
- [ ] cargo test --test stage_test 全通过

## Phase 4: @规则解析重构

### Task 4.1: parse_at_rule 入口
- [ ] `parse_at_rule(tokens: &[Token], ctx: &ParseContext) -> Result<(Node, &[Token])>`
- [ ] 从 tokens 提取 @rule name（消费 AtRule token）
- [ ] AtRuleKind 分派保持不变

### Task 4.2: 模块系统（ParseContext 验证）
- [ ] `parse_use(tokens: &[Token], ctx: &ParseContext)` — 读取 ctx.in_body + ctx.saw_other_rule
- [ ] `parse_forward(tokens: &[Token], ctx: &ParseContext)` — 同上
- [ ] 错误消息保留不变："This at-rule is not allowed here." / "@use rules must be written before any other rules."

### Task 4.3: 流控制
- [ ] `parse_if(tokens: &[Token]) -> Result<(Node, &[Token])>` — 含 @else if / @else 链
- [ ] `parse_for(tokens: &[Token]) -> Result<(Node, &[Token])>` — from through/to
- [ ] `parse_each(tokens: &[Token]) -> Result<(Node, &[Token])>` — multi-var + in
- [ ] `parse_while(tokens: &[Token]) -> Result<(Node, &[Token])>`

### Task 4.4: Mixin/Function
- [ ] `parse_mixin_def(tokens: &[Token])` — 保留 namespace 检查
- [ ] `parse_include(tokens: &[Token])` — 保留 private mixin 检查
- [ ] `parse_function_def(tokens: &[Token])` — 保留保留名检查（type/url/element/and/or/not）

### Task 4.5: 其他 @规则
- [ ] `parse_extend(tokens: &[Token])` — `!optional` 支持
- [ ] `parse_at_root(tokens: &[Token])` — 查询 + 选择器
- [ ] `parse_warn` / `parse_debug` / `parse_error`
- [ ] `parse_generic_at_rule(name, tokens)` — 默认 fallthrough
- [ ] `parse_at_params(tokens)` — params 序列化规则保留

### Task 4.6: 验证
- [ ] cargo test --test compile_test 全通过
- [ ] cargo test --test common_test 全通过

## Phase 5: 表达式解析重构

### Task 5.1: parse_prefix 纯函数化
- [ ] `parse_prefix(tokens: &[Token]) -> Result<(Value, &[Token])>`
- [ ] 完整保留：Minus / Literal / LParen / Not / LBracket / Percent 分支
- [ ] recursive descent：parse_prefix → parse_expr → ...

### Task 5.2: parse_literal 纯函数化
- [ ] `parse_literal(tokens: &[Token]) -> Result<Option<(Value, &[Token])]>>`
- [ ] 所有 token 类型分支保留

### Task 5.3: parse_ident_followup 纯函数化
- [ ] `parse_ident_followup(tokens: &[Token], name: String) -> Result<(Value, &[Token])>`
- [ ] 保留 interp 拼接、module.$var、module.func()、裸标识符、calc/rgb/hsl raw 处理
- [ ] CSS Level 4 rgb/hsl/hwb 空格分隔语法

### Task 5.4: Pratt 引擎纯函数化
- [ ] `parse_expr(tokens: &[Token], min_bp: u8) -> Result<(Value, &[Token])>`
- [ ] `parse_expr_slash(tokens: &[Token], min_bp, slash_as_sep) -> Result<(Value, &[Token])>`
- [ ] 斜杠分隔语义（slash_as_sep + lhs 不是 Paren/Variable → 斜杠列表）
- [ ] 空格列表（min_bp==0 + is_value_start → 空格分隔 List）
- [ ] 二元运算符优先级攀爬

### Task 5.5: Lookahead 辅助纯函数化
- [ ] `is_value_start(tokens: &[Token]) -> bool` — 关键字排除
- [ ] `slash_followed_by_arith_op(tokens: &[Token]) -> bool` — / 后有 +−*% → 除法语义
- [ ] `peek_binding_power(tokens: &[Token]) -> Option<(BinOpKind, u8)>` — 一元负号检测

### Task 5.6: interp_adjacent 纯函数化
- [ ] `parse_interp_adjacent(tokens: &[Token], segments: Vec<InterpSegment>) -> Result<(Value, &[Token])>`
- [ ] 保留函数调用后缀检测

### Task 5.7: parse_value / parse_decl_value
- [ ] `parse_value(tokens: &[Token]) -> Result<(Value, &[Token])>` — 逗号分隔列表
- [ ] `parse_decl_value(tokens: &[Token]) -> Result<(Value, &[Token])>` — slash_as_sep=true

### Task 5.8: parse_expr_rest
- [ ] `parse_expr_rest(tokens: &[Token], lhs: Value, min_bp: u8) -> Result<(Value, &[Token])>`
- [ ] 空格列表内算术子表达式

### Task 5.9: 验证
- [ ] cargo test --test interp_test 全通过
- [ ] cargo test --test expr_test（如存在）通过

## Phase 6: 参数解析重构

### Task 6.1: parse_params 纯函数化
- [ ] 形如 `( $p1: default, $p2, ... )`
- [ ] 保留 `...` rest 参数支持

### Task 6.2: parse_args 纯函数化
- [ ] 关键字参数 `$name: value` / `name: value`
- [ ] 位置参数 + 三元 `condition : value : else`
- [ ] `...` spread 支持
- [ ] 保留完整错误（双分号、; 后紧跟 , 等）

### Task 6.3: parse_config 纯函数化
- [ ] `allow_default` 标志区分 @use/@forward with()
- [ ] 保留变量名 drafts 去重（`-` vs `_` 正规化）

### Task 6.4: parse_member_list 纯函数化
- [ ] `@forward show $var, mixin, fn` 等成员列表

### Task 6.5: 验证
- [ ] cargo test --test param_expr_test 通过（如存在）
- [ ] cargo test --test bs_spec 通过

## Phase 7: 结构重组 + 入口保留

### Task 7.1: Parser 类型清理
- [ ] 从 `struct Parser { tokens, pos, in_body, saw_other_rule }` 改为承载纯函数的 namespace 结构体
- [ ] 保留 `Parser::parse(tokens) -> Result<Ast>` 关联函数作为 Reactor 兼容入口

### Task 7.2: Reactor 接口无变化
- [ ] `Reactor::parse()` 内部仍调用 `crate::parse::Parser::parse(&tokens)?`
- [ ] 不改变任何 Reactor 公开方法签名

### Task 7.3: lib.rs 入口保留
- [ ] `pub use parse::{parse as parse_scss, ast::Ast};` 不变
- [ ] `compile()` / `compile_file()` 等不变

### Task 7.4: 清理
- [ ] 搜索所有 `pos` 引用 → 确认无残留
- [ ] 搜索所有 `advance()` → 确认无残留
- [ ] 搜索所有 `self.tokens.get(self.pos` → 确认无残留

### Task 7.5: 验证
- [ ] cargo check 无错误
- [ ] cargo clippy 无新增 warning
- [ ] 核心测试 202 全绿
- [ ] sass-spec 不退化（SPEC_STORE_CMD=run 基线对比）

## Non-Goals

- 不改变 AST/Node/Value 类型
- 不改变 Evaluator / Serializer 结构的函数签名
- 不引入外部解析框架
- 不在本轮添加类型状态泛型（除非发现强制需求）
