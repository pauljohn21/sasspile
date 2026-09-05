## 1. color/to_space 失败诊断与修复

- [ ] 1.1 分析 color/to_space 失败模式：抽样 5 个失败用例提取 trace evidence，确定是解析失败还是序列化失败
- [ ] 1.2 修复 `color()` 函数解析：在 `color_conv.rs` 中补充 `color(srgb/display-p3/lab/lch/oklab/oklch/xyz/xyz-d50)` 语法解析分支
- [ ] 1.3 修复 `to-space` 输出格式：确保跨空间转换后输出 `color(target-space ...)` 格式字符串
- [ ] 1.4 运行 `cargo test --test color_algorithm_test` 确认无回退

## 2. color/scale 算法修复

- [ ] 2.1 分析 scale 失败模式：对比 sass-spec 期望值，确认是公式错误还是 clamp 边界错误
- [ ] 2.2 修正 `color-scale` 算法：采用 `new = current + direction * (|max - current| * percent/100)` 公式
- [ ] 2.3 扩展 scale 支持 oklab/lch/oklab 色彩空间的通道识别
- [ ] 2.4 运行 `cargo test --test color_algorithm_test` 确认无回退

## 3. color/change 修复

- [ ] 3.1 分析 change 失败模式：确认是参数校验缺失还是现代空间通道不支持
- [ ] 3.2 修复 `color-change` 参数类型校验：`<channel> requires a number` 错误消息
- [ ] 3.3 扩展 change 支持 oklab/lab/lch/oklch 色彩空间的通道设置
- [ ] 3.4 运行 `cargo test --test color_algorithm_test` 确认无回退

## 4. color/adjust + mix + hsl 修复

- [ ] 4.1 分析 adjust/mix/hsl 失败模式：确认是输出格式还是精度问题
- [ ] 4.2 修复 `color-adjust` 增量计算的通道 clamp 行为
- [ ] 4.3 修复 `mix` 权重边界（0%/100%/默认 50%）
- [ ] 4.4 修复 HSL/HWB 函数创建的颜色保持原始格式输出（不降级为 hex）
- [ ] 4.5 运行 `cargo test --test color_algorithm_test` 确认无回退

## 5. directives/import + function 修复

- [ ] 5.1 分析 @import 失败模式：确认是路径解析还是源映射问题
- [ ] 5.2 修复 @import 嵌套导入链的解析逻辑
- [ ] 5.3 修复 @function 返回值类型处理（空值/列表/map）
- [ ] 5.4 运行 `cargo test --test compile_test` 确认无回退

## 6. 全量回归验证

- [ ] 6.1 运行 `cargo test --test compile_test --test stage_test --test ast_test --test common_test` 确认 202/202
- [ ] 6.2 运行 `cargo test --test bs_spec --test ep_full` 确认 136/136
- [ ] 6.3 运行 `RUST_LOG="sass_spec_full=info,sasspile=warn" cargo test --test sass_spec_full -- --nocapture` 确认通过率提升
- [ ] 6.4 更新 AGENTS.md 基线统计数字
