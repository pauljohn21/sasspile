## ADDED Requirements

### Requirement: HRX 内存文件系统

系统 SHALL 实现 `HrxVfs` 结构体，将 HRX 文件中所有条目解析到 `HashMap<String, String>`，使多文件测试能在内存中运行。

#### Scenario: 从 HRX 内容构建 VFS

- **WHEN** 调用 `parse_hrx_to_vfs(content: &str)`
- **THEN** 系统 MUST 复用 `hrx_parser::parse_hrx()` 解析文件列表
- **AND** 将所有 `(path, content)` 对组装到 `HashMap<String, String>`
- **AND** 标记 `input_path` 为 HRX 中 `input.scss` 的路径

#### Scenario: VFS 查找文件

- **WHEN** 调用 `HrxVfs::get(path)`
- **THEN** 系统 MUST 从 `files` HashMap 中查找
- **AND** 返回 `Option<&str>`（找到返回内容，未找到返回 `None`）

### Requirement: VfsResolver 实现 ModuleResolver trait

系统 SHALL 实现 `VfsResolver`，实现 `ModuleResolver` trait，使 `@use`/`@import` 能从内存 VFS 查找文件。

#### Scenario: Sass 标准文件查找逻辑

- **WHEN** 调用 `VfsResolver::resolve(url, base_dir)`
- **THEN** 系统 MUST 按以下顺序查找：
  1. 精确匹配 `url`
  2. `url + ".scss"`
  3. `url + "/_" + basename + ".scss"` (partial)
  4. `url + ".css"`（标记 `is_css=true`，`raw_content=Some`）
- **AND** 找到时返回 `ResolvedModule { ast, is_css, raw_content, source_path }`
- **AND** 未找到时返回 `SassError`

#### Scenario: AST 缓存避免重复解析

- **WHEN** 同一模块被多次 `@use`/`@import`
- **THEN** 系统 MUST 从 `ast_cache` 中获取已解析的 AST
- **AND** MUST NOT 重复调用 `tokenize` + `parse`

#### Scenario: 循环引用检测

- **WHEN** 模块 A `@use` 模块 B，模块 B `@use` 模块 A
- **THEN** 系统 MUST 通过 `loading: HashSet<String>` 检测循环
- **AND** 返回循环引用错误

### Requirement: compile_with_resolver 公共 API

系统 SHALL 在 `src/lib.rs` 中新增 `compile_with_resolver` 函数，使测试能传入自定义的 `ModuleResolver`。

#### Scenario: compile_with_resolver 接收自定义 resolver

- **WHEN** 测试调用 `compile_with_resolver(source, resolver)`
- **THEN** 系统 MUST 执行 `tokenize → parse → evaluate(传 resolver) → serialize`
- **AND** MUST NOT 修改任何编译器逻辑
- **AND** MUST 添加 `#[instrument]` span

#### Scenario: 多文件测试不再被 SKIPPED

- **WHEN** `run_hrx_tests` 遇到多文件测试用例
- **THEN** 系统 MUST 构建 `HrxVfs` + `VfsResolver`
- **AND** MUST 调用 `compile_with_resolver` 执行编译
- **AND** MUST NOT 返回 `SKIPPED` 假通过
