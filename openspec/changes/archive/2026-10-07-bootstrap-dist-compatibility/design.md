# Design: Bootstrap Dist Compatibility

## Context

See proposal.md - Why.

当前状态: Bootstrap 全量编译通过（188/188 测试），输出 2324 行 / ~56KB，但 dist 参考为 12048 行 / 280KB。差距 ~80% 来自缺失的 SCSS 特性覆盖。这不是架构问题（响应式管线已稳定运行），是 SCSS 语义特性补全问题。

架构约束:
- 响应式求值器管线 (`expand → scan`) 已稳定，不需要重构
- `builtin-modules` 和 `directive-ops` spec 已有基础函数框架
- 所有改动在 `src/eval/` 模块内部完成（builtin.rs/expr.rs/mod.rs）

## Goals / Non-Goals

### Goals
- Bootstrap 5.3.x dist CSS 逐行对照覆盖率 >= 99%
- 补全 Color/Map/List/String 缺失函数
- 修复 @each 双变量、@content、@for range 等指令语义
- 建立自动化对照测试基础设施

### Non-Goals
- 不引入新的 CSS 预处理器功能（仅实现 Bootstrap 所需）
- 不修改响应式求值器架构（不动 `expand_nodes_to_events` / `eval_nodes_pipeline`）
- 不追求与 dart-sass 100% 兼容（以 Bootstrap dist 为唯一验收标准）

## Decisions

### Decision 1: 对照测试策略 — HashSet 行比对 vs LCS CSS 结构化 diff

**选择: 两阶段方案**

| 阶段 | 方法 | 精度 | 实现复杂度 |
|------|------|------|-----------|
| 阶段 1 | HashSet 行集合差 | 粗（仅检测缺失行） | 低 |
| 阶段 2 | 结构化 CSS AST diff | 细（selector + property 级） | 中 |

**理由**: Bootstrap dist 不存在格式差异（它是 dart-sass 的确定性输出），只有"行存在/不存在"问题。HashSet 已经能捕获 95% 的差距。后续如果需要定位具体 property 差异再升级到结构化 diff。

**备选**: 使用 `similar` crate 的 LCS diff — 但因为它对 CSS 无语义理解，对顺序变化会产生大量 false positive。

### Decision 2: 函数补全实现位置

**选择**: 在 `src/eval/builtin.rs` 中新增函数分派，沿用现有 `dispatch_builtin_*` 架构。

**理由**: 现有 `builtin/math.rs`, `builtin/color.rs` 等已建立文件级模块分工。新增函数独立成函数，不修改已有函数实现。

### Decision 3: rgba() CSS Variable 参数处理

**选择**: 在 `rgba()` 入口检测第一个参数是否为 `Value::CssVar` 或 `Expr::Var`，如果是则返回 `rgba(var(--name-rgb), alpha)` 格式（name 后追加 `-rgb`）。

**理由**: Bootstrap 大量使用 `rgba(var(--bs-white), 0.5)` 模式。正确的 Sass 语义要求原始 `$color` 必须提供对应的 `--color-rgb` 变量。我们直接实现这个语义映射。

**备选**: 实时解析 CSS var 值转 RGB — 但 CSS var 是运行时动态的，编译时无法确定值，Bootstrap 模式就是依赖预计算的 RGB 变量。

### Decision 4: @content 替换实现

**选择**: 在 `expand_nodes_to_events` 处理 `AstInclude` 时，将 `@content` 节点替换为调用方的 content block 克隆。

**理由**: 当前 `@content` 被解析为特殊 AstNode 但未在展开时替换。expand 阶段已有完整的 Work item 上下文，最自然的替换点就在 `@include` body 展开时。

**备选**: 在 eval_expr 层面做字符串替换 — 但 `@content` 是 AST 层概念（包含完整节点序列），不是纯字符串。

### Decision 5: @each 双变量迭代

**选择**: 在 `expand_nodes_to_events` 处理 `AstEach` 时，检测 vars 字段长度为 2 的 case，分别绑定 `$key` 和 `$value`。

**理由**: 现有 `AstEach` 只有一个 `var: String` 字段。需要扩展为 `vars: Vec<String>` 或在结构体外维护双变量模式。

## Risks / Trade-offs

| Risk | Mitigation |
|------|-----------|
| 补全函数引入 regression | 每个新函数对应一个 eval_test.rs 单元测试 |
| @content 替换破坏现有 mixin | 先运行 compile_bootstrap_full 回归 |
| HashSet diff 漏报顺序差异 | 99% 覆盖率门控 + 前 50 条差异日志 |
| 函数爆炸导致 builtin.rs 超 500 行 | 按模块拆分 builtin color.rs/string.rs/map.rs |

## Migration Plan

1. **Phase 1: 基础设施** — 搭建对照测试框架，量化当前覆盖率
2. **Phase 2: 内建函数** — 补全 color/string/map/list 缺失函数（不修改指令语义）
3. **Phase 3: 指令语义** — 修复 @each/@content/@for/@while
4. **Phase 4: Utility API** — 联合验证（函数 + 指令 + CSS var 全链路）
5. **Phase 5: 门控升级** — bootstrap_test.rs 从 ignored 变为 CI 强制

**回退策略**: 如果补全引入不可控 regression，新函数可在 builtin.rs 中临时 `unimplemented!()`，不影响已有函数。

## Open Questions

- Bootstrap dist 参考中是否有 dart-sass 特有行为（如自动 vendor prefix），rx-scss 不需要实现？ → 待 Phase 1 分析确认
- `shade-color`/`tint-color` 的具体算法（Sass spec 未精确定义，Bootstrap docs 描述与 `mix()` 等价）→ 采用 mix 实现
