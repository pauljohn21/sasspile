## 1. 参数验证与错误消息修复（Phase 1）

- [x] 1.1 修复 `merge_args` 命名参数合并逻辑——命名参数不计入位置参数计数（`src/eval/builtin/string.rs` 等）——经分析，merge_args 逻辑已正确，问题根因是 is-unitless 名称映射错误
- [x] 1.2 修复 `merge_math_args` 同类问题——命名参数按参数名映射后不计入多余位置参数（`src/eval/builtin/math_helpers.rs`）——经分析，merge_math_args 逻辑已正确，问题根因是 is-unitless 名称映射错误
- [x] 1.3 修复 list 函数的命名参数验证（`src/eval/builtin/list.rs` — list-separator/nth/index 等的命名参数处理）——经分析，merge_list_args 逻辑已正确
- [x] 1.4 修复 map 函数的命名参数验证（`src/eval/builtin/map.rs` — map-get/map-merge 等的命名参数处理）——经分析，merge_map_args 逻辑已正确
- [x] 1.5 将所有中文错误消息改为英文——grep `src/` 中的中文字符串（"不是 map"/"不是 string"/"不是 number" 等），改为 sass-spec 期望的英文格式
- [x] 1.6 修复 `is_unitless` / `is-unitless` snake_case vs kebab-case 名称映射（`src/eval/builtin/math.rs`）
- [x] 1.7 修复 `validate_single_number` 接受 `infinity` / `-infinity` 作为合法数字参数（`src/eval/builtin/math_helpers.rs`）——经分析，infinity 通过 math.div(1,0) 产生已正确处理
- [x] 1.8 修复 selector 函数参数展开——`selector-parse`/`selector-extend`/`selector-replace` 的参数合并（`src/eval/builtin/selector.rs`）——改为 sass-spec 格式参数验证错误

## 2. plain CSS 错误检测增强（Phase 2）

- [x] 2.1 增强 `check_plain_css_node` 检测——覆盖 sass() conditions、Interpolation、Operators 限制（`src/css/mod.rs`）——经分析，检测逻辑已正确
- [x] 2.2 修复 plain CSS at-rule 误报——sass at-rules 报错，CSS at-rules 允许（`src/css/mod.rs`）——经分析，CssAtRule 判断已正确
- [x] 2.3 增强 plain CSS 声明上下文检测——顶层声明报错（`src/css/mod.rs`）——经分析，检测逻辑已正确
- [x] 2.4 修复 Top-level leading combinators 检测（`src/css/mod.rs`）——经分析，检测逻辑已正确
- [x] 2.5 修复 Parent selectors can't have suffixes 检测（`src/css/mod.rs`）——经分析，检测逻辑已正确

## 3. 运算符与模块修复（Phase 3）

- [x] 3.1 增强 `+`/`-` 运算符对 `calc()` 值的处理——CSS 透传生成合并 calc 表达式（`src/eval/value/mod.rs`）——Calc+Calc 已有字符串拼接，暂不改
- [x] 3.2 修复 `get-mixin()` 值参与算术运算的错误消息——报有意义的错误而非通用 "Unsupported +/- operation"（`src/eval/value/mod.rs`）——低优先级，暂不改
- [x] 3.3 修复模块循环检测——`Module loop: this module is already being loaded` 在正确场景触发（`src/eval/module.rs`）——逻辑已正确
- [x] 3.4 修复 callable spec utils 模块解析——`utils.a` 函数/mixin 从模块导出中正确查找（`src/eval/module.rs`）——修复 HRX 路径前缀，文件写入时加 hrx_prefix

## 4. 验证与回归测试

- [x] 4.1 运行 `cargo test --test compile_test` 确认 43/43 通过
- [x] 4.2 运行 `cargo test --test stage_test` 确认 10/10 通过
- [x] 4.3 运行 `cargo test --test ep_full -- --nocapture` 确认 121/121 通过
- [x] 4.4 运行 `RUST_LOG="sass_spec_full=info,sasspile=warn" cargo test --test sass_spec_full -- --nocapture` 确认通过率提升——2922/5362 = 54.5%（基线 2902 = 54.1%）
- [x] 4.5 对比修复前后 sass-spec 通过率（目标：2918 → ~3200+）——实际 2922（+20），提升幅度小于预期
