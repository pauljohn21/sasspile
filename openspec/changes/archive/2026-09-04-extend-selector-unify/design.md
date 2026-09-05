## Context

sasspile 的 `@extend` 实现基于选择器 AST（`Selector` / `ComplexSelector` / `CompoundSelector` / `SimpleSelector`）代数运算。
当前通过率仅 29%（5/17），根因通过 tracing 证据链定位如下：

### 证据链

**Step 1 — SPAN 插桩**：在 `apply_extends` 入口加 `debug!(selector, n_extends, extends_debug)`。
**Step 2 — TRACE 采集**：`RUST_LOG="sasspile::extend=debug" cargo run -- /tmp/test.scss`
**Step 3 — 根因定位**：

```
span: apply_extends[selector=a, n_extends=1]
  extends_debug=[("d", "a", false, Some("/tmp/test.scss"))]
  → module scope 检查：module_selectors 为空，module_path 不在缓存中
  → fallback: module_selectors.values().any(|s| s.contains("a")) → false（空迭代器）
  → !false = true → return sel_ast（跳过 extend）
Root cause: module scope 检查在非模块化编译时误判，所有 extend 被跳过
```

### 当前实现架构

```
eval_extend_node(selector, optional, env)
  → env.add_extend(extender, selector, optional, module=base_path)
    → extends: Rc<Vec<(extender, target, optional, module)>>

apply_extends(css, extends, module_selectors)
  → 对每个 CssNode::Rule:
    sel_ast = extends.iter().fold(parse_selector(selector), |sel, (ext, target, _, module)| {
      // bogus 检测 → skip
      // module scope 检查 → skip if not in scope ← BUG
      extend_selector(&sel, &parse_selector(target), &parse_selector(ext))
    })
```

## Goals / Non-Goals

**Goals:**
- 修复 module scope 检查，使非模块化编译的 extend 正常工作
- 支持逗号分隔多目标 `@extend .a, .b`
- 新增 extend 参数校验（复杂/复合/空选择器报错）
- 提升 `directives/extend` sass-spec 通过率到 60%+

**Non-Goals:**
- `@extend` 跨文件继承的完整实现（`@use` 模块选择器传播）
- `:is()`/`:where()` 伪类内部的 extend 传播（后续优化）
- bogus 组合器的 deprecation warning（仅静默跳过）
- 注释穿透（`@extend a /**/` — 后续优化）

## Decisions

### D1: Module scope 检查降级策略

**决策**：当 `module_selectors` 为空或 `module_path` 不在缓存中时，不跳过 extend。

**理由**：非模块化编译（直接编译 SCSS 文件）时，所有 extend 都应该全局生效。module scope 检查只对 `@use` 模块化加载的文件有意义。

**替代方案**：将 `module` 字段改为 `Option<PathBuf>`，非模块化编译时设为 `None`——但这需要修改 `eval_extend_node` 中的 `env.get_base_path()` 逻辑，影响面更大。

### D2: 逗号分隔多目标拆分

**决策**：在 `eval_extend_node` 中按 `,` 拆分 target，为每个目标生成独立的 extend 条目。

**理由**：`@extend .a, .b` 语义等价于 `@extend .a; @extend .b;`，在求值阶段拆分最简单。

### D3: 复杂/复合选择器校验

**决策**：在 `eval_extend_node` 中校验 target：
- 包含空格（多 compound）→ 报错 "complex selectors may not be extended"
- 包含伪类/伪元素且非纯伪类 → 报错 "compound selectors may no longer be extended"
- 空字符串 → 报错 "expected selector"

**理由**：这些校验在求值阶段执行，不改变解析器。

### D4: `extend_complex` 中间位置匹配

**决策**：当前 `extend_complex` 只在后缀位置匹配 extendee。改为在所有可能位置匹配。

**理由**：`a.b.c` extend `.b` 应该匹配中间的 `.b`，而不仅仅是后缀。

### D5: fold 去重重构

**决策**：将 `apply_extends` 和 `extend_selector` 中的 `fold(Vec::new(), |mut acc, c| { if !acc.contains(&c) { acc.push(c) } acc })` 替换为函数式写法。

**理由**：遵循项目函数式 Rust 规范——禁止可变 `Vec` + `push` 累积。

## Risks / Trade-offs

- **[module scope 降级可能跳过跨模块 extend 校验]** → 后续在 `@use` 模块加载时正确填充 `module_selectors`
- **[逗号拆分可能破坏带逗号的选择器如 `a, b`]** → 拆分前先检查是否是合法选择器列表，如果整体匹配则不拆分
- **[中间位置匹配可能改变现有通过测试的输出]** → 必须验证不引入回归
