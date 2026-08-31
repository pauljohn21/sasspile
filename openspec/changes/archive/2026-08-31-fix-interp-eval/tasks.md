## 1. 修复 eval_interp_str + Value::Interp 架构重构

- [x] 1.1 将 `Value::Interp(String)` 改为 `Value::Interp(Vec<InterpSegment>)`，新增 `InterpSegment` 枚举区分 `Expr` 和 `Text`
- [x] 1.2 修改 parser `Token::Interp` 分支，使用 `parse_interp_adjacent` 保留表达式与文本边界
- [x] 1.3 修改 parser `Token::Ident` 分支，当后面紧跟 `Token::Interp` 时也走拼接逻辑（处理 `hey#{$y}ho`）
- [x] 1.4 实现 `eval_interp_segments` 函数，遍历片段列表分别求值
- [x] 1.5 更新 `Value::Interp` 的 Display 实现
- [x] 1.6 更新 `partial_eval_condition` 中的 `Value::Interp` 分支

## 2. 测试用例

- [x] 2.1 `tests/default_config_test.rs` 的 `distributed_vars` 测试验证 `content: a` 和 `content: b`
- [x] 2.2 新增 `tests/interp_test.rs`，覆盖 sass-spec 场景：
  - 裸插值 `#{$a}`、字符串内插 `"#{$a}"`、前缀后缀 `hey#{$y}ho`
  - 字符串拼接 `foo#{"hey"}bar`、表达式插值 `#{1+2}px`
  - 多段插值 `#{$a}#{$b}`、null 插值 `#{null}`
  - 属性名插值 `bar#{1+2}`、数字带单位 `#{$n}px`

## 3. 回归验证

- [x] 3.1 `cargo test --test compile_test` — 43/43 通过
- [x] 3.2 `cargo test --test stage_test` — 10/10 通过
- [x] 3.3 `cargo test --test ep_full` — 121/121 通过
- [x] 3.4 `cargo test --test default_config_test -- --test-threads=1` — 9/9 通过
