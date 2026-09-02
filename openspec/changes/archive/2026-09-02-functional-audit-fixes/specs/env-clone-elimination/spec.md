## ADDED Requirements

### Requirement: call_user_function MUST NOT use env.clone()

`Evaluator::call_user_function` MUST NOT 使用 `env.clone()` 创建函数作用域。MUST 使用 `Env` move 语义 + `exit_scope` 恢复外层作用域（与 `eval_rule` 一致）。

#### Scenario: call_user_function uses move semantics
- **WHEN** 检查 `src/eval/mixin.rs` 中 `call_user_function` 函数
- **THEN** 签名 MUST 为 `fn call_user_function(func: &FunctionDef, pos_args: &[Value], kw_args: &HashMap<String, Value>, env: Env) -> Result<Value>` — `env` 为 move 而非 `&Env`
- **AND** 函数体内 MUST NOT 出现 `env.clone()`

#### Scenario: call_user_function uses exit_scope
- **WHEN** `call_user_function` 求值函数体后
- **THEN** MUST 通过 `exit_scope` 恢复外层作用域的 local_vars/local_mixins/local_functions/forwarded_*，仅传播命名空间变量和 !global 变量

#### Scenario: Function local variables do not leak
- **WHEN** 执行 `$x: 1; @function f() { $x: 2; @return $x; } $y: f();` 后
- **THEN** `$x` MUST 保持为 `1` — 函数内局部变量不传播到外层

#### Scenario: !global variables propagate
- **WHEN** 执行 `@function f() { $x: 1 !global; @return $x; } $y: f();` 后
- **THEN** `$x` MUST 在外层可见且为 `1` — !global 变量传播

### Requirement: bind_params MUST NOT use env.clone()

`Evaluator::bind_params` MUST NOT 使用 `env.clone()` 创建参数绑定环境。MUST 接收 `Env`（move）并返回新 `Env`。

#### Scenario: bind_params uses move semantics
- **WHEN** 检查 `src/eval/mixin.rs` 中 `bind_params` 函数
- **THEN** 签名 MUST 为 `fn bind_params(params: &[Param], args: &[Arg], env: Env) -> Result<Env>` — `env` 为 move 而非 `&Env`
- **AND** 函数体内 MUST NOT 出现 `env.clone()`

#### Scenario: bind_params preserves all env fields
- **WHEN** 调用 `bind_params` 绑定参数
- **THEN** 返回的 Env MUST 保留输入 Env 的所有字段（base_path、load_paths、namespaces、extends、module_cache 等），仅追加参数绑定

#### Scenario: All callers updated
- **WHEN** `bind_params` 签名变更
- **THEN** 调用者 `exec_mixin` MUST 传入 `env`（move）而非 `&env`，并使用返回的 `Env`

#### Scenario: All tests pass after refactor
- **WHEN** 运行完整测试套件
- **THEN** 202/202 测试 MUST 全通过
