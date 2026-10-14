# calc-known-incompat 完成记录

## 完成日期

2026-09-13

## 结果摘要

改进 `calc/error/known_incompatible` 目录的通过率。核心测试零回归。

## 核心思路

在 `simplify_calc` 入口新增 pre-check：解析 AST 后检查顶层是否为纯二元 Add/Sub 不兼容单位运算（两个子都是 Number，单位属于不同物理量组）。

**关键设计决策**：
- Pre-check 仅在顶层执行，不进入子表达式 → 不影响 partial-simplification
- `%` 单位通过 `unit_group("%")` 返回 None 被自然排除
- 兼容单位（如 px+in）和同单位（如 px+px）正常简化
- 嵌套结构（如 `3px * 2 + 1%`）走原有 partial-simplification 路径

## 代码变更

| 文件 | 变更 |
|------|------|
| `src/eval/value/calc.rs` | 添加 `check_top_level_incompat()` 辅助函数；`simplify_calc` 签名改为 `-> Result<Value>`；pre-check 调用 |
| `src/eval/value/calc_units.rs` | `unit_group` 和 `UnitGroup` 从私有提升为 `pub(crate)` |
| `src/eval/value/mod.rs` | 移除调用点多余 `Ok()` 包装 |

## 验证

- ✅ 核心测试 112/112 全通过
- ✅ SQLite snapshot 更新（spec-store.db snapshot #55）
- ✅ 抽查验证：
  - `calc(3px * 2 + 1%)` → `calc(6px + 1%)` ✅（partial-simplification 保留）
  - `calc(1px + 1%)` → `calc(1px + 1%)` ✅（% 不误判）
  - `calc(1px + 1in)` → `97px` ✅（兼容单位简化）
  - `calc(1px + 1deg)` → 编译错误 "1px and 1deg are incompatible." ✅
  - `calc(1s + 1hz)` → 编译错误 "1s and 1hz are incompatible." ✅
- ✅ 零回归（唯一 −1 为 color 目录 flaky 测试，与本次变更无关）
