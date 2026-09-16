## Why

`sass-spec per-directory` 通过率在 `unlock-core-functions` fix 后没有提升，根因诊断是 `CssNode::render()` 输出为单行 `a {b: 0;}`，而 spec 期望的多行格式 `a {\n  b: 0;\n}`。`normalize_css` 在两阶段 normalize 后因行结构差异无法匹配。这是序列化层面的 pre-existing 缺陷 — 修复后预计可释放 5000~7000+ core_functions 测试。

## What Changes

- **修改** `src/ast.rs` `CssNode::render()`，让 `Rule` 输出为多行格式 `selector {\n  decl;\n}`（与 dart-sass / sass-spec 一致）
- **修改** 同函数让每个 Declaration 独占一行
- **新增** 序列化格式单元测试 `tests/serialize_format.rs` 验证单条 / 多条 / 空规则场景

## Capabilities

### New Capabilities

- `css-multiline-serialize`: CSS 序列化阶段输出 dart-sass 兼容的多行缩进格式

### Modified Capabilities

- 无新增 delta spec — 本缺陷属于"已有需求的正确输出格式未对齐"

## Impact

- **受影响的代码**: `src/ast.rs` `CssNode::render()` 中 `Self::Rule` 和 `Self::Declaration` 的格式化字符串
- **行为影响**: 所有非空的 Rule 输出从 `a {b: 0;c: 1;}` 变为多行格式；空 rule 仍可输出 `a {}` 或省略
- **兼容性**: 不影响 Bootstrap / Element Plus 编译语义（人类可读性提升）
- **风险**: 极低 — 仅改变 output 字符串格式，不改动 AST 结构或 dispatch 逻辑
