## Context

sasspile 的 sass-spec 全量通过率停留在 54.3%（6427/11824），核心测试 202/202 全通过但大量 SCSS 功能边界未覆盖。经过 trace 分析，失败集中分布在：颜色函数深层面（3194 fail）、values 解析（644 fail）、CSS 兼容性（374 fail）、模块系统高级特性（57 fail）、指令和运算符边界（~45 fail）。

当前架构已经稳定——Source→Lex→Parse→Eval→Serialize 的函数式管线，Env 的 move 语义零 clone 设计，22,694 个节点的 CodeGraph 索引。本次变更是在现有架构上**扩展行为覆盖**，**不改变 API 或架构模式**。

## Goals / Non-Goals

**Goals:**
- 系统性补齐 sass-spec 失败的功能点，覆盖 6 个优先级分组
- 每个 Phase 可独立实施、独立验证，不影响其他模块
- 保持函数式风格 + move 语义的设计约束（AGENTS.md）
- 核心测试 202/202 全通过不回归
- 目标通过率从 54.3% 提升至 63%+

**Non-Goals:**
- 不引入新的公共 API 或改变现有 API 签名
- 不重构架构（架构已稳定）
- 不实现纯 CSS 兼容性修复（那是 AGENTS.md 之外的独立工作）
- 不追求 100% sass-spec 通过率（那个需要数年工作）

## Decisions

### Decision 1: 分阶段实施而非一次性大变更

**WHY**: 5397 个失败 case 分布在 8+ 个功能领域，一次性修复风险高、验证困难。
**HOW**: 按失败量和影响分成 Phase 0~5，每个 phase 是一个独立变更单元。
**ALTERNATIVE**: 一次性大 PR → 被否决，因验证困难、回归定位复杂

### Decision 2: 颜色函数修复对齐 CSS Color Level 4 规范

**WHY**: 颜色是失败最多的区域（3194），但近期已有大量修复经验，有成熟的色彩空间转换矩阵和序列化工具。
**HOW**: 使用 W3C 规范的有理数分数矩阵，对 hsl/hwb/lab/lch/oklab/oklch 逐一诊断修复序列化精度和 NaN 处理。
**ALTERNATIVE**: 逐个试错修复 → 被否决，因需要系统性理解规范

### Decision 3: Meta 反射函数统一到现有 builtin-dispatch-macro 架构

**WHY**: 新的 meta 函数（get-function, get-mixin 等）遵循现有的命名参数合并 + 分派模式，应纳入 `#[derive(BuiltinRegistry)]` 宏。
**HOW**: 在 `MetaBuiltins` 结构体上增加新字段，宏自动生成 is_known_builtin 和 dispatch 逻辑。
**ALTERNATIVE**: 手工在 manual_dispatch.rs 追加 → 可用但不一致，不利于维护

### Decision 4: 错误消息格式集中管理

**WHEN**: 当前错误消息散落在各 eval 函数中，格式各不相同。
**HOW**: 对高频错误类型（未定义变量/函数、类型错误、模块错误）建立统一的错误消息模板，包含文件位置信息。
**ALTERNATIVE**: 在每个调用点独立 format → 当前做法，保持灵活性但易不一致

### Decision 5: 模块系统高级特性基于现有 parse + eval 扩展

**WHY**: `@use`/`@forward` 的解析和求值基础设施已就位，缺失的是边界校验和组合场景处理。
**HOW**: 在 `parse_use`/`parse_forward` 中增加冲突检测逻辑，在 `eval_use`/`eval_forward` 中实现模块缓存命名空间管理。

## Risks / Trade-offs

- **[颜色修复范围]** → 颜色 spec 极大（6027 case），全量修复耗时长。Mitigation：按子目录优先级推进，先修 channel(81%→90%)、adjust(56%→70%) 中修，再修 hsl/hwb/lab 深度修复。
- **[回归]** → 修改共享的值类型和序列化逻辑可能影响现有通过 case。Mitigation：每次提交前跑 202 核心测试 + 修改涉及的 sass-spec 子目录。
- **[性能]** → 新增边界校验和反射函数可能增加运行时开销。Mitigation：保持 O(1) 查找模式，不引入额外遍历。
- **[维护负担]** → 新增大量 builtin 函数增加代码量。Mitigation：继续用 builtin-dispatch-macro 宏统一注册，手工分派保持最小化。
