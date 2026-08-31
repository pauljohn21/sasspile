## Context

sasspile 当前 sass-spec 通过率 54%（2918/5362），2444 次失败中 1689 次为编译错误（ERROR 日志可见），755 次为输出不匹配（编译成功但输出与期望不同）。

失败集中在 5 大根因：
1. **参数验证过严**（286 次）：`merge_args` / `merge_math_args` 把命名参数当作多余位置参数计数，导致 `str-length($string: "hello")` 报 "Only 1 argument allowed, but 2 were passed"
2. **plain CSS 限制检测**（120 次）：`css/plain` 目录测试期望编译错误但 sasspile 编译成功（`expected_error_but_ok`）
3. **selector 函数参数**（76 次）：`selector-parse`/`selector-extend`/`selector-replace` 的参数展开不正确
4. **中文错误消息**（22 次）：`"1 不是 map"` 等中文消息不匹配 sass-spec 期望
5. **运算符对特殊值不支持**（50 次）：`calc()` / `get-mixin()` 等特殊值参与 `+`/`-` 运算时报错

## Goals / Non-Goals

**Goals:**
- 通过率从 54% 提升到 ~60%（+300~400 用例）
- 修复参数验证逻辑，消除命名参数误报
- 统一错误消息为英文
- 增强 plain CSS 错误检测覆盖
- 修复运算符对特殊值的处理

**Non-Goals:**
- 颜色系统修复（已跳过，需 `--ignored` 手动触发）
- 输出序列化深度对齐（Phase 4，ROI 较低，逐个排查困难）
- 架构变更（所有修复都是逻辑层修改）

## Decisions

### Decision 1: 参数合并逻辑修复 — `merge_args` 统一入口

**选择**：在 `merge_args` 函数中过滤掉已在 `kw_args` 中出现的命名参数，不计入 `pos_args` 的长度检查。

**理由**：sass-spec 大量使用 `str-length($string: "hello")` 形式调用单参数函数。当前 `merge_args` 先将 `pos_args` 和 `kw_args` 合并为统一列表，再检查 `args.len()`，导致命名参数被当作多余位置参数。

**替代方案**：在各函数内单独处理 — 否，每个函数都要改，维护成本高。统一在 `merge_args` 入口修复更安全。

### Decision 2: 中文错误消息改英文 — grep + 替换

**选择**：用 `grep` 搜索 `src/` 中所有中文字符串（`不是`、`要求`、`参数`等），改为 sass-spec 期望的英文格式。

**理由**：sass-spec 的期望输出和错误消息都是英文。中文消息必定不匹配。

### Decision 3: plain CSS 错误检测 — 增强 `check_plain_css_node`

**选择**：在 `check_plain_css_node` 函数中增加对 sass 特有 at-rules、Interpolation、Operators 的检测，使 plain CSS 模式下这些构造报错。

**理由**：sass-spec 的 `css/plain` 目录测试了 `.css` 文件中不应包含 sass 特有语法的场景。当前 sasspile 对这些场景编译成功（不应成功），导致 `expected_error_but_ok` 失败。

### Decision 4: 运算符特殊值处理 — CSS 透传

**选择**：对 `+`/`-` 运算符遇到 `calc()`/`get-mixin()` 等特殊值时，尝试 CSS 透传（生成 `calc(a + b)` 形式）而非直接报错。

**理由**：sass-spec 允许 `calc(1px + 2px)` 等 CSS 原生 calc 语法，sasspile 应支持而非拒绝。

### Decision 5: infinity 参数接受

**选择**：在 `validate_single_number` 中接受 `Value::Number(f64::INFINITY, ..)` 作为合法参数，不报 "is not a number" 错误。

**理由**：sass-spec 测试 `abs(infinity)` / `sqrt(infinity)` 等边界场景，当前 sasspile 报 "$number: infinity is not a number" 是错误的。

## Risks / Trade-offs

- **[Risk] 参数验证修复可能放宽过多** → 修复后运行 `compile_test` 和 `ep_full` 确认无回归
- **[Risk] plain CSS 错误检测增强可能误报** → 仅对 sass 特有语法报错，CSS 原生语法保持透传
- **[Risk] 运算符特殊值透传可能影响数值精度** → 仅对非 Number 类型透传，Number 运算保持原逻辑
- **[Trade-off] Phase 4 输出序列化对齐 ROI 低** → 暂不实施，后续单独提案处理
