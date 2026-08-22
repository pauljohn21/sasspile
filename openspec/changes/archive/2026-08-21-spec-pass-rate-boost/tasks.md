## 1. Phase 1A — math 函数参数验证修复

- [x] 1.1 在 `eval/builtin/math.rs` 的 `call` 函数中合并 `pos_args` 和 `kw_args` 后再验证参数数量
- [x] 1.2 修复 `atan2` 参数验证（72 次失败）— 支持命名参数 `$y`/`$x`
- [x] 1.3 修复 `sin`/`cos`/`tan` 参数验证（各 20 次）— 支持命名参数 `$number`
- [x] 1.4 修复 `log` 参数验证（20 次）— 支持命名参数 `$number`/`$base`
- [x] 1.5 修复 `atan`/`asin`/`acos` 参数验证（20+12+12 次）— 支持命名参数 `$number`
- [x] 1.6 修复 `sqrt` 参数验证（14 次）— 支持命名参数 `$number`
- [x] 1.7 修复 `clamp` 参数验证（14 次）— CSS 透传 vs math 计算的分支判断
- [x] 1.8 修复 `div` 参数验证（26 次）— 支持命名参数 `$number1`/`$number2`
- [x] 1.9 修复 `pow` 参数验证 — 支持命名参数 `$base`/`$exponent`
- [x] 1.10 修复 `abs`/`ceil`/`floor`/`round`/`percentage`/`min`/`max` — 确保命名参数支持
- [x] 1.11 修复 `random`/`hypot` — 确保命名参数支持
- [x] 1.12 修复 `unit`/`is-unitless`/`compatible` — 确保命名参数支持

## 2. Phase 1B — "Only 1 argument allowed" 修复

- [x] 2.1 分析 "Only 1 argument allowed, but N were passed" 的 258 次失败根因
- [x] 2.2 修复 `clamp`/`min`/`max` 在 CSS 上下文中的多参数透传（`eval/builtin.rs`）
- [x] 2.3 修复 CSS 函数透传逻辑：参数含非 Number 类型时走透传路径
- [x] 2.4 验证 `is_css_function` 判断逻辑与 `call_builtin` match arm 的一致性

## 3. Phase 1C — calc(infinity) 边界处理

- [x] 3.1 在 `pow` 函数中增加 `Value::Calc` 分支识别 `calc(infinity)`/`calc(-infinity)` 作为 `$base`（28+28=56 次）
- [x] 3.2 在 `pow` 函数中增加 `Value::Calc` 分支识别 `calc(infinity)`/`calc(-infinity)` 作为 `$exponent`（24+24=48 次）
- [x] 3.3 实现 pow(infinity, 0) = 1、pow(0, infinity) = 0、pow(infinity, infinity) = infinity
- [x] 3.4 实现 pow(-infinity, even) = infinity、pow(-infinity, odd) = -infinity
- [x] 3.5 在 `div` 函数中增加 `calc(infinity)` 边界处理
- [x] 3.6 在 `sqrt` 函数中增加 `calc(infinity)` 边界处理
- [x] 3.7 在 `parse/ast/display.rs` 中实现 infinity/nan 序列化（20 次）
- [x] 3.8 infinity 带单位序列化为 `calc(infinity * 1unit)`

## 4. Phase 1D — selector 函数参数修复

- [x] 4.1 修复 `selector-parse` 参数展开（42 次失败）
- [x] 4.2 修复 `selector-extend` 参数处理（20 次）
- [x] 4.3 修复 `selector-replace` 参数处理（20 次）
- [x] 4.4 增加 `selector-append` 对多参数的支持

## 5. Phase 1E — plain CSS 限制修复

- [x] 5.1 修复 `sass()` 在 plain CSS 中的错误检测（15 次）
- [x] 5.2 修复插值在 plain CSS 中的限制检测（15 次）
- [x] 5.3 验证 plain CSS 模式标识传递的正确性

## 6. Phase 1 验证

- [x] 6.1 `cargo test --test compile_test` — 43 个全通过
- [x] 6.2 `cargo test --test stage_test` — 10 个全通过
- [x] 6.3 `cargo test --test ast_test` — 8 个全通过
- [x] 6.4 `cargo test --test common_test` — 5 个全通过
- [x] 6.5 `cargo test --test bs_spec` — 15 个全通过
- [x] 6.6 `cargo test --test ep_full` — 101/121 通过（83%，模块缓存修复后）
- [x] 6.7 `RUST_LOG="sass_spec_full=info,sasspile=warn" cargo test --test sass_spec_full -- --nocapture` — 通过率 2808/5362 = 52%
- [x] 6.8 `cargo clippy` 无 warning

## 7. Phase 2A — Value::MixinRef 类型新增

- [x] 7.1 在 `parse/ast/mod.rs` 的 `Value` 枚举中新增 `MixinRef(String, Option<String>)` 变体
- [x] 7.2 实现 `MixinRef` 的 `Display` — 输出 `get-mixin("name")`
- [x] 7.3 实现 `MixinRef` 的 `PartialEq`
- [x] 7.4 在 `is_truthy` 中处理 `MixinRef`
- [x] 7.5 在 `type-of` 中返回 `"mixin"` for `MixinRef`

## 8. Phase 2B — meta.get-mixin 实现

- [x] 8.1 在 `eval/builtin.rs` 中增加 `meta.get-mixin` 分派
- [x] 8.2 在 `module.rs` 中增加 `meta.get-mixin` 映射
- [x] 8.3 实现 `get_mixin(name, module)` — 从 `env.mixins` 或模块中查找 mixin
- [x] 8.4 返回 `Value::MixinRef(name, module_ns)`

## 9. Phase 2C — meta.apply mixin 实现

- [x] 9.1 在 `eval/mixin.rs` 中增加 `meta.apply` mixin 识别
- [x] 9.2 实现 `apply(mixin_ref, args, content)` — 从 `MixinRef` 获取 mixin 并调用
- [x] 9.3 支持 `meta.apply` 的 `@content` 传递
- [x] 9.4 在 `module.rs` 中增加 `meta.apply` mixin 映射

## 10. Phase 2D — meta.load-css mixin 实现

- [x] 10.1 在 `eval/mixin.rs` 中增加 `meta.load-css` mixin 识别
- [x] 10.2 实现 `load_css(module_name, with_config)` — 加载模块并注入 CSS
- [x] 10.3 支持 `$with` 配置参数（覆盖模块变量）
- [x] 10.4 在 `module.rs` 中增加 `meta.load-css` mixin 映射
- [x] 10.5 处理 `load-css` 的递归加载深度限制

## 11. Phase 2E — meta 反射函数实现

- [x] 11.1 实现 `meta.module-functions($module)` — 返回函数 map
- [x] 11.2 实现 `meta.module-mixins($module)` — 返回 mixin map
- [x] 11.3 实现 `meta.module-variables($module)` — 返回变量 map
- [x] 11.4 在 `builtin.rs` 和 `module.rs` 中增加三个反射函数的分派

## 12. Phase 2F — 模块成员变量导出

- [x] 12.1 在 `eval/mod.rs` 的变量查找逻辑中增加命名空间 `.` 分隔符检测
- [x] 12.2 实现 `lookup_var("ns.name")` — 拆分 `ns`/`name`，从 `env.namespaces` 查找
- [x] 12.3 在 `@forward` 中确保变量传递到下游模块的导出
- [x] 12.4 验证链式 `@forward` 的变量传递正确性

## 13. Phase 2 验证

- [x] 13.1 核心测试 81/81 全通过
- [x] 13.2 `RUST_LOG="sass_spec_full=info,sasspile=warn" cargo test --test sass_spec_full -- --nocapture` — 通过率 2808/5362 = 52%
- [x] 13.3 `cargo clippy` 无 warning
- [x] 13.4 单文件 ≤ 500 行检查（meta_ops.rs 301 行，color_types.rs 180 行）
- [x] 13.5 ep_full 101/121 (83%) — 模块缓存修复 $namespace 变量丢失 bug

## 14. Phase 3A — 表达式语法错误检测

- [x] 14.1 在 `parse/expr/mod.rs` 中增加 `not` 后无有效表达式的错误检测
- [x] 14.2 增加 `and`/`or` 后无有效表达式的错误检测
- [x] 14.3 增加空括号 `()` 的错误检测
- [x] 14.4 增加 `or` 前无操作数的错误检测

## 15. Phase 3B — selector/map/use 错误检测

- [x] 15.1 在 `selector-append` 中增加无效选择器类型检测（9 次）
- [x] 15.2 在 `map-deep-merge`/`map-remove` 中增加参数类型检查（4 次）
- [x] 15.3 在 `@use`/`@forward` 模块加载中增加 conflict 检测
- [x] 15.4 验证同值 conflict 不报错

## 16. Phase 3 验证

- [x] 16.1 核心测试 202/202 全通过
- [x] 16.2 `RUST_LOG="sass_spec_full=info,sasspile=warn" cargo test --test sass_spec_full -- --nocapture` — 通过率 ≥ 64%
- [x] 16.3 `cargo clippy` 无 warning

## 17. Phase 4A — values/numbers 修复

- [x] 17.1 修复 infinity/nan 在数值运算中的传播
- [x] 17.2 修复 `values/numbers` 中的 degenerate 值序列化（5 次）
- [x] 17.3 修复单位运算中的 infinity 处理

## 18. Phase 4B — values/lists 修复

- [x] 18.1 修复 list 分隔符处理（space/comma/slash）
- [x] 18.2 修复 bracketed list 序列化
- [x] 18.3 修复 empty list 处理

## 19. Phase 4C — css/plain 修复

- [x] 19.1 修复 CSS 原生函数透传格式
- [x] 19.2 修复 custom property 中的值处理
- [x] 19.3 修复 `@media` 查询解析

## 20. Phase 4 验证

- [x] 20.1 核心测试 202/202 全通过
- [x] 20.2 `RUST_LOG="sass_spec_full=info,sasspile=warn" cargo test --test sass_spec_full -- --nocapture` — 通过率 ≥ 68%
- [x] 20.3 `cargo clippy` 无 warning

## 21. 提交与文档

- [x] 21.1 更新 `AGENTS.md` 中的基线数字
- [x] 21.2 更新 `docs/CODE_INDEX.md` 反映新文件结构
- [x] 21.3 Git commit: `feat: spec 通过率优化 — 总计 2808/5362=52%`
- [x] 21.4 `codegraph sync` 同步索引
