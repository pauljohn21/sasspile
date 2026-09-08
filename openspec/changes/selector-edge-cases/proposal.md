## Why

当前 `selector` 模块基础功能已完成，但 `selector-extend` 在复杂替换场景下无法正确处理 parent/grandparent unification，且 `value_to_selector_format` 函数不支持混合类型输入（string 和 list 混合），导致多 extendees 和嵌套列表选择器传递失败。这些问题影响 sass-spec 通过率约 100+ cases。

## What Changes

- **selector-extend 算法修复**：实现完整的 parent reference 匹配与替换逻辑，支持 `.c.x .d` 替换 `.c` 为 `.e` 时生成 `.x.e .d`
- **value_to_selector_format 混合类型支持**：支持 `(c, d e)` 这种包含 string 和 list 的混合列表格式
- **错误检测增强**：当输入是无效选择器（如未闭合的属性选择器 `[c`）时正确抛出解析错误
- **selector-extend 多 extendees 格式化**：确保 extendees 列表格式正确传递

## Capabilities

### New Capabilities

- `selector-extend-unification`：完整的 parent/grandparent 匹配与替换算法
- `selector-format-mixed-input`：支持混合类型（string + list）的 selector format 输入
- `selector-parser-error-detection`：增强解析器对无效选择器的错误检测

### Modified Capabilities

- `selector-nest`：修复嵌套参数接受混合列表格式（如 `(c, d e)`）

## Impact

- `src/css/selector_format.rs`：`value_list_to_format` 函数支持混合类型输入
- `src/eval/builtin/selector.rs`：`call_extend` 函数 algorithm 支持 parent unification
- `src/css/selector_parser.rs`：增强错误检测（未闭合属性选择器等）
- 预计 sass-spec 通过率提升约 100 cases（+0.8%）
