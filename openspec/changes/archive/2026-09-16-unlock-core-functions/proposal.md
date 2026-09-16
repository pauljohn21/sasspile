## Why

sass-spec 当前通过率仅 0.4%（37/10274），其中最大的失败集中区是 `core_functions` 目录 —— 7793 条测试全部失败。经探查，根因是 `substitute_vars` 的标识符扫描器不吸收 `.` 字符，导致 `math.abs(-5px)` 这类点分模块函数调用被错误拆解为 `math.` 泄漏到 CSS 输出中。这是一个字符级 bug — 修复一处即可释放 ~7000+ 测试。

## What Changes

- **修复** `src/evaluate_dst/mod.rs` `substitute_vars` 的标识符扫描逻辑，使 `.` 作为模块分隔符被吸收到标识符中
- **新增** 核心函数冒烟测试 `tests/core_functions_smoke.rs` 覆盖 math/string/list/map/color/meta 各子模块的基础场景
- **不修改** 对外 API 或数据结构 — 仅内部解析行为变更

## Capabilities

### New Capabilities

- `module-dot-evaluate`: 求值阶段支持 `module.function(args)` 点分调用语法，与现有裸函数调用 (`abs(...)`) 行为一致

### Modified Capabilities

- 无新增 delta spec — 本缺陷属于"已有需求未被正确实现"而非需求变改

## Impact

- **受影响的代码**: `src/evaluate_dst/mod.rs` 第 232-239 行的 char 扫描循环
- **行为影响**: 所有形如 `module.fn(args)` 的 value 字符串都会正确 dispatch 到 builtin 函数表；不再泄漏 module 前缀
- **兼容性**: 非函数调用的点分标识符（如 CSS 类选择器 `a.bar`）会因为缺少 `(` 触发 fallback 路径原样输出，不受影响
- **风险**: 极低 — Absorb 后的标识符若后续字符不是 `(` 则进入 else 分支原样输出，行为等价于当前
