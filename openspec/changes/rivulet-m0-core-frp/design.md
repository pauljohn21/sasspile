## Context

Rivulet 是一个全新项目，受 sasspile 管线架构启发，采用 Behavior/Event 经典 FRP 模型。M0 是框架地基——所有 Event/Behavior/Runtime 核心逻辑，纯 Rust 无 DOM 依赖。设计文档详见 `docs/superpowers/specs/2026-08-11-rivulet-design.md`。

## Goals / Non-Goals

**Goals:**
- 实现 Arena-based Event<T>/Behavior<T> 轻量句柄（8 字节 NodeId）
- 实现 Event 组合子：map、filter、merge、sample
- 实现 Behavior 组合子：now、map、accumulate
- 实现 Runtime 推送引擎：事件入队 → 批量 flush → 沿依赖图 propagate
- 实现 Emitter 事件触发器
- 实现 Context 全局状态传递

**Non-Goals:**
- Widget/VDom/Diff（M1）
- Renderer/Web/DOM 操作（M2/M3）
- `#[widget]` 宏（M4）
- 路由（M5）
- 小程序（M6）
- 错误处理完善（M0 使用 panic/expect，M1 完善）
- Box::leak 内存回收（M0 简化，后续改进）

## Decisions

### 1. Arena + NodeId 替代 Rc<RefCell>

**选择**：统一 Arena（slotmap）分配所有节点，Event/Behavior 是 8 字节 NodeId 句柄。

**理由**：
- 句柄 8 bytes vs Rc 的 16 bytes + 引用计数开销
- 连续 slab 内存布局，缓存友好
- 依赖图遍历通过索引查表 O(1)，无需递归跟指针
- Arena 统一回收，无需追踪 Rc 环

**替代方案**：Rc<RefCell<T>>——更简单但开销更大、内存分散。

### 2. Box::leak 提供 'static Runtime 引用

**选择**：`runtime()` 函数通过 `Box::leak` 创建 'static Runtime 引用，Event/Behavior 持有 `&'static Runtime`。

**理由**：
- Event/Behavior 组合子需要访问 Runtime 注册新节点
- 'static 生命周期避免生命周期参数传染整个 API
- WASM 进程生命周期内 Runtime 存在，leak 无实际问题

**替代方案**：thread_local Runtime——可行但调试困难、不显式。

### 3. 事件入队 + 批量 flush

**选择**：`Emitter::fire()` 将事件入队，`Runtime::flush()` 批量处理。

**理由**：
- 避免同步递归推送导致栈溢出
- 批量处理减少不必要的中间渲染
- 类似 sasspile Lexer 逐 token 产出后 Parser 批量消费

### 4. 类型擦除用 Box<dyn Any>

**选择**：Arena 中 BehaviorNode.value 为 `Box<dyn Any>`，通过 downcast 恢复类型。

**理由**：统一 Arena 需要存储不同类型的 Behavior 值，类型擦除是唯一可行方案。downcast 开销在 M0 可接受。

## Risks / Trade-offs

- **[Box::leak 内存不回收]** → M0 可接受（WASM 进程生命周期）。后续可通过 scoped arena 或 arena 分配器改进。
- **[Sample 类型擦除复杂]** → Sample 事件需要读取 Behavior 当前值作为新 payload，类型擦除使传播逻辑复杂。需仔细测试。
- **[Filter payload 传递]** → Filter 通过后传递原始 payload 引用，依赖 Any 的引用语义。需验证不丢失类型信息。
- **[Box<dyn Any> 运行时开销]** → downcast 有运行时检查开销。M0 可接受，后续可考虑泛型化关键路径。
