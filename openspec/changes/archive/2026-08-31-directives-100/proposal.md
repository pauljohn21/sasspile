## Why

sasspile 的 `@directives` 子目录 sass-spec 通过率仅 55%（333/605 evaluated，含 skip 为 333/775 = 43%）。这是项目中失败最集中的区域之一，272 个失败测试 + 170 个 skip 阻碍了整体通过率提升。其中 `expected_error_but_ok`（~80 个）和 `content_diff`（~80 个）是两大瓶颈，修复 ROI 最高。

## What Changes

- **@use with 配置验证**：在 `@use ... with (...)` 中检测未定义变量、非默认值引用、namespace 错误、嵌套配置、重复变量、多配置冲突等 ~40 种错误场景
- **@forward 冲突与语法验证**：检测 @forward 的 variable/function/mixin 冲突（含 same_value、because_of_as）、`as` 语法验证（nothing/asterisk/no_star）、私有成员访问
- **@import 冲突与顶层声明检测**：检测 @import 文件冲突（partial/extension/all/index/import_only）、顶层 @include/@at-root 声明错误
- **@extend 跨文件传递修复**：修复 @extend 在 @use/@import 跨模块场景下的选择器传递（bogus 选择器输出、pseudo 嵌套）
- **use+import 交互输出修复**：修复 @use 和 @import 组合使用时的 CSS 输出缺失（注释顺序、嵌套导入、CSS import 位置）
- **特殊函数名序列化**：修复 calc/clamp/expression/url/element/type 等特殊函数名的序列化差异和错误检测
- **加载优先级修复**：修复 @import 加载优先级（sass/css/partial/index 之间的顺序）
- **变量遮蔽/覆盖修复**：修复 @forward/@use 中的变量遮蔽（through_forward）、覆盖（override）、优先级（precedence）差异
- **嵌套 @规则输出**：修复 @import 内嵌套 @at-rule 的 CSS 输出缺失
- **for 循环声明内输出**：修复 @for 在声明内使用时的输出格式
- **skip 解除**：逐步解除 170 个 skip（at_root 21, mixin 29, if 19, forward 30, use 30, for 21, 其他 20）

## Capabilities

### New Capabilities
- `use-with-validation`: @use with 配置参数验证（未定义变量、非默认值、namespace、嵌套、重复、多配置冲突）
- `import-conflict-detection`: @import 文件冲突检测（partial/extension/all/index/import_only）和顶层声明错误检测
- `extend-cross-file`: @extend 跨文件选择器传递（bogus 选择器输出、pseudo 嵌套、diamond 依赖）
- `use-import-interaction`: @use 和 @import 组合使用时的 CSS 输出（注释顺序、嵌套导入、CSS import 位置）
- `special-function-names`: 特殊函数名（calc/clamp/expression/url/element/type）的序列化和错误检测
- `load-precedence`: @import 文件加载优先级（sass/css/partial/index 之间的查找顺序）
- `directives-skip-removal`: 逐步解除 @directives 子目录的 170 个 skip 测试

### Modified Capabilities
- `forward-conflict-detection`: 扩展冲突检测覆盖 same_value、because_of_as、syntax/as 验证、私有成员访问
- `error-detection-coverage`: 扩展错误检测覆盖 @use with 配置验证和 @import 冲突场景
- `module-member-access`: 修复 @use/@forward 变量遮蔽（through_forward）、覆盖（override）、优先级（precedence）差异
- `dependency-management`: 修复 use+import 交互场景下的模块加载和 CSS 输出顺序

## Impact

- **src/eval/**: `module_dispatch.rs`、`env.rs` 需增强 with 配置验证和冲突检测
- **src/eval/**: `import.rs`、`use.rs`、`forward.rs` 需修复加载优先级和交互逻辑
- **src/eval/**: `extend.rs` 需修复跨文件选择器传递
- **src/css/**: `serializer.rs` 可能需调整特殊函数名序列化
- **src/parse/**: `at_rule.rs` 可能需增强 @use with 语法验证
- **tests/**: `sass_spec_full.rs` 需逐步解除 skip 标记
- **无 BREAKING 变更**：所有修复都是让输出匹配 sass-spec，不改变已通过测试的行为
