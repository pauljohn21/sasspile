# Proposal

## Why

Bootstrap 5.3.x 编译覆盖率仅 1.25%（67/5342 唯一参考行匹配，5275 行缺失），远低于目标 99%。通过系统性诊断发现 6 个根因：(B6) 缩进 double-counting（8 空格 vs 4 空格，影响 ~50% 行）(B7) 选择器缺失空格 before `{`（影响所有行）(B8) `!important` 未处理（651 唯一受影响行）(B9) 厂家前缀缺失（-webkit-, -moz-）(B10) Sass Maps 格式错误 (B11) 复杂 mixin 展开不全（@media 规则 25 vs 参考 109）。修复这些根因可将 coverage 从 1.25% 提升至 ≥ 99%，使 Bootstrap 编译成为编译器正确性的强约束。

## What Changes

- **修复序列化缩进（B6）**: `format_expanded` 中 depth 起始值从 1 开始、每层 +1，确保 decl 缩进为 2/4 空格而非 8/16 空格
- **修复选择器空格（B7）**: 所有选择器输出时 `{` 前插入 1 个空格（`.a {` 而非 `.a{`）
- **实现 `!important` 标记（B8）**: parser 解析 `!Token` 为 `important: true`，序列化时追加 ` !important`
- **修复厂家前缀生成（B9）**: 确保 auto-prefixer mixin 完整展开，输出 `-webkit-`/`-moz-`/`-ms-` 变体
- **修复 Sass Maps 格式（B10）**: map 值在 CSS decl 中输出为 `null`（跳过）而非 Sass 内部格式
- **修复复杂 mixin 展开（B11）**: `media-breakpoint-up` 等调用链完整展开为 @media 规则（25→109 条）

## Capabilities

### New Capabilities

- _（无新增 capability — 修改现有 capability 的要求）_

### Modified Capabilities

- `bootstrap-dist-alignment`: 新增 4 个 Requirements（序列化缩进正确性、`!important` 标记处理、厂家前缀生成、覆盖率目标 ≥ 99%），扩展现有 spec 覆盖 B6-B11 修复

## Impact

- **代码变更**: `src/serialize/mod.rs`（B6/B7）、`src/parser/mod.rs`（B8）、`src/eval/mod.rs`（B9/B11）、`src/parser/values.rs`（B10）
- **依赖**: 无新增依赖
- **API**: 无公共 API 变更（仅内部实现修复）
- **测试**: 新增针对性单元测试（缩进、空格、!important），Bootstrap dist coverage 从 1.25% 提升至 ≥ 99%
- **波及**: 修复 B6/B7 会影响所有编译输出的格式（正向改善）
