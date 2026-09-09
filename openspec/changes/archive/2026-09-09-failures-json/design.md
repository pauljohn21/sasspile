## Context

sasspile 现有 sass-spec 诊断工具（`cfs_diag.rs`、`css_diag.rs`、`expr_diag.rs`）通过 `tracing::error` 输出失败信息到 stdout。4513 个失败散落在 ~1000 行日志中，无法结构化查询。需要新建独立工具将失败原因输出为 JSON 文件，支持编程分析和回归检测。

## Goals / Non-Goals

**Goals:**
- 创建 `tests/failures_json.rs`，评估全部 sass-spec case，输出结构化 JSON
- 每条失败记录包含完整的 expected/actual/error 全文
- 支持按目录和失败类型聚合统计
- 复用 `hrx_support` 基础设施，保持与 `sass_spec_full.rs` 行为一致

**Non-Goals:**
- 不实现失败修复逻辑（只诊断）
- 不修改现有 `sass_spec_full.rs` 或诊断工具
- 不实现增量/差异模式（每次全量输出）
- 不优化编译速度（与 sass_spec_full 相同 ~4 分钟）

## Decisions

### Decision 1: 新建独立 test 文件

**选择**: 新建 `tests/failures_json.rs`，不扩展 `sass_spec_full.rs`
**理由**: 
- `sass_spec_full.rs` 关注统计聚合，运行方式不同（info span 日志）
- 独立文件避免耦合，可独立运行不影响统计流程
- 如果扩展 sass_spec_full 需要引入环境变量切换模式，增加复杂度

### Decision 2: JSON 结构

**选择**: 四顶层字段 — `metadata`、`failures`、`by_dir`、`by_type`
**理由**: 
- `metadata` 概览总数，便于快速判断是否有回归
- `failures` 数组保留每条详情，是核心价值
- `by_dir` / `by_type` 预计算聚合，消费方无需再遍历
- 不嵌套分组（按目录分组再列 failures），保持扁平结构易解析

### Decision 3: 全文存储 expected/actual

**选择**: 存储完整 expected CSS 和 actual 输出，不做截断
**理由**: 全量 JSON（4513 × ~2KB ≈ 9MB）在现代系统中可接受，调试时无需重跑工具获取全文。

### Decision 4: 复用 parse_hrx_to_cases 编译模式

**选择**: 直接调用 `hrx_support::parse_hrx_to_cases` + 自写编译逻辑（不调用 `run_case`）
**理由**: 
- 需要获取编译结果全文（Ok(String) 或 Err(String)），而 `run_case` 只返回 bool
- 编译逻辑（写临时目录→compile→读结果）与 run_case 几乎相同，但需要完整输出
- 直接复制 ~20 行编译逻辑比修改 run_case 签名影响更小

## Risks / Trade-offs

- **[JSON 文件大小]** → 全文存储可能导致 6-10 MB JSON。Mitigation: 消费方可按需 grep 特定目录，git 可压缩
- **[运行耗时 ~4 分钟]** → 与 sass_spec_full 相同，无额外开销
- **[临时目录清理]** → 编译异常时可能残留临时文件。Mitigation: 测试结束后标记 tmp_dir，但当前 run_case 模式已是 `let _ = remove_dir_all`，无泄漏风险
