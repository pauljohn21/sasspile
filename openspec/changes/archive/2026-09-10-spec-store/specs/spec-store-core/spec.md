## ADDED Requirements

### Requirement: HRX 解析入库
系统 SHALL 扫描 `sass-spec/spec/` 目录下的全部 HRX 文件，解析每个 case 的 input/output，结构化存入 SQLite。

#### Scenario: 首次全量扫描
- **WHEN** 运行 `spec_store run --index`
- **THEN** 系统解析全部 3025 个 HRX 文件，在 `spec_cases` 表存入约 11869 条记录，每条含 case_id、hrx_file、function、dir、input_scss、expected_css

#### Scenario: 增量更新
- **WHEN** 重复运行 `spec_store run --index`（已有数据）
- **THEN** 系统跳过已存在的 case_id，只插入新增 case，不重复

### Requirement: Case 编译执行
系统 SHALL 对每个 spec case 执行编译（调用 `sasspile::compile_file_with_load_paths`），记录实际输出和状态。

#### Scenario: 编译成功且匹配期望
- **WHEN** case 编译输出 expected_output.trim() == actual.trim()
- **THEN** 记录 status = PASS，不存储 actual_css

#### Scenario: 编译成功但不匹配
- **WHEN** 编译成功但输出 != 期望
- **THEN** 记录 status = FAIL，failure_type = DIFF，存储 actual_css

#### Scenario: 编译错误但期望错误
- **WHEN** case expect_error = true 且编译返回 Err
- **THEN** 记录 status = PASS

#### Scenario: 编译错误且不期望错误
- **WHEN** case expect_error = false 且编译返回 Err
- **THEN** 记录 status = FAIL，failure_type = ERR，存储 error message

### Requirement: SQLite Schema
系统 SHALL 维护以下核心表结构：`spec_cases`、`snapshots`、`case_results`、`case_deltas`。

#### Scenario: Schema 初始化
- **WHEN** 首次运行 `spec_store run` 或 `spec_store init`
- **THEN** 系统在 DB 路径创建 SQLite 数据库，包含全部 4 个表 + 索引（case_id、function、snapshot_id）

#### Scenario: Schema 迁移
- **WHEN** DB file 版本低于代码期望版本
- **THEN** 系统自动执行迁移脚本，不丢失数据

### Requirement: 函数名提取
系统 SHALL 从 HRX 路径中提取被测函数标识（`core_functions/math/sin.hrx` → `math.sin`）。

#### Scenario: 标准函数路径
- **WHEN** HRX 文件路径为 `core_functions/math/sin.hrx`
- **THEN** function = "math.sin"

#### Scenario: 子目录分组路径
- **WHEN** HRX 文件在子目录 `core_functions/math/pow/positive/`
- **THEN** function = "math.pow"

#### Scenario: 非函数目录
- **WHEN** HRX 在 `values/numbers/` 或 `css/` 目录
- **THEN** function = NULL，dir 字段记录目录路径
