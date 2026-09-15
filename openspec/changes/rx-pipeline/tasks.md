# Tasks: rx-pipeline

## Phase I: 依赖 & 骨架 ✅

- [x] **1.1 添加 rxrust 依赖**
  - `Cargo.toml`: `rxrust = "1.0.0-rc.5"` 升为核心依赖
  - `cargo build` 通过

- [x] **1.2 创建 `src/pipeline/mod.rs`**
  - `compile(source: &str) -> LocalBoxedObservable<char, Infallible>`
  - `pipe` 辅助函数

## Phase II: 词素发射（tokenize）✅

- [x] **2.1 创建 `src/lex/stream.rs`**
  - `LexerState { buf, mode }` + `feed(state, ch) -> (LexerState, Vec<Token>)` 纯函数
  - 从 `scanner.rs` 迁移词素识别算法

- [x] **2.2 pipeline 接入**
  - `scan(LexerScanner) → flat_map → flat_map` 模式
  - flush 空白 + trim_end 处理

- [x] **2.3 测试验证**
  - `tests/pipeline_test.rs` 10 个测试全通过
  - 与旧 Lexer 逐 token 序列对比通过

## Phase III: 结构累积（parse）

- [ ] **3.1 创建 `src/parse/stream.rs`**
  - `ParseState { brace_stack, current_buffer }` scan 累加器
  - `absorb(state, token) -> (ParseState, Vec<Node>)` 纯函数
  - 从 `at_rules.rs` 迁移 brace 嵌套结构规则

- [ ] **3.2 基本结构**
  - `LBrace` 压栈新 frame
  - `RBrace` 弹栈，emit 完整 Rule node
  - selector 解析（从 buffer 提取）
  - property 累积到 frame.props

- [ ] **3.3 at-rules 结构**
  - `@media` / `@supports` — 带条件的嵌套 block
  - `@keyframes` — 百分比 stops
  - `@font-face`, `@page`, `@charset`, `@namespace`

- [ ] **3.4 flow control 结构**
  - `@if` / `@else if` / `@else` — 条件 node
  - `@for` — 循环 node
  - `@each` — 列表迭代 node
  - `@while` — 条件循环 node

- [ ] **3.5 SCSS 扩展**
  - `@mixin` / `@include` — node 定义和调用
  - `@function` — 函数定义 node
  - `@use` / `@forward` / `@import` — 模块 node
  - 嵌套属性（`font: { weight: bold }`）

- [ ] **3.6 测试验证**
  - `test_parse_simple_rule`: `a { color: red }`
  - `test_parse_nested_rules`: `.a { .b { color: red } }`
  - `test_parse_media_query`: `@media screen { a { color: red } }`
  - `test_parse_mixin_include`: `@mixin foo { ... }` + `@include foo`
  - 与旧 Parser AST 对比

## Phase IV: 流式求值（evaluate）

- [ ] **4.1 创建 `src/eval/stream.rs`**
  - `EvalState { env }` scan 累加器
  - `eval_node(state, node) -> (EvalState, Vec<CssNode>)` 纯函数
  - 从 `env_impl.rs` / `control_flow.rs` 迁移

- [ ] **4.2 声明求值**
  - `Define(name, expr)` — eval expr → bind to env
  - `Prop(name, expr)` — eval expr → emit CssNode::Decl

- [ ] **4.3 规则求值**
  - `Rule { selector, body }` — enter_scope → eval body → exit_scope
  - 子作用域继承父 scope chain（现有 Env 设计已支持）

- [ ] **4.4 @if/@else 条件**
  - 条件表达式求值 → is_truthy
  - true 分支：eval then_body → emit CssNodes
  - false 分支：eval else_body 或空

- [ ] **4.5 @for/@each 展开**
  - @for: range 求值 → 每次迭代 clone body + bind var → eval → emit
  - @each: list 求值 → 每次迭代 bind → eval → emit
  - 展开的 nodes 注入 emissions Vec

- [ ] **4.6 @while 循环**
  - 条件求值 → truthy 时展开 body → 循环

- [ ] **4.7 @include / mixin 展开**
  - MixinDef 在 env 中注册
  - @include 时 lookup mixin → bind params → eval body → emit
  - @content 块传递（需要嵌套订阅，特殊处理）

- [ ] **4.8 @function 调用**
  - FunctionDef 在 env 中注册
  - 调用时 bind params → eval body → 返回计算值

- [ ] **4.9 内建函数透传**
  - builtin 函数不需要改动（已经是纯函数）
  - `call_builtin(name, args)` 直接调用
  - 验证：所有 color/math/string/list/map/selector builtins 正常工作

- [ ] **4.10 测试验证**
  - `test_eval_variable`: `$x: 1; a { width: $x }`
  - `test_eval_if_true` / `test_eval_if_false`
  - `test_eval_for_loop`: `@for $i from 1 through 3`
  - `test_eval_each`: `@each $item in (a, b, c)`
  - `test_eval_mixin`: `@mixin foo` + `@include foo`
  - `test_eval_nested_scope`: 变量遮蔽与恢复

## Phase V: 序列化 + 全管线

- [ ] **5.1 创建 `src/css/stream.rs`**
  - `render_node(node: &CssNode) -> String` 纯函数
  - `serialize(nodes) -> Observable<char>` flat_map 展开
  - 从 `css/serialize.rs` 迁移格式规则

- [ ] **5.2 CssNode 定义**
  - `Decl { prop, value }` — CSS 属性声明
  - `Rule { selector, body }` — CSS 规则块
  - `AtRule { name, params, body }` — at-rule
  - `Comment(text)` — CSS 注释

- [ ] **5.3 全管线串联**
  - compile() 整合 tokenize → parse → evaluate → serialize
  - 端到端编译：`"a { color: red }"` → `"a {\n  color: red;\n}\n"`

- [ ] **5.4 便利函数**
  - `compile_to_string(input) -> String`（用于测试）
  - `compile_to_string(input, style)` 支持 expanded/compressed

- [ ] **5.5 测试**
  - `test_e2e_basic_rule`
  - `test_e2e_nested_rules`
  - `test_e2e_variables`
  - `test_e2e_if_for_each`

## Phase VI: 模块流合并

- [ ] **6.1 创建 `src/pipeline/module_stream.rs`**
  - `ModuleEvalState { env, module_cache: HashMap<PathBuf, Observable<Node>> }`
  - `resolve_use(path, base_path) -> Observable<Node>` — 文件加载 + tokenize + parse
  - 在 eval_node 中处理 Node::Use → flat_map 依赖流

- [ ] **6.2 @forward 处理**
  - 类似 @use，但只导出不直接 emit CssNodes

- [ ] **6.3 循环检测**
  - imports_seen 集合检测循环依赖
  - 错误传播到 Observable 流

- [ ] **6.4 测试**
  - `test_use_basic`
  - `test_use_with_override`: `@use 'lib' with ($x: 2)`
  - `test_forward`
  - `test_circular_dependency_error`

## Phase VII: OTel 操作符 ✅

- [x] **7.1 创建 `src/pipeline/otel_op.rs`**
  - `otel_span<T>(name)` Map 操作符
  - `feature = "otel"` 条件编译

- [x] **7.2 管线接入**（占位，全管线完成后统一接入）
  - `.pipe(otel_span("lex"))` / `"parse"` / `"eval"` / `"serialize"`

## Phase VIII: 验证 & 清理

- [ ] **8.1 核心测试全绿**
  - `cargo test --test compile_test` —— 46/46
  - `cargo test --test reactor_test` —— 14/14
  - `cargo test --test pipeline_test` —— 10/10+
  - `cargo test --test stage_test` —— 8/8
  - `cargo test --test common_test` —— 5/5
  - `cargo test --test interp_test` —— 15/15
  - `cargo test --test bs_spec -- --nocapture` —— 15/15
  - `cargo test --test ep_full -- --nocapture` —— 121/121

- [ ] **8.2 sass-spec 基线**
  - `SPEC_STORE_CMD=run cargo test --test spec_store -- --nocapture` ≥ 7620

- [ ] **8.3 clippy 检查**
  - `cargo clippy --all-targets` —— 零错误

- [ ] **8.4 旧代码清理**
  - 删除旧 pipeline 包装代码（如果全管线替代）
  - 保留经典 Reactor 路径作为 fallback（通过 feature gate）

## 验证清单

```bash
# 编译
cargo build                               # 零错误（rxrust 是默认依赖）
cargo build --features otel               # OTel 模式

# 测试
cargo test --test pipeline_test           # pipeline 专用
cargo test --test compile_test            # 46/46
cargo test --test reactor_test            # 14/14
cargo test --test interp_test             # 15/15

# sass-spec
SPEC_STORE_CMD=run cargo test --test spec_store -- --nocapture

# lint
cargo clippy --all-targets                # 零错误
```

## 关键风险与缓解

| 风险 | 缓解 |
|------|------|
| scan 累加器 clone 开销 | LexerState / ParseState 体积小（<100 字节），clone 开销可忽略 |
| @content 闭包嵌套订阅 | 使用 Local 调度器 + 手动递归流 |
| 错误传播中断流 | scan 闭包返回 Result，Err 时 downstream error |
| round-trip flush 空白 | trim_end + 只在顶层 compile 添加 flush 字符 |
