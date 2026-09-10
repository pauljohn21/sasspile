## 1. 基础设施

- [x] 1.1 创建 `tests/spec-store/` 目录结构（mod.rs + 8 个子模块文件）
- [x] 1.2 在 Cargo.toml 添加 `rusqlite` 依赖（features = ["bundled"]）
- [x] 1.3 实现 `db.rs`：SQLite schema 创建 + 迁移逻辑（spec_cases / case_files / snapshots / case_results / case_deltas 五表 + 索引）
- [x] 1.4 实现 `mod.rs`：CLI 参数解析 + 子命令分派（run / stats / trend / link / bisect / diff / snapshot / index）

## 2. spec-store-core（核心能力）

- [x] 2.1 实现 `hrx_loader.rs`：扫描 HRX，提取函数名，写入 `spec_cases` + `case_files` 表
- [x] 2.2 实现 `runner.rs`：编译 case（调用 `sasspile::compile_file_with_load_paths`），捕获 PASS/FAIL/SKIP + failure_type + actual/error
- [x] 2.3 实现 `snapshot.rs`：创建 snapshot（记录 commit_sha + timestamp），计算 case_deltas
- [x] 2.4 实现 `run` 子命令：全量运行 + 自动创建快照
- [x] 2.5 函数名提取逻辑：路径解析（`core_functions/math/sin.hrx` → `math.sin`）

## 3. spec-store-stats（统计报告）

- [x] 3.1 实现 `stats.rs`：目录聚合查询（pass/fail/skip/total/pct），排序输出
- [x] 3.2 `--md` 格式输出：生成 Markdown 表格报告
- [ ] 3.3 `--json` 标志：生成 JSON 结构化输出（后续迭代）
- [ ] 3.4 `--compare <snapshot_id>` 基线对比模式（后续迭代）

## 4. spec-store-trend（历史趋势）

- [x] 4.1 实现 `trend.rs`：函数级时间序列查询（snapshot.pct 按时间）
- [x] 4.2 ASCII 折线图输出
- [x] 4.3 `--dir` 标志：目录级聚合趋势
- [ ] 4.4 `--since` / `--from` / `--to` 时间范围过滤（后续迭代）

## 5. spec-store-link（CodeGraph 桥接）

- [x] 5.1 实现 `link.rs`：调用 `codegraph callers <fn>` 获取调用链
- [x] 5.2 `--function` 模式：输出函数 → spec cases + CodeGraph callers
- [ ] 5.3 `--case` 模式：输出 case → 涉及的 CodeGraph 符号（后续迭代）
- [ ] 5.4 `--impact` 标志（后续迭代）

## 6. spec-store-bisect（回归定位）

- [x] 6.1 实现 `bisect.rs`：函数 pass/fail 历史查询
- [x] 6.2 `diff <id1> <id2>` 子命令：输出两个 snapshot 间的 case 状态变化
- [ ] 6.3 `--summary` 标志（后续迭代，当前已有 format_diff）

## 7. Git Hook + 清理

- [x] 7.1 创建 `.githooks/post-commit` hook：每次 commit 后自动 snapshot
- [x] 7.2 删除 `tests/failures_json.rs`
- [x] 7.3 删除 `tests/sass_spec_stats.rs`
- [x] 7.4 删除 `tests/sass-spec-failures.json`、`tests/sass-spec-baseline.json`、`tests/sass-spec-stats.md`
- [x] 7.5 更新 `AGENTS.md`：替换 failures_json/sass_spec_stats 工作流为 spec_store 命令

## 8. 验证

- [x] 8.1 运行 `cargo test --test compile_test` 确认零回归（48/48 ✓）
- [x] 8.2 运行 `SPEC_STORE_CMD=index` 确认 HRX 入库（12,131 cases ✓）
- [x] 8.3 运行 `SPEC_STORE_CMD=run` 确认全量运行 + 快照创建（7,489 PASS / 4,380 FAIL / 262 SKIP ✓）
- [x] 8.4 运行 `SPEC_STORE_CMD=stats` 确认 Markdown 报告输出
- [x] 8.5 运行 `SPEC_STORE_CMD=trend FN=math.sin` 确认 ASCII 折线图
- [x] 8.6 CodeGraph 桥接查询可用（依赖 codegraph CLI）
- [x] 8.7 运行 `SPEC_STORE_CMD=diff FROM=1 TO=2` 确认 delta 查询
