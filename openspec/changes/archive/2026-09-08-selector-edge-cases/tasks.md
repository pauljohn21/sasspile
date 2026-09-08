## 1. Mixed Format Input Support

- [x] 1.1 修改 `value_list_to_format` 函数支持混合类型输入（string + list 混合）
- [x] 1.2 增加单元测试验证 mixed input 场景（`(c, d e)` 格式）
- [x] 1.3 验证 strictly-typed list 场景不破坏（pure strings, pure lists）
- [x] 1.4 运行 `cargo test --test selector_unify_test` 确认不回归

## 2. selector-extend Parent Unification

- [x] 2.1 实现 `selector_ops::extend_complex` 中 parent compound 替换逻辑
- [x] 2.2 增加单元测试 `selector.extend(".c.x .d", ".c", ".e")` → `.c.x .d, .x.e .d`
- [x] 2.3 增加单元测试多 compound extender（如 `.e .f`）展开
- [x] 2.4 增加单元测试多 extender 列表（如 `.e, .f`）展开
- [x] 2.5 运行 `cargo test --test selector_unify_test` 确认不回归

## 3. selector-extend Grandparent Unification

- [x] 3.1 实现 grandparent context 遍历和替换逻辑
- [x] 3.2 增加单元测试 `selector.extend(".c .d.x .e", ".d", ".f")` → `.c .d.x .e, .c .x.f .e`
- [x] 3.3 增加单元测试复杂 multi-compound grandparent 展开
- [x] 3.4 增加单元测试 grandparent list extender 场景
- [x] 3.5 运行 `cargo test --test selector_unify_test` 确认不回归

## 4. selector-extend Leading Combinator Handling

- [x] 4.1 实现 extender 以 combinator 开头时的正确处理逻辑
- [x] 4.2 增加单元测试 extender 以 `>` 开头
- [x] 4.3 增加单元测试 extender 以 `+` 开头
- [x] 4.4 运行 `cargo test --test selector_unify_test` 确认不回归

## 5. Parser Error Detection

- [x] 5.1 在 `parse_compound` 中增加 attribute selector 闭合验证
- [x] 5.2 增加单元测试 `[c` 抛出解析错误
- [x] 5.3 增加单元测试 `[foo=bar` 抛出解析错误
- [x] 5.4 增加单元测试合法属性选择器 `[foo="bar"]` 通过
- [x] 5.5 运行 `cargo test --test selector_unify_test` 确认不回归

## 6. Integration and Verification

- [x] 6.1 运行 `cargo test --test cf_diag diag_selector` 统计修复后通过率
- [x] 6.2 运行 `cargo test --test compile_test` 确认核心测试全部通过
- [x] 6.3 运行 `cargo test --test selector_unify_test` 确认 selector 测试全部通过
- [x] 6.4 提交代码并推送到 origin/main
- [x] 6.5 归档 openspec 变更
