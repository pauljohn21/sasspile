## Context

sasspile 的 `Env` 结构体模仿 GC 语言的不可变模式：每个方法 `&self -> Self`，内部 `let mut new = self.clone()` 再改一个字段。这在 GC 语言里没问题，但在 Rust 里是反模式——即使加了 `Rc` COW 把 clone 降到 O(1)，仍有引用计数开销和 `make_mut` 深拷贝问题。

同时 `@import` 没有 AST 缓存——同一文件被 import 多次时重复 read + lex + parse。`@use` 有 `module_cache` 所以快，`@import` 每次从头来。

## Goals / Non-Goals

**Goals:**
- `Env` 所有方法改 `&mut self`，消除全部 `clone()`
- `eval_xxx` 方法签名从 `&Env -> (Vec, Env)` 改为 `&mut Env -> Vec`
- `@import` 加 AST 缓存，同一文件只 read + lex + parse 一次
- ep_full ≤15 秒，bs_spec ≤10 秒
- 所有现有测试通过率不变

**Non-Goals:**
- 不引入 `im` / `im-rc` crate
- 不改变 SCSS 语义（@import 仍是内联模式，@use 仍隔离）
- 不改公开 API（`compile_file` / `compile_expanded` 等签名不变）

## Decisions

### D1: Env 字段去掉 Rc，恢复 HashMap/Vec

```rust
// 之前（Rc COW）：
local_vars: Rc<HashMap<String, Value>>,

// 之后（&mut 原地修改）：
local_vars: HashMap<String, Value>,
```

### D2: 方法签名 &self -> Self 改为 &mut self

```rust
// 之前：
pub fn bind(&self, name: String, value: Value) -> Self {
    let mut new = self.clone();
    new.local_vars.insert(name, value);
    new
}

// 之后：
pub fn bind(&mut self, name: String, value: Value) {
    self.local_vars.insert(name, value);
}
```

### D3: eval_xxx 签名 &Env -> (Vec, Env) 改为 &mut Env -> Vec

```rust
// 之前：
fn eval_nodes(nodes: &[Node], env: &Env) -> Result<(Vec<CssNode>, Env)> {
    nodes.iter().try_fold((Vec::new(), env.clone()), |(mut css, env), node| {
        let (mut out, new_env) = Self::eval_node(node, &env)?;
        css.append(&mut out);
        Ok((css, new_env))
    })
}

// 之后：
fn eval_nodes(nodes: &[Node], env: &mut Env) -> Result<Vec<CssNode>> {
    let mut css = Vec::new();
    for node in nodes {
        let mut out = Self::eval_node(node, env)?;
        css.append(&mut out);
    }
    Ok(css)
}
```

### D4: @import AST 缓存

```rust
// Env 新增字段：
ast_cache: HashMap<PathBuf, Rc<Ast>>,

// load_import / load_module 中：
let ast = if let Some(cached) = env.ast_cache.get(path) {
    cached.clone()
} else {
    let source = std::fs::read_to_string(path)?;
    let tokens: Vec<Token> = Lexer::new(&source)...;
    let ast = Rc::new(Parser::parse(&tokens)?);
    env.ast_cache.insert(path.to_path_buf(), ast.clone());
    ast
};
```

### D5: load_import 不再 clone Env

```rust
// 之前：
let mut env = caller_env.clone();
env.base_path = Some(path.to_path_buf());
let (css, mut final_env) = Self::eval_nodes(&ast.nodes, &env)?;

// 之后：
// 保存调用者状态，原地修改
let saved_base = env.base_path.take();
let saved_depth = env.depth;
env.base_path = Some(path.to_path_buf());
env.depth += 1;
let css = Self::eval_nodes(&ast.nodes, env)?;
// 恢复调用者状态
env.base_path = saved_base;
env.depth = saved_depth;
```

### D6: @content 词法作用域

`@content` 块在调用方作用域执行。当前用 `content_env: Option<Rc<Env>>` 传递。
改 `&mut` 后，`exec_mixin` 需要临时交换 Env：
- 保存 mixin_env 的状态
- 切换到 content_env 执行 @content
- 切换回 mixin_env 继续

用 `std::mem::swap` 或 `std::mem::take` 实现。

### D7: rule.rs 作用域传播

当前 `eval_rule` 结束后把 `new_env` 的变更选择性传播到 `return_env`（clone 外层 env + 合并部分字段）。
改 `&mut` 后，规则体直接在传入的 `&mut Env` 上修改，结束后恢复不需要传播的字段（局部变量），保留需要传播的字段（!global、mixin 定义、@extend）。

方案：进入规则体前保存 `local_vars` 的状态（`std::mem::take` + 新建空 HashMap），结束后用 `std::mem::swap` 恢复——但保留 `global_writes` 和新增的 mixin/function。

## Risks / Trade-offs

- **[Risk] @content 作用域切换复杂** → 用 `mem::swap` 临时交换，确保正确恢复
- **[Risk] rule.rs 作用域传播** → 需要仔细处理哪些字段传播、哪些不传播
- **[Trade-off] 方法签名全面改动** → 涉及 ~20 个方法，但模式固定
- **[Trade-off] AST 缓存内存占用** → 同一文件只缓存一份 AST，内存开销可忽略

## 改动范围

| 文件 | 改动 |
|------|------|
| `src/eval/mod.rs` | Env 定义去 Rc + 方法签名改 &mut + 新增 ast_cache |
| `src/eval/mixin.rs` | exec_mixin / bind_params / call_user_function 改 &mut + @content 交换 |
| `src/eval/module.rs` | load_module / load_import 改 &mut + AST 缓存 |
| `src/eval/rule.rs` | eval_rule 改 &mut + 作用域传播用 mem::swap |
| `src/eval/control_flow.rs` | eval_for / eval_each / eval_while 改 &mut，去掉 clone |
| `src/eval/value/mod.rs` | eval_variable 改 &mut |
| `src/eval/module_helpers.rs` | bind_exports / merge_module_cache 改 &mut |
| `src/eval/import.rs` | eval_import 改 &mut |
| `src/eval/meta_ops.rs` | eval_meta_apply / eval_meta_load_css 改 &mut |
