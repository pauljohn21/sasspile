# Spec Delta

## MODIFIED Requirements

### Requirement: System SHALL define CompilerBus struct with multicast Subjects / 系统 SHALL 定义包含多播 Subject 的 CompilerBus 结构体

系统 SHALL 定义 `pub struct CompilerBus<E: crate::Error>`，包含每个通信通道的多播 Subject：`var_events: Subject<ValueEvent, E>`、`module_events: Subject<ModuleEvent, E>`、`css_scope_subject: Subject<ScopeEvent, E>`。所有 Subject 实例 SHALL 使用默认无界通道（`Subject::new()`）。**MODIFIED**: 原规定错误类型为 `Infallible`，改为泛型 `E: crate::Error`，使其能携带错误事件。

The system SHALL define `pub struct CompilerBus<E: crate::Error>` containing multicast Subjects for each communication channel: `var_events: Subject<ValueEvent, E>`, `module_events: Subject<ModuleEvent, E>`, and `css_scope_subject: Subject<ScopeEvent, E>`. **MODIFIED**: Original specification used `Infallible` error type, changed to generic `E: crate::Error` for carrying error events.

#### Scenario: CompilerBus is created with fresh Subjects / 创建 CompilerBus 带全新 Subject
- **WHEN / 当** 调用 `CompilerBus::<SassError>::new()` 时
- **THEN / 那么** 三个 Subject 被创建，error 类型为 `SassError`，没有任何 observer 连接

#### Scenario: CompilerBus can be cloned for shared access / 克隆 CompilerBus 共享访问
- **WHEN / 当** 克隆 `CompilerBus<E>` 实例时
- **THEN / 那么** 克隆产生指向相同底层多播的引用计数指针（即 `CompilerBus` 包装内部 `Rc<InnerBus<E>>`）

#### Scenario: Subject propagates error events / Subject 传播错误事件
- **WHEN / 当** 某阶段向 `var_events` 发射错误时
- **THEN / 那么** 已订阅的 observer 通过 `on_error(E)` 收到错误通知
