## ADDED Requirements

### Requirement: sass-spec HRX 是行为契约的唯一来源（排除弃用目录）
SCSS 编译器的行为 MUST 严格遵循 `sass-spec/spec/` 下 **活跃规范目录** 中 HRX 文件描述的 input/output 对。以下目录 MUST 被屏蔽，它们是已弃用的历史规范，不可作为行为来源：

| 目录 | 状态 | 原因 |
|------|------|------|
| `spec/libsass/` | ❌ 弃用 | LibSass 已停止维护 |
| `spec/libsass-closed-issues/` | ❌ 弃用 | LibSass 历史 issue 回归测试 |
| `spec/libsass-todo-issues/` | ❌ 弃用 | LibSass 待办（永远不会实现）|
| `spec/libsass-todo-tests/` | ❌ 弃用 | LibSass 待办测试 |
| `spec/non_conformant/` | ❌ 弃用 | 非 Dart Sass 兼容行为 |

活跃目录包括（不限于此）：`core_functions/`、`directives/`、`css/`、`expressions/`、`parser/`、`values/`、`variables/`、`operators/`。

#### Scenario: 读 HRX 文件推断行为
- **WHEN** 看到 `spec/directives/extend/pseudo.hrx` 中 input 为 `:is(midstream) {@extend upstream}`
- **THEN**编译器 MUST 实现的选择器展开行为应收敛为该文件定义的 output 格式

#### Scenario: 屏蔽 LibSass 目录
- **WHEN** 浏览 sass-spec/spec/ 下的 HRX 文件
- **THEN** MUST NOT 使用 `libsass/`、`libsass-closed-issues/`、`libsass-todo-issues/`、`libsass-todo-tests/`、`non_conformant/` 下的任何文件作为行为契约来源

#### Scenario: 禁止查阅 dart-sass 实现
- **WHEN** 不确定 `@extend` 的展开规则
- **THEN** MUST 查阅 sass-spec 的 HRX 文件和 README.md 理解行为边界；MUST NOT 打开 `dart/sass/lib/src/extend/` 之类的源码

### Requirement: 禁止 println/eprintln
所有代码（含 src/ 和 tests/）MUST 使用 `tracing` 宏（info!/warn!/error!/debug!、trace!）替代 `println!` / `eprintln!`。

#### Scenario: 输出编译结果
- **WHEN** subscribe 闭包需要展示编译出的 CSS
- **THEN** MUST 使用 `info!("{css}")` 或同级别的 tracing 宏；MUST NOT 使用 `println!("{css}")`

### Requirement: 实现 rx-only，保持纯函数式流编程范式
所有源码 MUST 基于 rxrust 的理念：每个函数是流到流的变换，每个副作用发生在线程安全的 Observable 操作内。不接受命令式的代数结构（如大面积的 for 循环 + 显式的 HashSet 检查），而 MUST 使用 `.scan()` / `.flat_map()` / `.filter()` 等算子等价表达。

#### Scenario: 实现选择器去重
- **WHEN** 需要消除重复选择器以避免输出重复 CSS
- **THEN** MUST 使用 `.distinct()` 或 `.distinct_until_changed()` rxrust 算子；MUST NOT 在 `evaluate` step 中手动维护 `HashSet<String>` 并显式写入

#### Scenario: 实现嵌套规则扁平化
- **WHEN** 遇到 `a { b { c: d } }` 嵌套结构
- **THEN** MUST 使用 `.flat_map(Nesting::expand)` 三步展开；MUST NOT 手动栈循环

### Requirement: 编译器的最终入口是一行 Observable subscribe
`sasspile-rx` crate 的最终入口 MUST 是一行或极短的 subscribe 调用，将 Observable 源流形变换到结果订阅。入口 MUST 反映 `Observable<PathBuf/String> → ... → subscribe(Result)` 的签名，且 MUST NOT 与任何实验性原型代码耦合。

#### Scenario: 使用默认管道
- **WHEN** 用户触发编译
- **THEN** 入口 MUST 类似：`compile_pipeline(input).last().subscribe(handler)`，其中 `compile_pipeline` 返回类型擦除后的 `LocalBoxedObservable`

#### Scenario: 使用 conduit 链
- **WHEN** 用户组合自定义管道（如只 introspect 中间 AST）
- **THEN** MUST 提供函数式 API：用户可组建 `compile_pipeline().map(...)...subscribe(...)`，而不是修改任何内部文件

### Requirement: 企业级 SCSS 项目 100% 通过验证
编译器 MUST 将两个大型企业级 SCSS 项目当作最终验收标准：Bootstrap (`twbs/bootstrap`) 与 Element Plus (`element-plus/element-plus`)。这两个项目 MUST 以 git submodule 引入，并使用 `git clone --depth 1` 浅拷贝（submodule status 标记为 `-d`）。

#### Scenario: Bootstrap 入口 SCSS 编译零错误
- **WHEN** 编译器处理 `bootstrap/scss/bootstrap.scss`（及其全部 `@import` / `@use` 依赖树）
- **THEN** MUST 输出与 `bootstrap/dist/css/bootstrap.css` 语义等价的 CSS（空白差异允许），且 MUST NOT 产生编译错误

#### Scenario: Element Plus 入口 SCSS 编译零错误
- **WHEN** 编译器处理 `element-plus/packages/theme-chalk/src/index.scss`（及其全部依赖树）
- **THEN** MUST 输出有效 CSS，且 MUST NOT 产生编译错误

#### Scenario: 100% 通过率验收
- **WHEN** 运行 sass-spec 活跃目录 + Bootstrap + Element Plus 的全部 SCSS 编译测试套件
- **THEN** 整体通过率 MUST 为 100%（历史目录已显式排除，不计入分母）
