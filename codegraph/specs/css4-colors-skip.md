# CSS4 颜色特性跳过

## 背景

CSS Color Module Level 4 和 5 引入了新的颜色空间和函数：
- `lab()`, `lch()`, `oklab()`, `oklch()`
- `color()` 函数
- `hwb()`, `color-mix()`, `color-contrast()`
- 广色域 P3 支持

这些特性在 sass-spec 中占 462 个文件，**本阶段全部跳过**。

## 跳过策略

在测试运行器中：
```rust
const CSS4_COLOR_DIRS: &[&str] = &[
    "spec/css/colors/",
    "spec/css/css-color/",
    "spec/basic/color/",
    // ...
];

fn should_skip(path: &str) -> bool {
    CSS4_COLOR_DIRS.iter().any(|dir| path.starts_with(dir))
}
```

## 预留接口

`SassColor` 类型设计时预留扩展位：
```rust
pub struct SassColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: f64,
    // 未来添加：color_space: ColorSpace
}
```

## 未来适配

Phase 10 计划：
1. 扩展 SassColor 支持 lab/lch/oklab/oklch
2. 实现 `color()` 解析
3. 实现 color-mix/contrast
4. 重新启用跳过的 462 个测试
