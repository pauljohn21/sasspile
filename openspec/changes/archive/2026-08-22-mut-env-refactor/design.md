## Context

README 定义了纯函数式管线：`Source → Lexed → Parsed → Evaluated → Serialized`，Iterator + fold + 不可变数据。但代码偏离了设计：
- `Env` 方法 `&self -> Self` 内部 `clone()` — 应该是 `self -> Self`（move）
- `load_import` clone env 后原地改 — 应该 move 进去、move 出来
- `Rc` COW 模仿 GC — 应该用 Rust move 语义
- `@import` 无 AST 缓存 — 同文件重复解析

## Goals / Non-Goals

**Goals:**
- 代码回到设计的函数式管线
- 用 Rust move 语义实现不可变数据传递，零 clone
- `@import` AST 缓存
- ep_full ≤15s，bs_spec ≤10s
- 测试通过率不变

**Non-Goals:**
- 不改公开 API
- 不改 SCSS 语义
- 不引入新依赖

## Decisions

### D1: Env 方法 &self -> Self 改为 self -> Self（move）

```rust
// 之前（clone）：
pub fn bind(&self, name: String, value: Value) -> Self {
    let mut new = self.clone();
    new.local_vars.insert(name, value);
    new
}
// 之后（move）：
pub fn bind(mut self, name: String, value: Value) -> Self {
    self.local_vars.insert(name, value);
    self
}
```

move 语义：所有权转移，零拷贝。调用方 `env = env.bind(...)` 编译为原地修改。

### D2: eval_xxx 签名 &Env -> (Vec, Env) 改为 Env -> Vec

```rust
// 之前：
fn eval_nodes(nodes: &[Node], env: &Env) -> Result<(Vec<CssNode>, Env)>
// 之后：
fn eval_nodes(nodes: &[Node], mut env: Env) -> Result<(Vec<CssNode>, Env)>
```

env 被 move 进函数，求值后 move 出来。调用方 `let (css, env) = Self::eval_nodes(nodes, env)?;` 零 clone。

### D3: eval_nodes try_fold 保留，env 用 move

```rust
fn eval_nodes(nodes: &[Node], mut env: Env) -> Result<(Vec<CssNode>, Env)> {
    let mut css = Vec::new();
    for node in nodes {
        let (mut out, new_env) = Self::eval_node(node, env)?;
        css.append(&mut out);
        env = new_env;  // move 赋值，零 clone
    }
    Ok((css, env))
}
```

### D4: load_import 用 move 代替 clone

```rust
// 之前：
let mut env = caller_env.clone();
env.base_path = Some(path.to_path_buf());
let (css, final_env) = Self::eval_nodes(&ast.nodes, &env)?;

// 之后（move）：
let mut env = caller_env;  // move，零 clone
env.base_path = Some(path.to_path_buf());
let (css, final_env) = Self::eval_nodes(&ast.nodes, env)?;
// final_env 里保留了修改，caller 拿回 final_env
```

### D5: eval_rule 作用域

规则体创建局部作用域。进入前 `let mut rule_env = env;`（move），求值后从 `rule_env` 中提取需要传播的字段（global_writes、mixin/function 定义、@extend），再 move 回 env。

### D6: @content 词法作用域

content_env 通过 move 传递。`exec_mixin` 中 `env` move 给 `eval_nodes`，`@content` 节点用 `content_env`（已 move 进 Env.content_env 字段）。

### D7: @import AST 缓存

```rust
let ast = if let Some(cached) = env.ast_cache.get(path) {
    cached.clone()  // Rc clone O(1)
} else {
    let ast = Rc::new(Parser::parse(&tokens)?);
    env.ast_cache.insert(path.to_path_buf(), ast.clone());
    ast
};
```

### D8: control_flow 改 move

```rust
// 之前：
let mut current_env = env.clone();
current_env = current_env.bind(...);

// 之后（move）：
let mut current_env = env;  // move
current_env = current_env.bind(...);  // move bind
```

### D9: Env 字段去 Rc，恢复 HashMap/Vec

move 语义下不需要 Rc——所有权转移本身就是零拷贝。只有 `ast_cache` 和 `content_env` 用 `Rc` 共享（真正的共享场景）。

### D10: lib.rs compile_file 保持管道

`compile_file` 的 read→lex→parse→evaluate→serialize 管道不变。evaluate 内部用 move。

## Risks

- **move 后原变量不可用** → 调用方需 `let (css, env) = eval(...)` 接回
- **borrow checker** → move 语义比 `&mut` 更直观，不易冲突
- **@content 作用域** → content_env 需要 Rc 共享（多个 mixin 嵌套时）
- **递归 load_import** → env 被 move 进去后需要正确传回

## 改动范围

| 文件 | 改动 |
|------|------|
| `src/lib.rs` | evaluate 调用接 move 返回 |
| `src/eval/mod.rs` | Env 定义去 Rc + 方法改 move + eval_nodes/eval_node |
| `src/eval/mixin.rs` | exec_mixin/bind_params/call_user_function 改 move |
| `src/eval/module.rs` | load_module/load_import 改 move + AST 缓存 |
| `src/eval/rule.rs` | eval_rule 作用域用 move |
| `src/eval/control_flow.rs` | eval_for/each/while 改 move |
| `src/eval/value/mod.rs` | eval_variable 改 move |
| `src/eval/module_helpers.rs` | bind_exports/merge_module_cache 改 move |
| `src/eval/import.rs` | eval_import 改 move |
| `src/eval/meta_ops.rs` | eval_meta_apply/load_css 改 move |
