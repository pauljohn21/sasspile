## ADDED Requirements

### Requirement: Keyframes with mixin include
当 `@keyframes` 内部使用 `@include` mixin 且 mixin 输出多声明时，系统 SHALL 将声明内联到对应的 keyframe 百分比块内，保持正确的嵌套结构。

#### Scenario: Dialog/Drawer slide-in keyframes
- **WHEN** dialog.scss 中 `@keyframes dialog-fade-in { 0% { @include slide-in-from-top; } 100% { ... } }`
- **THEN** mixin 展开后声明在 `0% { ... }` 内正确嵌套
- **AND** 不因 @at-root 破坏 keyframes 上下文

### Requirement: Empty keyframe block preservation
当 keyframe 百分比块内 content 为空时，系统 SHALL 输出空块而非省略百分比选择器。

#### Scenario: Empty 100% block
- **WHEN** `100% { }` 块内无任何声明
- **THEN** 输出 `100% {}`（保留空结构以匹配 EP）

### Requirement: Calc expression in keyframes
当 keyframe 块内使用 `calc()` 表达式时，系统 SHALL 正确序列化并保留在 keyframe 上下文。

#### Scenario: Transform with calc
- **WHEN** `50% { transform: translateY(calc(-50% + 10px)); }`
- **THEN** `calc()` 输出正确，不因 keyframe 上下文丢失
