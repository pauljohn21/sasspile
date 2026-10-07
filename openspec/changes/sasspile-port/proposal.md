# Proposal

## Why

rx-scss 当前只有 ~500 行基础 SCSS 骨架（变量/嵌套/控制流/mixin），缺少完整 Lexer/Parser、内建函数、模块系统、`@extend`/`@at-root`、色彩空间。sasspile 已完成 ~60% sass-spec（含 Bootstrap 和 Element-Plus 全量通过），拥有成熟的函数式编译器核心。为了复用 sasspile 已验证的编译语义，同时保持 rx-scss 的 rxrust 响应式哲学，需要将 sasspile 的算法**以响应式风格完全重写**移植到 rx-scss——算法来自 sasspile，实现风格遵循 rxrust Observable 管线。

## What Changes

- **完整重写所有模块** — Lexer/Parser/Evaluator/Serializer 全部从零重写，算法参考 sasspile，实现风格为 rxrust Observable 流
- **新增完整 Value 类型** — 从 7 变体扩展到 ~20 变体（新增 Calc/Color(17 色彩空间)/ArgList/MixinRef/Selector 等）
- **新增完整 AstNode 语法** — 覆盖 @use/@forward/@import/@extend/@at-root/@content/@error/@keyframes 等所有 SCSS 语法
- **新增 Lexer 模块** — 移植 sasspile scanner.rs 算法，包装为 `Shared Observable<Token>` 流
- **新增 Parser 模块** — 移植 sasspile ParseStream + Pratt 表达式解析，产出 `Observable<AstNode>` 流
- **新增 Evaluator 模块** — 用 `scan(EvalState)` 实现求值状态机，`sasspile eval_nodes` 的递归语义转为响应式流
- **新增内建函数** — ~50 个内建函数（math/string/list/map/color/selector/meta），Bootstrap 所需全部覆盖
- **新增 Serializer 模块** — 支持 Expanded/Compressed/Nested 风格，@media 合并，@at-root hoist
- **新增模块系统（Phase 2）** — `@use`/`@forward`/`@import`、File Resolver、load paths
- **新增 @extend 选择器运算（Phase 2）** — 复杂选择器继承、placeholder
- **适配 Observable 调度** — 所有管线阶段通过 `Shared Observable` 连接，支持 `observe_on(ThreadPool)` 多线程调度
- **删除 CompilerBus + 算术 scope_id** — 改为 `EvalState` + `scan()` 状态机（Env 等价物）
- **删除简单 Value/AstNode/CssStmt** — 升级为 sasspile 级别的完整类型
- **升级错误系统** — `CompileError` → `thiserror` 风格详细错误（Lex/Parse/Eval/Type/Unit）
- **新增 saas-spec 全量测试** — 参考 sasspile 测试架构，搭建 sass-spec HRX 解析 + 验证测试
- **新增 Bootstrap 5.3.x 全量验证** — 编译 `bootstrap/scss/bootstrap.scss`，字节级对比
- **新增 Element-Plus 全量验证（Phase 2）** — 编译 EP 全量 SCSS，121 文件通过
- **BREAKING**: `from_string`/`from_path` 签名不变但内部类型全部升级；`CompileBuilder` API 不变但 builder 字段可能扩展

## Capabilities

### New Capabilities

- `value-system-complete`: 完整 Sass 值系统 — 20 变体 Value enum（Number/String/Color/Calc/List/Map/Bool/Null/ArgList/MixinRef/Selector 等），Display/PartialEq，色彩空间转换、calc 表达式、单位运算
- `lexer-complete`: 完整词法分析器 — 从 sasspile scanner 算法移植，覆盖字符串（单/双引号/转义）、数字（单位/科学计数）、插值 `#{}`、注释（单行/多行/ silent）、所有 @-rules、操作符、标识符（kebab-case/camelCase）
- `parser-complete`: 完整语法分析器 — Pratt 表达式解析、选择器解析、@-rule 全量解析（@use/@forward/@import/@include/@mixin/@function/@if/@for/@each/@while/@extend/@at-root/@content/@error/@warn/@debug）、嵌套规则、变量声明（!default/!global）
- `eval-complete`: 完整求值器 — `scan(EvalState)` 状态机实现 sasspile 求值语义：变量作用域链、mixin/include/function/return、控制流、选择器组合（`&` 引用）、@at-root 解析、@media/@supports 合并
- `builtin-functions`: 内建函数库 — ~50 个函数分 7 模块（math/string/list/map/color/selector/meta），提供 Bootstrap/Element-Plus 所需全部内建函数
- `serializer-complete`: 完整 CSS 序列化器 — CssNode 中间表示 + Expanded/Compressed/Nested 输出 + @media 合并 + @at-root hoist + var() null 规范化 + appearance -moz 前缀 + :not() 多参包装
- `modules`（Phase 2）: 模块系统 — `@use`/`@forward`/`@import`、File Resolver（含 load paths/partial/index 决议/循环检测）、模块缓存、命名空间管理、`with()` 配置
- `extend`（Phase 2）: @extend 选择器继承 — 复杂选择器解析/匹配/组合、placeholder 选择器、media 内 extend 限制

### Modified Capabilities

- `bootstrap-validation`: 升级为全量编译 — 从"基础功能验证"升级为"逐字节比对 Bootstrap 5.3.x dist CSS"
- `pipeline`: 管线内部实现升级 — 从简单 scan/parse/eval/serialize 升级为 sasspile 算法级别的完整管线
- `reactive-pipeline`: 求值器风格变更 — 从 `CompilerBus + Arc<Mutex>` 共享状态改为 `scan(EvalState)` 响应式状态机
- `value-system`: 值系统扩展 — 从 7 变体简单 Value 升级为 ~20 变体完整 Value 系统（含 Calc/Color 色彩空间/ArgList/MixinRef）

## Impact

- **代码**: rx-scss `src/` 从 ~500 行扩展到 ~4000 行（对齐 sasspile 代码量），`tests/` 新增 sass-spec 验证框架 + Bootstrap + Element-Plus 测试
- **依赖**: 新增 `thiserror`（错误类型），其余无新增（rxrust 已有）
- **工作区影响**: rx-scss 保持独立项目，不修改其他 workspace 成员
- **API**: `from_string`/`from_path`/`Options`/`CompileBuilder` 公共 API 不变，内部类型全部重写
- **测试策略**: 新增 `<-strategy`: 新增 `hrx_support.rs`（HRX 解析）、`sass_spec.rs`（全量框架）、`bootstrap_spec.rs`、`ep_spec.rs`；保留现有单元测试
- **风险**: 这是一次**完整的核心重写**，应以 feature flag 或 git 分支管理，确保现有 70+ 测试在重构期间仍可验证基础功能
