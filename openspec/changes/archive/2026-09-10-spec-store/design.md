## Context

sasspile 当前的 sass-spec 测试数据散落在三个位置：运行日志（`/tmp/sass-spec-full.log`）、失败 JSON（`sass-spec-failures.json`）、基线 JSON（`sass-spec-baseline.json`）。数据无时间维度，无法做趋势分析，且解析日志的方案格式脆弱。

现有工具链：
- `failures_json.rs` — 直接编译 → 写 JSON（覆盖式，无历史）
- `sass_spec_stats.rs` — 解析日志文本 → 写 Markdown + 基线 JSON
- 其他工具（`diag_helper.rs`/`sass_spec_full.rs`）不依赖 JSON，只依赖 HRX 解析

## Goals / Non-Goals

**Goals:**
- 统一 spec 数据结构：HRX 解析 + 编译执行 + 结果存储 单一入口
- SQLite 存储取代 JSON 出口
- 历史快照能力：每次 commit 自动记录 pass/fail 变化
- 函数级趋势查询 + 回归定位
- CodeGraph 调用链 ↔ spec case 桥接

**Non-Goals:**
- 不改 HRX 解析逻辑（复用 `hrx_support` 模块）
- 不改 CodeGraph 内部（作为外部 CLI 调用）
- 不实现 Web UI（CLI + SQLite 查询足够）
- 不实现实时追踪（运行时插桩），仅在 commit 时 snapshot

## Decisions

### Decision 1: SQLite vs JSON

**Chosen:** SQLite (rusqlite + WAL mode)

**Rationale:**
- JSON 是覆盖式快照，SQLite 支持时间序列查询
- WAL mode 读写并发好（编译写结果 + 查询读历史不冲突）
- 结构化查询：`SELECT function, COUNT(*) WHERE status='FAIL'` 直接出结果
- 存储效率：case_deltas 只记变化，95% 不变的不记

**Alternatives:**
- PostgreSQL：过重，不适合本地开发
- 纯 JSON 目录树（每 commit 一个文件）：无可查询性

### Decision 2: 函数名映射策略

**Chosen:** L1 路径映射（目录结构即映射）+ L2 内容解析（@use + fn.call 扫描）

**Rationale:**
- `core_functions/math/sin.hrx` → 函数名 `math.sin` 从路径直接提取
- 输入文件扫描 `@use "sass:math"` + `math.sin(` 确认精确调用
- 无需运行时插桩（L3），80% 场景 L1+L2 足够

**Alternatives:**
- 运行时追踪：精确但需要插桩编译器，复杂度高
- 纯正则全覆盖：误报率高，维护成本大

### Decision 3: Snapshot 触发策略

**Chosen:** 三种模式并行 — git hook + 手动 + CI 定时

**Rationale:**
- commit hook：本地开发自动记录，不丢历史
- 手动 `spec_store snapshot`：修复后立即记录
- CI 定时：兜底，防止 commit hook 被跳过

### Decision 4: 模块结构

**Chosen:** `tests/spec-store/` 单模块，内部按职责分文件

**文件拆分（确保 ≤500 行）：**
```
tests/spec-store/
├── mod.rs            # 入口 + CLI 分派
├── db.rs             # SQLite schema + 连接
├── hrx_loader.rs     # HRX 解析 → case 存入 DB
├── runner.rs         # 编译 case + 记录结果
├── snapshot.rs       # snapshot 创建 + delta 计算
├── stats.rs          # 目录统计报告
├── trend.rs          # 趋势查询
├── link.rs           # CodeGraph 桥接
└── bisect.rs         # 回归定位
```

## Risks / Trade-offs

| Risk | Mitigation |
|------|-----------|
| SQLite WAL 文件膨胀 | 定期 `PRAGMA wal_checkpoint(TRUNCATE)` |
| commit hook 失败阻塞 git | hook 设为 `|| true`，失败只 warn |
| HRX 格式边缘情况 | 复用已有 `hrx_support` 模块，不重写 |
| 首次运行编译耗时 (~87s) | 与 failures_json 同量级，可接受 |
| rusqlite 编译时间 | Cargo.toml optional feature，不影响主构建 |
