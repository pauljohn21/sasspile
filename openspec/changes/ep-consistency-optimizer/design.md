## Context

EP 官方 SCSS 编译管线：`dart-sass compile → lightningcss minify → autoprefixer`。sasspile 对标 dart-sass 的 SCSS 语义层（不负责 lightningcss/autoprefixer 后处理）。

当前 EP 121 文件中 84 个完全一致，30 个有差异，其中诊断确认：
- **A 类** (8 files)：BEM `@mixin e()` 链式嵌套丢失中间层——`&` 作用域解析错误
- **B 类** (3 files)：`@keyframes` 空步骤未剥离
- **C 类** (~12 files)：各类 selector/value semantic bug
- **D 类** (7 files)：autoprefixer 注入（非 sasspile 职责）
- **E 类** (3 files)：EP 技术改版（SVG mask → border，无法通过修 sasspile 解决）

**核心原则**：基于 sass-spec 规范修复真正的 sasspile bug，不模仿 dart-sass 实现。

## Goals / Non-Goals

**Goals:**
- 修复 A 类 BEM 嵌套作用域 bug（+8 files）
- 修复 B 类 keyframes 空步骤 bug（+3 files）
- 修复 C 类 semantic bug（+6-10 files）
- 增强 `ep_normalized_test` 管线，区分 sasspile bug 与 EP 管线产物

**Non-Goals:**
- 不实现 autoprefixer 后处理层（EP 管线职责）
- 不实现 lightningcss 的 CSS 化简逻辑
- 不修复 D/E 类（非 sasspile bug）

## Decisions

### Decision 1: BEM 嵌套 `&` 作用域修复方案

**问题根因**：
```scss
@include b(anchor) {                          // .el-anchor
  &.--horizontal {                            // → .el-anchor.el-anchor--horizontal
    @include e(list) {                        // @mixin e() 生成 AtRootDirect
      @include e(item) {}                     // ← & 应解析为 .el-anchor__list，实际解析为外层
    }
  }
}
```

`@mixin e()` 使用 `@at-root { #{$currentSelector} { @content } }`。当 `@content` 内嵌套另一个 `@include e()` 时，内层 mixin 的 `$selector: &` 应指向 `@at-root` body 的 `#{$currentSelector}`（`.el-anchor__list`），而非外层的 `.el-anchor.el-anchor--horizontal`。

**修复选择**：`AtRootDirect` 在 flatten 阶段通过 `nest_rule_in_children` 时，需要携带 parent selector 上下文。当前 `push_atroot_direct` 仅使用 `self.selector`（外层），丢失了 `AtRootDirect` 自身的 selector 作为嵌套 parent。

**备选方案比较**：
- **方案 A（选用）**：在 `eval_body`（`RuleBuilder::build` 期间）维护一个 `Vec<String>` stack 表示 `@at-root` 嵌套路径。`AtRootDirect` 入栈自身 selector，子节点使用栈顶 + AtRootDirect selector 联合作为 parent。
- **方案 B（弃用）**：在 `CssNode::AtRootDirect` 增加 `parent_context: String` 字段。缺点：破坏已有 `AtRootDirect` 构造点（mixin.rs 中构造），改动面广。

**实现位置**：`src/eval/rule.rs` 的 `RuleBuilder` 结构 + `push_atroot_direct` + `build` 方法。

### Decision 2: Keyframes 空步骤剥离

**规范依据**：CSS Animations Level 1 spec 规定空的 keyframe 步骤（无声明的 `0%{}` / `100%{}`）应被移除。这是 dart-sass 序列化时的标准行为。

**修复选择**：在 `Serializer::serialize_*` 之前或之中检测空 keyframe 步骤并移除。

**实现位置**：新增 `css/keyframes.rs` 模块，提供 `strip_empty_keyframe_steps(nodes: &mut Vec<CssNode>)` 函数，在 `Serializer::serialize_expanded` / `serialize_compressed` 入口调用。

### Decision 3: 诊断管线增强

**方案**：在 `tests/ep_normalized_test.rs` 的 `normalize_css` 后增加 `strip_autoprefixer_properties` 步骤，移除 `-webkit-user-select`, `-webkit-appearance`, `-moz-user-select`, `-ms-user-select` 等 autoprefixer 注入属性。使测试暴露真正的语义差异。

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| `&` 作用域修改影响所有嵌套规则 | 全量跑 sass-spec + 核心测试 |
| keyframes 剥离影响自定义动画 | 仅剥离**完全空**的步骤（无任何声明） |
| AtRootDirect 语义变化影响现有 EP 文件 | 跑 `ep_full` 对比修改前后 |
| 修复一个 bug 可能 regression 其他文件 | 每次修改后跑全量 EP diff |

## Migration Plan

无需迁移——本 change 是纯 bug 修复，不改变公共 API。

实施顺序：
1. 先修 A 类（最高 impact，+8 files），验证无回归
2. 再修 B 类（+3 files）
3. 逐个修 C 类（每文件独立 commit，方便回归定位）

## Open Questions

1. **A 类根因需要 SPAN 插桩确认**：`push_atroot_direct` 的 `self.selector` 在嵌套 AtRootDirect 场景下到底是什么值？需要 tracing 证据。
2. **C 类中 button.scss 变量值差异**：`--el-button-disabled-color` vs `--el-button-text-color`——是变量作用域 bug 还是 EP 版本问题？需诊断。
3. **`@at-root` 嵌套链深度 > 2 的正确语义**：`@at-root` 内 `@at-root` 内 `@at-root` 如何处理？需查 sass-spec。
