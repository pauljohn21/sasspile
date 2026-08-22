## Why

sasspile 设计文档（README + DESIGN.md）定义了纯函数式管线：`Source → Lexed → Parsed → Evaluated → Serialized`，类型状态机 + Iterator + fold + 不可变数据。但实际代码偏离了设计——`load_import` 里 clone env 后原地改字段，`eval_for`/`eval_each` 循环 clone，`Rc` COW 模仿 GC 语言。

需要纠正代码，使其回到设计的函数式管线，用 Rust 原生的所有权和 move 语义实现不可变数据传递，而非 `Rc` + `clone`。

同时 `@import` 没有 AST 缓存，同一文件被 import 多次时重复 read+lex+parse。

## What Changes

1. **Env 改 move 语义**：方法从 `&self -> Self`（clone）改为 `self -> Self`（move），消除 clone
2. **eval_xxx 改 move**：`&Env -> (Vec, Env)` 改为 `Env -> Vec`（env 被 move 进去，求值后 move 出来）
3. **eval_nodes**：`try_fold` 保留，但 env 用 move 不用 clone
4. **load_import**：env 用 move 传入，求值后 move 回来，不 clone
5. **eval_rule 作用域**：用 move + 重组代替 clone 合并
6. **@content**：content_env 用 move 传递
7. **@import AST 缓存**：同一文件只 read+lex+parse 一次
8. **回退 Rc 改动**
9. **README 保持函数式描述**，代码纠正回去
