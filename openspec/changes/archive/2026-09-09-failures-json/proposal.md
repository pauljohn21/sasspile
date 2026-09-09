## Why

当前 sass-spec 失败诊断依赖文本日志（tracing::error），4513 个失败散落在 stdout 中，无法结构化查询、统计和持久化对比。每次修复后需要手动对比日志来确认哪些 case 被修复、哪些新增，效率极低且易遗漏。需要一个工具将所有失败原因输出为结构化 JSON，使失败可编程查询、支持回归检测和修复优先级排序。

## What Changes

- 新建 `tests/failures_json.rs` 测试文件，运行全部 sass-spec case，将每个失败的完整 expected/actual/error 写入 JSON
- 输出文件：`tests/sass-spec-failures.json`
- JSON 包含：metadata（时间戳/passed/failed/total）、failures 数组（每条含完整 expected/actual）、by_dir 聚合、by_type 聚合
- 复用 `hrx_support` 的 `parse_hrx_to_cases` + 编译逻辑（与 `sass_spec_full.rs` 一致）
- 运行方式：`cargo test --test failures_json -- --nocapture`

## Capabilities

### New Capabilities

- `failures-json-export`: 运行全部 sass-spec case，将失败原因（expected/actual/error）以结构化 JSON 输出到文件，支持按目录/类型聚合统计

### Modified Capabilities

（无）

## Impact

- **新增文件**: `tests/failures_json.rs`（~150 行）
- **新增输出**: `tests/sass-spec-failures.json`（~6-10 MB）
- **依赖**: 复用已有 `hrx_support` 模块、`spec_manifest::SKIP_DIRS`、`sasspile::compile_file_with_load_paths`
- **不影响**: 生产代码、现有诊断工具、sass_spec_full 统计流程
- **耗时**: 约 4 分钟（与 sass_spec_full 相同，每个 case 只编译一次）
