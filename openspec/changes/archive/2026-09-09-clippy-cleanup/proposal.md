# Proposal: Clippy 违规全量清理

## Why（为什么）

多次 AI 迭代后，代码中引入了大量 clippy 违规：
- 1 个阻塞性 `unwrap()` 错误导致编译失败
- 48 个 clippy 警告散布在 11 个文件中
- 418 处 `.clone()` 调用中大量可精简
- 多处 for+push 累积应为迭代器链
- format! 误用、类型转换不安全等问题

这些问题影响代码质量、可读性、维护性，统一清理可恢复 AGENTS.md 规定的函数式 Rust 风格。

## What Changes（变更范围）

### 阻塞错误（1 处，必修）
| 位置 | 违规 | 修复方案 |
|------|------|----------|
| `src/eval/builtin/selector.rs:202` | `unwrap()` | `expect("build_parents: _ branch guarantees non-empty")` |

### Clippy 警告分类（48 处）

| 类别 | 数量 | 修复方案 |
|------|------|----------|
| intra-doc link (引号→反引号) | 8 | `"..."` → `` `...` `` |
| RangeInclusive::contains | 6 | `a <= x && x <= b` → `range.contains(&x)` |
| u128→u64 截断转换 | 6 | `as u64` → `u64::try_from(x)?` |
| format! 变量直接插入 | 5 | `format!("{}", x)` → `format!("{x}")` |
| to_string on Display | 3 | 移除 `.to_string()` |
| useless format! | 2 | `format!("{x}")` → `x.clone()` 或直接返回 |
| expect() on Option | 2 | 转为 `?` 或 proper error |
| u64→u128 as 转换 | 2 | `as u128` → `u128::from(x)` |
| Default 实现缺失 | 1 | 为 `ReactorTrace` 添加 `#[derive(Default)]` |
| cloned→copied | 1 | `.cloned()` → `.copied()` (Copy 类型) |
| unused variable | 1 | `has_suffix` → `_has_suffix` |
| boolean not | 1 | `!x.not()` → `x` |
| let...else | 1 | 简化为 `let...else` 形式 |
| float_cmp | 1 | `==` → `(a - b).abs() < EPSILON` |
| returning let | 1 | 直接返回表达式 |
| missing # Panics | 1 | 添加文档注释 |
| deref 冗余 | 1 | 移除不必要的 `*` |
| binding to `_` | 1 | 移除无效绑定 |
| items after statements | 1 | 调整代码顺序 |

### 文件级修复优先级

| 优先级 | 文件 | 违规数 |
|--------|------|--------|
| P0 | `src/eval/builtin/selector.rs` | 6 |
| P1 | `src/eval/reactor.rs` | 9 |
| P2 | `src/eval/builtin/color_adjust.rs` | 7 |
| P3 | `src/css/selector_format.rs` | 6 |
| P4 | `src/eval/builtin/color_hwb_hsl.rs` | 6 |
| P5 | `src/css/selector_ops.rs` | 5 |
| P6 | `src/parse/ast/display_color.rs` | 4 |
| P7 | `src/eval/builtin/math.rs` | 2 |
| P8 | `src/eval/value/calc.rs` | 1 |
| P9 | `src/eval/builtin/math_trig.rs` | 1 |

## Impact（影响范围）

- **功能影响**: 纯风格修复，不改变任何业务逻辑
- **API 影响**: 无
- **性能影响**: copied() 替代 cloned() 可能有微小提升
- **测试影响**: 无（仅 clippy lint 修复）

## Risk（风险）

- **低风险**: format! 修复、文档注释修复
- **中风险**: unwrap→expect 需确认语义正确
- **中风险**: u128→u64 try_from 需确认截断行为可接受

## Verification（验证策略）

```bash
# 1. clippy 清零
cargo clippy --all-targets 2>&1 | grep -c "warning:\|error:"  # 应输出 0

# 2. 核心测试全通过
cargo test --test compile_test      # 57/57
cargo test --test stage_test        # 10/10
cargo test --test ast_test          # 8/8
cargo test --test common_test       # 5/5
cargo test --test interp_test       # 15/15
cargo test --test bs_spec           # 15/15
cargo test --test ep_full           # 121/121
```
