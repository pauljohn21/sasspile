## Context

当前 sasspile 项目在 ~60 个 Rust 文件（src/ + tests/）中存在风格不一致问题。通过系统性代码审查发现 6 个核心不一致维度：

| 维度 | 现状 | 目标 |
|------|------|------|
| Module Header | 从无到完整文档 4 级并存 | 分层模板统一 |
| Section Divider | `═══`/`───`/`——`/无 + 长度不齐 | 单字符+统一长度 |
| 测试函数声明 | `#[test] fn x() {` 紧凑同行 vs 展开 | 全部展开 |
| 错误消息 | `"unexpected failure"` / 中文 / 空混用 | 统一默认消息 |
| 命名前缀 | `test_` / `diag_` / `stats_` 三套 | 保留但明确定义 |
| 注释对齐 | 部分文件有行内注释对齐 | 不强制 |

**利益相关者**：项目维护者、新贡献者、AI 辅助代码生成。

**约束**：
- 不改变任何逻辑/测试断言/编译行为
- 保留中文注释/文档
- 保持 AGENTS.md 既有规则不动
- 单文件 ≤ 500 行

## Goals / Non-Goals

**Goals:**
- 建立可执行的单一风格规范 `STYLE_GUIDE.md`
- 批量统一全部 ~60 个文件
- 消除 AI 生成代码风格飘忽问题
- 每批修改后跑 `cargo test` 确认零回归

**Non-Goals:**
- 不引入新 linter 或格式化工具（如 `rustfmt` 自定义配置）
- 不修改 AGENTS.md 中的既有规则
- 不统一代码逻辑/算法实现（纯风格变更）
- 不扩展到非 Rust 文件（如 `.scss` 测试用例）

## Decisions

### Decision 1: Section Divider 字符选择

**选择**: `// ─── Name ───────────────────────────` (U+2500 BOX DRAWINGS LIGHT HORIZONTAL)

**候选**:
- `───` (U+2500) — ✅ 选用，与既有 `diag_helper.rs`、`compile_color_test.rs` 一致
- `═══` (U+2550) — ❌ 仅限 reactor_test.rs，字符宽度不一致
- `——` (U+2014 em dash) — ❌ 仅限 interp_test.rs，双字符混淆

**理由**: 单字符线、78 列右对齐、与最多文件现有风格一致（改造成本最低）。

### Decision 2: Module Header 分层模板

**选择**: 两类模板（src 模块 vs tests 文件）

**src/ 模块**:
```rust
//! —— 模块名 ——
//!
//! 概要：一句话说明模块职责。
//!
//! ## 核心概念
//! - 关键类型/算法
//! - 与上下游模块关系
//!
//! ## 示例（可选）
//! ```rust
//! use ...;
//! ```
```

**tests/ 文件**:
```rust
//! —— 测试范围说明 ——
//!
//! 概要：一句话说明测试覆盖内容。
//!
//! ## 覆盖场景
//! - 场景 A
//! - 场景 B
//!
//! ## sass-spec 参照（如适用）
//! - `core_functions/list/join.hrx` — 命名参数校验
```

**理由**: src 需要说明架构角色，tests 需要说明覆盖范围和 sass-spec 参照。interp_test.rs 已有详细 sass-spec 参照说明，现状合理利用。

### Decision 3: 测试函数声明风格

**选择**: 全部展开 `#[test]\nfn name() {`

**候选**:
- 全部展开 — ✅ 选用，消除 `diagnostic_runner.rs` 的特例
- 允许单行紧凑 — ❌ 引入例外增加决策成本

**理由**: 统一展开消除 reviewer 犹豫"这个能不能写一行"。`diagnostic_runner.rs` 有 20+ 个单行测试，展开后仅增加 ~20 行，可接受。

### Decision 4: 错误消息默认值

**选择**: 保留 `expect("unexpected failure in test")` 作为标准消息，中文消息（如 `"编译应成功"`）在纯中文测试场景允许并存。

**理由**: AGENTS.md 已有 `expect()` 规则；强制统一语言会损害部分测试消息的可读性。

### Decision 5: STYLE_GUIDE.md 存放位置

**选择**: 项目根目录 `STYLE_GUIDE.md`

**候选**:
- 根目录 `STYLE_GUIDE.md` — ✅ 最高可见性
- `docs/STYLE_GUIDE.md` — ❌ 与架构文档混合
- 嵌入 `AGENTS.md` — ❌ 文件已 783 行，接近上限

**理由**: 独立文件便于引用和 review，未来可作为 CI 风格检查的基础。

## Risks / Trade-offs

| 风险 | 缓解措施 |
|------|----------|
| 批量修改引入 typo 破坏编译 | 每批修改后立即 `cargo test` 确认 |
| header 模板不适用于某些特殊文件 | 模板标记"可选"部分，允许灵活调整 |
| 中文 header 与英文代码的视觉冲突 | 注释语言不作强制，header 保留中文现有风格 |
| diagnostic_runner 展开后行数增加 | 确认仍 ≤ 500 行上限 |
