> ⛔ **禁止参照 dart-sass**：dart-sass 依赖 GC（垃圾回收），其嵌套结构依赖 GC 保。sasspile 是纯 Rust 项目，无 GC，所有权语义完全不同。任何实现必须基于 Rust 所有权模型和 sass-spec 规范，不得参照 dart-sass 的实现。

## Context

sasspile 当前 sass-spec 通过率 3068/5362 = 57%。经诊断分析，2294 个失败分布在六大类别。所有修复均在当前 Rust move 语义架构下完成，不需要 GC、共享引用或架构变更。

当前架构：
```
Source → Lexer → Parser → Evaluator → Serializer → CSS
                                     ↑              ↑
                               RuleBuilder      serialize_expanded
                               (展平选择器)      (空行/格式)
```

关键现状：
- `RuleBuilder::push` 展平选择器是**正确的** SCSS 行为（sass-spec 期望展平输出）
- `serialize_expanded` 顶层规则间一律加空行，但 sass-spec 期望同源规则不加空行
- `merge_math_args` / `validate_single_number` 不区分命名参数和位置参数
- `call_module_function` 的 `module_builtin_name` 映射可能遗漏部分函数

## Goals / Non-Goals

**Goals:**
- 将 sass-spec 通过率从 57% (3068/5362) 提升到 70%+ (3750+/5362)
- 六个 Phase 独立可实施，每个 Phase 不依赖其他 Phase 的结果
- 所有修复不改变已通过测试的行为（无 BREAKING）
- 修复后 `compile_test 43 + stage_test 10 + ep_full 121 = 202/202` 保持通过

**Non-Goals:**
- 不做嵌套输出重构（sass-spec 期望展平，RuleBuilder 行为正确）
- 不做颜色函数修复（颜色测试已跳过，需单独处理）
- 不做 GC / 共享引用架构变更
- 不做其他语言实现的移植

## Decisions

### D1: 空行处理 — 序列化阶段标记同源规则

**决策**：在 `flatten_nodes` 输出中给展平产生的规则添加 `flatten_group` 标记，`serialize_expanded` 根据标记决定是否加空行。

**理由**：空行问题本质是序列化格式问题，不是求值问题。在 `flatten_nodes` 阶段已经有足够信息判断两个规则是否同源。

**方案**：
```rust
// CssNode::Rule 新增可选字段 flatten_group: Option<usize>
// flatten_nodes 时，同一父选择器展平的规则共享相同的 flatten_group
// serialize_expanded 中：相邻规则如果 flatten_group 相同，不加空行
```

**替代方案**：在 `serialize_expanded` 中比较选择器前缀（如 `a` 和 `a .d` 是否同源）— 但这在选择器复杂时不可靠。

### D2: 参数验证 — merge_args 返回合并后参数，不再传 kw_args 给 validate

**决策**：`merge_math_args` / `merge_args` 已经正确合并命名参数到位置参数。问题在 `validate_single_number` 接收的是**合并前**的 `args`（包含命名参数重复计数）。

**方案**：所有内建函数的参数验证改为检查**合并后**的参数列表，不再分别检查 `pos_args.len()` 和 `kw_args.len()`。

### D3: 内建函数补全 — 补全 module_builtin_name 映射

**决策**：检查 `module_builtin_name` 函数是否正确映射了所有 `STRING_NAMES` / `LIST_NAMES` 等条目。当前 `string.str-insert` 已在注册表中，但 `call_module_function` 的解析路径可能有问题。

**方案**：在 `call_module_function` 中，当 `sass:xxx` 内建模块的函数未被命名空间找到时，确保正确映射到 `call_builtin`。

### D4: 输出格式 — 逐目录对齐

**决策**：针对 `content_diff` / `extra_output` / `missing_output` 三类失败，逐个目录排查具体差异。

**方案**：使用 `cf_diag` 和 `css_diag` 的逐行 diff 输出定位具体问题，逐个修复。

### D5: plain CSS 错误 — 增强 check_plain_css_node

**决策**：在 `plain_css.rs` 的 `check_plain_css_node` 中增加对更多节点的错误检测。

**方案**：针对 `expected_error_but_ok` 场景，增加缺失的错误检测路径。

### D6: 模块系统 — module loop 检测修正

**决策**：当前 `eval_use` 的 module loop 检测逻辑：`already_loaded && !cache.contains` 判断 loop。但 `loaded_modules` 和 `module_cache` 的更新时机可能不同步。

**方案**：确保 `loaded_modules` 在 `load_module` 开始时插入（而非结束时），使循环引用能在递归调用中被检测到。

## Risks / Trade-offs

- [Phase 1 空行修复影响所有输出] → 逐目录验证 sass-spec，先跑 compile_test + ep_full 确保无回归
- [Phase 2 参数验证改动可能改变错误消息] → 仅修改验证逻辑，保持错误消息格式不变
- [Phase 3 内建函数补全可能引入新函数路径] → 确保 `is_known_builtin` 同步更新
- [Phase 4 输出格式修复是逐个 case 处理] → 控制每批修复规模，避免上下文丢失
- [Phase 5 plain CSS 错误检测增强可能误报] → 只在 sass-spec 期望错误的场景添加检测
- [Phase 6 module loop 检测改动可能影响 @use 递归] → 先跑 ep_full 121 个测试验证
