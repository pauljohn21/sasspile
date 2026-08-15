# Decision 003: 模块系统使用有向图 + 拓扑排序

## Status
Accepted

## Context
@use/@forward 形成复杂依赖关系，需支持循环检测和增量编译。

## Decision
构建依赖有向图，Kahn 算法拓扑排序确定编译顺序。

## Alternatives Considered
- 简单递归 → 无法检测循环，可能栈溢出
- DFS 递归 → 难以并行

## Consequences
- ✅ 支持循环依赖检测
- ✅ 入度为 0 的模块可并行编译
- ✅ 增量编译时只重新编译变更模块

## Related
- semantic.md
- tasks.md Phase 4
