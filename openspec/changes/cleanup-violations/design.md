## Context

sasspile 经多次 AI 辅助迭代后积累三类技术债务：(1) 4 个 `println!` 阻塞 clippy 编译；(2) 11 个文件超 500 行上限；(3) 21 个 lib-clippy warning。本设计描述清理策略与顺序。

## Decision: 分层推进策略

### Layer 0 — 编译阻塞修复 (必做)

`tests/specstore/mod.rs` 中 4 处 `println!` 直接替换为 `tracing::info!`。spec-store 已通过 `--nocapture` 运行，输出仍可见。

### Layer 1 — clippy 自动修复 (必做)

```bash
cargo clippy --fix --lib -p sasspile -- --allow-no-vcs --warn clippy::all
```

自动修复项：
- `unused_imports` 删除
- `unused_variables` 加 `_` 前缀或删除
- `format!` 内联变量
- `manual_range_contains` 等

非自动但手动的：
- `let...else` 重写（clippy 会建议但可能不全部自动修复）
- unused functions / fields 删除或加 `#[allow(dead_code)]` + TODO 注释

### Layer 2 — 文件拆分 (按优先级顺序)

目标：减少超 500 行的文件到 ≤500。

#### selector_extend.rs (873 行)

拆分方案：
```
src/css/
  selector_extend.rs  → 入口 + ReExports (thin)
  selector/           → 新模块目录
    extend.rs          (~300 行 — extend 算法)
    unify.rs           (~200 行 — unify 算法)
    format.rs          (~150 行 — 格式化输出)
    simplify.rs        (~已存在, 迁移合并)
    mod.rs             — 公开 API re-export
```

#### reactor.rs (659 行)

拆分方案：
```
src/eval/reactor.rs           → Export + Pipeline 链式主体 (≤500)
src/eval/reactor_states.rs    → 状态类型定义 (StateRaw, StateLexed, ... ≤159)
```

#### color_adjust.rs (636 行)

拆分方案：
```
src/eval/builtin/color_adjust.rs    → adjust 函数 (~200 行)
src/eval/builtin/color_change.rs    → change 函数 (~200 行)  
src/eval/builtin/color_scale.rs     → scale 函数 (~200 行)
```

#### selector.rs builtin (603 行)

拆分方案：
```
src/eval/builtin/selector.rs         → 入口 + dispatch (≤200 行)
src/eval/builtin/selector_ops.rs     — selector-append/is-super/nest 等 (~400 行)
```

#### color_hwb_hsl.rs (583 行)

拆分方案：
```
src/eval/builtin/color_hwb_hsl.rs    → HSL 函数族 (~280 行)
src/eval/builtin/color_hwb.rs        → HWB 函数族 (~300 行)
```

#### display_color.rs (567 行)

拆分方案：
```
src/parse/ast/display_color.rs       → 主入口 + ColorOutput 辅助 (≤350 行)
src/parse/ast/display_color_spaces.rs → 各色彩空间序列化 (~220 行)
```

### Layer 3 — 可选项（不纳入本次）

- `module.rs` (540)、`serialize.rs` (539)、`env_impl.rs` (517)、`calc.rs` (505)、`list.rs` (505) — 仅超标 1-8%，拆分收益低、回归风险高，留待后续
- clone() 密度优化 — 独立 OpenSpec change，需要 OTel 热点数据支撑

## Verification

每完成一层即运行验证：
```bash
# Layer 0
cargo clippy --test spec_store

# Layer 1-2
cargo clippy --lib
cargo test --test compile_test    # 43 cases
cargo test --test stage_test      # 10 cases
cargo test --test ast_test        # 8 cases
cargo test --test common_test     # 5 cases
cargo test --test bs_spec         # 15 cases
cargo test --test ep_full          # 121 cases (~38s)

# 最终
cargo clippy --all-targets
```
