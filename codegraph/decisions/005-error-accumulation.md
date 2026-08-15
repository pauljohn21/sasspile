# Decision 005: 错误不中断管道，沿途收集

## Status
Accepted

## Context
用户希望一次看到所有错误，而非逐个修复。

## Decision
错误不中断管道，收集所有错误后统一报告。每个 AST 节点附带 `SourceLocation`。

## Consequences
- ✅ 用户一次看到所有错误
- ✅ 源码位置对诊断至关重要
- ⚠️ 需要设计良好的错误类型系统

## Related
- diagnostics.md
- parser.md
- tasks.md Phase 1, 3
