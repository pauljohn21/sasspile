# boost-85to100 — Technical Design

## 根因分组策略

194 个失败 case 经分析按根因归为 12 个主题。每个主题有明确的代码修复点，
修复一个根因通常能一次性解决多个 case。

```
                    ┌─────────────────────────────────────┐
                    │     194 failures → 12 root causes    │
                    └─────────────────────────────────────┘
                                      │
        ┌──────────────┬──────────────┼──────────────┬──────────────┐
        ▼              ▼              ▼              ▼              ▼
   ┌─────────┐   ┌─────────┐   ┌─────────┐   ┌─────────┐   ┌─────────┐
   │ Parser  │   │Serialize│   │ Eval    │   │ CSS     │   │ Module  │
   │ Issues  │   │ Issues  │   │ Logic   │   │ Extend  │   │ System  │
   └────┬────┘   └────┬────┘   └────┬────┘   └────┬────┘   └────┬────┘
        │              │              │              │              │
   T1: use/fwd     T2: empty     T3: list eq   T4: transit   T11: fwd member
     comment        string        T6: map        extend       T12: use css
   T8: @if expr    T7: selector   T9: list                 T5: @import
     parsing       formatting     T10: math                 hoisting
```

## 各主题设计

### T1: @use/@forward 注释解析 (12 case, 12 ERR)

**根因**：`parse_use_declaration` / `parse_forward_declaration` 解析 `@use "url" as ... with (...)`
时只在固定位置调用 `skip_ws()`，未处理注释。

**修复点**：类似已修复的 `parse_variable`，在 identifier/keyword/`(`/`:`/`"/"`)` 等关键位置调用
`skip_ws_and_comments()` 而非 `skip_ws()`。

**文件**：`src/parse/nodes.rs`

### T2: 空字符串属性抑制 (1+ case)

**根因**：CSS 序列化器遇到 `key: ""` 输出 `key: ;`，但规范要求整行属性不输出。

**修复点**：`serialize_property` 或等效函数在 value 为 `Value::String(s)` 且 `s.is_empty()` 时
返回空字符串（跳过该属性）。

**文件**：`src/css/serialize.rs`

### T3: List 等值容器类型 (1 case)

**根因**：`[]` (bracket list) 和 `()` (paren list) 在 equality 比较中被视为相同。

**修复点**：`List::eq` 或 `values_eq` 在比较前检查 `bracketed` 字段。

**文件**：`src/eval/value/list.rs`

### T4: 传递 extend 穿越伪选择器 (1+ case)

**根因**：extend 图构建时 `:is(midstream)` 中的 `midstream` 未被识别为 extend 目标，
导致 `downstream` 对 `midstream` 的 extend 无法传递到 `upstream` 的结果中。

**修复点**：`extends_after` 中的选择器图遍历需要处理伪选择器参数作为 extend 链的一环。

**文件**：`src/css/selector_extend.rs`

### T5: @import 从规则内提升 (3 case, 3 ERR)

**根因**：当前 at-rule 处理器在规则内遇到 `@import` 抛出 "not allowed here"，
但规范要求将 `@import` 提升到根级别（类似原生 Sass 行为）。

**修复点**：在 `eval_ruleset` 或规则处理入口检测到 `@import` 时，将其移入
pending imports 列表并在求值完毕后执行。

**文件**：`src/eval/directives.rs`

### T6: Map 函数 edge case (8+1 case)

**根因**：多个独立 — `deep_remove` 未实现 / `deep_merge` 处理空 map / `has_key` 类型错误消息 /
`get/nested` 的 "not a map" 错误 / `remove` 的 positional_and_named 错误。

**修复点**：逐个分析每个子功能的实际行为差异。

**文件**：`src/eval/functions/map.rs`

### T7: CSS 选择器格式化 (7 case, 7 DIFF)

**根因**：选择器序列化差异 — 数字逃逸、combinator 空格、reference combinator (`/`),
`::slotted()` 参数处理。

**修复点**：逐个对比 expected vs actual，调整 `serialize_selector` 的分支。

**文件**：`src/css/serialize.rs`

### T8: @if 表达式 (14 case)

**根因**：
- 9 个 `error/*` — Sass 将 `and`/`or`/`not` 在特定无效组合下报告错误，
  our error message 格式或时机不同
- 3 个 `raw/*` — 原始 `and`/`or`/`not` 关键字在 `@if` 中的解析差异
- 1 个 `css/alone` — 三元 `?` 字符 lex 错误
- 1 个 `trailing_semi` — 尾部分号解析

**修复点**：可能需要分别处理 error message 格式化、raw conditional 解析。

**文件**：`src/parse/expr.rs`, `src/lex/lexer.rs`

### T9: List 函数 edge case (20 case)

**根因**：
- `join` separator 校验：`"auto"` 应触发明确错误（当前错误消息不匹配）
- `join` separator truthiness: 空字符串 `""` 的处理
- `zip` 单列表 bracketed 行为
- `index` 在 map 中的行为

**修复点**：调整 `list-join` 的 separator 校验逻辑和错误消息。

**文件**：`src/eval/functions/list.rs`

### T10: Math 函数 (73 case)

**根因**：这是最大的主题，涵盖多个独立子问题：
- **negative_zero 处理**: sin(-0), atan(-0), asin(-0) 等应返回 -0 而非 0
- **atan2 单位**: 带单位参数的 atan2 行为
- **clamp 单位保持**: `clamp(1px, 2px, 3px)` 应保留单位
- **pow 边界**: 负底数 + 非整数指数的 NaN 处理
- **unit 归一化**: 复合单位的简化输出格式
- **命名常量**: `math.$e`, `math.$epsilon`, `math.$max-number` 等

**修复点**：按子问题分组修复。

**文件**：`src/eval/functions/math.rs`, `src/eval/functions/math_ops.rs`

### T11: @forward 成员处理 (12 case)

**根因**：`@forward` 的成员导入优先级、覆盖规则、`as` 分隔符处理。

**修复点**：forward 语义逻辑。

**文件**：`src/eval/module_import.rs`

### T12: @use CSS 排序 (5 case)

**根因**：`@use` 与 `@import` 混合使用时的 CSS 输出顺序，命名空间变量赋值行为。

**修复点**：use 模块加载与 CSS 生成顺序。

**文件**：`src/eval/module_import.rs`, `src/eval/directives.rs`

## 优先级与依赖

```
Phase 1 (Quick Wins, ~16 cases 直接修复):
  T1 use/forward comment  ──→  12 ERRs
  T2 empty string          ──→  1 DIFF
  T3 list equality         ──→  1 DIFF
  T8 if (partial)          ──→  2 ERRs
  T5 @import hoisting      ──→  3 ERRs

Phase 2 (Medium, ~29 cases):
  T6 map functions         ──→  8+1 cases
  T9 list functions        ──→  20 cases
  T7 selector formatting   ──→  7 cases

Phase 3 (Substantial, ~76 cases):
  T4 extend pseudo         ──→  1+ cases
  T11/T12 module system    ──→  17 cases

Phase 4 (Heavy, 73 cases):
  T10 math functions       ──→  73 cases
```

## 验证策略

每个 Phase 完成后：
1. `cargo test --test compile_test --test stage_test --test ast_test --test common_test --test bs_spec --test ep_full` — 核心测试
2. `SPEC_STORE_CMD=run SPEC_STORE_CMD=stats` — 确认 delta
3. 目标目录全部 100%
