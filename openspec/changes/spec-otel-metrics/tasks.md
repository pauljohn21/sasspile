## 1. Cargo.toml 依赖更新

- [x] 1.1 `opentelemetry-stdout` features 从 `["trace"]` 改为 `["trace", "metrics"]`
- [x] 1.2 确认 `opentelemetry_sdk` 和 `opentelemetry` 默认 features 包含 metrics（如不包含则显式添加）
- [x] 1.3 `cargo check` 确认依赖编译通过

## 2. tracing_init.rs 加 Metrics 初始化

- [x] 2.1 新增 `static METRICS_INIT: Once` 和 `static METER_PROVIDER: OnceLock<SdkMeterProvider>`
- [x] 2.2 实现 `init_metrics(label: &str)` — 创建 `MetricExporter` + `SdkMeterProvider`，注册到 `global::set_meter_provider`
- [x] 2.3 实现 `shutdown_metrics()` — `force_flush()` + `shutdown()`
- [x] 2.4 添加 tracing span（`metrics_init`, stage=`tracing`, 字段 `label`）
- [x] 2.5 `cargo check --tests` 确认编译通过

## 3. HrxVfs + VfsResolver（tests/hrx_vfs.rs）

- [x] 3.1 定义 `HrxVfs { files: HashMap<String, String>, input_path: String }` 结构体
- [x] 3.2 实现 `parse_hrx_to_vfs(content: &str) -> HrxVfs` — 复用 `hrx_parser::parse_hrx()`，将文件列表组装到 HashMap
- [x] 3.3 实现 `HrxVfs::get(path) -> Option<&str>`
- [x] 3.4 定义 `VfsResolver { vfs: HrxVfs, ast_cache: HashMap<String, Vec<Stmt>>, loading: HashSet<String> }`
- [x] 3.5 实现 `ModuleResolver` trait for `VfsResolver` — Sass 标准查找逻辑（精确→.scss→_partial→.css）
- [x] 3.6 AST 缓存 + 循环引用检测
- [x] 3.7 添加 tracing span（`vfs_resolve`, stage=`test`, 字段 `url`/`resolved_path`/`is_css`）

## 4. compile_with_resolver 公共 API（src/lib.rs）

- [x] 4.1 在 `src/lib.rs` 新增 `pub fn compile_with_resolver(source: &str, resolver: &mut dyn ModuleResolver) -> Result<String, SassError>`
- [x] 4.2 添加 `#[instrument(name = "compile", skip_all, fields(stage = "compile"))]`
- [x] 4.3 函数体：tokenize → parse → evaluate(传 resolver) → serialize
- [x] 4.4 `cargo check` 确认编译通过（不改编译器逻辑，只加入口）

## 5. SpecOtelRunner（tests/spec_otel_runner.rs）

- [x] 5.1 定义 `SpecOtelRunner { domain: &str, meter: Meter, stats: DomainStats, results: Vec<SpecTestResult> }`
- [x] 5.2 实现 `new(domain: &str) -> Self` — 从 `global::meter_provider()` 获取 meter
- [x] 5.3 实现 `run_spec_test(name, domain, input, expected, expected_error)` — 编译+比对+Counter.add+Histogram.record
- [x] 5.4 失败时 `tracing::error!` 记录不 panic（字段 domain/test_name/expected/actual）
- [x] 5.5 实现 `run_hrx_tests(hrx_path)` — 单文件走 `run_spec_test`，多文件走 VFS 模式
- [x] 5.6 多文件模式：构建 HrxVfs → VfsResolver → `compile_with_resolver` → 比对 output
- [x] 5.7 实现 `finalize() -> DomainStats` — 注册 ObservableGauge 回调报告通过率
- [x] 5.8 实现 `assert_results()` — 全跑完后统一 assert（`if failed > 0 { panic! }`）
- [x] 5.9 添加 tracing span（`spec_test`/`spec_hrx`/`spec_domain`）
- [x] 5.10 `#[test]` 验证 SpecOtelRunner 基本功能（在 tests/ 目录）

## 6. 改造 17 个 spec_*.rs 测试文件

- [x] 6.1 `spec_plain.rs` — 改用 SpecOtelRunner + assert
- [x] 6.2 `spec_css.rs` — 改用 SpecOtelRunner + assert（6 个子测试合并为 1 个）
- [x] 6.3 `spec_directives.rs` — 改用 SpecOtelRunner + assert
- [x] 6.4 `spec_expressions.rs` — 改用 SpecOtelRunner + assert
- [x] 6.5 `spec_operators.rs` — 改用 SpecOtelRunner + assert
- [x] 6.6 `spec_parser.rs` — 改用 SpecOtelRunner + assert
- [x] 6.7 `spec_values.rs` — 改用 SpecOtelRunner + assert
- [x] 6.8 `spec_variables.rs` — 改用 SpecOtelRunner + assert
- [x] 6.9 `spec_callable.rs` — 改用 SpecOtelRunner + assert
- [x] 6.10 `spec_core_functions_color.rs` — 改用 SpecOtelRunner + assert
- [x] 6.11 `spec_core_functions_list.rs` — 改用 SpecOtelRunner + assert
- [x] 6.12 `spec_core_functions_map.rs` — 改用 SpecOtelRunner + assert
- [x] 6.13 `spec_core_functions_math.rs` — 改用 SpecOtelRunner + assert
- [x] 6.14 `spec_core_functions_meta.rs` — 改用 SpecOtelRunner + assert
- [x] 6.15 `spec_core_functions_string.rs` — 改用 SpecOtelRunner + assert
- [x] 6.16 `spec_core_functions_selector.rs` — 改用 SpecOtelRunner + assert
- [x] 6.17 `spec_core_functions_misc.rs` — 改用 SpecOtelRunner + assert

## 7. spec_baseline.rs（全量基线）

- [x] 7.1 创建 `tests/spec_baseline.rs`，定义 17 个域的路径映射
- [x] 7.2 实现 `#[test] #[ignore] fn test_baseline_all()` — 遍历全部 HRX
- [x] 7.3 `init_otel("spec_baseline")` + `init_metrics("spec_baseline")`
- [x] 7.4 RecordOnly 模式跑完，不 assert
- [x] 7.5 `shutdown_otel()` + `shutdown_metrics()` flush
- [x] 7.6 实现 `write_baseline_json(stats)` — 产出 `spec_baseline_{timestamp}.json`
- [x] 7.7 添加 tracing span（`spec_baseline`, stage=`spec_test`, 字段 `total_hrx`/`total_cases`/`passed`/`failed`/`pass_rate`）

## 8. spec_diff.rs（rust-script baseline diff）

- [x] 8.1 创建 `scripts/spec_diff.rs`（rust-script 格式）
- [x] 8.2 接收两个 baseline JSON 文件路径参数
- [x] 8.3 解析 JSON，对比 per-domain 通过率
- [x] 8.4 输出：新增通过 / 新增失败 / 回归 / 新增跳过
- [x] 8.5 输出 Markdown 格式报告

## 9. 验证与基线

- [x] 9.1 `cargo test --test spec_plain -- --nocapture` 验证 OTel trace + metrics 输出
- [x] 9.2 `cargo test --test spec_css -- --nocapture` 验证多文件测试不再 SKIPPED
- [x] 9.3 `cargo test` 全量回归确认无 panic（失败用例用 error! 不 panic）
- [x] 9.4 `cargo test --test spec_baseline -- --nocapture --ignored` 首次全量基线
- [x] 9.5 检查产出物：`otel-metrics-spec_*.jsonl`、`otel-trace-spec_*.jsonl`、`spec_baseline_*.json`
- [x] 9.6 `rust-script scripts/spec_diff.rs --old spec_baseline_old.json --new spec_baseline_new.json` 验证 diff 工具

## 10. 独立 spec 数据集 + 对照工具

- [x] 10.1 创建 `scripts/gen_spec_dataset.rs`（rust-script 格式）— 从 sass-spec HRX 提取纯数据集 JSON
- [x] 10.2 数据集格式：`id`/`domain`/`hrx_file`/`case_name`/`files[]`/`entry`/`expected_output`/`expected_error`/`options`/`is_multi_file`
- [x] 10.3 生成全量数据集 `spec_dataset.json`（20504 cases, 16MB, 2177 HRX files）
- [x] 10.4 创建 `scripts/spec_check.rs` — 独立对照工具，给定编译器命令 + 数据集，用 tracing span 记录证据链
- [x] 10.5 对照工具使用 `catch_unwind` 兜住编译器 panic，产出 `spec_check_*.json` 报告
- [x] 10.6 创建 `src/main.rs` — sasspile CLI 入口（compile_file）
- [x] 10.7 用 sasspile CLI 全量对照验证（2346/20504 passed, 11.44%）
- [x] 10.8 更新 README.md 添加 spec 数据集 + 对照工具章节
