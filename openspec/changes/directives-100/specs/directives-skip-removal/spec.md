## ADDED Requirements

### Requirement: 逐步解除 @directives skip 测试

系统 SHALL 逐步解除 sass_spec_full 中 @directives 子目录的 skip 标记，每批不超过 10 个，解除后立即验证。

#### Scenario: at_root skip 解除
- **WHEN** 解除 `directives/at_root` 的 21 个 skip 测试
- **THEN** 系统 SHALL 对每个测试验证输出匹配 sass-spec 或记录失败原因

#### Scenario: mixin skip 解除
- **WHEN** 解除 `directives/mixin` 的 29 个 skip 测试
- **THEN** 系统 SHALL 保持 100% 通过率

#### Scenario: if skip 解除
- **WHEN** 解除 `directives/if` 的 19 个 skip 测试
- **THEN** 系统 SHALL 验证每个 if 测试的分支逻辑

#### Scenario: forward skip 解除
- **WHEN** 解除 `directives/forward` 的 30 个 skip 测试
- **THEN** 系统 SHALL 验证 forward 配置和冲突检测

#### Scenario: use skip 解除
- **WHEN** 解除 `directives/use` 的 30 个 skip 测试
- **THEN** 系统 SHALL 验证 use 交互和作用域隔离

#### Scenario: for skip 解除
- **WHEN** 解除 `directives/for` 的 21 个 skip 测试
- **THEN** 系统 SHALL 验证 for 循环边界条件
