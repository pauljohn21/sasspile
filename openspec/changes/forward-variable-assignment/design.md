## 核心问题

`namespace.$prefix-var: value` 形式的命名空间变量赋值未能正确转发到 upstream 模块的原始变量。

### 根因

`src/eval/value/mod.rs` 中 `eval_assign` 处理命名空间变量时，仅检查 `forwarded_vars` 表查找精确匹配，未处理 `@forward ... as prefix-*` 的前缀映射。

## 设计方案

### 1. 前缀映射表

```rust
// Env 中新增：forward_prefix: HashMap<String, (String, String)>
// 记录 namespace → (forward_prefix, source_module)
// 例如: "midstream" → ("d-", "upstream")
```

当 `@forward "upstream" as d-*` 求值时，在 `midstream` 命名空间注册前缀映射。

### 2. 变量名解前缀

```
midstream.$d-a: new value;
→ 查找 midstream 的前缀表: "d-" → "upstream"
→ 去掉前缀: "d-a" → "a"
→ 赋值到 upstream 模块的 $a 变量
```

### 3. 嵌套作用域处理

即使在 CSS rule 内执行命名空间赋值，也始终转发到 upstream 模块的全局变量（SCSS 规范：命名空间赋值忽略块级作用域）。

## 源文件修改

| 文件 | 修改 |
|------|------|
| `src/eval/forward.rs` | `@forward as prefix-*` 求值时注册前缀映射 |
| `src/eval/value/mod.rs` | `eval_assign` 增加前缀解映射逻辑 |
| `src/eval/env.rs` | Env 新增 `forward_prefix` 字段 |

## 验证

- `directives/forward/member/as/variable_assignment/*`（2 cases）
- `directives/forward/member/shadowed/*`（2 cases）
- `directives/forward/member/import/*`（2 cases）
- 核心测试 202/202 无回归
