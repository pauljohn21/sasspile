## Why

`@forward "url" as prefix-*` 转发的模块中，命名空间变量赋值（`namespace.$prefix-var: value`）未正确转发到 upstream 模块的原始变量。这是 `directives/forward/member/as` 失败的主因（2 ERR + 4 DIFF = 6 fails）。

## What Changes

- `midstream.$d-a: new value;` 正确赋值到 `@forward "upstream" as d-*` 转发的 `$a`
- 嵌套上下文中（CSS rule 内）的命名空间赋值同样正确传播
- `@forward` + `@import` 混合作用时变量值正确传播

## Capabilities

### New Capabilities

- `forward-prefixed-variable-assignment`: `namespace.$prefix-var` 正确转发到 upstream 模块
- `forward-variable-shadow`: 多层 forward 链中变量阴影正确工作

### Modified Capabilities

- 无

## Impact

- 受影响源文件：`src/eval/forward.rs`、`src/eval/value/mod.rs`、`src/eval/env_impl.rs`
- 受影响测试：`directives/forward/member/as/*`（约 4 cases）、`directives/forward/member/shadowed/*`（约 2 cases）
- API 兼容性：无 breaking change
