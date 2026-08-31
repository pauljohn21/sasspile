## ADDED Requirements

### Requirement: @use 和 @import 组合 CSS 输出

系统 SHALL 正确处理 @use 和 @import 组合使用时的 CSS 输出顺序和内容。

#### Scenario: use_into_use 输出
- **WHEN** 文件通过 @use 引入另一个文件，该文件也通过 @use 引入第三个文件
- **THEN** 系统 SHALL 按正确顺序输出 CSS（import_above_rule / import_below_rule 模式）

#### Scenario: use_into_import 输出
- **WHEN** 文件通过 @import 引入另一个文件，该文件通过 @use 引入第三个文件
- **THEN** 系统 SHALL 按正确顺序输出 CSS（css_import_above_rule / css_import_below_rule 模式）

#### Scenario: import_into_use 输出
- **WHEN** 文件通过 @use 引入另一个文件，该文件通过 @import 引入第三个文件
- **THEN** 系统 SHALL 按正确顺序输出 CSS（css_import_above_rule / css_import_below_rule 模式）

#### Scenario: 注释顺序
- **WHEN** 文件中注释、CSS 规则和 @use/@import 混合出现
- **THEN** 系统 SHALL 按正确顺序输出注释和 CSS 内容

#### Scenario: 嵌套 import 到 use
- **WHEN** @import 嵌套在 @use 加载的文件中
- **THEN** 系统 SHALL 正确处理嵌套导入，输出完整 CSS

#### Scenario: use module used by import
- **WHEN** 同一模块同时被 @use 和 @import 引用
- **THEN** 系统 SHALL 正确处理模块重用，避免重复 CSS 输出
