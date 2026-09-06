## Context

sasspile 经过多轮 AI 迭代后积累结构性违规：

| 违规类型 | 数量 | 严重程度 |
|---------|------|---------|
| unwrap() 滥用 | 10 处（生产代码） | 🔴 直接 panic 风险 |
| 文件 >500 行 | 7 个（共超额 1358 行） | 🟠 AI 上下文劣化 |
| clone() 滥用 | 362 处 | 🟡 所有权设计退化 |
| for + push | ~15 处 | 🟡 函数式违规 |

关键约束：AGENTS.md 规则是静态文档，AI 在长上下文窗口中会自然遗忘。需要**结构性威慑**而非更多规则。

## Goals / Non-Goals

**Goals:**
- 消除所有 unwrap() panic 风险
- 将所有 src/**/*.rs 文件缩减到 ≤500 行
- 建立 CI 自动化检测（clippy deny + 行数检测）
- 不改变任何公开 API 或 sass-spec 行为

**Non-Goals:**
- 不重构 Env/Scope 架构（已在 scope-chain-arch 完成）
- 不清理所有 clone()（仅处理本次拆分中直接涉及的）
- 不新增任何 SCSS 功能

## Decisions

### Decision 1: unwrap() 修复策略

**选择**: 全部替换为 `?` 传播或 `expect("context")`

**理由**:
- `selector_parser.rs` 处于 parse 热路径，expect() 提供可调试信息
- `calc_ast.rs` 和 `calc_simplify.rs` 处于 calc() 表达式求值，? 直接传播更简洁
- `deny` 级 lint 确保未来 CI 拦截

**替代方案**: 用 `unwrap_or_default()`。但会掩盖错误——解析器遇到意外 EOF 应报错而非静默。

### Decision 2: 文件拆分粒度

**选择**: 按"逻辑功能组"拆分，而非简单的行数均分

**具体拆分映射**:

```
display.rs (636 行)
  ┌─ display.rs (核心 ~200 行)        ← Value 基础 fmt 逻辑
  └─ color_display.rs (~400 行)       ← Color/ColorSpace::fmt 分派

color.rs (627 行)
  ┌─ color.rs (核心 ~180 行)          ← invert/grayscale/complement/hsl 入口
  ├─ color_channels.rs (~150 行)      ← red/green/blue/alpha/hue/saturation/lightness/whiteness/blackness
  ├─ color_adjust.rs (~180 行)         ← adjust-color/change-color/scale-color (旧版)
  └─ color_hsl_hwb.rs (~120 行)       ← hsl/hsla/hwb/hwb-a 构造

color_adjust.rs (614 行)
  ┌─ color_adjust.rs (现代空间 ~200 行) ← Oklch/Lab/Lch/Oklab/DisplayP3 adjust/change/scale
  └─ color_adjust_legacy.rs (~150 行)  ← sRGB/HSL 专用路径

css/mod.rs (549 行)
  ├─ css/mod.rs (核心 ~350 行)        ← Serializer 主逻辑
  └─ css/merge.rs (~100 行)           ← merge_at_rules 折叠逻辑
```

**理由**:
- 每个子模块 ≤300 行，为后续 AI 扩展留余量
- 按"功能边界"拆分，避免跨文件理解负担
- 每个子模块的 pub 函数不超过 5 个

### Decision 3: CI 检测机制

**选择**: clippy deny + 自定义 compile_test

**实现**:
```toml
# Cargo.toml [lints.clippy] 添加
unwrap_used = "deny"
clippy::unwrap_used = "deny"
clippy::todo = "deny"
clippy::unimplemented = "deny"
clippy::expect_used = "warn"  # 允许但警告
```

```rust
// tests/file_size_check.rs
#[test]
fn check_file_size_limits() {
    // 遍历 src/**/*.rs，找出 >500 行的文件
    // 失败时打印: "file.rs: 636 行，超出 136 行"
}
```

**理由**:
- clippy deny 是 Rust 生态标准，无额外依赖
- 自定义 compile_test 可以在 CI 中执行，且失败信息直接明显
- 比 pre-commit hook 轻量（不需要全局 git 配置）

## Risks / Trade-offs

- **[sass-spec 回归]** → 每次修改后跑 `cargo test --test sass_spec_full -- --nocapture` 验证
- **[编译性能轻微下降]** → deny 级 lint 增加约 5-10% clippy 检查时间，可接受
- **[文件数增加]** → 从 N 个文件增加到 N+7 个，但每个 ≤300 行
- **[API 兼容性]** → 内部模块拆分，pub 函数路径不变（通过 `pub use` 重新导出）
