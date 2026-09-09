## Context

选择器模块（`src/css/selector_*.rs` + `src/eval/builtin/selector.rs`）当前通过率 45%（404/893），535 个 spec 失败。根因是 2026-09-04 selector-ast 变更时建立了正确的 AST 层级结构（`Selector → ComplexSelector → CompoundSelector → SimpleSelector`），但 `SimpleSelector::Type(String)` 保留了不透明字符串表示，导致命名空间、伪元素二义性、伪类链式三个核心语义无法实现。

当前数据流：`Value → SelectorFormat（字符串）→ parse_selector → AST → 算法 → to_string → Value`。算法层（`selector_ops.rs`）的 `unify_compound`/`is_super_compound`/`extend_complex` 全部基于字符串级比较，无法感知命名空间语义。

## Goals / Non-Goals

**Goals:**
- 将选择器通过率从 45% 提升至 73~84%（+250~350 passes）
- 建立命名空间感知的数据模型，统一 unify/extend/is_superselector 三套算法
- 修复伪元素单/双冒号归一化和伪类链式合并
- 保持外部 API 不变（`selector.unify()`/`extend()`/`is-superselector()` 签名不变）

**Non-Goals:**
- 不改变 `Selector → ComplexSelector → CompoundSelector` 层级结构
- 不触及 `@extend` 指令层（`directives/extend` 的 32 fails 独立问题）
- 不实现选择器嵌套（`@at-root` 等高级特性）
- 不改变 CSS 输出层的选择器序列化（`css/selector.rs` 独立模块）

## Decisions

### Decision 1: Namespace 作为独立枚举 vs 内联到 Type

**选择**：独立 `Namespace` 枚举

**Rationale**：
- 独立枚举使统一规则矩阵可表达（`match (ns_a, ns_b)` 四象限）
- `Namespace::Explicit(String)` 利用 `String` 所有权，无需生命周期
- 未来扩展（如 `@namespace` 声明）只需增加枚举变体

**替代方案**：`Type { namespace: Option<String>, name: String }` — 无法区分 `None`（无前缀）和 `Some("")`（空命名空间），语义模糊。

### Decision 2: 伪元素归一化策略

**选择**：归一化到 class syntax（单冒号）

**Rationale**：
- sass-spec 期望输出 `:before` 而非 `::before`
- 比较时归一化 `is_class_syntax` 字段，输出时保持归一化形式
- 不影响 CSS 输出层（`css/selector.rs` 独立处理 legacy 语法）

**替代方案**：归一化到 element syntax — 与 spec 输出不符，需要额外转换。

### Decision 3: 伪类链式合并策略

**选择**：同 name 覆盖、不同 name 链式

**Rationale**：
- `:c` + `:d` → `:c:d` 符合 CSS 规范
- `:nth-child(2n)` + `:nth-child(3n)` → `:nth-child(3n)`（后者覆盖）
- 实现：`unify_compound` 中伪类按 name 分组，同 name 取后者，不同 name 全部保留

**替代方案**：不允许链式 — 与 spec 冲突，`:c` + `:d` 应产生 `:c:d`。

### Decision 4: 统一算法中 Universal 与命名空间 Type 的交互

**选择**：`*`（`SimpleSelector::Universal`）与无命名空间 Type 兼容，与有命名空间 Type 不兼容

**Rationale**：
- spec 测试用例：`unify("*", "c|d")` → `null`，`unify("*", "c")` → `"c"`
- `*` 隐含 "any namespace OR no namespace"，但与显式命名空间 `c|d` 不兼容
- 实现：`unify_compound` 中 Universal + Type 分支检查 namespace

**替代方案**：`*` 与任何 Type 兼容 — 与 spec 冲突。

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| extend 算法复杂，NO-OP 边界多 | 每个 spec case 对应一个 scenario，按 scenario 逐步验证 |
| 命名空间规则矩阵遗漏边界 | 从 universal.hrx / and_type.hrx / and_universal.hrx 反推完整矩阵 |
| 伪类链式改变现有通过测试 | 全量跑 sass_spec_full 对比 before/after |
| 数据模型改造波及 Display/Debug | 全量搜索 `SimpleSelector::Type` 引用点（19 个符号），逐一适配 |

## Migration Plan

1. **Day 1**：`selector_ast.rs` — 定义 `Namespace` + 改造 `Type` + 更新 `Display`
2. **Day 2**：`selector_parser.rs` — `take_type_with_ns` 输出结构化 `Namespace`
3. **Day 3**：`selector_ops.rs` — `unify_compound`（命名空间矩阵 + 伪类链式 + 伪元素归一）
4. **Day 4**：`selector_ops.rs` — `is_super_compound` + `extend_complex` + `compounds_conflict`
5. **Day 5**：`selector_format.rs` + `selector.rs` — 适配 + 全量测试验证

**回退策略**：独立 change，`git revert` 即可回退到当前状态。

## Open Questions

1. **`:not()`/`:is()`/`:where()`/`:has()` 伪类参数是否需解析为子选择器？** — 当前 arg 为 `String`，暂不改。spec 中这些伪类的参数保持字符串级比较。
2. **`@extend` 指令的 32 fails 是否同步修复？** — 否，`@extend` 走独立路径（`eval/extend.rs`），不在本次范围。
3. **命名空间规则是否覆盖所有 spec case？** — 从 universal.hrx (29) + and_type.hrx (23) + and_universal.hrx (17) 反推，覆盖 69 个已知 case。
