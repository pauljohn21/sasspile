## 1. 编译阻塞修复 (Layer 0)

- [x] 1.1 `tests/specstore/mod.rs` 中 4 处 `println!` → `tracing::info!`
- [x] 1.2 验证 `cargo clippy --test spec_store` 零 error

## 2. lib clippy 自动清理 (Layer 1)

- [x] 2.1 运行 `cargo clippy --fix --lib` 自动修复（21→9 warnings）
- [x] 2.2 手动清理 unused code（`pseudo_args_match`/`css_round`/`clear_selector`/`add_css_imports` 已删除）
- [x] 2.3 处理 `css_imports` field（添加 `#[allow(dead_code)]` + 注释说明）
- [x] 2.4 手写 `let...else` 重写 3 处（selector_extend.rs ×2、map.rs ×1）
- [x] 2.5 修复 `&mut Vec` → `&mut [_]` + `args.iter()` → `args` + 其他 clippy --fix 自动修复
- [x] 2.6 验证 `cargo clippy --lib` 零 warning 零 error

## 3. 大文件拆分 (Layer 2) — selector_extend.rs

- [ ] 3.1 创建 `src/css/selector/` 模块目录 + `mod.rs`
- [ ] 3.2 将 extend 算法迁移到 `selector/extend.rs` (~300 行)
- [ ] 3.3 将 unify 算法迁移到 `selector/unify.rs` (~200 行)
- [ ] 3.4 将 format 逻辑迁移到 `selector/format.rs` (~150 行)
- [ ] 3.5 从 `selector_simplify.rs` 合并已有逻辑
- [ ] 3.6 更新 `src/css/mod.rs` re-export
- [ ] 3.7 验证 cargo clippy --lib + 全量测试

## 4. 大文件拆分 — reactor.rs

- [ ] 4.1 创建 `src/eval/reactor_states.rs` 存放 StateRaw/StateLexed/StateParsed/StateEvaluated/StateSerialized 类型定义
- [ ] 4.2 将状态类型与 impl 从 `reactor.rs` 移出，保留 Pipeline 链式主体
- [ ] 4.3 更新 `src/eval/mod.rs` 重新导出
- [ ] 4.4 验证 cargo clippy --lib + 全量测试

## 5. 大文件拆分 — color_adjust.rs

- [ ] 5.1 创建 `src/eval/builtin/color_change.rs` 承载 change-color 函数实现
- [ ] 5.2 创建 `src/eval/builtin/color_scale.rs` 承载 scale-color 函数实现
- [ ] 5.3 `color_adjust.rs` 仅保留 adjust-color 核心逻辑 + re-export
- [ ] 5.4 验证 cargo clippy --lib + 全量测试

## 6. 大文件拆分 — selector.rs (builtin)

- [ ] 6.1 创建 `src/eval/builtin/selector_ops.rs` 承载 selector-append/is-super/nest/replace/parse 实现
- [ ] 6.2 `selector.rs` 简化为入口 dispatch + re-export (≤200 行)
- [ ] 6.3 验证 cargo clippy --lib + 全量测试

## 7. 大文件拆分 — color_hwb_hsl.rs

- [ ] 7.1 创建 `src/eval/builtin/color_hwb.rs` 承载 hwb 系列函数
- [ ] 7.2 `color_hwb_hsl.rs` 仅保留 hsl 系列函数
- [ ] 7.3 验证 cargo clippy --lib + 全量测试

## 8. 大文件拆分 — display_color.rs

- [ ] 8.1 创建 `src/parse/ast/display_color_spaces.rs` 承载各色彩空间序列化
- [ ] 8.2 `display_color.rs` 保留主入口 + ColorOutput 辅助逻辑
- [ ] 8.3 验证 cargo clippy --lib + 全量测试

## 9. 最终验证

- [ ] 9.1 运行 `cargo clippy --all-targets` 确认整项目零 error
- [ ] 9.2 运行全部核心测试确认 202/202 通过
- [ ] 9.3 确认所有源文件 ≤ 500 行 (`find src -name '*.rs' -exec wc -l {} + | sort -rn | awk '$1 > 500'`)
