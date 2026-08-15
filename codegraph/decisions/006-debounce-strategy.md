# Decision 006: 文件防抖使用事件合并 + 可配置静默窗口

## Status
Accepted

## Context
编辑器保存通常触发 1-3 个事件（内容变更、元数据变更），需要避免单次保存触发多次无效编译。

## Decision
fsnotify 事件进入 mpsc channel，debouncer 在每轮静默期后批量 flush。

## Key Parameters
- `debounce_ms: u64` — 默认 200ms
- `CompileMode::Watch | CompileMode::Ci` — CI 模式下禁用防抖

## Consequences
- ✅ 避免单次保存触发多次无效编译
- ✅ 批量处理复用已编译模块缓存

## Related
- incremental.md
- tasks.md Phase 8
