## ADDED Requirements

### Requirement: 失败 JSON 输出
测试文件 `tests/failures_json.rs`  SHALL 运行全部 sass-spec case，将所有失败原因以结构化 JSON 写入 `tests/sass-spec-failures.json`。

#### Scenario: 基本运行
- **WHEN** 执行 `cargo test --test failures_json -- --nocapture`
- **THEN** 全部 12131 个 case 被评估，JSON 文件被写入磁盘

#### Scenario: JSON 结构完整性
- **WHEN** JSON 文件生成后
- **THEN** 包含 `metadata`（timestamp/pass/fail/skip/total）、`failures` 数组、`by_dir` 聚合、`by_type` 聚合

### Requirement: 失败记录内容
每条失败记录 SHALL 包含完整的 expected 输出、actual 输出或编译错误信息。

#### Scenario: DIFF 类型失败
- **WHEN** case 编译成功但输出与 expected 不匹配
- **THEN** 记录 `type: "DIFF"`、`expected`（完整 expected CSS）、`actual`（完整实际输出）

#### Scenario: ERR 类型失败
- **WHEN** case 编译返回错误但标记为期望成功
- **THEN** 记录 `type: "ERR"`、`error`（错误信息字符串）

#### Scenario: ERR_EXP_OK 类型失败
- **WHEN** case 标记为期望错误但编译成功输出 CSS
- **THEN** 记录 `type: "ERR_EXP_OK"`、`actual`（实际输出）

### Requirement: 失败记录标识
每条失败记录 SHALL 包含唯一标识符和所属目录。

#### Scenario: 记录标识
- **WHEN** 记录一条失败
- **THEN** 包含 `id`（case 的唯一路径标识，如 `core_functions/color/scale/hue/percentage`）和 `dir`（一级目录名）

### Requirement: 复用现有基础设施
工具 SHALL 复用 `hrx_support::parse_hrx_to_cases` 和 `compile_file_with_load_paths`，不重复实现 VFS 或编译逻辑。

#### Scenario: HRX 解析
- **WHEN** 读取 HRX 文件
- **THEN** 通过 `hrx_support::parse_hrx_to_cases` 解析，不自行实现解析器

#### Scenario: 编译调用
- **WHEN** 编译一个 case
- **THEN** 使用 `sasspile::compile_file_with_load_paths` 与 `OutputStyle::Expanded`

### Requirement: 跳过规则一致性
工具 SHALL 使用与 `sass_spec_full.rs` 一致的目录跳过列表（`spec_manifest::SKIP_DIRS`），并跳过 `.sass` 缩进式语法 case。

#### Scenario: 跳过目录
- **WHEN** `SKIP_DIRS` 中的目录（如 `libsass`、`non_conformant`）
- **THEN** 这些目录的 HRX 文件不被评估

#### Scenario: 跳过 .sass 文件
- **WHEN** case 的 `input_path` 以 `input.sass` 结尾
- **THEN** 该 case 被跳过（与 `sass_spec_full.rs` 一致）

### Requirement: 聚合统计
JSON SHALL 包含按目录和按失败类型的聚合统计。

#### Scenario: by_dir 聚合
- **WHEN** 生成聚合数据
- **THEN** `by_dir` 是一个 map，key 为目录名，value 包含该目录下各失败类型的计数

#### Scenario: by_type 聚合
- **WHEN** 生成聚合数据
- **THEN** `by_type` 包含 `DIFF`、`ERR`、`ERR_EXP_OK` 三种类型的总计数

### Requirement: 执行耗时与进度
测试 SHALL 通过 `tracing::info` 输出进度信息，便于监控 4 分钟运行的进展。

#### Scenario: 进度日志
- **WHEN** 工具运行中
- **THEN** 输出当前处理的目录名、已评估 case 数、发现失败数
