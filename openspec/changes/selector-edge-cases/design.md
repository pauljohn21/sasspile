## Context

### 当前状态
- `selector` 模块基础功能（append/nest/unify/extend/replace/parse/is-super/simple-selectors）已完成
- `selector_format.rs` 提供 selector 格式与 Value 之间转换
- `value_to_selector_format` 当前仅支持纯 string list 或纯 list of lists，不支持混合类型

### 问题
1. **混合类型输入失败**：`selector.nest((c, d e), "f")` 中 `(c, d e)` 是 string + list 混合，当前返回错误
2. **extend unification 不正确**：`selector.extend(".c.x .d", ".c", ".e")` 期望输出 `.c.x .d, .x.e .d`，实际只返回原始 selector
3. **错误检测缺失**：`selector.append("[c", "d")` 中的 `[c` 是无效选择器，应报解析错误而非静默忽略

### 约束
- 纯 Rust 实现，禁止参照 dart-sass（GC 依赖）
- 函数式风格强制：迭代器链优于 for+push，match 优于 if-else
- 单文件 ≤ 500 行

## Goals / Non-Goals

**Goals:**
- 修复 `value_list_to_format` 支持混合类型输入（string 和 list 混合）
- 实现 `selector-extend` 完整 unification 算法，正确替换 parent reference
- 增强 `selector_parser` 错误检测，对无效选择器（未闭合属性选择器、无效字符）抛出明确错误
- 提升 sass-spec selector 相关测试通过率约 100 cases

**Non-Goals:**
- 不改变已有的 selector-append/nest/unify 等正常功能
- 不引入新的内建函数
- 不改变 selector 的基础数据结构和 AST 表示

## Decisions

### 1. 混合类型输入处理

**决策**：在 `value_list_to_format` 中增加混合类型分支，string 视为单 compound complex，list 视为多 compound complex

**理由**：
- Sass selector format 规范允许 `(c, d e)` 表示多个 complex selectors，其中每个 complex 可以是单个 compound（string）或多个 compounds（list）
- 相比严格模式拒绝混合类型，支持混合类型能覆盖更多合法用例

**备选方案**：仅在 selector 函数入口预处理混合类型 → 拒绝，因为可能破坏当前严格类型检查逻辑

### 2. selector-extend unification 算法

**决策**：实现基于 compound 遍历的 parent 匹配算法

**算法思路**：
1. 遍历 selector 的每个 complex 的每个 compound
2. 对每个 compound，检查是否能匹配 extendee
3. 匹配时，用 extender 中的对应 compound 替换，并保持前后缀关系
4. 如果 extender 包含多个 compounds，需生成笛卡尔积展开

**理由**：
- Sass 的 extend 算法本质上是在复杂选择器中查找并替换匹配的子序列
- 需要考虑 parent（前面 compounds）和 grandparent（更前面）的上下文

**备选方案**：直接字符串替换 → 拒绝，因为无法正确处理嵌套关系和组合符

### 3. 解析器错误检测增强

**决策**：在 `parse_compound` 中增加属性选择器闭合验证

**理由**：
- `[c` 缺少闭合 `]` 是明确的语法错误
- 当前对无效字符的验证基于 `is_valid_selector_token`，但它无法检测结构不完整的 token
- 在解析阶段检测能提供更好的错误消息和位置信息

## Risks / Trade-offs

| 风险 | 缓解措施 |
|------|----------|
| 混合类型支持可能破坏当前严格的类型检查逻辑 | 增加穷尽测试用例，确保纯 string 和纯 list 场景仍正确工作 |
| extend unification 算法复杂，可能引入边界 bug | 逐步实现：先支持简单 parent 匹配，再支持 grandparent |
| 解析器错误检测改变可能影响现有合法输入 | 仅验证 attribute selector 闭合性，不影响其他语法规则 |
