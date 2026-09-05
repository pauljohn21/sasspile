## Context

sasspile 是纯 Rust SCSS 编译器（edition 2024 / toolchain 1.97），架构为链式 stage 管线：Source → Lexed → Parsed → Evaluated → Serialized → CSS。当前 sass-spec 通过率 52%（6209/12086），缺失集中在 4 个维度：

1. **CSS at-rules 全链路缺失** — @keyframes/@font-face/@page/@charset/@namespace/@layer 从 parser 到 serializer 均未实现
2. **meta 反射函数不完整** — feature-exists/content-exists/global-variable-exists 等边界行为未对齐规范
3. **颜色算法偏差** — scale/change/invert 的计算逻辑与 sass-spec 期望值存在系统性差异
4. **CSS 细节** — @supports 嵌套、CSS custom properties、selector.replace() 等

## Goals / Non-Goals

**Goals:**
- 补全所有 CSS at-rules（parser + eval + serializer 全链路）
- 修复 meta 反射函数使符合 sass-spec
- 修复颜色算法精度
- 补齐 CSS custom properties / selector-operations / @supports 细节
- 通过率从 52% 提升至 70%+

**Non-Goals:**
- 不改变公开 API（`Source::lex()?.parse()?.evaluate()?.serialize()` 链式调用保持不变）
- 不优化性能（先正确后性能）
- 不新增 sass-spec 不支持的实验性功能
- 不参照 dart-sass 实现（Rust 所有权模型优先）

## Decisions

### 决策 1: CSS at-rules 解析策略

**选择**: 在现有 `AtRuleKind` 枚举 + `parse_at_rules.rs` 框架内扩展，新增 Keyframes/FontFace/Page/Charset/Namespace/Layer/Container 变体和对应 parse 方法。

**替代方案**: 引入独立的 at-rule 插件注册表 — 过度设计，当前枚举 + match 更符合项目函数式风格。

**理由**: 项目已有 `AtRuleKind::Keyframes/FontFace/...` 定义，只需补全从识别到 CssNode 完整链路。

### 决策 2: 颜色算法修复策略

**选择**: 在 `color_adjust.rs` / `color.rs` 中定位偏差函数，用 tracing span 插桩 → trace 采集对比 → 根因定位 → 修复（遵循 4 步调试协议）。

**替代方案**: 参照 dart-sass — ⛔ 禁止（Rust 所有权语义完全不同）。

**理由**: sass-spec 提供精确期望值，可直接对比。

### 决策 3: meta 反射函数修复策略

**选择**: 在 `manual_dispatch.rs` 中修复 arm，feature-exists 维护静态 feature 集合，content-exists 检查 env 上下文标志。

**替代方案**: 将 meta 函数独立为 `meta.rs` 模块 — 当前文件 ≤ 500 行限制下暂不需拆分。

### 决策 4: 模块化拆分遵循现有模式

**选择**: 每个新 at-rule 在 `src/css/node.rs` 中添加 CssNode 变体，在 `src/parse/at_rules.rs` 添加 parse 方法，在 `src/eval/rule.rs` 添加 eval 逻辑。

**理由**: 符合现有 parser → eval → css 三阶段分离架构。

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| at-rules 修复影响现有 @media/@supports 行为 | 每步跑 202 核心测试回归 |
| 颜色算法改动影响大量现有通过用例 | 先插桩收集基线 → 修复 → 对比 |
| 单文件超 500 行限制 | 按功能拆分（如 at_rules_keyframes.rs） |
| @layer 实现复杂度高 | 优先支持基本块语法，复杂嵌套后续迭代 |

## Migration Plan

无迁移影响（纯功能补全）。每完成一个 Tier 即运行 sass-stat 统计对比：

```bash
RUST_LOG="sass_spec_full=info,sasspile=warn" cargo test --test sass_spec_full -- --nocapture
```

## Open Questions

1. **@keyframes 是否需要支持 `+` 复合选择器关键帧**（如 `from, 50% { }`）— 取决于 sass-spec 测试覆盖范围
2. **CSS custom properties 插值是否需要在编译时求值** — sass 规范中 CSS 变量保留运行时语义，插值仅在值位置求值
3. **@layer 是否需要实现层叠优先级影响** — sasspile 当前不计算层叠，仅保持语法结构
