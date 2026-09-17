## 1. Panic 修复与命名空间路由重构

- [ ] 1.1 修改 `src/evaluate_dst/mod.rs:255` index-out-of-bounds panic：在 `i += 1` 后加 `if i < chars_vec.len()` 保护,否则 `args_start_byte = s.len()`.引用: specs/ns-routing-module-fn/spec.md.扫描/无需 rxrust (纯 tracing span).

- [ ] 1.2 重构 `substitute_vars` 中 module-dot 标识符路由逻辑：将 `module.fn` 归一化为 `module-fn` (hyphenated) 传给 `eval_builtin`.引用: specs/ns-routing-module-fn/spec.md + specs/module-dot-evaluate/spec.md.

- [ ] 1.3 重构 `eval_builtin` match：顶层改用 `module-fn` (hyphenated) canonical key,保留旧短名 alias.引用: specs/ns-routing-module-fn/spec.md.

- [ ] 1.4 新增 `tests/ns_routing.rs`：selector.append vs list.append 不再冲突;短名 alias 不回归;map.get 路由为 map-get.引用: specs/ns-routing-module-fn/spec.md.

## 2. Color 内置函数扩展

- [ ] 2.1 实现 `color.adize`, `color.change`：接受 keyword args `$red/$green/$blue/$hue/$saturation/$lightness/$alpha`.引用: specs/color-adjust-change-scale/spec.md.

- [ ] 2.2 实现 `color.scale`：按比例缩放 HSL/RGB/alpha.引用: specs/color-adjust-change-scale/spec.md.

- [ ] 2.3 实现 `color.opacify` / `color.fade-in` / `color.transparentize` / `color.fade-out`.引用: specs/color-adjust-change-scale/spec.md.

- [ ] 2.4 在 `eval_builtin` 注册 `color-adjust`, `color-change`, `color-scale`, `color-opacify`, `color-transparentize`, `color-fade-in`, `color-fade-out` 路由.引用: specs/color-adjust-change-scale/spec.md.

- [ ] 2.5 在 `tests/color_builtins.rs` 补充 adjust/change/spec 测试用例,验证 sass-spec upstream baseline.引用: specs/color-adjust-change-scale/spec.md.

## 3. Math 内置函数扩展

- [ ] 3.1 实现 `math.pow`, `math.sqrt`, `math.log`.引用: specs/math-advanced/spec.md.

- [ ] 3.2 实现 `math.sin`, `math.cos`, `math.tan` (单位 deg/rad/grad/turn);`math.asin`, `math.acos`, `math.atan`, `math.atan2`.引用: specs/math-advanced/spec.md.

- [ ] 3.3 实现 `math.random($limit?)`：采用 thread_rng + 范围生成.引用: specs/math-advanced/spec.md.

- [ ] 3.4 实现 `math.hypot`, `math.clamp`.引用: specs/math-advanced/spec.md.

- [ ] 3.5 在 `eval_builtin` 注册 `math-pow`, `math-sqrt`, `math-log`, `math-sin`, `math-cos`, `math-tan`, `math-asin`, `math-acos`, `math-atan`, `math-atan2`, `math-random`, `math-hypot`, `math-clamp`.引用: specs/math-advanced/spec.md.

- [ ] 3.6 在 `tests/math_builtins.rs` 补充对应 test 用例.引用: specs/math-advanced/spec.md.

## 4. String 内置函数扩展

- [ ] 4.1 实现 `string.insert`, `string.slice`, `string.split`.引用: specs/string-advanced/spec.md.

- [ ] 4.2 实现 `string.to-upper-case`, `string.to-lower-case`, `string.unique-id`, `string.replace`.引用: specs/string-advanced/spec.md.

- [ ] 4.3 在 `eval_builtin` 注册 `string-insert`, `string-slice`, `string-split`, `string-to-upper-case`, `string-to-lower-case`, `string-unique-id`, `string-replace`.引用: specs/string-advanced/spec.md.

- [ ] 4.4 在 `tests/string_builtins.rs` 补充测试.引用: specs/string-advanced/spec.md.

## 5. List / Map 内置函数扩展

- [ ] 5.1 实现 `list.set-nth`, `list.zip`, `list.is-bracketed`.引用: specs/list-advanced/spec.md.

- [ ] 5.2 实现 `map.remove`, `map.deep-merge`, `map.deep-remove`.引用: specs/map-advanced/spec.md.

- [ ] 5.3 在 `eval_builtin` 注册 `list-set-nth`, `list-zip`, `list-is-bracketed`, `map-remove`, `map-deep-merge`, `map-deep-remove`.引用: specs/list-advanced/spec.md + specs/map-advanced/spec.md.

- [ ] 5.4 在 `tests/list_builtins.rs` / `tests/map_builtins.rs` 补充测试.引用: specs/list-advanced/spec.md + specs/map-advanced/spec.md.

## 6. Selector 内置函数扩展

- [ ] 6.1 实现 `selector.parse`, `selector.extend`, `selector.replace`, `selector.unify`, `selector.is-superselector`, `selector.simple-selectors`.引用: specs/selector-advanced/spec.md.

- [ ] 6.2 在 `eval_builtin` 注册对应 `selector-*` 规范名.引用: specs/selector-advanced/spec.md.

- [ ] 6.3 在 `tests/selector_builtins.rs` 补充测试.引用: specs/selector-advanced/spec.md.

## 7. Meta 内置函数扩展

- [ ] 7.1 实现 `meta.module-variables`, `meta.module-functions`, `meta.keywords`.引用: specs/meta-advanced/spec.md.

- [ ] 7.2 实现 `meta.calc-args`, `meta.calc-name`.引用: specs/meta-advanced/spec.md.

- [ ] 7.3 在 `eval_builtin` 注册 `meta-module-variables`, `meta-module-functions`, `meta-keywords`, `meta-calc-args`, `meta-calc-name`.引用: specs/meta-advanced/spec.md.

- [ ] 7.4 在 `tests/meta_builtins.rs` 补充测试.引用: specs/meta-advanced/spec.md.

## 8. Parser 增强（仅 sass-spec 所需语法子集）

- [ ] 8.1 解析 `(key: value, ...)` map 字面量作为函数参数值,保留冒号 key.引用: design.md D3.

- [ ] 8.2 解析 `(a, b, c)` 列表字面量作为函数参数值.引用: design.md D3.

- [ ] 8.3 解析 keyword argument `$name: value`.引用: design.md D3.

## 9. 验证与回归

- [ ] 9.1 跑全量 `cargo test` — 确保所有已有测试(含 Bootstrap/Element Plus 相关 fixture)不回归.引用: design.md Risks.

- [ ] 9.2 跑 `cargo test --test sass_spec_detail -- --nocapture` — 采集 core_functions 通过数量增量.引用: specs/ns-routing-module-fn/spec.md.目标: 从当前 ~40/7793 至少翻倍到 2000+.

- [ ] 9.3 清理临时 `diag_corefn_*.rs / diag.rs`.引用: 调试协议.
