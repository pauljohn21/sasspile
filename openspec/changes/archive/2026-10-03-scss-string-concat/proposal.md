## Why

EP 全量对比中剩余 91/121 文件不一致，根因是 SCSS **字符串拼接表达式** (`'.' + $B + '__' + $unit + ','`) 输出乱码而非正确连接结果。输出呈现 `\.\' \ \+ \a \  \  \  \  \  \ null\ \+ \a \ __\ \+ \a \ separator...` 形态，每个字符被单独转义并分隔，说明 `+` 运算符未正确实现字符串连接语义。

此 bug 阻断 EP 所有使用 BEM mixin 的组件（breadcrumb、button-group、calendar、card、carousel、cascade 等约 60 个文件），一致性无法突破 21/121 (17.4%)。

## What Changes

- **修复字符串 `+` 运算符**：`'a' + 'b'`、`'prefix' + $var + 'suffix'` 正确返回连接后的字符串
- **修复混合类型连接**：`Number + String`、`String + Number` 等混合类型的字符串化逻辑
- **修复插入语上下文**：`#{$a + '.' + $b}` 正确展开为 `value_a.value_b`
- **添加对比回归测试**：确保 BEM selector 构建结果正确

## Capabilities

### New Capabilities
- `string-concat-fix`: SCSS `+` 运算符正确实现字符串连接语义

### Modified Capabilities
- 不改变已通过的 sass-spec 行为（如果现有行为正确）

## Impact

- **受影响代码**: `src/eval/value/ops.rs`（`Add` 操作的字符串处理路径）、`src/eval/value/calc_ast.rs`
- **API 变更**: 无公开 API 变更
- **性能影响**: 无
- **风险**: 修复可能影响依赖当前行为的 case；需全量 sass-spec 回归确认
