# Design: functional-cleanup

## 问题分类与重构策略

### 1. `else-if` 链 → `match`（81 处，32 个文件）

**模式**：
```rust
// ❌ 当前
if text.eq_ignore_ascii_case("true") { Token::True }
else if text.eq_ignore_ascii_case("false") { Token::False }
else { Token::Ident(text.to_string()) }
```

**策略**：
```rust
// ✅ match + to_lowercase（scan_escape_ident 已用此模式）
match text.to_lowercase().as_str() {
    "true" => Token::True,
    "false" => Token::False,
    _ => Token::Ident(text.to_string()),
}
```

**目标文件（按 else-if 数量排序）**：

| 文件 | else-if 数 | 策略 |
|------|-----------|------|
| `eval/color.rs` | 8 | 颜色空间分派 → match ColorSpace |
| `css/selector.rs` | 8 | 伪类/组合器分派 → match |
| `lex/scanner.rs` | 6 | scan_ident 关键字分派 → match |
| `eval/builtin/color_conv_ops.rs` | 6 | 颜色空间转换分派 → match |
| `eval/mixin.rs` | 5 | mixin 参数分派 → match |
| `parse/at_rules.rs` | 4 | @规则分派 → match |
| `parse/ast_impl.rs` | 4 | AST 节点分派 → match |
| `parse/ast/color_fmt.rs` | 4 | 颜色格式分派 → match |
| `eval/value/display.rs` | 4 | Value Display 分派 → match |
| 其余 1-3 处的文件 | 28 | 逐个转换 |

### 2. `for + push` → 迭代器链（76 处）

**模式**：
```rust
// ❌ 当前
let mut result = Vec::new();
for x in items {
    if pred(x) { result.push(transform(x)); }
}
```

**策略**：
```rust
// ✅ filter + map + collect
items.into_iter()
    .filter(pred)
    .map(transform)
    .collect::<Vec<_>>()
```

**关键目标文件**：

| 文件 | for 循环数 | 重构策略 |
|------|-----------|----------|
| `parse/ast/display.rs` | 6 | 枚举 + format! → iterator chain |
| `eval/file_resolver.rs` | 7 | 嵌套 for 路径搜索 → flat_map |
| `css/mod.rs` | 4 | 序列化 children → iterator chain |
| `css/selector.rs` | 3 | tokens 遍历 → windows/iter |
| `css/selector_ast.rs` | 2 | Display → iterator chain |
| `eval/value/mod.rs` | 1 | eval_args → try_fold |
| `eval/color.rs` | 1 | 颜色转换 → iterator chain |

### 3. `if-let` 链 → `apply_kw` 链式

**目标文件**：`color_adjust.rs` 的 `adjust_legacy`/`change_legacy`/`scale_legacy`

**模式**：
```rust
// ❌ 当前——9 个 if-let 修改可变变量
let mut r = c.legacy_rgb[0];
if let Some(v) = get_num(kw_args, "red")? { r += v; }
if let Some(v) = get_num(kw_args, "green")? { g += v; }
```

**策略**：已有 `apply_kw` 函数，只需将 legacy 函数对齐：
```rust
// ✅ apply_kw 链式（现代空间函数已用此模式）
let r = apply_kw(c.legacy_rgb[0], kw_args, "red", |v, d| v + d)?;
let g = apply_kw(c.legacy_rgb[1], kw_args, "green", |v, d| v + d)?;
```

**注意**：`has_hsl`/`has_hwb` 标志位需要特殊处理——用 `Option` 链或 `bool::then` 替代 `let mut has_hsl = false`。

### 4. `while let` 循环（scanner.rs 等）

**判断**：Lexer 的 `while let Some(c) = self.peek()` 是**合理的内部状态循环**——封装在 `&mut self` 内部，不跨函数传递。与 `selector_parser.rs` 的 peek+next 模式一致。**保留不改**。

### 5. `&mut` 参数

| 类型 | 数量 | 处理 |
|------|------|------|
| `Display::fmt(&mut Formatter)` | 10 | 保留（Rust 标准 trait） |
| `css/mod.rs write_node_*` | 3 | 改为消费 + 返回 String |
| `css/mod.rs process_node` | 1 | 改为消费 + 返回 Vec |

### 6. `.clone()` 消减策略

| 场景 | 策略 |
|------|------|
| `HashMap::get().clone()` | 改 `HashMap::remove()` 消费或保留 clone（不可消除） |
| 函数参数 `&Value` → clone | 改为 move 语义消费 |
| `Rc::clone` | 保留（原子计数器，零开销） |

## 不改的部分

- `while let` 在 lexer/parser 内部（合理的状态循环）
- `Display::fmt` 的 `&mut Formatter`（Rust 标准 trait 签名）
- `for _ in 0..6` 固定次数循环（hex 解析，语义清晰）
- `for window in windows(2)` （已经是迭代器，合理使用）

## 验证策略

```bash
# 核心测试
cargo test --test compile_test --test stage_test --test ast_test --test common_test \
  --test interp_test --test bs_spec --test calc_ast_test --test calc_simplify_test \
  --test calc_units_test --test selector_ast_test --test selector_unify_test \
  --test selector_super_test --test selector_extend_test --test default_config_test

# ep_full
cargo test --test ep_full

# sass-spec 全量
RUST_LOG="sass_spec_full=info,sasspile=warn" cargo test --test sass_spec_full -- --nocapture
```

**通过标准**：251/251 核心测试 + sass-spec ≥ 3366/5624（不回退）
