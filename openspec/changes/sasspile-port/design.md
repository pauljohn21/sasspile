# Design

## Context

rx-scss 已有完整的响应式框架（已通过现有测试验证）：

```text
source → scan(LexerState) → TokenStream → parse_stream() → AstStream → eval_stream() → CssStream → serialize() → CSS
```

各模块已使用 rxrust Observable 连接：lexer 用 `Shared::create` 产出 Token 流，parser 订阅 Token 产出 AstNode 流，evaluator 用 CompilerBus 管理状态。

sasspile 的**算法内容**（scanner 状态机、Pratt 解析、eval 分派、CssNode 序列化、内建函数）需要填充进这个框架——**架构不变，内容升级**。

## 现有框架概览

| 模块 | 文件 | 当前实现 | 当前限制 |
|------|------|---------|---------|
| Lexer | `lexer/state.rs` + `lexer/mod.rs` | `scan()` + `Shared::create` → TokenStream | 字符串转义不全、注释不完整、数字无单位 |
| Parser | `parser/state.rs` + `parser/mod.rs` | `parse_stream()` → AstStream | 无 Pratt 解析（优先级不做）、@rule 不全 |
| Evaluator | `eval/mod.rs` + `runtime.rs` + `bus.rs` | CompilerBus + arithmetic scope_id | 无内建函数、选择器组合不完整 |
| Serializer | `serialize/mod.rs` | 纯函数 `serialize(stmts, opts) -> String` | 仅基础 Expanded/Compressed、无 @media 合并 |
| Pipeline | `pipeline.rs` | `from_string()` 串联全部 | 无 include_path 解析 |

## Goals / Non-Goals

**Goals:**
- 保留 rx-scss 现有响应式架构（scan Observable、CompilerBus、Shared::create/from_iter 管线）
- 将 sasspile 的算法内容填充进对应模块
- 编译通过现有全部测试（builder_test 15 + value_test 11 + 其他）
- 编译 `bootstrap/scss/bootstrap.scss` 产出有效 CSS

**Non-Goals:**
- 不改架构为 scan(EvalState)（现有 CompilerBus 足够）
- 不引入 17 色彩空间（保持 `Value::Color(u8,u8,u8,u8)`）
- 不做 ObservablePipe trait 的管道重构
- 不做 CompilerService 多播层

## Decisions

### Decision 1: Lexer 内容补全 — 延伸到 lexer/state.rs

**选择**: 在现有 `LexerState::feed()` 中增加 sasspile scanner 算法：
- 字符串：双引号/单引号 + 转义符（`\"` `\\` `\n`）+ `#{}` 插值追踪
- 注释：silent `//`（抑制不输出）+ preserved `/*! */`（输出 Comment）
- 数字：`16px`, `1.5em`, `1e3` — 单位作为 Token::Number 的 Option<String> 字段
- 操作符：`==` `!=` `<=` `>=` 双字符识别（当前只有单字符）
- 特殊标志：`!default` `!global` `!important`

**文件**: `lexer/state.rs`（扩展现有文件，≤500 行）

### Decision 2: Parser 内容补全 — 增加 Pratt + @rule

**选择**: 在现有 `parser/mod.rs` 中增加：
- Pratt 表达式解析器（优先级: or<and<cmp<add<sub<mul/div<unary<primary），替换现有 `parse_value()` 简单字面量
- `@use` / `@forward` / `@import` 模块规则解析（产出无 CSS 节点，仅注册）
- `@at-root` / `@content` / `@error` / `@keyframes` 解析
- `!default` `!global` 标志解析 → `VarFlags { is_default, is_global }`
- 选择器内插值 `#{}` 解析

**文件**: `parser/mod.rs` + `parser/state.rs`（扩展现有文件）

### Decision 3: Evaluator 内容补全 — 内建函数 + 选择器组合

**选择**: 在现有 `eval/mod.rs` 中增加：
- 内建函数分派：`call_builtin(name, args) -> Option<Value>` — 覆盖 Bootstrap 所需的 math/string/list/map/color 函数
- 选择器组合：`&` 引用替换 + descendant 组合
- `@at-root` 求值：hoist 子规则到根层级
- `@media` query 合并
- function @return 求值（当前 FunctionCall 未走完整 eval）

**不改为 scan(EvalState)**: 现有 CompilerBus 已能处理变量/作用域/mixin/function 注册。scope_id 算术模式改为 parent_idx 链是可选改进，非必须。

**文件**: `eval/mod.rs`（扩展现有文件，若超 500 行则拆出 `eval/builtin.rs`）

### Decision 4: Serializer 内容补全 — @media 合并 + 三种 style

**选择**: 在现有 `serialize/mod.rs` 中增加：
- Nested 输出风格（当前只区分 Expanded/Compressed）
- @media 相邻同 query 合并
- @at-root hoist 语义（AtRoot / AtRootDirect 作为新 CssStmt 变体）
- 重要注释 `/*! */` 保留

**如有必要新增 CssStmt 变体**（AtRoot / AtRootDirect / Import / Raw），保持向后兼容。

**文件**: `serialize/mod.rs`（扩展现有文件）

### Decision 5: 类型最小化扩展

**选择**: 仅扩展 sasspile 内容所需的最小类型变更：

```rust
// types.rs 中新增
struct VarFlags { is_default: bool, is_global: bool }

// AstNode 变体新增
VariableDecl { name, value, flags: VarFlags }  // 替换原 scope_id
AtRoot { query: Option<String>, inner }
Extend(String)
Error(Box<AstNode>)
Content
Forward(String, Option<String>)

// CssStmt 变体新增（如需 @at-root）
AtRoot { inner: Vec<CssStmt> }
AtRootDirect { selector: String, inner: Vec<CssStmt> }
Import(String)
Raw(String)

// Token 变体新增
AtImport, AtAtRoot, AtContent, AtError, Atkeyframes, AtFontFace, AtPage, AtCharset
```

**不扩展**: 不引入 ColorSpace（17 空间）、CalcExpr、ArgList、SelectorValue 等 sasspile 复杂类型。Bootstrap 编译仅需 `Value::Color(u8,u8,u8,u8)` + 简单颜色函数。

### Decision 6: 内建函数清单（Bootstrap 所需）

数学: `math.abs` `math.ceil` `math.floor` `math.round` `math.percentage` `math.max` `math.min` `math.random`

字符串: `string.quote` `string.unquote` `string.to-upper-case` `string.to-lower-case` `string.length` `string.index`

列表: `list.length` `list.nth` `list.join` `list.append` `list.separator`

Map: `map.get` `map.merge` `map.keys` `map.values` `map.has-key`

颜色: `rgb` `rgba` `hsl` `hsla` `red` `green` `blue` `alpha` `lighten` `darken` `saturate` `desaturate` `adjust-hue` `mix` `invert` `complement` `grayscale` `opacify` `transparentize` `scale-color` `adjust-color` `change-color`

元: `meta.type-of` `meta.inspect` `meta.unit` `if`

**文件**: `eval/builtin.rs`（直接填充进原 eval 模块）

### Decision 7: 文件结构（保持现有 + 增量添加）

```
src/
├── lexer/
│   ├── mod.rs          (保持 scan + Shared::create)
│   └── state.rs        (扩展: sasspile scanner 算法)
├── parser/
│   ├── mod.rs          (扩展: Pratt + @rule 全量)
│   └── state.rs        (保持 ParserState)
├── eval/
│   ├── mod.rs          (扩展: 内建函数分派 + 选择器组合)
│   └── builtin.rs      (新增: ~30 个内建函数实现)
├── bus.rs              (保持 CompilerBus)
├── runtime.rs          (保持 EvalContext — 可选改进为 scope chain)
├── types.rs            (最小扩展: VarFlags + 少量 AstNode 变体)
├── serialize/
│   └── mod.rs          (扩展: @media 合并 + Nested style + @at-root)
├── pipeline.rs         (保持 from_string + 可选 include_path)
├── builder.rs          (保持)
├── lib.rs              (保持)
├── observable_ext.rs   (保持 ObservablePipe trait)
└── error.rs            (可选: thiserror CompileError)
```

## Migration Plan

### Phase 1: Lexer 补全（1 天）
1. 扩展 `lexer/state.rs` feed() — 字符串转义、注释、数字单位
2. 更新 `lexer/mod.rs` scan() 处理 flush
3. 验证: `cargo test --test lexer_test`

### Phase 2: Parser 补全（1-2 天）
1. 增加 Pratt 表达式解析器
2. 扩展 @rule 解析（use/forward/import/at-root/error/keyframes）
3. 增加 VarFlags
4. 验证: `cargo test --test parser_test`

### Phase 3: Evaluator 补全（2-3 天）
1. 增加 `call_builtin()` 分派
2. 增加 `eval/builtin.rs` 内建函数
3. 修正 function @return 求值
4. 增加选择器组合 (& 引用)
5. 验证: `cargo test --test eval_test`

### Phase 4: Serializer 补全（1 天）
1. 增加 Nested style
2. 增加 @media 合并
3. 增加 @at-root hoist
4. 验证: 编译简单 SCSS

### Phase 5: Bootstrap 验证（1-2 天）
1. 运行 `bootstrap/scss/bootstrap.scss`
2. 迭代修复缺失的内建函数 / 解析错误
3. 验证: 产出 > 100KB CSS，含 .btn/.container 选择器

总计: ~6-9 天

## Risks / Trade-offs

| 风险 | 影响 | 应对 |
|------|------|------|
| Pratt 解析器复杂度 | 高度 3-5 层时易出错 | 用测试驱动：1 + 2 * 3 → 正确 AST |
| Bootstrap 需要的内建函数数量 | 遗漏任何函数 → 编译失败 | 逐个补充 + Bootstrap 编译迭代测试 |
| 类型扩展导致现有代码冲突 | 新 AstNode 变体使现有 match 报错 | 增量添加，每次添加后立即编译 |
| 现有 CompilerBus scope_id 算术模式 | 深层嵌套可能溢出 | Phase 3 可选改为 parent_idx 链 |
