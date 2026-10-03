## ADDED Requirements

### Requirement: 空的首尾 keyframe 步骤在序列化时被移除
`@keyframes` 规则中，声明块完全为空的首尾步骤（`0%{}` / `100%{}` / `from{}` / `to{}`）SHALL 在序列化前被移除。这是 CSS Animations spec 的标准行为，空步骤无视觉效果。

#### Scenario: v-modal-in — 尾部 100% 空步骤被移除
- **WHEN** SCSS `@keyframes v-modal-in { 0% { opacity: 0 } 100% {} }`
- **THEN** 序列化输出 SHALL 为 `@keyframes v-modal-in { 0% { opacity: 0 } }`（100% 空步骤移除）

#### Scenario: v-modal-out — 头部 0% 空步骤被移除
- **WHEN** SCSS `@keyframes v-modal-out { 0% {} 100% { opacity: 0 } }`
- **THEN** 序列化输出 SHALL 为 `@keyframes v-modal-out { 100% { opacity: 0 } }`（0% 空步骤移除）

#### Scenario: 中间空步骤不移除
- **WHEN** SCSS `@keyframes x { 0% { opacity: 0 } 50% {} 100% { opacity: 1 } }`
- **THEN** 序列化输出 SHALL 保留 50% 空步骤（只有首尾空步骤被移除）

#### Scenario: 全部非空步骤不受影响
- **WHEN** SCSS `@keyframes x { 0% { opacity: 0 } 50% { opacity: 0.5 } 100% { opacity: 1 } }`
- **THEN** 输出 SHALL 完整保留所有步骤

#### Scenario: 仅有一个空步骤的 keyframes
- **WHEN** SCSS `@keyframes x { 0% {} }`
- **THEN** 输出 SHALL 为 `@keyframes x { }`（唯一步骤为空则整个 keyframes 为空块）

#### Scenario: from/to 等价写法
- **WHEN** SCSS `@keyframes x { from { opacity: 0 } to {} }`
- **THEN** 输出 SHALL 为 `@keyframes x { from { opacity: 0 } }`（to 等价于 100%）

### Requirement: 空步骤剥离不影响 @keyframes 内的非 Rule 节点
`@keyframes` children 中的注释等非 Rule 节点 SHALL 在空步骤剥离过程中保持不变。

#### Scenario: keyframes 含注释
- **WHEN** `@keyframes x { 0% { opacity: 0 } /* comment */ 100% {} }`
- **THEN** 输出 SHALL 保留 `/* comment */` 并移除 100% 空步骤
