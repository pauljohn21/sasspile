## ADDED Requirements

### Requirement: Declaration merge in grouped extend
当多个选择器共同 extend 同一 %placeholder 时，系统 SHALL 输出逗号分隔的选择器组，声明集中在一对 `{}` 内，而非每个选择器独立成规则。

#### Scenario: Shared placeholder with multiple selectors
- **WHEN** `.a, .b, .c { @extend %shared; }`
- **THEN** 输出 `.a, .b, .c { /* shared declarations */ }`

#### Scenario: Competing declaration merge
- **WHEN** `.a { color: red; @extend %shared; }` 和 `%shared { padding: 0; }`
- **THEN** `.a { padding: 0; color: red; }`（placeholder 声明前置或按 dart-sass 规则）

### Requirement: Placeholder with descendant selector extend
当 %placeholder 内部使用后代选择器且被顶层规则 extend 时，系统 SHALL 在输出中保留完整选择器链。

#### Scenario: EP component with nested selectors
- **WHEN** `%has-icon { & + .el-input__suffix { ... } }` 被 `.el-input--suffix { @extend %has-icon }`
- **THEN** 输出 `.el-input--suffix + .el-input__suffix { ... }`（正确组合 & 上下文）

### Requirement: Merge output rule ordering
当多个规则 extend 同一 %placeholder 且有自身声明时，系统 SHALL 按原文顺序排列输出规则。

#### Scenario: Sequential rule ordering
- **WHEN** 文件末尾 `.a @extend %x; .b @extend %x; .c @extend %x;`
- **THEN** 输出中 .a/.b/.c 规则按 .a/.b/.c 原文顺序，不重组
