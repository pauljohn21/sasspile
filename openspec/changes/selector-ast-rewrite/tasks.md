## 1. 数据模型改造（selector_ast.rs）

- [ ] 1.1 新增 `Namespace` 枚举（`None`/`Empty`/`Any`/`Explicit(String)`）及其 `Display` 实现
- [ ] 1.2 将 `SimpleSelector::Type(String)` 改造为 `Type { namespace: Namespace, name: String }`
- [ ] 1.3 更新 `Display for SimpleSelector::Type` 输出 `ns|name` 格式
- [ ] 1.4 运行 `cargo test --test compile_test` 确认编译通过

## 2. 解析器适配（selector_parser.rs）

- [ ] 2.1 修改 `take_type_with_ns` 返回 `(Namespace, String)` 而非拼接字符串
- [ ] 2.2 更新 `parse_compound` 中 Type 构造调用适配新结构体
- [ ] 2.3 更新降级路径（`parse_selector` 末尾 `Type(rest)` 改为结构化）
- [ ] 2.4 运行 `cargo test --test compile_test` 确认编译通过

## 3. 统一算法重写（selector_ops.rs — unify）

- [ ] 3.1 重写 `unify_compound` Type 分支：实现命名空间规则矩阵（16 种组合）
- [ ] 3.2 修改 `unify_compound` Universal + Type 分支：检查 namespace 兼容性
- [ ] 3.3 实现伪类链式合并：同 name 覆盖、不同 name 链式
- [ ] 3.4 实现伪元素归一化比较：`is_class_syntax` 归一后比较
- [ ] 3.5 运行 `cargo test --test compile_test` + 手动验证 universal.hrx 29 case

## 4. 超选择器判断重写（selector_ops.rs — is_super）

- [ ] 4.1 修改 `is_super_compound` 伪元素比较：归一化 `is_class_syntax`
- [ ] 4.2 修改 `is_super_compound` 伪类比较：super 中每个伪类必须在 sub 中存在
- [ ] 4.3 运行 `cargo test --test compile_test` + 手动验证 is_superselector 相关 case

## 5. 扩展算法重写（selector_ops.rs — extend）

- [ ] 5.1 重写 `is_more_specific_than`：增加命名空间兼容性判断
- [ ] 5.2 重写 `compounds_conflict`：Type 比较增加命名空间感知
- [ ] 5.3 修改 `compounds_conflict` 伪元素比较：归一化 `is_class_syntax`
- [ ] 5.4 修改 `extend_complex` NO-OP 条件：命名空间不兼容时不触发 NO-OP
- [ ] 5.5 运行 `cargo test --test compile_test` + 手动验证 extend/no_op.hrx 25 case

## 6. 适配层更新（selector_format.rs + selector.rs）

- [ ] 6.1 更新 `selector_format.rs` 中 `SimpleSelector::Type` 匹配（如有）
- [ ] 6.2 更新 `selector.rs` 中 `SimpleSelector::Type` 匹配（`merge_selector_args` 等）
- [ ] 6.3 运行 `cargo test --test compile_test` 确认编译通过

## 7. 全量测试验证

- [ ] 7.1 运行 `cargo test --test compile_test` 确认 57/57 通过
- [ ] 7.2 运行 `cargo test --test stage_test` 确认 10/10 通过
- [ ] 7.3 运行 `cargo test --test ast_test` 确认 8/8 通过
- [ ] 7.4 运行 `cargo test --test common_test` 确认 5/5 通过
- [ ] 7.5 运行 `cargo test --test bs_spec` 确认 15/15 通过
- [ ] 7.6 运行 `cargo test --test ep_full` 确认 121/121 通过
- [ ] 7.7 运行 `RUST_LOG="sass_spec_full=info,sasspile=warn" cargo test --test sass_spec_full -- --nocapture` 统计 selector 通过率
- [ ] 7.8 对比 before/after：确认 selector 通过率从 45% 提升至 73%+ 且无回归
- [ ] 7.9 运行 `cargo clippy --all-targets` 确认零警告

## 8. 提交

- [ ] 8.1 `git add` 所有改动文件
- [ ] 8.2 `git commit -m "feat: selector AST 全量重写 — 命名空间 + 伪元素归一 + 伪类链式 — 总计 +N/M"`
- [ ] 8.3 `codegraph sync` 同步索引
- [ ] 8.4 等待用户确认后推送
