## 1. Diagnostic Instrumentation

- [ ] 1.1 在 `ops.rs` 的 `eval_op(Add, ...)` 入口添加 `#[instrument]` 记录左右操作数和返回类型
- [ ] 1.2 在 `display.rs` 的 `eval_interp_str` 和 `eval_simple_expr` 入口添加 span 记录输入/输出
- [ ] 1.3 在 `mod.rs` 的 `eval_variable` 添加 span 记录变量名、值、flags

## 2. Trace Collection

- [ ] 2.1 构造最小 repro 测试：`'$a' + '$b'`、`'.' + $var + '__'` 
- [ ] 2.2 采集 trace 确认 `Add` 操作进入哪个代码分支
- [ ] 2.3 确认中间结果类型（String / List / ArgList / 其他）
- [ ] 2.4 确认乱码的 escape 序列由哪个 Display/Debug impl 产生

## 3. Root Cause Analysis

- [ ] 3.1 检查 `ops.rs` 中 `(String, String) +` 是否存在正确连接分支
- [ ] 3.2 检查是否有路径错误创建了 `ArgList` 或 `List`
- [ ] 3.3 检查 `Value::Add(a, b)` 的 stringification 路径
- [ ] 3.4 确认 `Value::String(arglist, separator, bracketed).to_string()` 的表现

## 4. Implementation Fix

- [ ] 4.1 修复 `Add` 操作符使其正确连接字符串（返回 `Value::String(concatenated, false)`）
- [ ] 4.2 确保混合类型（Number + String、String + Boolean）正确字符串化
- [ ] 4.3 确保 `ArgList` 类型的字符串化不产生转义字符

## 5. Regression Testing

- [ ] 5.1 运行 `cargo test --tests` 全量核心测试，确认 241/241 通过
- [ ] 5.2 运行 sass-spec 全量，确认无回归
- [ ] 5.3 运行 `cargo test --test ep_normalized_test`，记录新一致性数
- [ ] 5.4 验证 BEM selector 构建结果：`.el-breadcrumb__separator { ... }`

## 6. Cleanup

- [ ] 6.1 降级/移除临时 debug span
- [ ] 6.2 添加永久性回归测试到 `compile_test.rs` 或新建测试文件
- [ ] 6.3 更新 openspec 状态为归档
