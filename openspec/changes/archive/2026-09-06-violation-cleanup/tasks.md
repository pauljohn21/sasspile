## 1. 基础设施 — Clippy deny + 行数检测

- [ ] 1.1 在 Cargo.toml 添加 clippy deny lint 配置（unwrap_used、todo、unimplemented）
- [ ] 1.2 创建 `tests/file_size_check.rs`，实现 `check_file_size_limits` 测试（遍历 src/**/*.rs，>500 行则失败）
- [ ] 1.3 验证：`cargo clippy -- -D warnings` 编译通过，`cargo test --test file_size_check` 全部通过

## 2. 消除 unwrap() panic 风险

- [ ] 2.1 修复 `css/selector_parser.rs` 7 处 unwrap()：
  - 改用 `?` 传播或 `expect("take_ident: ...")` 格式
  - take_ident / take_number 返回 Result<String, SassError>
- [ ] 2.2 修复 `eval/value/calc_ast.rs` 3 处 unwrap()：
  - `self.advance().unwrap()` → `self.advance().ok_or(SassError::Parse { expected: "token", found: "EOF" })?`
- [ ] 2.3 修复 `eval/value/calc_simplify.rs` 1 处 unwrap()：
  - `nums.last().unwrap().1.clone()` → `nums.last().expect("calc_simplify: nums non-empty").1.clone()`
- [ ] 2.4 验证：运行 `cargo test --test compile_test` 确认无 panic 且 57/57 通过

## 3. 文件拆分 — display.rs（636 行）

- [ ] 3.1 创建 `parse/ast/color_display.rs`，迁移 Color/ColorSpace::fmt 分派逻辑
- [ ] 3.2 在 `parse/ast/mod.rs` 中添加 `pub mod color_display;`
- [ ] 3.3 从 `display.rs` 中移除已迁移的代码，仅保留 Value::String/Number/List 等基础 fmt
- [ ] 3.4 验证：`display.rs` ≤ 250 行，编译通过，`cargo test --test compile_test` 通过

## 4. 文件拆分 — color.rs（627 行）

- [ ] 4.1 创建 `eval/builtin/color_channels.rs`，迁移 red/green/blue/alpha/hue/saturation/lightness/whiteness/blackness/color-channel
- [ ] 4.2 创建 `eval/builtin/color_adjust_legacy.rs`，迁移旧版 adjust-color/change-color/scale-color
- [ ] 4.3 创建 `eval/builtin/color_hsl_hwb.rs`，迁移 hsl/hsla/hwb/hwb-a 构造
- [ ] 4.4 修改 `eval/builtin/color.rs`：添加 `pub mod` 声明，保留入口路由（call 函数分派）
- [ ] 4.5 验证：`color.rs` ≤ 200 行，编译通过，cargo test 无回归

## 5. 文件拆分 — color_adjust.rs（614 行）

- [ ] 5.1 创建 `eval/builtin/color_adjust_legacy.rs`（如果之前未创建），迁移 sRGB/HSL 专用路径
- [ ] 5.2 在 `color_adjust.rs` 中保留现代空间（Oklch/Lab/Lch/Oklab/DisplayP3）处理逻辑
- [ ] 5.3 验证：`color_adjust.rs` ≤ 300 行

## 6. 文件拆分 — css/mod.rs（549 行）

- [ ] 6.1 创建 `css/merge.rs`，迁移 `merge_at_rules` 函数及辅助逻辑
- [ ] 6.2 在 `css/mod.rs` 中添加 `pub mod merge;` 并重新导出必要接口
- [ ] 6.3 验证：`css/mod.rs` ≤ 400 行

## 7. 最终验证

- [ ] 7.1 运行 `cargo clippy -- -D warnings` 确认无 clippy 错误
- [ ] 7.2 运行 `cargo test --test file_size_check` 确认所有文件 ≤500 行
- [ ] 7.3 运行完整核心测试套件：57/57 + 10/10 + 8/8 + 5/5 + 15/15 + 121/121
- [ ] 7.4 运行 `cargo test --test sass_spec_full test_sass_spec_full_stats` 确认 sass-spec 无回归
- [ ] 7.5 `codegraph sync` 同步索引
