## Context

sasspile 的 directives 测试通过率 87.8%（15660/17844），2184 个失败 case 集中在 7 类模式。当前架构：

- **Parser**: `src/parse/parser.rs` — 指令解析入口 `parse_for`/`parse_mixin`/`parse_if`/`parse_function` 在读取 keyword 后直接读下一个 token，未跳过注释
- **Forward**: `src/eval/eval_forward.rs` — `@forward` 转发 scope 时未处理 `!default` 变量的合并
- **Import**: `src/eval/eval_import.rs` — `@import` 共享作用域中 `!default` 变量覆盖不正确
- **Function resolve**: function resolver 对大小写不一致的函数名处理不统一
- **At-rule nesting**: `@at-root` 内 `@use` 被错误拒绝
- **CSS serializer**: `@use`/`@import` 与 CSS 混合时输出顺序不保留

## Goals / Non-Goals

**Goals:**
- 7 类失败模式全部修复，directives 通过率达到 100%
- 核心测试维持 202/202 全通过
- 不引入新的回归

**Non-Goals:**
- 不重构整体架构
- 不新增公开 API
- 不优化性能（除非修复需要）

## Decisions

### Decision 1: 指令注释跳过 — 统一在 Parser 层解决

**选择**: 在 `parse_for`/`parse_mixin`/`parse_if`/`parse_function` 入口调用 `skip_comments()` 而非修改 lexer

**理由**: 
- Parser 层已有 `skip_comments()` 辅助函数
- 各指令解析入口模式一致：`keyword → skip_comments → 读参数`
- 不改变 lexer 行为，影响范围最小

**备选**: 在 lexer 层面让 comment token 自动跳过 — 被否决，因为 comment 在某些上下文需要保留（如 CSS 选择器内的注释）

### Decision 2: @function 名称大小写 — 统一在 function resolve 层折叠

**选择**: function resolver 在查找用户定义函数时，将名称转为小写后比较

**理由**:
- SCSS 规范定义函数名大小写不敏感
- 只需在 `call_function` 查找时做一次 `to_lowercase()`
- CSS 原生函数（`element()`/`url()`/`expression()`）在 builtin dispatch 层已处理

### Decision 3: @forward !default 传播 — 在 evaluate_forward 合并 scope

**选择**: `evaluate_forward` 在合并 forwarded scope 时，检查目标 scope 中同名变量是否有 `!default` 标记，若有且源 scope 有定义则覆盖

**理由**:
- `!default` 语义是"默认值可被覆盖"
- forward 链上的变量传播需要保持此语义
- 在 scope 合并时处理最自然

### Decision 4: @at-root 内 @use — 放宽 at-rule 嵌套检查

**选择**: 修改 at-rule 嵌套验证逻辑，允许 `@at-root` 内部使用 `@use`

**理由**:
- sass-spec 测试明确定义此行为为合法
- 只需在 `validate_at_rule_nesting` 中添加例外

### Decision 5: @use CSS 排序 — 在序列化阶段保持原始顺序

**选择**: 在 CSS serializer 中，保持 `@use`/`@import` 规则和 CSS 规则在原始 AST 中的相对顺序

**理由**:
- sass-spec 测试要求注释和 CSS 声明顺序保留
- 在序列化阶段处理最合适，因为 AST 已包含顺序信息

### Decision 6: @use + @extend 跨模块可见性 — 扩展 extend 查找范围

**选择**: `@extend` 查找目标选择器时，除了当前 scope 还搜索通过 `@use` 导入的模块选择器

**理由**:
- sass-spec 定义 `@use` 导入的选择器对 `@extend` 可见
- private selector（`%-name`）需排除

### Decision 7: @import 变量作用域 — 修复 !default 覆盖逻辑

**选择**: `@import` 处理时，将 importing context 的变量作用域与 imported file 合并，`!default` 变量在已有定义时不覆盖

**理由**:
- SCSS 规范中 `@import` 是共享作用域模型
- `!default` 标记的变量只在未定义时生效

## Risks / Trade-offs

| Risk | Mitigation |
|------|-----------|
| 注释跳过可能破坏现有注释保留行为 | 只在指令入口跳过，不影响 CSS 规则内的注释 |
| 函数大小写折叠可能影响 CSS 原生函数 | builtin dispatch 优先于用户函数查找 |
| forward 变量传播可能引入循环依赖 | 保持现有循环检测逻辑不变 |
| @use + @extend 可能影响现有 extend 性能 | 只在 extend 查找时增加模块搜索，不改变 extend 合并算法 |
