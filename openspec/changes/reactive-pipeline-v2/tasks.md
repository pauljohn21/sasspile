# Tasks

## 1. Setup & Scaffolding

- [x] 1.1 创建 `Cargo.toml` 依赖块（确认 `rxrust = "1.0.0.0-rc.5"` 已在位）并通过 `cargo check` 验证
- [x] 1.2 创建 `src/lexer/`、`src/parser/`、`src/lowering/`、`src/builtin/` 目录及 mod.rs 入口文件，通过 `cargo check` 验证模块解析
- [x] 1.3 在 `src/error.rs`（或新建模块）定义 `crate::Error` 枚举（LexerError / ParserError / LoweringError / EvalError / SerializerError），并实现 `std::error::Error` + `Display`，通过 `cargo check` 验证

## 2. Lexer Implementation

- [x] 2.1 实现 `Token` 结构体（`kind: char`、`pos: u32`、可选 `token_type` 枚举区分 ident/number/string 等），含单元测试验证基本构造
- [x] 2.2 实现 `LexerState` 结构体，管理字符位置、行列号，支持 `advance()` / `peek()` 接口，含单元测试验证位置跟踪
- [x] 2.3 实现 `Lexer` 入口函数 `fn lex<Input: Observable<char>>(input: Input) -> Observable<Token>` 使用 `Observable::create` + `scan(LexerState)`，通过 `cargo test` 验证 `"abc"` 产出三个 Token
- [x] 2.4 实现换行符标准化（`\r\n` → `\n`，`\x0C` → `\n`），含单元测试验证 `\r\n` 输入产出单个 `\n` Token
- [x] 2.5 实现错误传播：无效字符通过 `on_error` 通道报告，含单元测试验证错误类型

## 3. Parser Implementation

- [x] 3.1 实现 `InputSyntax` 枚举（Scss / Sass / Css）和 `ParserState` 结构体，含 token 缓冲区与游标（支持 `peek_n` / `set_cursor`），通过单元测试验证前瞻/回溯
- [x] 3.2 实现 `SassAstNode` 枚举定义（`VariableDecl`、`Rule`、`StyleDecl`、`Interpolated`、`ParentSelector`、`MapLiteral`、`ListLiteral`、`Comment` 等变体），通过 `cargo check` 验证
- [x] 3.3 实现 `Parser` 入口函数 `fn parse<Input: Observable<Token>>(input: Input) -> Observable<SassAstNode>` 使用 `Observable::create` + `scan(ParserState)`，通过单元测试验证 `"a { color: red; }"` 产出正确 SassAstNode
- [x] 3.4 实现 SCSS 变量声明解析（`$color: red;`、`$color: red !default;`），含单元测试验证 `has_default` 字段
- [x] 3.5 实现嵌套规则 + 父选择器解析（`a { &:hover { ... } }`），含单元测试验证 `&` 保留在解析树中
- [ ] 3.6 实现插值表达式解析（`.#{$class}` 中的 `#{$class}` 作为 `Interpolated` 节点），含单元测试验证
- [ ] 3.7 实现 Map 和 List 字面量解析（`(blue: #0d6efd)` → `MapLiteral`），含单元测试验证
- [ ] 3.8 实现错误传播：无效语法通过 `on_error` 报告，含单元测试验证未闭合括号错误

## 4. Lowering Implementation

- [ ] 4.1 实现 `LoweringContext` 结构体（含祖先选择器栈、变量环境引用），通过 `cargo check` 验证
- [ ] 4.2 实现 `lower_to_ast(SassAstNode, &LoweringContext) -> Result<AstNode>` 函数骨架，能够处理简单的 `StyleDecl` 降级，含单元测试验证
- [ ] 4.3 实现插值展开（`Interpolated` → 具体字符串），含单元测试验证 `$prefix = "bs-"` 时 `#{$prefix}btn` 展开为 `"bs-btn"`
- [ ] 4.4 实现父选择器展开（`&:hover` + 祖先栈 `"a"` → `"a:hover"`），含单元测试验证单层/多层嵌套
- [ ] 4.5 实现 Map/List 字面量 → `Value::Map` / `Value::List` 转换，含单元测试验证
- [ ] 4.6 实现 `!default` 语义：已存在同名变量时丢弃声明，含单元测试验证丢弃和生效两种情况
- [ ] 4.7 实现错误传播：未定义变量插值返回错误，含单元测试验证

## 5. Evaluator Refactor (SassOp → Observable::create)

- [ ] 5.1 修改 `SassOp` trait：`into_operator` 返回 `Observable<AstNode>`（不再是 `Box<dyn Fn>`），通过 `cargo check` 验证 trait 定义
- [ ] 5.2 改造 `AstIf` 的 `SassOp` 实现：用 `Observable::create` + `switch_map` 选择分支，含单元测试验证 `@if $x { ... } @else { ... }`
- [ ] 5.3 改造 `AstFor` 的 `SassOp` 实现：用 `Observable::create` + `flat_map` 迭代范围，含单元测试验证 `@for $i from 1 through 3`
- [ ] 5.4 改造 `AstEach` 的 `SassOp` 实现：支持列表和 Map 迭代，含单元测试验证 `@each $color in red, green`
- [ ] 5.5 改造 `AstWhile` 的 `SassOp` 实现：支持 MAX_WHILE_ITERATIONS 边界，含单元测试验证迭代次数
- [ ] 5.6 改造 `AstMixin` / `AstInclude` 的 `SassOp` 实现：`tap` 注册 + 展开 body，含单元测试验证 `@include foo(20px)`
- [ ] 5.7 改造 `AstFunctionDecl` / `AstReturn` 实现：注册 + 终止内部 Observable，含单元测试验证自定义函数返回值
- [ ] 5.8 改造 `AstMediaRule` / `AstErrorRule` / `AstWarnRule` / `AstDebugRule` 实现，含单元测试验证各指令行为

## 6. Multicast Bus & Error Type

- [ ] 6.1 将 `CompilerBus` 从 `Subject<_, Infailable>` 改为 `Subject<_, E: crate::Error>`，通过 `cargo check` 验证
- [ ] 6.2 更新所有 `CompilerBus` 相关结构体（`EvalContext` 等）为泛型错误类型，通过 `cargo check` 验证
- [ ] 6.3 消除 `collect_css` 中的 `Rc<RefCell<CssBuffer>>`：用 `Observable::create` + `scan(CssBuffer)` 替代，含单元测试验证 `@media` 包装
- [ ] 6.4 实现 `VarEvent::Bind` 在泛型 `E` 通道上的传播，含单元测试验证变量订阅

## 7. Serializer & Output

- [ ] 7.1 实现 `Serializer` 接受 `Observable<CssStmt>` 产出 `Observable<String>`，支持 Expanded / Compressed / Nested 三种样式，含单元测试验证
- [ ] 7.2 实现 `OutputStyle::Compressed` 输出格式（匹配 Bootstrap dist），含单元测试验证 `"a{color:red}"`
- [ ] 7.3 实现 `OutputStyle::Expanded` 输出格式，含单元测试验证 `"a {\n  color: red;\n}\n"`
- [ ] 7.4 实现 `from_string(&str, &Options) -> Result<String>` 公共 API（替代当前 stub），含单元测试验证基本编译
- [ ] 7.5 实现 `from_path(&Path, &Options) -> Result<String>` 公共 API 和 `Fs` trait，含单元测试验证文件读取 + 编译

## 8. Built-in Modules

- [ ] 8.1 实现 `src/builtin/color.rs`（darken / lighten / mix / rgba / transparentize / opacify），含单元测试验证 darken 0% = 原色
- [ ] 8.2 实现 `src/builtin/math.rs`（clamp / max / min / round / abs / percentage），含单元测试验证 percentage(0.5) = 50%
- [ ] 8.3 实现 `src/builtin/string.rs`（index / length / slice / to-upper / to-lower），含单元测试验证 string.index
- [ ] 8.4 实现 `src/builtin/list.rs`（append / index / length / nth / join），含单元测试验证 list.nth
- [ ] 8.5 实现 `src/builtin/map.rs`（get / has-key / keys / merge / remove / values），含单元测试验证 map.get
- [ ] 8.6 实现 `src/builtin/mod.rs` 统一注册接口 `register_all(scope: &mut Scope)`，通过 `cargo test` 验证全部内置函数可调用

## 9. Bootstrap E2E 验证

- [ ] 9.1 创建 `tests/bootstrap_dist_verify_test.rs`，让 `from_string("body{margin:0}")` 产出与 Grass 一致的压缩输出，作为基线验证
- [ ] 9.2 编译 Bootstrap `_variables.scss` 主入口，手动验证 `$primary` 等变量正确赋值
- [ ] 9.3 编译 Bootstrap `_buttons.scss` 组件，验证 `.btn { ... }` 规则正确生成
- [ ] 9.4 编译 Bootstrap 全量入口 `bootstrap.scss`，与 `dist/css/bootstrap.css` 进行逐字节比对，差异数应为 0
- [ ] 9.5 所有 Cargo test 通过（包括旧有 27 个测试 + 新增 Lexer/Parser/Lowering/Evaluator/Bootstrap 测试）

## 10. Open Questions Resolution

- [ ] 10.1 回溯 Decision 2 的 Q1：验证 `Observable::create` + `scan(ParserState)` 前瞻无需额外 backpressure 策略（或补充策略）
- [ ] 10.2 回溯 Decision 3 的 Q2：验证 `Value` 枚举设计足以表达 Sass 所有值类型（如不满足则扩展 Value）
- [ ] 10.3 回溯 Decision 的 Q3：确认 `SassAstNode` 不需要保留注释节点（Bootstrap 编译验证）
