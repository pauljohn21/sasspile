# Spec Delta

## Purpose

全量 Bootstrap 5.3.x 编译验证：编译 Bootstrap SCSS 源码（约 130 个文件），对 Expanded 输出与官方 dist `bootstrap.css` 做逐字节比对，确保编译正确性。Bootstrap SCSS 通过 git submodule（浅拷贝）引入到 `bootstrap/scss/` 目录，测试直接引用该路径，无需运行时下载。

## ADDED Requirements

### Requirement: Bootstrap 源码准备（Git Submodule 浅拷贝）
Bootstrap SCSS SHALL 通过项目 `.gitmodules` 配置的 git submodule 引入到 `bootstrap/` 目录，使用浅拷贝 (`shallow = true`, depth=1) 仅获取最新一份 commit 以节省磁盘和克隆时间。源码 SHALL 包含完整的 `scss/` 目录和 `@import` 文件树。`tests/bootstrap_test.rs` 直接使用 `bootstrap/scss/bootstrap.scss` 作为编译入口。

#### Scenario: First clone with submodule
- **WHEN** 开发者首次执行 `git submodule update --init --depth 1 bootstrap`
- **THEN** SHALL 克隆 Bootstrap 仓库浅拷贝到 `bootstrap/` 目录（仅最新 commit，无历史）

#### Scenario: CI environment setup
- **WHEN** CI 执行 `git submodule update --init --recursive --depth 1`
- **THEN** SHALL 各 submodule （sass-spec / bootstrap / element-plus）均以浅拷贝方式就绪

#### Scenario: Bootstrap SCSS directory accessible
- **WHEN** submodule 初始化完成后
- **THEN** `bootstrap/scss/bootstrap.scss`  SHALL 可供 `CompileBuilder` 引用

#### Scenario: Override with force flag
- **WHEN** 环境变量 `BOOTSTRAP_FORCE_REFRESH=1`
- **THEN** SHALL `rm -rf bootstrap && git submodule update --init --depth 1 bootstrap` 重新克隆

### Requirement: 全量编译测试
`tests/bootstrap_test.rs` SHALL 包含 `#[test] fn compile_bootstrap_full()`。该测试 SHALL 使用 `CompileBuilder::new().include_path("bootstrap/scss/").build("bootstrap/scss/bootstrap.scss")` 编译 Bootstrap 入口文件（submodule 浅拷贝路径）。

#### Scenario: Full Bootstrap compilation succeeds
- **WHEN** 使用 CompileBuilder 编译 Bootstrap 成功
- **THEN** SHALL 返回 Ok(String) 且输出长度 > 100,000

#### Scenario: Compilation error reports location
- **WHEN** Bootstrap 源码包含任何编译错误
- **THEN** SHALL 返回 Err(CompileError::Parse(...)) 或 Err(CompileError::Eval(...)) 并携带尽可能准确的错误位置

### Requirement: 逐字节比对
编译测试 SHALL 将 `build` 输出与 `tests/fixtures/bootstrap-5.3.x-dist.css`（项目内维护的官方 Expanded 输出）做逐字节比对。若存在差异，SHALL 使用 `diff` crate 输出具体差异位置和上下文。

#### Scenario: Byte-for-byte match
- **WHEN** Bootstrap 编译输出与 fixtures 文件完全一致
- **THEN** SHALL 测试通过 (assert_eq!)

#### Scenario: Single byte difference
- **WHEN** 输出在第 1024 字节处有一个字符差异
- **THEN** SHALL 断言失败，打印 diff 上下文显示差异位置

#### Scenario: Length mismatch
- **WHEN** 输出文件比 fixtures 多/少 N 字节
- **THEN** SHALL 打印长度差异和最后一个匹配位置

### Requirement: IncludePath 配置
测试配置 SHALL 在 `CompileBuilder` 时将 `bootstrap/scss/` (submodule 路径) 注册为 IncludePath，使 `@use`/`@import` 能解析相对路径。IncludePath SHALL 遵循 Sass 模块解析规则：先查找相对路径，再查找已注册绝对路径。

#### Scenario: Bootstrap import resolution
- **WHEN** Bootstrap scss 中包含 `@import "variables"`
- **THEN** SHALL 在 `bootstrap/scss/` 中查找 `_variables.scss` 并加载

### Requirement: 压缩模式比对验证
测试 SHALL 提供 `#[test] fn compile_bootstrap_compressed()` 选项以 Compressed 模式编译并与 `tests/fixtures/bootstrap-5.3.x-dist.min.css` 比对。Compressed 模式比对 SHALL 忽略末尾换行差异。

#### Scenario: Compressed output matches
- **WHEN** 编译模式为 Compressed，与 min.css fixtures 比对
- **THEN** SHALL 在 trim 末尾空白后 assert_eq

### Requirement: 模块覆盖度
Bootstrap SCSS 覆盖的编译器特性 SHALL 包含：Maps（`$theme-colors`）、`@use ... with (...)` 配置、`@include` mixin 调用、`@for`/`@each`/`@while` 循环、内置 `color.adjust()` / `map.get()` / `math.div()` 函数、选择器嵌套、父选择器 `&`、插值 `#{}`。Bootstrap 编译测试是编译器的**集成测试天花板**。

#### Scenario: Maps compilation
- **WHEN** Bootstrap `_maps.scss` $theme-colors 参与循环
- **THEN** SHALL 产生正确的 CSS 规则

#### Scenario: @use with configuration
- **WHEN** Bootstrap 入口文件 `@use "sass:color" with ($black: #000)`
- **THEN** SHALL 正确传递配置值到模块内部

### Requirement: 测试隔离与性能
Bootstrap 编译是重操作（130+ 文件、数百行 CSS），SHALL 使用 `#[test]` + `#[ignore]` 标记永不自动运行，仅在手动执行 `cargo test --test bootstrap_test -- --ignored` 时编译。常规 `cargo test` 单元测试 SHALL 不受 Bootstrap 测试影响。

#### Scenario: Default test run skips Bootstrap
- **WHEN** `cargo test`
- **THEN** Bootstrap 编译测试 SHALL 被 skip

#### Scenario: Manual full validation
- **WHEN** `cargo test --test bootstrap_test -- --ignored`
- **THEN** SHALL 执行完整 Bootstrap 编译和比对

#### Scenario: Parallel safe
- **WHEN** 多个开发者同时执行 bootstrap_test
- **THEN** 源码下载 SHALL 使用 atomic write 或 lock file 避免冲突

### Requirement: 输出 dump
编译测试 SHALL 在比对失败时 dump 实际输出到 `target/bootstrap-output-{style}.css`。开发者可 diff 该文件与 fixtures 文件定位错误。

#### Scenario: Dump on mismatch
- **WHEN** bootstrap_test 比对失败
- **THEN** SHALL 写入 `target/bootstrap-output-expanded.css` 和 `target/bootstrap-compressed.css`
