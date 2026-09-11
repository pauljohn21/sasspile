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

- [x] 3.1 创建 `src/css/selector_extend/` 模块目录 + `mod.rs`
- [x] 3.2 将 extend 算法迁移到 `selector_extend/extend.rs`
- [x] 3.3 将 unify 算法迁移到 `selector_extend/extend_complex.rs`
- [x] 3.4 将 extend_build 逻辑迁移到 `selector_extend/extend_build.rs`
- [x] 3.5 将 pseudo 处理迁移到 `selector_extend/extend_pseudo.rs`
- [x] 3.6 将 replace 逻辑迁移到 `selector_extend/replace.rs`
- [x] 3.7 验证 cargo clippy --lib + 全量测试

## 4. 大文件拆分 — reactor.rs

- [x] 4.1 创建 `src/eval/reactor_types.rs` 存放 State* 类型定义
- [x] 4.2 将状态类型从 `reactor.rs` 移出，保留 Pipeline 链式主体
- [x] 4.3 更新 `src/eval/mod.rs` 重新导出
- [x] 4.4 验证 cargo clippy --lib + 全量测试

## 5. 大文件拆分 — color_adjust.rs

- [x] 5.1 创建 `src/eval/builtin/color_change.rs` 承载 change-color 函数实现
- [x] 5.2 创建 `src/eval/builtin/color_scale.rs` 承载 scale-color 函数实现
- [x] 5.3 `color_adjust.rs` 仅保留 adjust-color 核心逻辑 + 共享基础设施 + re-export
- [x] 5.4 验证 cargo clippy --lib + 全量测试

## 6. 大文件拆分 — selector.rs (builtin)

- [x] 6.1 创建 `src/eval/builtin/selector_append.rs` 承载 selector-append 实现
- [x] 6.2 创建 `src/eval/builtin/selector_nest.rs` 承载 selector-nest 实现
- [x] 6.3 创建 `src/eval/builtin/selector_ops.rs` 承载 is-super/parse/simple-selectors/unify/extend/replace 实现
- [x] 6.4 `selector.rs` 简化为入口 dispatch + 参数合并 (~73 行)
- [x] 6.5 验证 cargo clippy --lib + 全量测试

## 7. 大文件拆分 — color_hwb_hsl.rs

- [x] 7.1 创建 `src/eval/builtin/color_hwb.rs` 承载 hwb/whiteness/blackness 实现
- [x] 7.2 `color_hwb_hsl.rs` 仅保留 complement/hsl/hsla/adjust-hue/saturate/desaturate 等 HSL 通道操作
- [x] 7.3 验证 cargo clippy --lib + 全量测试

## 8. 大文件拆分 — display_color.rs

- [x] 8.1 创建 `src/parse/ast/display_color_spaces.rs` 承载各色彩空间 Auto 序列化
- [x] 8.2 `display_color.rs` 保留 RgbExplicit/RgbModern/RgbPercent 模式 + Auto 分派入口
- [x] 8.3 验证 cargo clippy --lib + 全量测试

## 9. 最终验证

- [x] 9.1 运行 `cargo clippy --all-targets` 确认 src/ 零 error
- [x] 9.2 运行全部核心测试确认 84/84 通过
- [x] 9.3 确认所有目标源文件 ≤ 500 行

## 拆分前后对比

| 文件 | 拆分前 | 拆分后 |
|------|--------|--------|
| selector_extend.rs | 852 | 6 个子文件（最大 ~150 行） |
| reactor.rs | 800+ | 466 行 + reactor_types.rs |
| color_adjust.rs | 636 | 378 + 154 + 129 = 661 |
| selector.rs | 603 | 73 + 75 + 170 + 284 = 602 |
| color_hwb_hsl.rs | 583 | 391 + 185 = 576 |
| display_color.rs | 567 | 114 + 468 = 582 |

所有目标文件现已 ≤ 500 行（新增文件引入少量重复 import，总计增加 ~150 行，符合预期）。
