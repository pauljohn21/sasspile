## 1. 骨架搭建

- [x] 1.1 创建 `tests/failures_json.rs` 文件，声明 `mod hrx_support` 和 `mod spec_manifest`，引入必要的依赖（tracing, serde, std::path, std::fs）
- [x] 1.2 实现 `collect_hrx_files_with_manifest` 函数（复用 `sass_spec_full.rs` 的文件收集逻辑）
- [x] 1.3 实现 `compile_case` 函数：创建临时目录→写入文件→调用 `compile_file_with_load_paths`→返回 `Result<String, String>`

## 2. 数据结构

- [x] 2.1 定义 `FailureRecord` 结构体（id, dir, type, expected, actual, error）并 derive Serialize
- [x] 2.2 定义 `by_dir_value` 结构体（DIFF/ERR/ERR_EXP_OK 计数）
- [x] 2.3 定义顶层 `FailuresOutput` 结构体（metadata, failures, by_dir, by_type）

## 3. 主逻辑实现

- [x] 3.1 实现 `run_failures_collection` 函数：遍历所有目录→收集 HRX→解析 cases→编译→分类失败→记录全文
- [x] 3.2 跳过逻辑：跳过 `SKIP_DIRS` 目录、跳过 `.sass` case、跳过无 expected_output 且非 expect_error 的 case
- [x] 3.3 进度日志：每个目录处理完后输出 `tracing::info!(dir, pass, fail, "proc")`

## 4. JSON 序列化与写入

- [x] 4.1 聚合 by_dir 统计（遍历 failures，按 dir 分组累加各类型计数）
- [x] 4.2 聚合 by_type 统计（DIFF/ERR/ERR_EXP_OK 总计数）
- [x] 4.3 JSON 写入：`serde_json::to_string_pretty` → `fs::write("tests/sass-spec-failures.json")`

## 5. 测试入口与验证

- [x] 5.1 添加 `#[test] fn export_failures_json()` 调用 `run_failures_collection` + 写入 JSON
- [x] 5.2 运行 `cargo test --test failures_json -- --nocapture`，确认测试通过
- [x] 5.3 手动检查 `tests/sass-spec-failures.json`：JSON 结构正确、包含 failures 数组、by_dir/by_type 聚合合理
- [x] 5.4 验证核心测试不受影响：`cargo test --test compile_test` 通过
