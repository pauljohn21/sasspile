## Context

sasspile v0.9.7 当前 sass-spec 通过率 2822/5362 = 52%。上一个 spec-pass-rate-boost 变更已完成参数验证修复、meta 模块功能、error 检测和 values/css 深度修复。本轮聚焦通过全量扫描发现的高频失败模式。

当前失败分布：
- `values/` 768 失败（34% 通过率）——最大失败池
- `css/` 482 失败（39%）——plain CSS 错误检测不完整
- `core_functions/` 923 失败（63%）——命名空间映射缺失+参数验证
- `directives/` 268 失败（55%）——@forward 冲突+@use 深层问题

## Goals / Non-Goals

**Goals:**
- 补全命名空间函数映射，消除所有 `Undefined function: <ns>.<name>` 错误
- 完善 plain CSS 模式错误检测，覆盖 at-rule/interpolation/operators/sass 变量
- 实现 @forward 冲突检测（variable/function/mixin 同名报错）
- 增强 math 函数参数验证（clamp/min/max/hypot/pow/log）
- 实现 `color.ie-hex-str` 函数
- 实现 `meta.load-css` mixin 和 `meta.apply` mixin 基础版
- 提升 sass-spec 通过率至 62-65%（+500~800 用例）

**Non-Goals:**
- 颜色系统改进（已跳过，需 `--ignored` 手动触发）
- values/ 目录的序列化格式深度修复（涉及面太广，留待下一轮）
- @use/@forward 模块加载的深层语义修复（如 `@forward ... as` 前缀转换）
- Dart Sass 兼容性修复（禁止参考 Dart Sass 实现）

## Decisions

### D1: 命名空间映射补全策略——直接映射表扩展

在 `module_dispatch.rs` 的 `module_builtin_name()` 函数中直接添加缺失映射行。

**替代方案**：动态查找（去除前缀后查找内建函数）——被否决，因为某些函数名带前缀后与内建名不同（如 `string.str-length` → `str-length`，不是 `str-length` 去前缀）。

### D2: plain CSS 错误检测——在 eval 阶段拦截

在 `eval/mod.rs` 的 `eval_node` 入口处，当 `env.plain_css == true` 时检查节点类型，对不允许的 at-rule/interpolation/operators 报错。

**替代方案**：在 parser 阶段拦截——被否决，因为 parser 不知道 plain CSS 模式（模式在 eval 阶段才确定）。

### D3: @forward 冲突检测——在模块合并 exports 时检查

在 `eval/module.rs` 的 `load_module` 或 `eval_forward` 中，合并多个 @forward 的 exports 时检测同名成员。

**替代方案**：在 `bind_exports` 阶段检测——被否决，因为 bind_exports 时已丢失来源信息，无法区分"同一模块两次 forward"和"两个不同模块 forward 同名成员"。

### D4: meta.load-css mixin 实现——eval 阶段动态加载

在 `eval/mixin.rs` 的 mixin 调用路径中拦截 `meta.load-css`，动态调用 `eval_use` 加载模块并输出 CSS。

**替代方案**：注册为真正的 mixin——被否决，因为 load-css 需要动态参数（模块名从变量来），普通 mixin 不支持。

### D5: math 参数验证——统一 helper 函数

在 `builtin/math.rs` 中提取 `require_number`、`require_int`、`require_unitless` 等 helper，统一参数验证逻辑。

## Risks / Trade-offs

- **[命名空间映射遗漏]** → 逐目录运行 sass-spec 验证，确保无回归
- **[plain CSS 错误误报]** → 只对明确不允许的语法报错，保守策略
- **[@forward 冲突检测过度]** → 仅检测同名不同来源的冲突，同值允许
- **[meta.load-css 副作用]** → 动态加载可能引入循环依赖，需检查 loaded_modules
- **[单文件行数限制]** → math.rs 可能超 500 行，需拆分验证
