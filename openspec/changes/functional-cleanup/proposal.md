## Why

sasspile 经过多轮迭代后，代码库中残留大量过程式模式（81 处 `else-if` 链、76 处 `for + push` 累积、16 处 `&mut` 参数、361 处 `.clone()`），与 AGENTS.md 的函数式规范不一致。同一文件内出现"现代空间函数已用 `apply_kw` 链式、legacy 函数仍是 `if-let + mut`"的风格分裂。需要一次性全量清理，消除 AI 多次迭代产生的风格不一致。

## What Changes

- **消除 81 处 `else-if` 链**：全部改为 `match` 表达式，按枚举值或字符串字面量分派
- **消除 76 处 `for + push` 累积**：改为 `map`/`filter`/`collect`/`try_fold`/`flat_map` 迭代器链
- **消除 16 处 `&mut` 参数**：`Display::fmt` 的 `&mut Formatter` 保留（Rust 标准），其余改为消费 self 返回新值
- **消除 `if-let` 链**：color_adjust.rs 等 legacy 函数中的连续 `if let Some(v)` 改为 `apply_kw`/`apply_pct_kw` 链式
- **减少不必要的 `.clone()`**：move 语义优先，`HashMap` 查找返回的值用 `take` 或 `cloned` 而非 `clone`
- **统一 `scan_ident` 等 lexer 函数**：已有 `scan_escape_ident` 用 match，补齐遗漏

## Capabilities

### New Capabilities
- `functional-style-enforcement`: 函数式风格强制规范——else-if 链→match、for+push→迭代器链、&mut 参数→move 语义、if-let 链→apply_kw 链式

### Modified Capabilities
（无 spec 级行为变更，纯内部重构）

## Impact

- **影响范围**：`src/` 下约 30 个文件
- **高风险文件**：`color_adjust.rs`（legacy 函数重写）、`value/mod.rs`（eval_args 重写）、`scanner.rs`（scan_ident 重写）、`selector.rs`（if-else 链重写）
- **测试验证**：全量核心测试（251 个）+ sass-spec 全量（3366/5624 基线）
- **无 API 变更**：纯内部重构，不改变对外行为
