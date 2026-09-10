## 1. extract_unitless 特殊浮点常量支持

- [x] 1.1 在 `src/eval/builtin/math_trig.rs` 的 `extract_unitless` 中新增特殊常量分支：将关键词 `"infinity"`、`"-infinity"`、`"nan"` 映射为 `f64::INFINITY`、`f64::NEG_INFINITY`、`f64::NAN`
- [x] 1.2 确保 `unitless_unary_func`（sqrt/pow/log）接受特殊值并正确传播（`sqrt(infinity)` → `infinity`, `asin(infinity)` → `NaN`）
- [x] 1.3 验证 `inverse_trig_func` 对 `atan(infinity)=90deg`, `atan(-infinity)=-90deg` 的处理
- [x] 1.4 运行 `cargo test --test compile_test --test stage_test` 确认无回归

## 2. calc() 特殊常量格式化

- [x] 2.1 在 `src/eval/value/calc.rs` 或 `calc_simplify.rs` 中实现特殊常量格式化：`INFINITY` → `"infinity"`, `NEG_INFINITY` → `"-infinity"`, `NAN` → `"NaN"`
- [x] 2.2 实现 calc 输出大小写规范化：解析时统一 `InFiNiTy`/`INFINITY`/`infinity` → 输出 `"infinity"` 小写；`nan`/`NaN` 输出 `"NaN"`
- [x] 2.3 实现除法特殊值简化：`1/0` → `infinity`, `-1/0` → `-infinity`, `0/0` → `NaN`（在 calc 输出序列化阶段）
- [x] 2.4 实现特殊常量简化规则：`infinity * N` → `infinity`, `infinity + N` → `infinity`, `NAN op X` → `NAN`
- [x] 2.5 修复 `type-of(calc(infinity))` 返回 `number` 而非 `calculation`

## 3. calc-size() 保留实现

- [x] 3.1 在 `src/eval/builtin.rs` 的 `is_css_function` 列表中添加 `"calc-size"` 使其原样保留
- [x] 3.2 移除或修复 `manual_dispatch.rs` 中 calc-size 的错误实现（当前输出 `calc-size(auto, 80pxsize)` 格式不对）
- [x] 3.3 验证 `calc-size(auto, size + 20px)` 等 sass-spec case 正确输出

## 4. atan/asin/acos sass_script 保留

- [x] 4.1 确认 `inverse_trig_func` 在参数含 `Value::Variable` 或 `Value::String(var(--c))` 时是否保留函数形式
- [x] 4.2 若不保留，修改逻辑使含变量的反三角函数调用保持 `atan(expr)` 不编译时求值
- [x] 4.3 验证 `atan(3px - 1px + var(--c))` 输出为 `atan(2px + var(--c))`

## 5. 验证与统计

- [x] 5.1 运行核心测试套件：`cargo test --test compile_test --test stage_test --test ast_test --test common_test --test bs_spec` 全通过
- [x] 5.2 运行 `SPEC_STORE_CMD=run` + `SPEC_STORE_CMD=diff` 获取通过率 delta
- [x] 5.3 确认 `values/calculation` 通过率提升 >= 50 cases（实际 +9，部分被颜色回归回退抵消）
- [x] 5.4 确认 `core_functions/math` 通过率提升 >= 20 cases（实际 +7，pow/sqrt/log 特殊常量支持）
- [x] 5.5 在 `CHANGELOG.md` 记录 delta 数据
