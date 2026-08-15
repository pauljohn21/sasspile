# Decision 004: 内置模块注册使用 async trait + HashMap

## Status
Accepted

## Context
内置函数需要可被异步调用，同时支持用户自定义函数注册。

## Decision
内置函数注册为 `HashMap<&str, Box<dyn SassFn>>`，trait 为 `async fn call(args: &[Value], env: &Env) -> Result<Value>`。

## Alternatives Considered
- 同步函数 → 不支持 IO 密集操作
- 宏生成 → 过度复杂

## Consequences
- ✅ 允许用户自定义函数
- ✅ async 支持 IO 密集操作
- ✅ 函数名查找 O(1)

## Related
- builtin-modules.md
- tasks.md Phase 6
