## Why

sasspile 的 directives 测试通过率当前为 87.8%（15660/17844），距离 100% 还差 2184 个 case。这些失败集中在 7 类可修复的模式中，修复后将显著提升 sass-spec 整体通过率。

## What Changes

按优先级依次修复以下 7 类失败模式：

1. **指令关键字后注释跳过**: `@for`/`@mixin`/`@if`/`@function` 解析入口增加 comment skip 调用
2. **@function 名称大小写折叠**: function resolver 统一进行大小写不敏感匹配，确保 `ELEMENT()` 与 `element()` 等价
3. **@forward !default 变量传播**: evaluate_forward 转发 scope 时正确合并 `!default` 变量的"已定义则保留"语义
4. **@at-root 内嵌 @use**: 放宽 at-rule 嵌套限制，允许 `@at-root` 内部使用 `@use`
5. **@use CSS 输出排序**: 序列化阶段保持 `@use`/`@import` 与 CSS 规则的相对顺序和注释位置
6. **@use + @extend 跨模块可见性**: 实现通过 `@use` 导入的选择器对 `@extend` 可见，支持 private selector（`%-name`）不可 extend
7. @import 跨文件变量传播**: 修复 `@import` 共享作用域中 importing context 的变量对 imported file `!default` 变量的覆盖

## Capabilities

### New Capabilities
- `directive-comment-skipping`: 指令关键字后任意位置注释（`/* */`、`//`）的正确跳过
- `function-case-insensitive`: 用户定义函数名大小写不敏感匹配
- `forward-default-propagation`: @forward 链上 !default 变量的正确覆盖传播
- `at-root-use-nesting`: @at-root 内部允许嵌套 @use 规则
- `use-css-ordering`: @use/@import 与 CSS 混合时的输出顺序保留
- `use-extend-visibility`: @use 模块选择器的 @extend 跨模块可见性
- `import-variable-scope`: @import 共享作用域中 !default 变量的正确覆盖

### Modified Capabilities
无

## Impact

- **受影响模块**: `src/parse/parser.rs`（指令解析）, `src/eval/eval_forward.rs`（forward 处理）, `src/eval/eval_import.rs`（import 处理）, `src/eval/function_resolve.rs`（函数查找）, `src/css/serializer.rs`（输出排序）
- **测试影响**: 核心测试需维持 202/202 全通过，sass-spec 预计 +1500~2184 passes
- **API 变更**: 无公开 API 变更
- **性能影响**: 无显著影响
