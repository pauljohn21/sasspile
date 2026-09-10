# sass-spec 失败统计

> **此文档已废弃**，由 `spec_store` 工具取代。

使用 spec-store 获取实时失败统计：

```bash
# 全量运行 + 快照（~125 秒）
SPEC_STORE_CMD=run cargo test --test spec_store -- --nocapture

# 目录聚合报告
SPEC_STORE_CMD=stats cargo test --test spec_store -- --nocapture

# 两 snapshot 对比（回归检测）
SPEC_STORE_CMD=diff FROM=1 TO=2 cargo test --test spec_store -- --nocapture
```

数据持久化在 `tests/spec-store.db`（SQLite WAL 模式）。
