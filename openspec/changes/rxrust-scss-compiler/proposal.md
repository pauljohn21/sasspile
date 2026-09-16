## Why

sasspile-rx 项目需要一个完全以 **rxrust 算子机制** 为根源的设计哲学宣言和实践指南。设计不从任何实验性代码出发，而是从 rxrust 的四原语（Observable/Observer/Subscription/Operator）及其算子体系（Creation/Transformation/Filtering/Combination/Utility/Aggregation/Connectable）推导编译器的完整架构。

本 change 将建立：
1. rxrust 算子语义 → 编译器 stage 算子的完整映射
2. sass-spec HRX 作为行为契约的唯一来源
3. 企业级项目 100% 编译通过作为验收标准

## What Changes

- 确立 **rxrust-first 编译器架构**：每个 stage 是一个 `CoreObservable<C>` 实现，通过 Observer 包装模式构建
- 建立 **四原语为根本** 的编译器抽象：Observable（流载体）/ Observer（消费端）/ Subscription（生命周期）/ Operator（纯函数变换）
- 定义 **算子使用清单**：Transformation/Filtering/Combination/Utility/Aggregation/Connectable 每类至少使用一种
- 定义 **Infallible + Result<T, E>** 错误管道模式
- 确立 **tap-only 副作用** + **scan 状态累积** + **flat_map 嵌套展开** + **publish/ref_count 模块共享** 的 rxrust 实践
- 确立 **Enterprise 100% 验收**：Bootstrap + Element Plus 编译零错误

## Capabilities

### New Capabilities
- `rxrust-foundations`: rxrust 四原语 + 算子体系 + Observer 包装模式作为编译器的唯一设计根源
- `reactive-pipeline-core`: tokenize/parse/evaluate/serialize 四 stage 通过 `pipe` + `box_it` 链式组合
- `shared-context`: 模块缓存的 publish/ref_count 共享语义 + CompilerContext 的 scan 传播
- `error-as-value`: 编译错误作为流值继续传播（Observable<T, Infallible> + Result<T, CompileError>）
- `subscription-sink`: reduce/last/collect 收敛为 Result<CSS, CompileError>
- `spec-driven-design`: sass-spec HRX 为唯一行为契约，排除弃用目录

### Modified Capabilities
无现有 capability 需要修改（首次创建）。

## Impact

- **受影响代码**: 仓库现有 `main.rs` 为概念实验，不影响最终架构
- **新增代码**: 从零构建 `src/pipeline.rs` + stage 文件 + `src/shared/` + `tests/`
- **依赖**: 仅依赖 `rxrust` + 标准库
- **模块约束**: 单文件 ≤ 500 行
- **验证基准**: sass-spec + Bootstrap + Element Plus 100% 通过
