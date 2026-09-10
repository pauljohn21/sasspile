## 1. :not() 扩展基础设施

- [x] 1.1 在 `selector_extend.rs` 添加 `:not()` 检测函数 `has_not_pseudo(compound: &CompoundSelector) -> bool`
- [x] 1.2 添加 `extract_not_arg(compound: &CompoundSelector) -> Option<String>` 提取 `:not()` 参数
- [x] 1.3 添加 `append_not_pseudo(compound: &CompoundSelector, new_not: &SimpleSelector) -> CompoundSelector` 追加新 `:not()`

## 2. :not() 扩展核心逻辑

- [x] 2.1 在 `extend_selector_with_mode` 开始时检测 selector compound 中的 `:not()`
- [x] 2.2 实现 `:not()` 扩展路径：检查 extendee 是否匹配 `:not()` 内部
- [x] 2.3 处理 `:not()` 内部为简单选择器（`:not(.c)` + `.c` → `:not(.c):not(.d)`）
- [x] 2.4 处理 `:not()` 内部为 complex（`:not(.c .d)` + `.d` → `:not(.c .d):not(.c .e .f):not(.e .c .f)`）
- [x] 2.5 处理 `:not()` 内部为 compound（`:not(.c.d)` + `.c` → `:not(.c.d):not(.d.e)`）

## 3. :not() 列表和伪类扩展

- [x] 3.1 实现 `:not(.x, .y)` 已包含列表时的处理（添加新选择器到列表）
- [x] 3.2 实现 extender 为列表（`.d, .e`）时展开为多个 `:not()`
- [x] 3.3 实现 extender 为 `:is(.d, .e)` 时展开为多个 `:not()`
- [x] 3.4 实现 extender 为 `:where(...)` 时展开为多个 `:not()`
- [x] 3.5 实现 extender 为 `:matches(...)` 时展开为多个 `:not()`
- [x] 3.6 实现 extender 包含伪类在 compound 内（`.d:is(.e, .f)` 整体添加）

## 4. :is/:where/:matches 检测和 :where 扩展

- [x] 4.1 实现 `:is/:where/:matches` subselector 检测函数
- [x] 4.2 在 `is_more_specific_than` 中增加 no-op 判断
- [ ] 4.3 实现 `:where()` specificity_modification 扩展逻辑（已知限制，后续增量实现）

## 5. 参数化伪类匹配

- [x] 5.1 添加伪类参数匹配辅助函数
- [x] 5.2 实现 `:nth-child()` 参数匹配
- [x] 5.3 实现 `:nth-last-child()` 参数匹配

## 6. 测试和验证

- [x] 6.1 添加 `tests/pseudo_extend_test.rs` 单元测试
- [x] 6.2 运行 `cargo test --test pseudo_extend_test` 确认通过
- [x] 6.3 运行 `SPEC_STORE_CMD=run cargo test --test spec_store` 确认增量
- [ ] 6.4 更新 `AGENTS.md` 基线数据
