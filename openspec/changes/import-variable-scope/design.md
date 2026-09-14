## 核心问题

当前 `@import "file"` 时，imported file 在一个**全新的环境**中求值，无法访问 importing context 中定义的变量。

### 根因

`src/eval/import.rs` 中 `eval_import` 调用 `Evaluator::evaluate_with_env` 时传入的新 Env 未继承 importing context 的变量绑定。

## 设计方案

### 1. 变量作用域链继承

```
eval_import(url, env):
    loaded_module = load_module(url)
    // 创建 child_env，继承当前 env 的变量
    child_env = env.with_import_scope()
    evaluate_with_env(loaded_module.ast, child_env)
```

- `with_import_scope()` 创建新的 Env，其 `current` scope 的 parent 指向 importing context 的 current scope
- 只读访问：imported file 可以**读取** importing context 的变量
- 写隔离：imported file 中 `!default` 变量的写入不影响 importing context

### 2. `!default` 覆盖机制

```
// importing context:
$a: configured;

// imported file (other.scss):
$a: default !default;

// @import "other" 时：
// 1. child_env 查找 $a → 在 parent scope 找到 "configured"
// 2. 因已有值，跳过 !default 赋值
// 3. imported file 中的 $a 引用解析为 "configured"
```

### 3. 嵌套 Import 链

```
a.scss @import b.scss @import c.scss
→ c 的 scope chain: c → b → a
→ c 可以读取 a 和 b 的变量（只读）
```

### 4. 作用域模型

复用现有 `Scope` 的 `parent: Option<Rc<Scope>>` 链：
- importing context 的 `current` scope 作为 imported file 的 parent
- 不 clone 整个 Env，只 clone `Rc<Scope>`（引用计数递增，零拷贝）

## 源文件修改

| 文件 | 修改 |
|------|------|
| `src/eval/env_impl.rs` | 添加 `with_import_scope()` 方法 |
| `src/eval/import.rs` | `eval_import` 改用 `with_import_scope` |
| `src/eval/hoist.ts` | 确保 hoist 后仍保留作用域链 |

## 验证

- `directives/import/configuration/*`（20+ cases）
- `directives/import/with*`（12 cases）
- 核心测试 202/202 无回归
