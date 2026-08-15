# Decision 007: CSS 生成使用中间 AST + Formatter

## Status
Accepted

## Context
需要支持多种输出格式（expanded/compressed/compact）。

## Decision
先产生中间 CSS AST，再通过 Formatter 产生最终文本。

## Alternatives Considered
- 直接拼字符串 → 无法支持多种输出格式
- Display trait → 过于耦合

## Consequences
- ✅ 支持 expanded / compressed / compact 等多种格式
- ✅ CSS AST 可做后续优化（去重、合并）

## Related
- css-gen.md
- tasks.md Phase 7
