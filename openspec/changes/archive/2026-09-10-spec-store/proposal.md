## Why

sasspile 的 sass-spec 测试数据分散在两个独立工具中：`failures_json.rs`（case 级失败导出为 JSON）和 `sass_spec_stats.rs`（目录统计 + 基线对比）。两者均依赖外部 JSON 文件作为存储，无法回答"这个 case 何时开始失败"、"哪个 commit 修了 math.sin 的 3 个 case"这类历史问题。需要一个统一的 spec_store 工具，用 SQLite 结构化存储 sass-case 数据 + 历史快照，消灭 JSON 出口，并提供 trend/bisect/link 查询能力。

## What Changes

- 新建 `tests/spec-store/` 模块（Rust test binary），提供 `spec_store` CLI 工具
- 解析 sass-spec HRX 文件 → SQLite 存储（cases + function 索引）
- 运行时编译 → 记录 case 结果 → 自动/手动创建 snapshot
- `stats` 命令替代 `sass_spec_stats.rs`（Markdown 报告 + 基线对比）
- `trend` 命令提供函数级历史趋势
- `link` 命令桥接 CodeGraph 调用链 ↔ spec cases
- `bisect` 命令提供回归定位
- **删除** `tests/failures_json.rs`、`tests/sass_spec_stats.rs`
- **删除** `tests/sass-spec-failures.json`、`tests/sass-spec-baseline.json`、`tests/sass-spec-stats.md`
- 更新 `AGENTS.md` 工作流描述

## Capabilities

### New Capabilities

- `spec-store-core`: HRX 解析、SQLite 存储、case 编译、snapshot 创建
- `spec-store-stats`: 目录统计报告（Markdown/JSON 输出）、基线对比
- `spec-store-trend`: 函数级历史趋势查询、ASCII 图表
- `spec-store-link`: CodeGraph 调用链 ↔ spec cases 桥接
- `spec-store-bisect`: commit 间回归定位

### Modified Capabilities

- 无（纯新增能力，不修改现有 spec 要求）

## Impact

- **代码影响**: 删除 2 个文件，新建 1 个模块；其余 `diag_helper.rs`/`sass_spec_full.rs` 等间接引用 HRX 的工具不受影响（它们用 `hrx_support` 模块，不依赖 JSON）
- **存储影响**: 新增 `tests/spec-store.db`（SQLite，预估 <50MB/年）
- **依赖新增**: `rusqlite` crate（已有 `serde`/`serde_json` 可保留给 stats JSON 输出）
- **工作流影响**: 验证清单中的命令需从 `cargo test --test failures_json` 更新为 `spec_store run`
- **文档影响**: `AGENTS.md` 中 failures_json + sass_spec_stats 描述需更新
