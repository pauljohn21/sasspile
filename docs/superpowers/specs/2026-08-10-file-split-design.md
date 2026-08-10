# 项目文件拆分设计

> **日期**: 2026-08-10
> **状态**: 已批准，待实现
> **关联**: sasspile v2-rewrite-from-scratch

## 1. 背景与动机

### 1.1 问题

4 个源文件超过 500 行限制，影响 AI 上下文处理效率（"防抖"）：

| 文件 | 行数 | 超出 |
|------|------|------|
| `src/eval/mod.rs` | 2144 | 4x |
| `src/parse/mod.rs` | 1148 | 2x |
| `src/parse/ast.rs` | 698 | 1.4x |
| `src/lex/mod.rs` | 510 | 1.02x |

### 1.2 设计原则

- **按功能拆分**，不是简单按行数切割
- 每个文件有单一明确职责
- 利用现有 `// ——` 分节注释作为拆分边界
- 测试代码集中到 `tests/` 目录
- 所有文件 ≤ 500 行

## 2. eval/ 拆分（2144行 → 13 个文件）

### 2.1 核心模块

| 新文件 | 估计行数 | 职责 |
|--------|----------|------|
| `eval/mod.rs` | ~310 | `Env` + `ModuleExports` + `MixinDef`/`FunctionDef` + `Evaluator` 入口 + `eval_nodes`/`eval_node` |
| `eval/rule.rs` | ~110 | `eval_rule` + `combine_selectors` + `eval_at_root` |
| `eval/value.rs` | ~350 | `eval_value` + `eval_binop` + `add`/`sub`/`mul`/`div`/`modulo`/`compare` + `values_eq` + `inspect_value` + `eval_interp_str` + `eval_simple_expr` |
| `eval/control_flow.rs` | ~100 | `eval_if`/`eval_for`/`eval_each`/`eval_while` |
| `eval/mixin.rs` | ~110 | `eval_include` + `bind_params` + `call_function` + `call_user_function` + `eval_at_rule` |
| `eval/extend.rs` | ~80 | `apply_extends` |
| `eval/module.rs` | ~160 | `resolve_file` + `load_module` + `call_module_function` |

### 2.2 Builtin 子模块

`call_builtin`（~645行）拆分方式：`builtin.rs` 保留函数签名和 match 分派，每个 arm 调用子模块的 `pub fn call(name: &str, args: &[Value]) -> Result<Value>`。

| 新文件 | 估计行数 | 职责 |
|--------|----------|------|
| `eval/builtin.rs` | ~50 | `call_builtin` 分派入口 |
| `eval/builtin/math.rs` | ~80 | abs/ceil/floor/round/min/max/percentage |
| `eval/builtin/string.rs` | ~80 | str-length/to-upper/lower/quote/unquote |
| `eval/builtin/color.rs` | ~120 | rgb/rgba/darken/lighten/mix/invert/grayscale/complement/hwb/hsl/hsla/adjust-hue 等 |
| `eval/builtin/list.rs` | ~100 | length/nth/append/join/index/separator/set-nth/zip/is-bracketed |
| `eval/builtin/map.rs` | ~80 | map-get/merge/remove/keys/values/has-key/set |
| `eval/builtin/meta.rs` | ~100 | type-of/inspect/keywords/function-exists/variable-exists/mixin-exists/get-function/call |
| `eval/builtin/selector.rs` | ~80 | selector-nest/append/parse/simple-selectors/unify/extend/is-super |

### 2.3 颜色转换函数

| 新文件 | 估计行数 | 职责 |
|--------|----------|------|
| `eval/color.rs` | ~240 | `hsl_to_rgb`/`hwb_to_rgb`/`rgb_to_hsl` + `builtin_rgba`/`darken`/`lighten`/`mix` + `simple_random` |

### 2.4 实现方式

所有 `Evaluator` 方法保留在 `impl Evaluator` 块中，通过多个 `impl Evaluator` 块分散到不同文件。内部方法不加 `pub`，仅在模块内部可见。

## 3. parse/ 拆分（1148行 → 4 个文件）

| 新文件 | 估计行数 | 职责 |
|--------|----------|------|
| `parse/mod.rs` | ~70 | `mod` 声明 + `Parser` 结构 + `parse()` 入口 + 基础操作 |
| `parse/nodes.rs` | ~350 | `parse_node`/`parse_rule`/`parse_decl`/`parse_variable`/`parse_body` + 参数解析 `parse_params`/`parse_args`/`parse_config` + 辅助方法 |
| `parse/at_rules.rs` | ~350 | 所有 @规则解析方法 |
| `parse/expr.rs` | ~330 | Pratt 表达式 `parse_expr`/`is_value_start`/`parse_prefix`/`peek_binding_power` + `parse_number`/`parse_hash_color` |

所有 `Parser` 方法保留在 `impl Parser` 块中，分散到不同文件。

## 4. ast.rs 拆分（698行 → 2 个文件）

| 新文件 | 估计行数 | 职责 |
|--------|----------|------|
| `parse/ast.rs` | ~233 | 类型定义（Node, Value, BinOp, Color, Separator, Ast, Param, Arg, VarFlags 等） |
| `parse/ast_impl.rs` | ~335 | `impl Display for Value` + `impl Node { to_scss() }` |

## 5. lex/mod.rs（510行 → 无需拆分）

测试移走后 ~410 行，不超过 500 行。

## 6. 测试迁移

从 4 个大文件中提取测试代码到 `tests/` 目录：

| 新文件 | 来源 | 估计行数 | 内容 |
|--------|------|----------|------|
| `tests/eval_test.rs` | `eval/mod.rs` | ~30 | test_eval_simple/variable |
| `tests/parse_test.rs` | `parse/mod.rs` | ~50 | test_parse_rule/variable/if/mixin/expr_precedence |
| `tests/ast_test.rs` | `parse/ast.rs` | ~70 | test_number/string/color/list/map/bool_null_display |
| `tests/to_scss_test.rs` | `parse/ast.rs` | ~80 | test_rule/decl/variable/comment/if/for/include/extend/use/return/content |
| `tests/lex_test.rs` | `lex/mod.rs` | ~100 | test_ident/number/string/interp/amp/comment |

## 7. 最终项目结构

```
src/
├── lib.rs           (300)
├── main.rs          (36)
├── error.rs         (77)
├── css/
│   ├── mod.rs       (248)
│   └── node.rs      (73)
├── lex/
│   ├── mod.rs       (410)
│   └── token.rs     (131)
├── parse/
│   ├── mod.rs       (70)
│   ├── nodes.rs     (350)
│   ├── at_rules.rs  (350)
│   ├── expr.rs      (330)
│   ├── ast.rs       (233)
│   └── ast_impl.rs  (335)
├── eval/
│   ├── mod.rs       (310)
│   ├── rule.rs      (110)
│   ├── value.rs     (350)
│   ├── control_flow.rs(100)
│   ├── mixin.rs     (110)
│   ├── extend.rs    (80)
│   ├── module.rs    (160)
│   ├── color.rs     (240)
│   ├── builtin.rs   (50)
│   └── builtin/
│       ├── math.rs  (80)
│       ├── string.rs(80)
│       ├── color.rs (120)
│       ├── list.rs  (100)
│       ├── map.rs   (80)
│       ├── meta.rs  (100)
│       └── selector.rs(80)
└── stage/
    ├── mod.rs       (14)
    ├── source.rs    (77)
    ├── lexed.rs     (64)
    ├── parsed.rs    (57)
    ├── evaluated.rs (73)
    └── serialized.rs(89)

tests/
├── common/mod.rs   (158)
├── common_test.rs  (1)
├── eval_test.rs    (30)
├── parse_test.rs   (50)
├── ast_test.rs     (70)
├── to_scss_test.rs (80)
├── lex_test.rs     (100)
├── cf_diag.rs      (208)
├── minimize.rs     (265)
├── sass_spec.rs    (216)
├── sass_spec_full.rs(258)
├── bootstrap_spec.rs(421)
├── bs_spec.rs      (64)
├── cf_color.rs     (102)
└── ep_spec.rs      (54)
```

**全部源文件 < 500 行** ✅

## 8. 实现约束

1. **`impl` 块分散**——Rust 允许同一类型在多个文件中写多个 `impl` 块
2. **内部方法不加 `pub`**——仅在模块内部可见
3. **`mod` 声明**——父模块中声明子模块，`pub use` 重导出公共类型
4. **`lib.rs` 不变**——`pub mod eval;` 等声明不需要改
5. **`Cargo.toml` 不变**——无新依赖

## 9. 不在范围内

- 不引入 trait 抽象
- 不重构 `call_builtin` 的 match 结构（只拆文件，不改逻辑）
- 不改动 `css/`、`stage/` 目录
- 不改动现有测试逻辑（只迁移位置）
