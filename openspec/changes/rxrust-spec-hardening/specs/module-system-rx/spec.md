## ADDED Requirements

### Requirement: 模块系统不得绕过 rxrust 管线
AI 不得将 @use/@forward 实现为"纯函数预处理器"（接收 `&str` 返回 `String`）。MUST 将模块解析事件注入管线，由 rxrust 算子处理。

#### Scenario: @use 行处理
- **WHEN** AI 处理 `@use "url"` 行
- **THEN** MUST 将 @use 事件注入 `Shared::subject::<ModuleEvent, Infallible>()`，由 `scan_map(ModuleResolver::new(), resolve_module)` 处理
- **不得** 使用 `while let Some(line) = queue.pop_front() { ... out_injections.push(...) }` 模式在管线外展开

#### Scenario: @forward 链展开
- **WHEN** AI 展开 @forward 链
- **THEN** MUST 使用 `scan_map(ForwardResolver::new(), |resolver, fwd_event| resolver.resolve(fwd_event))`，状态（已加载集合、命名空间映射）在 `&mut ForwardResolver` 中维护
- **不得** 使用 `while let Some(fwd_line) = fwd_queue.pop_front() { ... ns_maps.push(...) }` 手动 BFS

### Requirement: 命名空间替换必须是结构化操作
AI 不得使用 `result.replace(&format!("{alias}.{old}"), new)` 文本替换实现命名空间映射。MUST 基于 token/AST 做结构化解析。

#### Scenario: 变量引用命名空间化
- **WHEN** AI 处理 `alias.$var` 形式的变量引用
- **THEN** MUST 解析 `(Namespace, VariableName)` 二元组后查表替换，不得使用字符串拼接 + replace

#### Scenario: @include 命名空间化
- **WHEN** AI 处理 `@include alias.mixin_name` 引用
- **THEN** MUST 解析 mixin 调用表达式后做结构化分发，不得使用 `result.replace(&format!("@include {alias}.{old}"), ...)` 模式

### Requirement: 模块加载状态用 scan_map Acc 持有
AI 不得使用外部 `HashSet<String>` 手动追踪已加载模块和命名空间映射。MUST 将状态封装在 scan_map 的 Acc 结构体中。

#### Scenario: 循环导入检测
- **WHEN** AI 检测循环导入
- **THEN** MUST 在 `ModuleResolver` Acc 中维护 `loaded: HashSet<String>`，通过 `&mut self.loaded` 检查
- **不得** 使用函数外 `let mut loading = HashSet::new();` + 闭包捕获 `&mut loading`

#### Scenario: 命名空间映射表
- **WHEN** AI 维护 alias → member 的映射
- **THEN** MUST 在 `ModuleResolver` Acc 中维护 `ns_maps: Vec<(String, NsMap)>`，通过 `&mut self.ns_maps` 就地修改
- **不得** 使用函数外 `let mut ns_maps = Vec::new();` + 闭包捕获 `&mut ns_maps`
