## Context

`sass-spec/core_functions` 7193 HRX cases 当前通过 38 (0.5%)。Diagnostic (diag_corefn.rs) 识别出四种失败模式：(1) 一行代码的 index-out-of-bounds panic at evaluate_dst/mod.rs:255，(2) `selector.append` 被 dispatch 到 `list.append`（命名空间冲突），(3) ~60 个 builtin 函数未在 `eval_builtin` match 名单中，(4) Parser 对部分 SCSS 语法（list/map 字面量、keyword args、嵌套函数调用）支持不完整。

**Primary gate**: Bootstrap + Element Plus 100% 全量编译不得回归 — 这些 builtin 是 Bootstrap/Element Plus 的真实子集。

**Current architecture**: `substitute_vars` 扫描标识符后调用 `eval_builtin(name, args, ctx)`，其中 match 有 ~50 分支，用短名（`abs`, `quote`, `append`, `get`, `values`, `red`, `green`, `blue` 等）。每次 `module.X(...)` 调用都剥掉 module 前缀留下 `X`。

## Goals / Non-Goals

**Goals:**
- 修掉 index-out-of-bounds panic（+76 例）
- 实现 `module-fn` hyphenated canonical key 路由（+30~50 例）
- 补全 sass-spec core_functions 所需的 color / math / string / list / map / selector / meta builtin 函数（+4000~5000 例）
- 所有新增/修改通过 Gherkin 风格 spec + tests/ 验收

**Non-Goals:**
- 不在本次实现所有 SCSS 语法（仅是实现 builtin 所需最小 Parser 增强）
- 不追求与 dart-sass 像素级精确对齐（仅对齐 sass-spec HRX 期望值）
- 不修改编译器核心 pipeline 结构（evaluate → builtin dispatch 属于局部改造）

## Decisions

### D1: `module-fn` hyphenated canonical key for builtin dispatch

**选择**: `substitute_vars` 将 `math.abs` → `"math-abs"`, `color.alpha` → `"color-alpha"`。在 `eval_builtin` 顶层 match 中用 hyphenated key（`math-abs`, `color-alpha`, `selector-append`, `list-append`, `map-get` 等）。旧短名作为 alias 分支保留。
**理由 (why)**:
- dart-sass 内部 module 系统就是用 hyphenated 的 canonical name（`math-abs`, `list-append` 等）
- 避免跨模块冲突：`selector.append` 与 `list.append` 不再都 dispatch 到 `append`
- 模块隔离语义：更易扩展，不会因在 eval_ctx 里命名与其他辅助函数冲突
**Alternatives considered**:
- *方案 A*（保留短名 + 加 context 参数传 module 前缀）: 复杂，需要每个 builtin 都改为 module-aware
- *方案 B*（单次剥前缀只留 fn）: 这就是现状，已证明不可行（冲突率过高）

### D2: 在 `evaluate_dst/builtins.rs` 实现新增 builtin

**选择**: 在现有 `src/evaluate_dst/builtins.rs`（已是 415 行，靠近 500 行上限）基础上扩展；模块化拆分（`color.rs`, `math.rs`, `string.rs`, `list.rs`, `map.rs`, `selector.rs`, `meta.rs`）一旦文件超过 500 行。
**理由**: 单个文件 500 行上限；拆分后保持读写性。
**Alternatives considered**:
- 单一 `builtins.rs` 包揽: 违反单文件 ≤ 500 行规则

### D3: Parser 增强范围（只做 sass-spec 必需的）

**选择**: 仅增加 sass-spec core_functions 测试所需的语法：
- `(key: value, key2: value2)` map 字面量（作为函数参数值时）
- `(item1, item2, item3)` list 字面量
- keyword 参数 (`$name: value` 形式)
不支持: `@each/for/while` 扩展语法 — 不在 scope。
**理由**: sass-spec 大量测试需要 map/list 字面量作为函数参数。
**Alternatives**: 完整 SCSS parser 重写 — 过度工程，不在 scope

### D4: 浮点精度对齐

**选择**: 遵循 dart-sass 惯例 — 浮点返回保留合理小数位数（如 10 位截断到有意义位），hex color 输出 2 位 alpha。
**理由**: sass-spec 输出格式通常保留 5-10 位小数或 2 位 hex。
**Alternatives**: 精确 bit-for-bit — 不可能跨平台，dart-sass 自身也有变动

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| 重构路由破坏已有 38 个 pass | 单元测试（tests/core_functions_smoke.rs 覆盖率保护）+ 全量 cargo test |
| sass-spec 内部模块加载失败（`@use "core_functions/...")`仍阻碍大量测试 | 优先级顺序：先修 panic + 路由 + 已实现函数；module loading 单独一个后续 change |
| trunk 编译时间增加（~60 新增函数） | 每个 builtin 是 O(1) 简单运算，影响可忽略 |
| Parser 增强引入新 bug | 用 normalize 断言输出，不直接 assert 原始字符串；对失败 case 增 tracing span 后再修 |
