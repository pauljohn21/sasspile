## 1. 构造函数注册

- [ ] 1.1 在 `manual_dispatch.rs` 中为 `lab` / `lch` / `oklch` 添加分派（与 `oklab` 对齐）
- [ ] 1.2 验证 `parse_color_fn` 对空格分隔和斜杠分隔参数的正确展开
- [ ] 1.3 测试 `lab(50% 20 30)` / `lch(50% 30 180deg)` / `oklch(50% 0.1 180deg)` 构造

## 2. 序列化精度

- [ ] 2.1 扩展 `display.rs` 以支持 Lab/Lch/Oklab/Oklch 精确输出（channels 保留 ~10 位有效数字）
- [ ] 2.2 修复 `format_num` 以正确处理负零和小于 1 的浮点数
- [ ] 2.3 确保 `none` 通道输出正确（NaN → `"none"`）
- [ ] 2.4 验证 chroma=0 时 hue 输出为 `none`（Lch/Oklch）

## 3. color.channel 支持

- [ ] 3.1 验证 `ChannelSet::from_str` 对 Lab/Lch/Oklab/Oklch 通道名解析
- [ ] 3.2 测试 `color.channel($color, 'chroma')` 对 Lch/Oklch
- [ ] 3.3 验证 `color.channel` 输出 NaN 通道时返回 `none`

## 4. color.to-space 支持

- [ ] 4.1 验证 `color_conv_ops.rs` 中 lab/lch/oklab/oklch 与其他空间的转换
- [ ] 4.2 测试 `color.to-space(lab(...), srgb)` 输出正确的 sRGB 值
- [ ] 4.3 测试 `color.to-space(red, oklab)` 输出正确的 Oklab 值
- [ ] 4.4 验证转换链：Lch -> Lab -> XYZ -> sRGB -> Oklab -> Oklch

## 5. 测试工具链修复

- [x] 5.1 修复 `test_directives_subdirs` 的 `.sass` 文件过滤
- [x] 5.2 确保 sass-spec `core_functions/color/utils` 共享模块可被发现
- [x] 5.3 在 compile_test 中添加 lab/lch/oklab/oklch 构造和序列化测试

## 6. 验证

- [x] 6.1 运行 `cargo test --test compile_test` 确认 57/57 通过（新增 14 个色彩空间测试）
- [x] 6.2 运行 `cargo test --test sass_spec_full test_sass_spec_full_stats` 记录颜色通过率
- [x] 6.3 运行 `cargo test --test bs_spec` 确认 15/15 通过（Bootstrap 不依赖现代颜色）
- [x] 6.4 运行完整核心测试套件 95/95 确认无回归（含新增 compile 测试）
