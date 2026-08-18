## 1. ModuleResolver trait 与默认实现

- [x] 1.1 创建 `src/resolver/mod.rs`，定义 `ModuleResolver` trait（`resolve` 方法返回 `ResolvedModule`）
- [x] 1.2 定义 `ResolvedModule` 结构体（`ast: Vec<Stmt>`, `is_css: bool`, `raw_content: Option<String>`, `source_path: PathBuf`）
- [x] 1.3 实现 `FileResolver` 默认实现，封装 tokenize → parse 流程，带模块缓存（`HashMap<PathBuf, Vec<Stmt>>`）和循环引用检测（`HashSet<PathBuf>`）
- [x] 1.4 为 `FileResolver` 的 `resolve` 方法添加 tracing span（`module_resolve`, stage=`eval`, 字段 `url`/`resolved_path`/`is_css`）
- [x] 1.5 在 `src/lib.rs` 中 `pub mod resolver;` 并 re-export `ModuleResolver` 和 `FileResolver`

## 2. 消除 eval → parser 循环依赖

- [x] 2.1 修改 `evaluate` 和 `evaluate_with_dir` 签名，接收 `&mut dyn ModuleResolver` 参数
- [x] 2.2 重构 `eval_use_rule`（`src/eval/mod.rs`），将直接 `crate::lexer::tokenize` + `crate::parser::parse` 调用替换为 `resolver.resolve(url, base_dir)`
- [x] 2.3 重构 `eval_import_rule`（`src/eval/atrule.rs`），同样替换为 `resolver.resolve` 调用
- [x] 2.4 修改 `compile`/`compile_file`（`src/lib.rs`），创建 `FileResolver` 实例并传入 evaluate
- [x] 2.5 重构 `eval_interpolation_expr`（`src/eval/interp.rs`），消除直接 `crate::lexer::tokenize` + `crate::parser::Parser` 调用，改为通过 `resolver.parse_expr` 回调
- [x] 2.6 用 CodeGraph 验证：`codegraph callers parse` 确认 eval 层不再直接调用 `parser::parse`

## 3. @use with 配置注入

- [x] 3.1 在 `eval_use_rule` 中实现 config 参数处理：在调用者环境中求值 config 表达式，得到 `(String, Value)` 对列表
- [x] 3.2 在通过 resolver 获取模块 AST 后，创建子环境并预设配置变量（非 `!default`，直接覆盖）
- [x] 3.3 在子环境中求值模块 AST，使模块内部的 `!default` 变量被配置值覆盖
- [x] 3.4 添加 tracing span（`use_config_inject`, stage=`eval`, 字段 `config_count`/`module`）
- [x] 3.5 在 `tests/` 创建 `tests/use_config.rs` 测试文件，验证 `@use with` 语法（单个配置、多个配置、引用已有变量、`!default` 覆盖场景）

## 4. 模块求值结果缓存（ModuleCache）

- [x] 4.1 创建 `src/eval/module_cache.rs`，定义 `ModuleCache` 和 `EvaluatedModule` 结构体
- [x] 4.2 在 `evaluate` / `evaluate_with_dir` 中创建 `ModuleCache` 实例，传入 `eval_stmts` → `eval_stmt` → `eval_use_rule`
- [x] 4.3 重构 `eval_use_rule`：检查 `ModuleCache`，有缓存则直接取 `EvaluatedModule`，跳过 `eval_stmts`
- [x] 4.4 无缓存时：`eval_stmts` 后收集公开成员（vars/funcs/mixins）+ CSS 输出，存入 `ModuleCache`
- [x] 4.5 后续 `@use` 同一模块时不重复输出 CSS（只输出第一次的 `css_output`）
- [x] 4.6 处理 `@use "module" with (...)` 的缓存约束：只有在第一次加载时才允许 `with`，后续报错
- [x] 4.7 添加 tracing span（`module_cache_hit`/`module_cache_store`, stage=`eval`）
- [x] 4.8 在 `tests/` 创建 `tests/module_cache.rs` 测试文件，验证模块只求值一次（同一模块被多个文件 @use 时变量值一致）

## 5. @forward 指令实现

- [x] 5.1 实现 `eval_forward_rule`（`src/eval/mod.rs`），替换当前的 no-op 处理
- [x] 5.2 复用 `ModuleResolver` 加载模块 + `ModuleCache` 缓存求值结果
- [x] 5.3 实现成员转发：将 `EvaluatedModule` 的公开成员注入当前模块的公开成员集合
- [x] 5.4 实现 `show`/`hide` 过滤逻辑（按成员名匹配，不区分类型）
- [x] 5.5 `@forward` 不产生 CSS 输出（与 `@use` 不同）
- [x] 5.6 添加 tracing span（`eval_forward`, stage=`eval`, 字段 `url`/`show_count`/`hide_count`）
- [x] 5.7 在 `tests/` 创建 `tests/forward_rule.rs` 测试文件，验证 `@forward` 基础语法（转发变量/函数/mixin、show 过滤、hide 过滤）

## 6. Env 作用域安全重构

- [x] 6.1 在 `Env`（`src/env.rs`）中实现 `with_child_scope<R>(&mut self, f: impl FnOnce(&mut Env) -> R) -> R` 方法
- [x] 6.2 添加 tracing span（`env_child_scope`, stage=`eval`, 字段 `scope_depth`/`caller`）
- [x] 6.3 重构 `call_user_function`（`src/eval/func.rs`），将 `mem::replace` + `take().unwrap()` 替换为 `with_child_scope`
- [x] 6.4 重构 `eval_include`（`src/eval/atrule.rs`），同样替换为 `with_child_scope`
- [x] 6.5 运行全部现有测试确认无回归：`cargo test`

## 7. 验证与收尾

- [x] 7.1 运行 `RUST_LOG=info cargo test --test bootstrap_otel -- --nocapture` 验证 Bootstrap 编译进展
- [x] 7.2 运行 `RUST_LOG=info cargo test --test element_plus_otel -- --nocapture` 验证 Element Plus 编译进展
- [x] 7.3 运行全部核心测试：`cargo test`，确认无回归
- [x] 7.4 用 CodeGraph 重新同步：`codegraph sync` 并检查循环依赖是否消除
- [x] 7.5 更新 `README.md` 中 `@use`/`@forward` 功能描述，标注配置注入和模块缓存已支持
- [x] 7.6 用 `codegraph impact eval_use_rule` 确认变更影响范围已全覆盖
