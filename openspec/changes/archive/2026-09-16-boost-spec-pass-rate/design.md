## Context

sass-spec 通过率 0.4%（37/10274），经 trace 分析发现三类根因：

```
mixin expanding params=["$b", "$c"] args=["1 2"]
→ 输出 body: [Declaration { prop: "d", value: "$c" }]
   ^^^ $c 应该是 "2"，未替换
```

```
MixinCall { name: "a", args: ["1 2"] }
                     ^^^^^ 应为 ["1", "2"]，html_arg 未在 parse 层正确 split
```

core_functions 子目录 7793 case 大多因 builtin 返回空字符串失败 — 框架已注册 50+ 函数但实现体有空漏。

**现状**：
- `evaluate_node_with_locals` 通过临时修改 `global_variables` HashMap 实现参数注入 → 线程不安全、同全局污染
- `split_args` 已存在但 parse 层收集 MixinCall args 时未调用它
- `builtins.rs` 已注册 map/list/string/color 函数映射，部分函数返回空/未处理边界

**约束**：
- 单文件 ≤ 500 行
- 不破坏 Bootstrap/Element Plus 编译（tracker enterprise 通过）
- tracing span 约定保持不变

## Goals / Non-Goals

**Goals:**
- Mixin 参数正确替换（独立局部作用域 → 优先查找）
- MixinCall args 多参数正确拆分
- darken/lighten/mix/rgba/hsl/hsla/invert/grayscale 正确输出
- list/string/map builtin 边界正确处理
- sass-spec 通过率从 0.4% 提升到 ≥10%

**Non-Goals:**
- 不做完整的 SCSS 模块系统（@use namespace、@forward）
- 不做 @at-root / @content 等内容注入指令
- 不做 sourcemap 生成
- 不做颜色通道的 ICC profile 精确计算

## Decisions

### Decision 1: Mixin 局部变量作用 — 引入 `local_scope` 栈

**当前问题**: `evaluate_node_with_locals` 通过临时 swap `global_variables` 实现。这导致：
1. 嵌套 mixin 互相污染
2. "`$c`" 如果全局恰好有同名变量会被错误覆盖
3. 恢复逻辑依赖 borrow 展开点时序

**选择**: 在 `CompilerContext` 增加 `local_vars: Vec<HashMap<String, String>>`（作用域栈）。`substitute_vars` 先查栈顶→栈底→`global_variables`。

```
原: ctx.global_variables.borrow() → 单源查找
新: for scope in stack.iter().rev() { if let Some(v) = scope.get(ident) return v }
    → fallback: global_variables
```

**替代方案 A**: 传参携带 `&[(String,String)]` 链到每个 evaluate_node — 需要改 20+ 签名，拒绝。
**替代方案 B**: Copy-on-write snapshot + rollback — 与当前实现类似但有 borrow 问题，拒绝。
**替代方案 C (selected)**: scope stack，最小改动，仅影响 `substitute_vars` 和 `evaluate_node_with_locals` 入口。

### Decision 2: MixinCall args 参数化 — 在 parse 层完成 split

**当前问题**: parse 层直接将 `"1 2"` 存入 args[0]，等 evaluate 阶段再拆分 → 嵌套函数调用内的空格/括号边界无法正确处理。

**选择**: 在 `parse_dst/directives.rs` 解析 `@include name(arglist)` 时，对 arglist 调用与 `evaluate/mod.rs::split_args` 同算法的 split，生成正确的 args Vec。不拆分为独立函数（DRY 考量）：evaluate 阶段 args 已经是 `Vec<String>`，不需要再 split。

### Decision 3: Color 函数输出格式 — 委托给 rgb_to_string/已有渠道

**选择**: 新增 `rgba_to_string(r,g,b,a)` 和 `hsla_to_string(h,s,l,a)`，保持 `#rrggbb` 与现有 `rgb_to_string` 一致。不引入 CSS Color Module Level 4 语法。

### Decision 4: 文件组织

- `builtins.rs` 若超 500行则拆分为 `color.rs`（darken/lighten/mix/hsl/rgba/invert/grayscale/alpha 等）+ `builtin_core.rs`（map/list/string/math 已存在部分移至 core）

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| scope stack 与现有 variable 恢复逻辑冲突 | `evaluate_node_with_locals` 直接 push/pop scope，不再操作 global_variables |
| parse 层 split 影响现有 passing 的 mixin 用例 | tap trace 对比 split 前后 output |
| Bootstrap/EP 编译退化 | 每次改完跑 `cargo test --test e2e_api` + tracker enterprise |

## Migration Plan

**顺序**: 
1. 先修 parse 层 args split → 原本 args=["1 2"] 的 case 变为 args=["1","2"]
2. 再修 evaluate 层 scope stack → mixin 参数正确注入
3. 最后补全 color builtin

每步独立可提交、独立可回滚 — 任一步失败不阻塞其余。
