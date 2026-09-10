## Why

当前 `selector.extend` 对伪类选择器（`:not()`, `:is()`, `:where()`, `:matches()`, `:nth-child()` 等）的处理不完整，导致 65 个 sass-spec case 失败。这些伪类有独特的 Sass 扩展语义，需要特殊处理逻辑。

## What Changes

- **新增**: `:not()` extend 算法——在 compound 上追加新的 `:not()` 而非替换内部
- **新增**: `:is()`, `:where()`, `:matches()` subselector 检测——避免无效扩展
- **新增**: 伪类参数匹配逻辑——支持 `:nth-child()` 等参数化伪类
- **修改**: `extend_complex` 函数增加伪类感知分支
- **修改**: `extend_selector_with_mode` 增加 `:not()` 特殊路径

## Capabilities

### New Capabilities
- `pseudo-not-extend`: `:not()` 选择器的特殊 extend 语义（追加 `:not()` 到 compound）
- `pseudo-is-where-matches`: `:is()`, `:where()`, `:matches()` 的 subselector 检测和 no-op 判断
- `pseudo-args-match`: 带参数伪类（`:nth-child`, `:nth-last-child` 等）的匹配逻辑

### Modified Capabilities
- `selector-extend`: extend 算法增加伪类感知分支

## Impact

- 影响文件: `src/css/selector_extend.rs`, `src/css/selector_ops.rs`
- 影响测试: `tests/sass-spec-failures.json`（预期减少 65 个失败）
- API 变更: 无（内部算法改进）
- 依赖变更: 无

## 预估收益

- sass-spec: +40~65 cases（伪类相关）
- 总体通过率: 62.7% → ~65%
