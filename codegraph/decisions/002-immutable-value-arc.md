# Decision 002: 值系统使用不可变数据结构 + Arc

## Status
Accepted

## Context
多阶段并行编译需要跨 Task 共享数据，同时避免数据竞争。

## Decision
所有 `Value` 类型实现 `Clone`，使用 `Arc<T>` 共享大数据。

## Alternatives Considered
- `Rc<T>` → 非 Send，无法跨 Task
- 可变引用 `&mut` → 违反借用规则（多阶段并行）
- Copy-on-write (`Cow`) → 过度复杂

## Consequences
- ✅ Arc 是 Tokio 跨 Task 共享的标准方式
- ✅ 不可变性消除数据竞争
- ✅ 符合函数响应式核心原则
- ⚠️ 内存使用可能偏高（需 moka 缓存缓解）

## Related
- architecture.md
- value-system.md
- tasks.md Phase 1
