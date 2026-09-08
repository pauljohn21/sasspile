## Context

sass-spec 当前通过率 62%（7376/12131），core_functions 非 color 子域存在 630 个失败 case：
- `selector/`: 381 fail (56%) — selector-extend 为主(format 错误 + combinator no_op + unification no_op)
- `meta/`: 156 fail (67%) — type-of, calc-args, call, global-variable-exists, equality, 颜色(跳过)
- `math/`: 79 fail (82%) — percentage, ceil/floor/round, clamp, comparable
- `list/`: 20 fail (**91%**) — 目标 90%+ 已达成,无需额外修复
- `modules/`: 14 fail (54%) — **全为颜色变更引入,跳过**

**诊断方法**: 使用 `tests/tmp_cf_diag.rs` 统一诊断测试,`SHOW_FAILS=1` 聚类分析。

**策略调整**: 原预估 +350~600 cases 需修正 — selector 需要 AST 级深度修复(高成本),meta 失败分散(中等成本),modules 跳过。实际可快速收益来自 meta + math 子域。

## Goals / Non-Goals

**Goals:**
- 将 5 个非 color 子域通过率从 45-80% 提升至 80-95%
- 总计修复约 +350~600 sass-spec cases
- 不引入回归（所有已有核心测试 +110 个必须维持通过）

**Non-Goals:**
- 不修改 color 子域（已在其他变更中处理）
- 不新增 SCSS 语法或功能
- 不动 Reactor 管线或 Env 核心架构
- 不追求 100%（有些 spec edge case 成本过高）

## Decisions

### Decision 1: 诊断驱动修复（非盲修）

**决策** — 每个子域先用 `SHOW_FAILS=1` 收集 20-30 个失败案例 → 聚类分析 → 找共性根因 → 一次修好一类

**理由**：selector 有 489 个失败，盲修效率极低。统计聚类后可发现"70% 的 selector-merge 失败是因为忽略 delimiter 参数"这类模式

**替代方案**：逐 case 修复 — 太慢，不采用

### Decision 2: 诊断测试文件复用诊断基础设施

**决策** — 每个阶段用临时诊断文件（`tests/tmp_cf_*.rs`）收集失败模式，修复后删除临时文件

**理由**：项目已有 `cf_diag.rs`、`css_diag.rs`、`cfs_units.rs` 等诊断基础设施，内联 `hrx_support` 模块可直接访问 `run_case`

### Decision 3: 每个子域独立 commit

**决策** — selector、meta、list、math、modules 分别独立 commit，每次提交前跑 `compile_test` + 快速 sass-spec 子目录统计验证无回归

**理由**：隔离风险，若某个子域修复引入回归只影响该子域，不影响其他

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| selector 选择器 AST 深度耦合，修复一个函数可能破坏其他 | 先跑 `selector-ast` 相关测试 + 核心 compile_test 验证 |
| meta 反射函数依赖内部环境状态，修复可能影响 @use/@forward | 跑完整 directives 子目录统计验证 |
| sass-spec 部分 spec case 本身有歧义（如精度舍入） | 记录"有意不修"的 case 列表，归档原因 |
| 修复期间其他分支的 color 变更已提交 | 先 rebase 到最新 main |

## Open Questions

1. selector-merge 失败是否因为 sub-selector 解析格式未处理？需要先看 10 个失败样本
2. meta 模块失败中多少是 `content-exists()` 的 `@content` 上下文问题 vs 简单类型检查？
3. `selector-unify` 和 `selector-merge` 是否共享底层 AST 操作？如果是可一并修复
