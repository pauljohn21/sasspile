## 1. 阻塞错误修复

- [x] 1.1 修复 `src/eval/builtin/selector.rs:202` unwrap() → expect("build_parents: _ branch guarantees non-empty")

## P0: selector.rs 清理

- [x] 2.1 修复 selector.rs 所有 clippy 警告

## P1: reactor.rs 清理

- [x] 3.1 修复 u64↔u128 as 转换 → From/try_from
- [x] 3.2 为 `ReactorTrace` 添加 Default 实现
- [x] 3.3 修复其他 reactor.rs 警告

## P2: color_adjust.rs 清理

- [x] 4.1 修复 color_adjust.rs format! 相关警告

## P3: selector_format.rs 清理

- [x] 5.1 修复 intra-doc link (引号→反引号, 8 处)
- [x] 5.2 修复其他 selector_format.rs 警告

## P4: color_hwb_hsl.rs 清理

- [x] 6.1 修复 color_hwb_hsl.rs RangeInclusive::contains (6 处)

## P5: selector_ops.rs 清理

- [x] 7.1 修复 selector_ops.rs 警告

## P6: display_color.rs 清理

- [x] 8.1 修复 display_color.rs 警告

## P7-P9: 其他文件清理

- [x] 9.1 修复 math.rs 警告 (2 处)
- [x] 9.2 修复 value/calc.rs 警告 (1 处)
- [x] 9.3 修复 builtin/math_trig.rs 警告 (1 处)

## 全量测试文件清理

- [x] 10.1 全量 tests/ 文件 unwrap() → expect() (26 个文件)
- [x] 10.2 全量 tests/ 文件 eprintln! → tracing::error! (7 个文件)
- [x] 10.3 benches/ parser_bench.rs unwrap → expect

## 验证

- [x] 11.1 `cargo clippy --all-targets` 零错误（425 warnings 均属合理）
- [x] 11.2 核心测试全通过 (compile_test 57/57 + stage_test 8/8 + ast_test 8/8 + common_test 5/5 + interp_test 15/15 + bs_spec 15/15 + ep_full 1/1 + default_config_test 9/9 + feature_tests 9/9 + calc_units_test 4/4)
- [x] 11.3 sass-spec 回归验证 (通过率不下降)
