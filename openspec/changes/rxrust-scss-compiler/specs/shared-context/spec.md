## ADDED Requirements

### Requirement: 编译器必须维护可共享的 CompilerContext
编译器 MUST 维护一个 `Arc<CompilerContext>` 实例，在 pipeline 执行期间注入到需要跨文件/跨阶段共享状态的所有算子。该上下文 MUST 至少包含以下三个数据结构：

| 字段 | 类型 | 职责 |
|------|------|------|
| `module_cache` | `DashMap<PathBuf, EvaluatedModule>` | 文件路径 → 已评估模块 的映射，避免重复 parse |
| `global_vars` | `RwLock<Scope>` | 存储 `@import` 引入的全局变量和 `!global` 写入 |
| `use_registry` | `DashMap<String, SourceLocation>` | 已 `@use` 的模块名 → 来源路径（用于增量 rebuild 的脏检查） |

#### Scenario: 初始化 CompilerContext
- **WHEN** pipeline 启动时
- **THEN** MUST 创建一个 `Arc::new(CompilerContext::default())` 实例
- **AND** MUST 在 evaluate stage 的闭包中被捕获并传递

#### Scenario: 跨 stage 共享上下文
- **WHEN** tokenize 识别到 `@use` 或 `@import` 指令
- **THEN** MUST 通过 `CompilerContext.module_cache` 查询或填充模块
- **AND** 后续文件请求 MUST 复用已评估模块

### Requirement: 模块缓存 MUST 避免重复 parse/eval
同一文件路径 MUST 仅被 tokenize/parse/evaluate 一次，第二次 `@use`/`@import` 同一文件 MUST 直接返回 EvaluatedModule 缓存。

#### Scenario: 多次 @import 同一文件
- **WHEN** 文件 A 执行 `@import "variables"`，文件 B 也执行 `@import "variables"`
- **THEN** 第二次 MUST 未触发 tokenize/parse/evaluate，直接返回缓存的 `EvaluatedModule`
- **AND** Observable MUST 不产生重复输出

#### Scenario: 缓存命中可被 tracing 观测
- **WHEN** module_cache 命中
- **THEN** MUST 记录 `debug_span!("sasspile.cache", path=path, hit=true)`

### Requirement: 全局变量表 MUST 支持 !global 写入可见性
SCSS 的 `!global` flag 允许在任何嵌套作用域写入全局变量。MUST 让 `global_vars: RwLock<Scope>` 对所有后续 Observable 元素可见。

#### Scenario: !global 写入后同文件中后续引用
- **WHEN** 代码中先出现 `$colors: map.merge((...), $colors) !global;`，然后 `@include set-color-mix-level($type, ...)`
- **THEN** `set-color-mix_level` mixin 内部 MUST 能读取到 mutation 后的 `$colors`

#### Scenario: !global 写入后跨文件可见
- **WHEN** `common/var.scss` 中 `set-color-mix-level` mixin 执行 `$colors: map.deep-merge((...)) !global;`
- **THEN** 后续 `@use 'common/var' as *` 的文件 MUST 能观察到混合后的 `$colors` 值

### Requirement: @use ... as * 命名空间注入
Element Plus 90%+ 文件使用 `@use 'X' as *` 去掉命名空间前缀。MUST 解析 `as *` 语义，将模块所有 public 成员注入当前 Scope。

#### Scenario: @use 带 * 注入
- **WHEN** 文件顶部写 `@use 'common/var' as *;`
- **THEN** 该文件 MUST 能直接访问 `$colors`, `$border-radius` 等变量而**不**需要写成 `common.var.$colors`

#### Scenario: @use 不带 * 保留命名空间
- **WHEN** 文件写 `@use 'common/var';`
- **THEN** 变量 MUST 仅在 `var.$colors` 路径下可访问
- **AND** MUST NOT 污染全局 scope

#### Scenario: @use 带别名
- **WHEN** 文件写 `@use 'common/var' as v;`
- **THEN** 变量 MUST 仅在 `v.$colors` 路径下可访问

### Requirement: CompilerContext MUST 线程安全
虽然当前 pipeline 使用 `Local` 单线程语义，`CompilerContext` interior MUST 实现 `Send + Sync`，以便未来扩展到 `Shared` 多线程 pipeline。

#### Scenario: Arc<CompilerContext> 跨 stage 传递
- **WHEN** pipeline.rs 将 `ctx: Arc<CompilerContext>` 传入 stage 闭包
- **THEN** MUST 编译通过（`Arc` 的 Send + Sync 边界检查）

### Requirement: Observable 管道 MUST 保持因果顺序（happens-before）
SCSS 文件依赖有严格的执行顺序，某个文件的 `@use` MUST 在该文件任何代码之前完成评估。

#### Scenario: @use 前代码不执行
- **WHEN** 文件写 `@use 'variables'; $x: $variables.primary;`
- **THEN** `variables` 模块 MUST **先完成** tokenize/parse/evaluate，**之后**才评估 `$x: $variables.primary`

#### Scenario: 循环依赖检测
- **WHEN** 文件 A `@use` 文件 B，文件 B `@use` 文件 A
- **THEN** MUST 产生 `CompileError::CircularDependency { path_a, path_b }`
- **AND** MUST NOT 陷入死循环或 stack overflow

### Requirement: 共享缓存对 Observable 是透明的（scan 传播）
CompilerContext 在 pipeline 中的传播 MUST 通过 rxrust 算子完成（如 `scan` 或闭包捕获 Arc），MUST NOT 使用全局静态变量（`lazy_static` / `thread_local`）。

#### Scenario: 测试中使用隔离的 CompilerContext
- **WHEN** 测试函数创建 pipeline
- **THEN** MUST 每个测试获得独立的 `Arc<CompilerContext>`，测试之间 MUST NOT 共享缓存状态
