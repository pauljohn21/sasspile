# Tasks — rc-env-perf

## 1. 回退 Rc 改动

- [ ] 1.1 Env 字段从 `Rc<HashMap>` 恢复为 `HashMap`
- [ ] 1.2 ModuleExports 字段同样恢复
- [ ] 1.3 删除所有 `Rc::make_mut` 调用
- [ ] 1.4 `cargo build` 编译通过

## 2. Env 方法改 &mut

- [ ] 2.1 `bind(&mut self, ...)` — 直接 insert
- [ ] 2.2 `define_local_mixin(&mut self, ...)` — 直接 insert
- [ ] 2.3 `define_forwarded_mixin(&mut self, ...)` — 直接 insert
- [ ] 2.4 `define_local_function(&mut self, ...)` — 直接 insert
- [ ] 2.5 `define_forwarded_function(&mut self, ...)` — 直接 insert
- [ ] 2.6 `incr_depth(&mut self)` — 直接 +1
- [ ] 2.7 `add_module(&mut self, ...)` — 直接 push
- [ ] 2.8 `add_namespace(&mut self, ...)` — 直接 insert
- [ ] 2.9 `set_content(&mut self, ...)` — 直接赋值
- [ ] 2.10 `with_base_path(&mut self, ...)` — 直接赋值
- [ ] 2.11 `with_load_paths(&mut self, ...)` — 直接赋值
- [ ] 2.12 `with_selector(&mut self, ...)` — 直接赋值
- [ ] 2.13 `add_extend(&mut self, ...)` — 直接 push
- [ ] 2.14 `with_module_cache(&mut self, ...)` — 直接赋值
- [ ] 2.15 `with_plain_css(&mut self, ...)` — 直接赋值

## 3. eval_xxx 方法签名改 &mut Env -> Vec

- [ ] 3.1 `eval_nodes(nodes, &mut Env) -> Result<Vec<CssNode>>`
- [ ] 3.2 `eval_node(node, &mut Env) -> Result<Vec<CssNode>>`
- [ ] 3.3 `eval_rule(selector, body, &mut Env) -> Result<Vec<CssNode>>`
- [ ] 3.4 `eval_if(branches, else_body, &mut Env) -> Result<Vec<CssNode>>`
- [ ] 3.5 `eval_for(var, from, to, inclusive, body, &mut Env) -> Result<Vec<CssNode>>`
- [ ] 3.6 `eval_each(vars, list, body, &mut Env) -> Result<Vec<CssNode>>`
- [ ] 3.7 `eval_while(cond, body, &mut Env) -> Result<Vec<CssNode>>`
- [ ] 3.8 `eval_include(name, args, content, &mut Env) -> Result<Vec<CssNode>>`
- [ ] 3.9 `exec_mixin(mixin, args, content, &mut Env) -> Result<Vec<CssNode>>`
- [ ] 3.10 `bind_params(params, args, &mut Env) -> Result<()>`
- [ ] 3.11 `call_user_function(func, pos_args, kw_args, &mut Env) -> Result<Value>`
- [ ] 3.12 `eval_variable(name, value, flags, &mut Env) -> Result<Vec<CssNode>>`
- [ ] 3.13 `eval_use(url, ns, star, config, &mut Env) -> Result<Vec<CssNode>>`
- [ ] 3.14 `eval_forward(url, prefix, config, &mut Env, show, hide) -> Result<Vec<CssNode>>`
- [ ] 3.15 `eval_import(url, modifier, &mut Env) -> Result<Vec<CssNode>>`
- [ ] 3.16 `eval_at_root(query, body, &mut Env) -> Result<Vec<CssNode>>`
- [ ] 3.17 `eval_at_rule(name, params, body, &mut Env) -> Result<Vec<CssNode>>`
- [ ] 3.18 `load_module(path, config, &mut Env) -> Result<ModuleExports>`
- [ ] 3.19 `load_import(path, &mut Env) -> Result<Vec<CssNode>>`

## 4. @content 作用域切换

- [ ] 4.1 `exec_mixin` 中用 `mem::swap` 交换 mixin_env 和 content_env
- [ ] 4.2 `Node::Content` 处理改为在交换后的 env 上执行
- [ ] 4.3 确保 @content 执行后正确恢复 mixin_env

## 5. rule.rs 作用域传播

- [ ] 5.1 进入规则体前用 `mem::take` 保存 local_vars
- [ ] 5.2 规则体执行（&mut Env 上直接改）
- [ ] 5.3 结束后恢复 local_vars（局部变量不传播）
- [ ] 5.4 保留 global_writes 传播
- [ ] 5.5 保留新增 mixin/function 传播
- [ ] 5.6 保留 @extend 传播
- [ ] 5.7 保留 namespaces 传播

## 6. @import AST 缓存

- [ ] 6.1 Env 新增 `ast_cache: HashMap<PathBuf, Rc<Ast>>` 字段
- [ ] 6.2 `load_import` 先查缓存，命中跳过 read+lex+parse
- [ ] 6.3 `load_module` 同样查缓存
- [ ] 6.4 缓存随 Env 传播（@import 内联时继承调用者的缓存）

## 7. module_helpers.rs 改 &mut

- [ ] 7.1 `bind_exports(&mut Env, ...)` — 直接操作
- [ ] 7.2 `merge_module_cache(&mut Env, ...)` — 直接操作
- [ ] 7.3 `builtin_module_exports` 保持不变（构造 ModuleExports）

## 8. 编译验证 + 回归测试

- [ ] 8.1 `cargo build` 编译通过
- [ ] 8.2 `cargo test --test compile_test` — 43/43
- [ ] 8.3 `cargo test --test stage_test` — 10/10
- [ ] 8.4 `cargo test --test ast_test` — 8/8
- [ ] 8.5 `cargo test --test common_test` — 5/5
- [ ] 8.6 `cargo test --test bs_spec` — 15/15 + ≤10s
- [ ] 8.7 `cargo test --test ep_full` — 121/121 + ≤15s
- [ ] 8.8 `cargo test --test sass_spec_full` — 基线 2828/5362 不回归

## 9. 更新文档

- [ ] 9.1 README.md — 去掉"纯函数式"描述，改为 Rust 所有权管线
- [ ] 9.2 design.md / proposal.md — 已更新
- [ ] 9.3 AGENTS.md — 更新架构描述
- [ ] 9.4 `codegraph sync` 同步索引
