## Context

**问题域**：sasspile 的 SCSS → CSS 编译管线在 3 个选择器处理场景中未能输出与 sass-spec 规范 + EP dist（SCSS 编译器产物）一致的 CSS，导致 EP (Element Plus) 8 个文件存在差异。差异模式纯正：均属于选择器展开路径问题，不涉及 EP 管线后处理（autoprefixer / lightningcss 那些归类为管线产物，另有 22 files，跟本变更无关）。

**行为验证标准**：本变更以 **sass-spec 官方测试套件**的行为规范为正确性标准，以 **EP dist（element-plus 官方构建产物）** 作为该规范在 EP 这一组 SCSS 文件上的具体行为结果对照。sasspile 编译结果应与 EP dist 100% 匹配（排除 22 files EP 管线产物）。

**当前实现关键路径**：
```
Parser → Evaluator::eval_nodes → eval_rule (selector resolution)
     ↓                              ↓
  Env.current_selector         RuleBuilder::push → combine_selectors
     ↓                              ↓
  @(include e()/m()/pseudo)    push_atroot_direct
     ↓                              ↓
  @at-root → AtRoot → exec_mixin → AtRootDirect
```

**四个故障点的根因分析**：

### 1. step-ampersand (literal `&` in output)

Step `/ last-of-type` 是 SCSS 标准选择器嵌套 sass-spec 测试覆盖范围：

- `pseudo(last-of-type)` mixin: `@at-root #{&}#{':#{$pseudo}'}` — 由 mixin `$selector: &` 捕获父引用，然后字符串插值为 `.el-step:last-of-type`
- mixin exec 返回 `AtRoot([Rule("&.is-flex")])` → 转为 `AtRootDirect(Rule("&.is-flex"))`
- `push_atroot_direct` 处理后，`&.is-flex` 需要结合 resolved_selector `.el-step:last-of-type` 展开为 `.el-step:last-of-type.is-flex`
- BUG：push_atroot_direct 内对 kid selector 的 combine 路径，literal `&` 未被展开 — 字面 `&` 存活到最终 CSS（违反 SCSS 规范：literal `&` 必须被替换）
- 修复方向：`push_atroot_direct` 中 combine child selector 时**强制展开字面 `&`**，使 `&.is-flex` → `resolved.is-flex`

### 2. sel-doubling (`.el-anchor .el-anchor.el-anchor--vertical`)

`sass-spec: parent-selector/ampersand-nesting` 官方测试覆盖此类：

- anchor.scss: `&.#{$namespace}-anchor--vertical { @include e(marker) { ... } }`
- `e(marker)` mixin 内部 `@at-root { .el-anchor__marker, { @content } }` — 按 sass-spec @at-root semantics，@at-root 后(inner selector)的输出应当在外层 RuleBuilder 环境下进一步 nest，最终输出 `.el-anchor.el-anchor--vertical .el-anchor__marker`
- BUG：`push_atroot_direct` 的 else 分支（clean_sel 不含 `&`、不以 `:` `[` 开头）将 `.el-anchor__marker` 视为完整路径直接使用，跳过必要的 nest 步骤
- sass-spec 规范要求：**mixin 输出 @at-root inner selector 后，outer selector context 仍需 nest 一次**
- 修复方向：else 分支改为**始终用 `combine_selectors` nest**

### 3. rate-focus-visible (`.el-rate .el-rate:focus-visible`)

相同根因路径：
- rate.scss: `&:focus-visible { @include e(item) { ...sass-spec cover- ... } }`
- 预期 (按 sass-spec nesting + @at-root): `.el-rate:focus-visible .el-rate__item .el-rate__icon.is-focus-visible`
- 实际 (sasspile): `.el-rate .el-rate:focus-visible .el-rate__item .el-rate__icon.is-focus-visible`（多一层 `.el-rate ` 前缀 — sel-doubling 同源 bug）

### 4. color-picker-panel 选择器缺失中间层（Phase 跳过）

BEM 嵌套链 `is-disabled e(color-selector)` 输出的选择器中间层缺失，属于另一个独立的 e() mixin scope chain 问题，留后续 batch。

## Goals / Non-Goals

**Goals:**
- 修复 `push_atroot_direct` 使 literal `&` 在 mixin 出口 always expand（匹配 sass-spec expand-parent-ref 规范）
- 修复 else 分支 selector nesting 缺失（匹配 sass-spec @at-root + outer-rule nest 规范）
- 修复后 sass-spec `at-root/` `parent-selector/` `nesting/` 所有相关 case 必须 PASS
- EP consistency 从 83/121 → 预期 **88~91/121**
- 核心测试 202/202 + ep_normalized_test 通过
- sass-spec 全量不退化

**Non-Goals:**
- 不修复 EP 管线产物差异 (22 files — autoprefixer/lightningcss)
- 不修复 bem-nesting-header/color-picker-panel (独立 batch)
- 不修改颜色系统
- 不重构 Reactor 管线架构

## Decisions

### Decision 1: 在哪里修复 literal `&` 展开失败？

**选项 A**: `eval_rule` 入口添加 fallback 父选择器传播 (top-level + `&` suffix → 使用父级)
**选项 B**: `push_atroot_direct` 中 combine child selector 时**强制展开字面 `&`**（当前 examine 的路径）
**选项 C**: Serializer 后处理 transform literal `&`

**选择：B**。理由：最直接修复 — `&.is-flex` 在 push_atroot_direct 内 combine 时必须替换为 `&` → `.el-step:last-of-type`（resolved_selector），符合 sass-spec "当 & 出现在 top-level compound selector 位置时必须展开为父选择器" 规范。C 被拒绝（CSS 后的 workaround 不合规）。

### Decision 2: 选择器重复前缀 nest 策略

**选项 A**: `push_atroot_direct` else 分支改为始终 `combine_selectors(&resolved_selector, clean_sel)` 而非直接使用
**选项 B**: 添加全局 `child.starts_with(parent)` 在 `combine_selectors` 入口判断
**选项 C**: 在 Serializer 层 dedupe compound selectors

**选择：A**。理由：else 分支当前实现的 "视为完整路径直接使用" 假设错误 — `@at-root` 语义只保证 inner selector 输出在根位置与 parent context 分离，但 outer rule 的 nest chain 仍需应用。**mixin 输出的 selector 总是需要与调用者 selector nest**。A 强制 nest 保持与 `clean_sel starts_with ':' / '['` 分支一致行为。

### Decision 3: 如何处理 bem-nesting-header？

**本次跳过**。理由：需要独立的 e() mixin scope chain 根因分析，不在本变更范围。

## Risks / Trade-offs

- **[R1: push_atroot_direct else 分支改动]** → 影响所有 EP 文件中 `#{&} `→ 字面 selector 经由 e() mixin 生成后输出的 nest path → Mitigation: sass-spec /nesting 与 /at-root 全量 + ep_normalized_test
- **[R2: literal `&` 强制 fallback 条件]** → top-level `&.is-flex` 需要知道有效的父级 → Mitigation: 仅在 `resolved_selector` 有效（非空、非 @-prefixed）时 fallback
- **[R3: sass-spec 可能新增 breakage]** → 改动 combine nest 路径可能影响既有 sass-pass 用例 → Mitigation: SPEC_STORE_CMD=run 全量 + diffsnapshot 比较
- **[R4: 修复后 EP dist 仍未完全对齐]** → 部分选择器差异可能属 EP 特有的 build-time transform，非 sass-spec 规范要求 不强求对齐

## sass-spec 验证策略

每修复一个 bug 后必须执行：

1. **快速门控**: `cargo test --test compile_test --test stage_test --test ast_test --test common_test` 确认 202/202 PASS
2. **行为门控**: `cargo test --test reactor_test` 确认 14/14 PASS（Reactor 管线完整性）
3. **EP 验证**: `cargo test --test ep_normalized_test -- --nocapture` 确认 IDENTICAL 计数 >= 当前基线 83
4. **sass-spec 门控**: `SPEC_STORE_CMD=run cargo test --test spec_store -- --nocapture` 30 秒确认通过率 >= 57%（3097/5362），无负向 regression

sass-spec 细节 case 验证：优先检查 `tests/sass-spec/spec/css/at_root/` `tests/sass-spec/spec/css/parent_selector/` `tests/sass-spec/spec/directives/at_root/` 三个子目录的所有 case 必须 PASS。
