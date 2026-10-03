## Context

EP 一致性已达 73/121 (60.3%)。Phase 1-6 已完成：CSS AST 架构统一、@atRootDirect 优化、AtRoot + @content 顺序一致性、@extend %placeholder 选择器分组（含 mixin 传播指数膨胀 bug 固定）。剩余 44 DIFF 文件按根因分为 5 类 capability 修复。

**关键约束**：
- sasspile 是纯 Rust 函数式架构，禁止参照 dart-sass
- 单一文件 ≤ 500 行
- 所有代码使用 tracing 宏 + span
- 修复必须遵循 4 步调试协议

## Goals / Non-Goals

**Goals:**
- EP 一致性从 73/121 提升至 121/121（100%）
- 每个修复点有明确的 spec 场景覆盖
- 不引入 sass-spec 回归（当前 65.7%）
- 不破坏现有核心测试（202/202 必须保持）

**Non-Goals:**
- 不重构核心管线架构（eval/serialize 已稳定）
- 不修复 dart-sass 兼容性问题以外的功能问题
- 不处理 color 测试（已隔离）

## Decisions

### Decision 1: 伪元素格式统一在 Sequencer 层处理

**问题**：`:before` 在部分 extend 路径输出为 `: before`，在 keyframes 中输出不理想。

**决定**：在 CssSerializer 层统一处理伪元素规范化，确保 `:before`/`:after`/`:first-line`/`:first-letter` 输出单冒号格式，`::before`/::after`/`::placeholder` 输出双冒号格式。extend/mixin 路径不应产生格式差异。

**替代方案**：在 extend.rs / mixin.rs 中逐个修复格式问题——被拒绝，因路径太多且易遗漏。

**理由**：Serializer 是 CSS 输出唯一出口，统一在此处理可永生消除格式差异。

**影响文件**：`src/eval/serializer.rs` 或相关输出模块。

### Decision 2: placeholder extend 子层级传播采用"增量快照"算法

**问题**：Phase 6 修复了 exec_mixin 中的指数膨胀 bug（通过 `&returned_env.get_extends()[base_len..]`），但 placeholder 在嵌套规则内的 extend 仍需传播到正确层级。

**决定**：在 `transform_nodes` 中，对每个嵌套层级独立执行 extend 应用。嵌套规则内的 `@extend %ph` 只影响当前层级及以下，不提升到外层。

**替代方案**：全局 flat extend 后递归分发——被拒绝，破坏 SCSS 作用域语义。

**理由**：增量传播 + 层级隔离保证声明在正确的祖先上下文中应用。

**影响文件**：`src/eval/extend.rs`

### Decision 3: @keyframes 内部求值采用延迟展开

**问题**：mixin 在 @keyframes 内的 @include 会产生中间 CssNode，需要正确嵌套在百分比块内。某些情况下 AtRootDirective 会错误提升。

**决定**：在 eval_at_rule 处理 @keyframes 时，对子节点使用 Context 标记 `{ in_keyframes: true }`，抑制 @at-root 提升逻辑。

**替代方案**：在 keyframe 序列化时过滤 at-root 节点——被拒绝，不够精确。

**理由**：上下文标记在 eval 阶段即可精确控制行为，不影响其他 @rule 类型。

**影响文件**：`src/eval/mixin.rs` (eval_at_rule)

### Decision 4: 模块变量解析增加"提升回溯"

**问题**：@use 导入的变量在嵌套 mixin include 的深层上下文中偶发不可见。

**决定**：在 `bind_exports` 时，将模块变量绑定复制到当前 mixin 即将进入的作用域内（而非依赖 parent chain 延迟查找），确保 mixin 在任意嵌套深度都能访问模块变量。

**替代方案**：在变量查找时增加 fallback 链——被拒绝，隐藏了真实的绑定问题。

**理由**：EP 文件中存在 @use → mixin → deep-nesting 的变量引用链，提升回溯确保变量在入口点已可见。

**影响文件**：`src/eval/env_impl.rs` (bind_exports), `src/parse/module_helpers.rs`

### Decision 5: 并行诊断 + 批量修复策略

**问题**：44 DIFF 文件修复需要系统性分析，不能逐个猜测。

**决定**：
1. 一次性运行 `ep_normalized_test` 收集完整 diff 信息
2. 用 CodeGraph 定位每个差异的代码路径根因
3. 按 capability 分类后批量实现修复
4. 每个 capability 修复后运行 `ep_normalized_test` 验证

**影响文件**：`tests/ep_normalized_test.rs`, `tests/ep_diag.rs`

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| 伪元素格式修复导致 sass-spec 回归 | 修复前后对比 sass-spec 统计，如有回滚需精准定位是哪个伪元素路径 |
| placeholder extend 层级隔离破坏 sass-spec 行为 | sass-spec 已有 placeholder extend 测试，修复后必须重跑全量 |
| 模块变量"提升回溯"改变现有绑定语义 | 仅在 mixin @include 入口作用域提升模块变量，不影响普通规则 |
| 44 文件修复引入新 diff | 按 capability 分阶段提交，每阶段对比 baseline |
| 修复耗时过长超过 500 行限制 | 每个 capability 拆分为子模块/子函数 |

## Migration Plan

1. 分 5 个 capability 顺序修复
2. 每个 capability 修复后运行 `cargo test --test ep_full` + `ep_normalized_test`
3. 每完成一个 capability 提交一次（等用户确认再 push）
4. 全部完成后运行 `SPEC_STORE_CMD=run` 确认 sass-spec 无回归

## Open Questions

1. Q: 伪元素 `: before` 空格是否来自 extend 合并时的选择器拼接？还是源文件本身？
   A: 需 span 插桩 extend.rs transform_nodes 后确认。

2. Q: 嵌套 keyframes 内的 @content 是否也需要 in_keyframes 标记？
   A: 待检查 @content 在 keyframes 内的使用模式后决定。

3. Q: base.scss 的 DIFF 是否全部来自颜色序列化格式？
   A: 首轮分析确认 base.scss 差异来源后再决定修复方案。
