## Context

当前 `selector.extend` 算法在 `src/css/selector_extend.rs` 中实现，主要处理简单选择器和类型选择器的扩展。伪类选择器（`:not()`, `:is()`, `:where()`, `:matches()` 等）有独特的 Sass 语义：

1. **:not() 特殊语义**: 扩展时追加 `:not()` 到 compound，而非替换内部
   - `extend(":not(.c)", ".c", ".d")` → `:not(.c):not(.d)`
   - 因为 `:not(X, Y)` ≡ `:not(X):not(Y)`

2. **:is/:where/:matches subselector**: 如果 extendee 是这些伪类的 subselector，扩展是 no-op
   - `extend(".c:is(d)", ":is(d)", "d.e")` → `.c:is(d)` (no-op)

3. **:not() 列表处理**: 如果 `:not()` 已包含列表，直接添加新选择器到列表
   - `extend(":not(.c, .d)", ".c", ".e")` → `:not(.c, .e, .d)`

## Goals / Non-Goals

**Goals:**
- 实现 `:not()` extend 算法
- 实现 `:is()`, `:where()`, `:matches()` 的 no-op 检测
- 支持 `:nth-child()` 等参数化伪类的匹配
- 通过 65+ sass-spec cases

**Non-Goals:**
- 不改变现有非伪类 extend 行为
- 不实现 CSS 新规范中的伪类（如 `:has()`）
- 不修改选择器 AST 结构

## Decisions

### Decision 1: :not() extend 实现位置
**选择**: 在 `extend_selector_with_mode` 中添加独立的 `:not()` 处理路径
**理由**: 保持现有逻辑清晰，`:not()` 逻辑复杂且独立
**备选**: 在 `extend_complex` 内部添加分支——但会使已经很复杂的函数更难维护

### Decision 2: 伪类检测方式
**选择**: 在 `extend_complex` 开始时检测 selector compound 中的 `:not()` 伪类
**理由**: 早期检测可以走专用路径，避免进入通用 extend 逻辑
**备选**: 在匹配过程中检测——但会导致逻辑分散

### Decision 3: :not() 参数匹配
**选择**: 解析 `:not()` 的 arg 字符串为伪选择器，检查 extendee 是否匹配
**理由**: 精确匹配，支持复杂嵌套
**备选**: 字符串匹配——不够健壮

## Risks / Trade-offs

- **[复杂度高]** → 添加详细 tracing span 便于调试
- **[性能影响]** → 仅在检测到 `:not()` 时进入特殊路径，无 `:not()` 时无开销
- **[回归风险]** → 保持现有逻辑不变，仅添加新分支