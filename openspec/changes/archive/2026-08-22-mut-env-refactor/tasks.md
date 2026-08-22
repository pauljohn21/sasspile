# Tasks — mut-env-refactor

## 1. 回退 Rc 改动

- [ ] 1.1 Env 字段从 `Rc<HashMap>` 恢复为 `HashMap`
- [ ] 1.2 ModuleExports 字段同样恢复
- [ ] 1.3 删除所有 `Rc::make_mut` 调用
- [ ] 1.4 `cargo build` 编译通过

## 2. Env 方法改 move（self -> Self）

- [ ] 2.1 `bind(mut self, ...) -> Self`
- [ ] 2.2 `define_local_mixin(mut self, ...) -> Self`
- [ ] 2.3 `define_forwarded_mixin(mut self, ...) -> Self`
- [ ] 2.4 `define_local_function(mut self, ...) -> Self`
- [ ] 2.5 `define_forwarded_function(mut self, ...) -> Self`
- [ ] 2.6 `incr_depth(mut self) -> Self`
- [ ] 2.7 `add_module(mut self, ...) -> Self`
- [ ] 2.8 `add_namespace(mut self, ...) -> Self`
- [ ] 2.9 `set_content(mut self, ...) -> Self`
- [ ] 2.10 `with_base_path(mut self, ...) -> Self`
- [ ] 2.11 `with_load_paths(mut self, ...) -> Self`
- [ ] 2.12 `with_selector(mut self, ...) -> Self`
- [ ] 2.13 `add_extend(mut self, ...) -> Self`
- [ ] 2.14 `with_module_cache(mut self, ...) -> Self`
- [ ] 2.15 `with_plain_css(mut self, ...) -> Self`

## 3. eval_xxx 方法签名改 move（Env -> (Vec, Env)）

- [ ] 3.1 `eval_nodes(nodes, mut env: Env) -> Result<(Vec<CssNode>, Env)>`
- [ ] 3.2 `eval_node(node, mut env: Env) -> Result<(Vec<CssNode>, Env)>`
- [ ] 3.3 `eval_rule(selector, body, env: Env) -> Result<(Vec<CssNode>, Env)>`
- [ ] 3.4 `eval_if(branches, else_body, env: Env) -> Result<(Vec<CssNode>, Env)>`
- [ ] 3.5 `eval_for(var, from, to, body, env: Env) -> Result<(Vec<CssNode>, Env)>`
- [ ] 3.6 `eval_each(vars, list, body, env: Env) -> Result<(Vec<CssNode>, Env)>`
- [ ] 3.7 `eval_while(cond, body, env: Env) -> Result<(Vec<CssNode>, Env)>`
- [ ] 3.8 `eval_include(name, args, content, env: Env) -> Result<(Vec<CssNode>, Env)>`
- [ ] 3.9 `exec_mixin(mixin, args, content, env: Env) -> Result<(Vec<CssNode>, Env)>`
- [ ] 3.10 `bind_params(params, args, env: Env) -> Result<Env>`
- [ ] 3.11 `call_user_function(func, pos_args, kw_args, env: Env) -> Result<(Value, Env)>`
- [ ] 3.12 `eval_variable(name, value, flags, env: Env) -> Result<(Vec<CssNode>, Env)>`
- [ ] 3.13 `eval_use(url, ns, star, config, env: Env) -> Result<(Vec<CssNode>, Env)>`
- [ ] 3.14 `eval_forward(url, prefix, config, env: Env, show, hide) -> Result<(Vec<CssNode>, Env)>`
- [ ] 3.15 `eval_import(url, modifier, env: Env) -> Result<(Vec<CssNode>, Env)>`
- [ ] 3.16 `eval_at_root(query, body, env: Env) -> Result<(Vec<CssNode>, Env)>`
- [ ] 3.17 `eval_at_rule(name, params, body, env: Env) -> Result<(Vec<CssNode>, Env)>`
- [ ] 3.18 `load_module(path, config, env: Env) -> Result<(ModuleExports, Env)>`
- [ ] 3.19 `load_import(path, env: Env) -> Result<(Vec<CssNode>, Env)>`

## 4. eval_nodes 改 move 赋值

- [ ] 4.1 `for` 循环中 `env = new_env`（move 赋值，零 clone）
- [ ] 4.2 `try_fold` 去掉 `env.clone()`

## 5. load_import / load_module 改 move

- [ ] 5.1 `load_import` 接收 `env: Env`（move），不再 `clone()`
- [ ] 5.2 `load_module` 同样改 move
- [ ] 5.3 调用方 `eval_import` 改为传 move

## 6. eval_rule 作用域传播改 move

- [ ] 6.1 规则体 env 用 move 传入
- [ ] 6.2 求值后从返回 env 中提取传播字段
- [ ] 6.3 合并回外层 env（move 重组）

## 7. control_flow 改 move

- [ ] 7.1 `eval_for` — `current_env = env`（move），循环 `current_env = current_env.bind(...)`
- [ ] 7.2 `eval_each` — 同上
- [ ] 7.3 `eval_while` — 同上

## 8. @content 作用域

- [ ] 8.1 `content_env` 保持 `Option<Rc<Env>>`（共享场景）
- [ ] 8.2 `Node::Content` 从 `content_env` 取 Rc clone 后 move 给 eval_nodes

## 9. @import AST 缓存

- [ ] 9.1 Env 新增 `ast_cache: HashMap<PathBuf, Rc<Ast>>`
- [ ] 9.2 `load_import` 先查缓存
- [ ] 9.3 `load_module` 先查缓存
- [ ] 9.4 缓存随 env move 传播

## 10. module_helpers.rs 改 move

- [ ] 10.1 `bind_exports(env: Env, ...) -> Result<Env>`
- [ ] 10.2 `merge_module_cache(env: Env, ...) -> Env`

## 11. lib.rs 调用适配

- [ ] 11.1 `evaluate_with_path` 接收 owned Env
- [ ] 11.2 返回 `(Vec<CssNode>, Env)`

## 12. 编译验证 + 回归测试

- [ ] 12.1 `cargo build` 编译通过
- [ ] 12.2 `cargo test --test compile_test` — 43/43
- [ ] 12.3 `cargo test --test stage_test` — 10/10
- [ ] 12.4 `cargo test --test ast_test` — 8/8
- [ ] 12.5 `cargo test --test common_test` — 5/5
- [ ] 12.6 `cargo test --test bs_spec` — 15/15 + ≤10s
- [ ] 12.7 `cargo test --test ep_full` — 121/121 + ≤15s
- [ ] 12.8 `cargo test --test sass_spec_full` — 2828/5362 不回归

## 13. 更新文档

- [ ] 13.1 README 保持函数式描述（代码纠正回去了）
- [ ] 13.2 `codegraph sync`
