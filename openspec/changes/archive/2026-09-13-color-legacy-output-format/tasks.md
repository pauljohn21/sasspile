## 1. invert 保留色彩空间

- [ ] 1.1 修改 `src/eval/builtin/color.rs` invert legacy 路径：HSL 输入使用 `Color::with_hsl()` 保持 HSL 空间，HWB 输入使用 `Color::with_space(Hwb, ...)`，RGB 输入保持现有 `with_rgb(..., Auto)`
- [ ] 1.2 修改 grayscale 路径：与 invert 类似，保留输入 `c.space`
- [ ] 1.3 修改 invert weighted 路径：输出使用 `ColorOutput::RgbPercent`
- [ ] 1.4 运行 `SPEC_STORE_CMD=run cargo test --test spec_store` 验证 invert 相关 case

## 2. complement 保留色彩空间

- [ ] 2.1 修改 `src/eval/builtin/color_hwb_hsl.rs` complement：输出使用输入 `c.space` + `ColorOutput::RgbPercent`
- [ ] 2.2 验证 complement HSL/RGB 输入的 sass-spec case

## 3. adjust-color HSL 通道输出格式

- [ ] 3.1 修改 `src/eval/builtin/color_adjust.rs` `adjust_legacy` HSL 分支：异构 RGB 输入走 `build_channel_modified_color` 路径输出 `RgbPercent`
- [ ] 3.2 确保同构 HSL 输入继续走 `build_channel_modified_color` 保持 HSL 输出
- [ ] 3.3 验证 adjust-color HSL 通道相关 sass-spec case

## 4. change-color 保留色彩空间

- [ ] 4.1 审查 `src/eval/builtin/color_change.rs` `change_hsl`：确保 `rgb_in_range` 判断不会错误地将 HSL 操作结果走 Auto(hex) 分支
- [ ] 4.2 如需要，修改 `change_hsl` 输出逻辑：HSL 操作结果优先保持 HSL space
- [ ] 4.3 验证 change-color HSL 相关 sass-spec case

## 5. scale-color 保留色彩空间

- [ ] 5.1 审查 `src/eval/builtin/color_scale.rs` `scale_color` HSL 分支：确保输出格式正确
- [ ] 5.2 如需要，修改 scale-color HSL 路径输出逻辑
- [ ] 5.3 验证 scale-color HSL 相关 sass-spec case

## 6. 全量验证

- [ ] 6.1 运行 `cargo test` 确保所有现有测试通过
- [ ] 6.2 运行 `SPEC_STORE_CMD=run cargo test --test spec_store` 创建新 snapshot
- [ ] 6.3 运行 `SPEC_STORE_CMD=stats cargo test --test spec_store` 对比通过率变化
- [ ] 6.4 确认无回归（对比前后 snapshot 的 delta）
