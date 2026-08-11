## 1. 项目脚手架

- [ ] 1.1 创建 `rivulet` 仓库 workspace 根 Cargo.toml（edition 2024, toolchain 1.97, members = ["rivulet-core"]）
- [ ] 1.2 创建 `rivulet-core` crate Cargo.toml（依赖 slotmap, smallvec, tracing）
- [ ] 1.3 创建 `rivulet-core/src/lib.rs` 入口 + 模块声明 + 公共 re-export
- [ ] 1.4 创建 `.gitignore`，`git init`，首次 commit

## 2. 辅助类型

- [ ] 2.1 编写 `tests/types_test.rs`：验证 NodeId 是 Copy 且 8 字节
- [ ] 2.2 实现 `src/types.rs`：NodeId（slotmap DefaultKey）、DomId、Key 枚举 + From 实现
- [ ] 2.3 运行测试验证通过，commit

## 3. Arena 节点类型 + 依赖图

- [ ] 3.1 编写 `tests/node_test.rs`：验证 EventSource 变体和 NodeKind 判别
- [ ] 3.2 实现 `src/node.rs`：Node 枚举（Event/Behavior）、EventNode、BehaviorNode、EventSource（Manual/Dom/Map/Filter/Merge/Sample）、NodeKind、as_event/as_behavior 方法
- [ ] 3.3 运行测试验证通过，commit

## 4. Runtime + Arena + 推送引擎

- [ ] 4.1 编写 `tests/runtime_test.rs`：runtime 创建、create_event、flush 空队列、emit+flush 更新 Behavior、批量 emit
- [ ] 4.2 实现 `src/context.rs`：Context（TypeId 索引的 HashMap，provide/use_value）
- [ ] 4.3 实现 `src/emitter.rs`：Emitter<T>（裸指针 rt + NodeId + PhantomData，fire 方法，Clone）
- [ ] 4.4 实现 `src/runtime.rs`：Runtime（SlotMap Arena + 事件队列 + Context）、runtime() 函数（Box::leak 'static）、register_event/register_behavior/add_dependency、emit/flush/propagate、EventSource::transform 辅助方法
- [ ] 4.5 创建 `src/event.rs` 和 `src/behavior.rs` 存根（todo!）
- [ ] 4.6 验证编译通过（测试可能 panic on todo!），commit

## 5. Event/Behavior 完整组合子实现

- [ ] 5.1 编写 `tests/event_test.rs`：map 变换、map 链式、merge 合并、sample 采样
- [ ] 5.2 编写 `tests/behavior_test.rs`：accumulate 初始值、accumulate 更新、map 派生、map 链式、同事件多 Behavior
- [ ] 5.3 修改 Runtime：添加 as_static()、with_behavior_value() 方法，修改 runtime() 签名为 `FnOnce(&'static Runtime)`
- [ ] 5.4 完整实现 `src/event.rs`：Event<T>（id + rt: &'static Runtime + PhantomData）、map/filter/merge/sample 组合子、Clone
- [ ] 5.5 完整实现 `src/behavior.rs`：Behavior<T>（id + rt: &'static Runtime + PhantomData）、now/accumulate/map 组合子、Clone
- [ ] 5.6 修改 Runtime::propagate：完善 Filter（should_propagate）、Sample（读取 behavior 值）分支处理
- [ ] 5.7 运行全部测试验证通过，commit

## 6. Context + Emitter + 端到端集成测试

- [ ] 6.1 编写 `tests/context_test.rs`：provide+use、None 返回、多类型、覆盖、自定义类型
- [ ] 6.2 编写 `tests/emitter_test.rs`：fire 入队、clone 触发同一事件、String payload
- [ ] 6.3 编写 `tests/integration_test.rs`：counter app（+1/-1）、todo list（add+toggle）、derived behaviors 链式、fanout 同事件
- [ ] 6.4 运行全部测试验证通过
- [ ] 6.5 Commit — M0 完成
