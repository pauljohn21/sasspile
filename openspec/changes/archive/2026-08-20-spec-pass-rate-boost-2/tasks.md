## 1. 命名空间函数映射补全

- [x] 1.1 在 `module_dispatch.rs` 中添加 `map.map-get`/`map.map-merge`/`map.map-keys`/`map.map-values`/`map.map-has-key`/`map.map-remove`/`map.map-set` 映射
- [x] 1.2 在 `module_dispatch.rs` 中添加 `string.str-length`/`string.str-index`/`string.str-insert`/`string.str-slice`/`string.str-split` 映射
- [x] 1.3 在 `module_dispatch.rs` 中添加 `color.lighten`/`color.darken`/`color.ie-hex-str` 映射
- [x] 1.4 在 `module_dispatch.rs` 中添加 `selector.selector-append`/`selector.selector-nest`/`selector.selector-extend`/`selector.selector-parse`/`selector.selector-replace`/`selector.selector-unify`/`selector.selector-simple-selectors`/`selector.selector-is-superselector` 映射
- [x] 1.5 在 `module_dispatch.rs` 中添加 `math.unitless`（→ `is-unitless`）映射
- [x] 1.6 在 `module_dispatch.rs` 中添加 `meta.load-css`/`meta.apply`/`meta.accepts-content` 映射
- [x] 1.7 运行 `cargo test --test cf_diag diag_map`/`diag_string`/`diag_selector`/`diag_meta`/`diag_math` 验证无 `Undefined function` 错误

## 2. color.ie-hex-str 函数实现

- [x] 2.1 在 `builtin/color.rs` 中实现 `ie-hex-str` 函数——将颜色转为 IE 兼容的 `#AARRGGBB` 格式
- [x] 2.2 在 `builtin.rs` 的 match 中添加 `"ie-hex-str"` 分派
- [x] 2.3 在 `is_known_builtin` 中添加 `"ie-hex-str"`
- [x] 2.4 运行 `cargo test --test cf_diag diag_meta -- --nocapture` 验证 `color.ie-hex-str` 测试通过

## 3. plain CSS 错误检测补全

- [x] 3.1 在 eval 阶段检测 plain CSS 模式下的 at-rule：`@use`/`@forward`/`@include`/`@function`/`@mixin` → 报 "This at-rule isn't allowed in plain CSS."
- [x] 3.2 在 eval 阶段检测 plain CSS 模式下的插值：选择器和属性名中的 `#{...}` → 报 "Interpolation isn't allowed in plain CSS."
- [x] 3.3 在 eval 阶段检测 plain CSS 模式下的运算符：声明值中的 `+`/`-`/`*`/`/` → 报 "Operators aren't allowed in plain CSS."
- [x] 3.4 在 eval 阶段检测 plain CSS 模式下的 Sass 变量：`$var` → 报 "Sass variables aren't allowed in plain CSS."
- [x] 3.5 检测 plain CSS 模式下的父选择器后缀 `&-suffix` → 报 "Parent selectors can't have suffixes in plain CSS."
- [x] 3.6 检测 plain CSS 模式下的占位选择器 `%placeholder` → 报 "Placeholder selectors aren't allowed in plain CSS."
- [x] 3.7 检测 plain CSS 模式下的顶级前导组合器 `> .child` → 报 "Top-level leading combinators aren't allowed in plain CSS."
- [x] 3.8 运行 `cargo test --test css_diag -- --nocapture` 验证 css/ 目录通过率提升

## 4. @forward 冲突检测

- [x] 4.1 在 `eval/module.rs` 的 `load_module` 或 `eval_forward` 中添加冲突检测逻辑：合并多个 @forward 的 exports 时检查同名 variable
- [x] 4.2 添加同名 function 冲突检测
- [x] 4.3 添加同名 mixin 冲突检测
- [x] 4.4 确认同值变量不报错（允许同值冲突）
- [x] 4.5 运行 `cargo test --test cf_diag diag_forward -- --nocapture` 验证 forward 冲突测试通过

## 5. math 函数参数验证增强

- [x] 5.1 在 `builtin/math.rs` 中提取 `require_number(name, value)` helper 函数
- [x] 5.2 增强 `clamp` 参数验证：参数数量（3 个）、类型（数字）、单位检查
- [x] 5.3 增强 `min`/`max` 参数验证：至少 1 个参数、所有参数为数字
- [x] 5.4 增强 `pow` 参数验证：2 个参数、无单位、数字类型
- [x] 5.5 增强 `hypot` 参数验证：所有参数为数字
- [x] 5.6 增强 `log` 参数验证：数字类型、可选 base 参数
- [x] 5.7 增强 `abs`/`ceil`/`floor`/`round`/`sin`/`cos`/`tan`/`asin`/`acos`/`atan`/`sqrt` 参数验证：1 个参数、数字类型
- [x] 5.8 确保 `math.unitless`（`is-unitless`）正确映射和参数验证
- [x] 5.9 确保 `math.compatible`（`comparable`）正确映射和参数验证
- [x] 5.10 运行 `cargo test --test cf_diag diag_math -- --nocapture` 验证 math 子目录通过率

## 6. meta.load-css mixin 和 meta.apply 实现

- [x] 6.1 在 eval 阶段拦截 `meta.load-css` mixin 调用——动态调用 `eval_use` 加载模块并输出 CSS
- [x] 6.2 支持 `$with` 配置 map 传递
- [x] 6.3 在 eval 阶段拦截 `meta.apply` mixin 调用——从 MixinRef 执行 mixin
- [x] 6.4 验证 `$mixin` 参数为 MixinRef 类型
- [x] 6.5 实现 `meta.accepts-content` 函数——检查 mixin 是否接受 @content
- [x] 6.6 运行 `cargo test --test cf_diag diag_meta -- --nocapture` 验证 meta 子目录通过率

## 7. 全量验证

- [x] 7.1 运行核心测试 `cargo test --test compile_test && cargo test --test stage_test && cargo test --test ast_test && cargo test --test common_test && cargo test --test bs_spec && cargo test --test ep_full`
- [x] 7.2 运行 sass-spec 全量 `RUST_LOG="sass_spec_full=info,sasspile=warn" cargo test --test sass_spec_full -- --nocapture` 确认通过率提升
- [x] 7.3 确认核心测试 100% 通过（43+10+8+5+15+121）
- [x] 7.4 记录最终通过率数据，更新 AGENTS.md 基线
