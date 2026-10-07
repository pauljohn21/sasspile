# Proposal

## Why

Bootstrap 5.3.x 全量编译产物仅覆盖参考 CSS 的 22%（2674/12048 行），阻塞项不是架构问题（响应式管线已稳定运行，188/188 测试通过），而是 evaluator 层缺失关键语义特性：嵌套 `$utilities` map 三层 `@each` 迭代**因作用域链或 map-get 提取问题未能**生成 spacing/display 工具类、`@include media-breakpoint-up/down` mixin 调用链未能正确展开为 `@media` 包裹块。补全这些语义特性预计可将覆盖率提升至 60%+。

## What Changes

- **诊断并修复嵌套 map 迭代**：现有 `@each` 已实现 Map→Pairs 转换，但三层嵌套迭代失败。需先诊断真实根因（作用域链 / map-get / panic），再针对性修复
- **修复响应式 mixin 展开**：使用 `flat_map` + `collect()` 消费旧事件流，生成带 `@media` 包裹的新事件流
- **CSS Custom Properties 验证**：`--bs-*-rgb` 系列变量（约 50+ 个）全部由 `to-rgb()` 或 `rgba()` var 模式生成
- **CSS var() fallback 修复**：`var(--bs-card-cap-bg, transparent)` 正确传递 fallback 值
- **新代码强制函数式风格**：所有集合变换使用 `into_iter().map().collect()`，禁止 `for + Vec::push`

## Capabilities

### New Capabilities
- `utility-api-generation`: 从嵌套 SCSS map 生成完整 Utility 类（spacing、display、flex、sizing），包括响应式断点变体

### Modified Capabilities
- `directive-ops`: 添加响应式 media-breakpoint mixin 展开规则（@media 包裹语义，使用 flat_map 消费旧事件流）
- `bootstrap-dist-alignment`: 收紧覆盖率门控从"已实现"提升到">= 60% 覆盖率"

## Impact

- **代码**: `src/eval/mod.rs`（`expand_nodes_to_events` 内 `@each`/`@include` 分支 + 新增诊断 span）
- **测试**: `tests/integration_test.rs`（新增 tool/display 端到端测试）、`tests/bootstrap_test.rs`（覆盖率门控阈值）
- **依赖**: 无新增
- **风险**: 作用域链诊断可能耗时；修复可能涉及 `child_scope()` 实现调整（需 trace 证据指导）
- **风格约束**: 新增代码强制迭代器链风格，可能与现有命令式模拟框架共存（渐进迁移）
