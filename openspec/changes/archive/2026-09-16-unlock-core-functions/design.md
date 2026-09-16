## Context

`substitute_vars`（`src/evaluate_dst/mod.rs`）是求值阶段的核心变量替换函数，负责将 `$var` 替换为变量值，并将函数调用 `fn(args)` / `module.fn(args)` 替换为 builtin 返回值。

**当前状态**:
- 第 232-239 行的标识符扫描循环只接受 `[A-Za-z0-9_-]`，无法吸收 `.`
- 后缀 `rsplitn(2, '.').next()`（第 244 行）虽然已经准备好从 `module.fn` 中提取 `fn`，但由于 ident 扫描器提前在 `.` 处断开，`ident` 永远不会包含模块前缀
- 行为结果：`math.abs(0)` → `"math."` 泄漏到输出 + `abs(0)` 被 dispatch → `"0"` → 最终输出 `"math.0"`

**约束**:
- `substitute_vars` 同时作用于 selector（规则选择器）和 value（声明值）、prop（属性名）—— 修复不能引入 false positive
- 同一函数中后续的 `rsplitn(2, '.')` 已经在正确的代码路径上 — 只需保证 ident 完整吸收点分路径

## Goals / Non-Goals

**Goals:**
- 让 `substitute_vars` 在 alphanumeric id-扫描中包含 `.`，形成 `module.fn` 路径，使现有 `rsplitn` 逻辑生效
- 对非函数调用的 dotted 标识符保持 fallback 输出 (absorb 后未匹配 `(` → 原样写出)

**Non-Goals:**
- 不处理 `@use "..as m"` 别名解析（该功能属于未来 scope）
- 不修改 parse 层的 AST 结构
- 不新增 builtin 函数实现（已有足够的 dispatch 表）

## Decisions

### Decision 1: 标识符扫描吸收 `.` 的条件

**选择**: 在 `.` 之后紧跟 alphanumeric 或 `_` 时才吸收进 ident

```
if c.is_alphanumeric() || c == '_' || c == '-'
|| (c == '.' && peek_next_char.is_alphanumeric() || peek_next_char == '_')
```

**理由**:
- 避免吞掉 CSS 类选择器前的独立 `.` (如 `.foo:hover` 的首字符 `.` 不在 alphanumeric 循环内，本来就不会进入此分支)
- 纯数字 dot (如 `1.5px`) 同样安全：数值 `.` 前有数字，也被吸收 → 进入 fallback 路径原样输出
- 确保 CSS `property: value.something` 中的 `.something` 也能被正确 fallback

**已拒绝的替代方案**:
- 在 `rsplitn` 处做反向回溯拼接 — 过于 hack，且需要回溯指针位置
- 在 parse 层预处理 — evaluator 已经在做 string-level 二次处理，两层修改复杂度高

### Decision 2: 错误 fallback 行为

**选择**: 点分路径 absorb 后若下一个非空白字符不是 `(`，则把完整 ident 原样输出（包含 `.`）

**理由**: 与当前对 `is_alphanumeric` 标识符的 fallback 处理完全一致 — 额外吸收的 `.` 只是在 path 内部不破坏 fallback 不变性

## Risks / Trade-offs

- **[Risk]** CSS 属性名或值中出现非函数调用的 dotted 字符串会被原样输出（恰好正确行为，与 `url(foo.bar.png)` 这类已有模式一致）
- **[Mitigation]** smoke 测试覆盖 selector 场景确保不回归

- **[Risk]** 部分 sass-spec `core_functions` 测试可能因 builtin 实现细节（如精度、返回格式）仍不通过 — 这是后续迭代内容，本 change 只管打通路径
- **[Mitigation]** 实施后跑 `snapshot` 观察分目录 pass rate 增量
