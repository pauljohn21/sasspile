## Context

sasspile 经过多轮 AI 迭代后，代码库中出现以下 5 类 GC 思维产物：

1. **文本手术式替换**：`substitute_vars` / `eval_all_calls` 使用 `while let Some(start) = result.find("#{")` + `result.replace_range()` + `break` 模式
2. **命令式累积**：`expand_while` / `merge_import_lines` / `process_module_imports` 等使用 `for`/`while` + `Vec::push`/`extend`
3. **模块系统绕道**：`module_system.rs` + `import_resolver.rs` 作为"预处理器"完全 bypass rxrust 管线，接收 `&str` 返回 `String`
4. **错误术语传播**：`lib.rs` / `pipeline.rs` / `mod.rs` 的 doc comment 使用 "Flux → rxrust 转译" 等表述，诱导 AI 用 GC 思维写代码
5. **控制流链替代算子**：`process_line` 函数使用 60 行 `if/return` 链而非枚举分发

## Goals / Non-Goals

**Goals:**
- 建立一套可被 AI 解析和遵守的 spec 约束体系
- spec 覆盖算子选择、禁止模式、模块系统响应式化、tracing 规范四大维度
- spec 以 SHALL/MUST 强制语气写成，消除 AI "理解偏差" 空间
- 作为后续 code review 和 CI check 的基准

**Non-Goals:**
- 不直接重构现有代码（重构是后续 change 的任务）
- 不改变编译器的输入/输出行为（spec 管代码风格，不管功能）
- 不替代 `AGENTS.md` / `CLAUDE.md` 的项目级规则（spec 是补充和细化）

## Decisions

### Decision 1: Spec 结构 — 能力域拆分 vs 单一文件

选择：按 5 个能力域拆为独立 spec 文件，每个聚焦一个维度。

**理由**：
- AI 读取 spec 时更精准：写管线代码时查 `reactive-dataflow/spec.md`，写模块系统时查 `module-system-rx/spec.md`
- 每个 spec 独立演进，不会因局部变更导致全文档重审
- 与 OpenSpec 的 `specs/<name>/spec.md` 结构对齐

**替代方案及否决原因**：
- 单一 `spec.md` 全写 → AI 容易遗漏局部约束；修改一处需验证全文

### Decision 2: 强制语气 — SHALL/MUST vs should/may

选择：所有约束使用 SHALL / MUST / 不得，不用 should / may / 建议。

**理由**：AI 对弱化语气（"建议"、"可以"）容易忽视或降级处理。强制语气无法被"理解偏差"消解。

**替代方案及否决原因**：
- should/may 语气 → AI 经常把"建议"当"可选"，形同虚设

### Decision 3: 反模式表 vs 纯文字描述

选择：`no-gc-patterns/spec.md` 同时包含"禁止表"和"正确替代"两列。

**理由**：AI 看到 `禁止: for + push` 时可能不知道该用什么；提供 `正确替代: flat_map + collect` 直接给出目标模式。这种"before/after"对照在 SKILL.md 中验证有效。

**替代方案及否决原因**：
- 只写禁止 → AI 改完可能写出另一种违规形式

### Decision 4: 模块系统响应式化策略

选择：spec 规定 `ModuleEvent` Subject + `scan_map(ModuleResolver)` 模式，但将实现细节留给后续 change。

**理由**：模块系统重构是大工程，本期 spec 只定义"必须怎么做"的约束（不绕过管线、状态在 Acc 中），不动现有行为。当前阶段先建立规范，下阶段照规范写代码。

**替代方案及否决原因**：
- 本期一并重构 → 范围太大，风险高；spec 建立和代码变更应分阶段

### Decision 5: Flux 术语清理策略

选择：spec 明确禁止所有 Flux 映射术语，给出替换示例（如 `Shared Subject 入口` 替换 `Flux.create() Sinks.Many`）。

**理由**：AI 写代码时经常参考已有 doc comment。如果 doc comment 还说 "Flux"，AI 产出自然会延续 Flux 思维。必须从"源头"切掉错误术语。

**替代方案及否决原因**：
- 不清理 → AI 每次读到 `// Flux 思维` 就激活 GC 思维模式

## Risks / Trade-offs

| Risk | Mitigation |
|------|-----------|
| spec 太详细导致 AI 抗拒（"约束太多不敢写"） | spec 以"可选择算子表"形式呈现，给 AI 具体路径而非仅有禁令 |
| spec 与现有 AGENTS.md 冲突 | spec 引用 AGENTS.md 更细化，AGENTS.md 是项目级、spec 是能力域级 |
| spec 难以 enforcement（AI 可能不看） | spec 作为 AGENTS.md 的补充路径，配合 CI lint（如 `grep -r "for.*push" src/`）形成检查网 |
