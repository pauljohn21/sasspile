# sass-spec 全量统计报告

> **此文档已废弃**，由 `spec_store` 工具取代。

使用 spec-store 获取实时统计数据：

```bash
# HRX 入库（一次性，12,131 个 case）
SPEC_STORE_CMD=index cargo test --test spec_store -- --nocapture

# 全量运行 + 快照
SPEC_STORE_CMD=run cargo test --test spec_store -- --nocapture

# Markdown 统计报告（按目录排序）
SPEC_STORE_CMD=stats cargo test --test spec_store -- --nocapture

# 函数级趋势查询
SPEC_STORE_CMD=trend FN=math.sin cargo test --test spec_store -- --nocapture

# 代码↔spec 桥接（CodeGraph）
SPEC_STORE_CMD=link FN=math.sin cargo test --test spec_store -- --nocapture
```

数据持久化在 `tests/spec-store.db`（SQLite WAL 模式）。
