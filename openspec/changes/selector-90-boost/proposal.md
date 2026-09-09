## Why

`selector-extend()` 函数对应 sass-spec 的 `core_functions/selector` 目录仍有 408 个 DIFF 失败（占该目录 46%）。上次 `selector-ast-rewrite` 修复命名空间数据模型，但遗留了四类算法缺陷：(1) NO-OP 检测丢失 `Namespace::Empty` 序列化信息；(2) 格式输出产生多余选择器；(3) 复杂选择器统合生成缺漏；(4) tail combinator 生成重复 compound。需用 `failures_json` 工具逐一对照修复，目标将该目录通过率从 54% 提升到 90%+。

## What Changes

- 修复 `src/css/selector_extend.rs` 中 `extend_complex` 的四类算法缺陷修复 `src/css/selector_unify.rs` 中 `unify_extendee_list` 对空命名空间的处理
- 确保 `Namespace::Empty` 在 extend 后保留（输出 `|name` 格式）
- 修复复杂选择器统合时的组合生成逻辑（生成所有排列）
- 修复 tail combinator 场景下重复 compound 的 bug

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- **`css-selector`** — selector-extend 函数的四类行为修复：空命名空间保留、格式输出、复杂统合、tail combinator
- **`css-selector`** — unify_extendee_list 保留 Namespace::Empty 信息

## Impact

| 范围 | 文件 | 说明 |
|------|------|------|
| extend 算法 | `src/css/selector_extend.rs` | 修复 extend_complex 四类 bug |
| unify 算法 | `src/css/selector_unify.rs` | 修复空命名空间序列化丢失 |
| 测试确认 | `tests/failures_json.rs` | 用 failures_json 工具验证修复 |

**预估收益**：core_functions/selector 408 个失败 → 目标 ≤30（90%+），加上 css 和 directives 中 selector 相关修复，总计 +450~500 cases。
