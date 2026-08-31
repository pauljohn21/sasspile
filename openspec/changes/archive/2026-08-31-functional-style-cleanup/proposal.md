## Why

chain-reaction 重构已覆盖核心管线（eval_nodes/eval_for/eval_each 用 try_fold，flatten_nodes 用 flat_map，hoist_css_imports 用 partition），但 CSS 序列化器、Map 内建函数、选择器规范化、AST 序列化等模块仍保留大量 `let mut result = Vec::new(); for ... { result.push(...) }` 命令式模式。这些模块共计约 25 个函数、~450 行代码可重构为 fold/collect/flat_map 链式风格，与项目函数式管线设计保持一致。

## What Changes

- **CSS 序列化器**：`flatten_children` 改为 `flat_map + collect`；`serialize_expanded` / `serialize_compressed` 外层 `for` 改为 `fold`；`merge_at_rules` 内部 `for + push` 改为 `fold`
- **Map 内建函数**：`nested_map_merge` / `nested_map_set` / `deep_merge_maps` / `map_deep_remove` / `map-get` 的 `for + push` 改为 `fold` 或 `scan + collect`
- **选择器规范化**：`tokenize_selector_with_pseudo` / `normalize_adjacent_compounds` / `normalize_attr_selectors` / `normalize_attr_content` 的 `while + push` 改为 `iterator + collect` 或 `fold`
- **AST 序列化**：`to_scss` 的 If 分支 `for + push_str` 改为 `enumerate + fold`
- **求值器杂项**：`nest_rule_in_children` 的 `for + push` 改为 `fold`；`eval_import` CSS 分支 `for + push` 改为 `map + collect`；`selector-simple-selectors` 的 `for + push` 改为 `fold`
- **color_adjust.rs helper 提取**：15 个 adjust/change/scale 函数的 `let mut x = ...; if let Some(v) = ... { x = ... }` 重复模式提取为 `apply_kw` helper 函数

## Capabilities

### New Capabilities

（无新能力——本次变更是已有能力的风格规范化）

### Modified Capabilities

- `chain-fold`: 扩展覆盖范围——从 eval 管线扩展到 CSS 序列化器、Map 内建函数、选择器规范化、AST 序列化等模块的循环累积器
- `fp-architecture`: 补充函数式风格约束——禁止 `let mut result = Vec::new(); for ... { result.push(...) }` 模式，要求使用 fold/collect/flat_map

## Impact

- `src/css/mod.rs` — flatten_children, serialize_expanded, serialize_compressed, merge_at_rules（~100 行重构）
- `src/css/selector.rs` — tokenize_selector_with_pseudo, normalize_adjacent_compounds, normalize_attr_selectors, normalize_attr_content（~145 行重构）
- `src/eval/builtin/map.rs` — nested_map_merge, nested_map_set, deep_merge_maps, map_deep_remove, map-get（~100 行重构）
- `src/eval/builtin/color_adjust.rs` — 提取 apply_kw helper 消除 15 个函数的 mut 变量重复（~150 行重构）
- `src/eval/builtin/selector.rs` — selector-simple-selectors（~12 行重构）
- `src/eval/rule.rs` — nest_rule_in_children（~30 行重构）
- `src/eval/import.rs` — eval_import CSS 分支（~15 行重构）
- `src/parse/ast_impl.rs` — to_scss If 分支（~20 行重构）
- 测试基线不变：202/202 + sass-spec 2902/5362
