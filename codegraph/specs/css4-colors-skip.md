# CSS 4.0 颜色跳过说明

## 跳过的特性

CSS Color Level 4 引入的新特性，本期不实现：

| 特性 | 用例数 | 示例 |
|------|--------|------|
| `oklch()` | ~80 | `oklch(50% 0.2 240)` |
| `oklab()` | ~80 | `oklab(50% 0.1 -0.2)` |
| `lch()` | ~40 | `lch(50% 50 240)` |
| `lab()` | ~40 | `lab(50% 20 -30)` |
| `color()` | ~60 | `color(display-p3 1 0.5 0)` |
| `color-mix()` | ~50 | `color-mix(in srgb, red 50%, blue)` |
| 相对颜色语法 | ~40 | `from red` |
| `light-dark()` | ~30 | `light-dark(red, blue)` |
| `hwb()` | ~20 | `hwb(120 20% 30%)` |

**总计：约 462 个文件**

## 保留接口

在 `value-system/color.rs` 中预留扩展点：

```rust
pub enum ColorSpace {
    SRgb,
    // 预留未来扩展
    // OKLch,
    // OKLab,
    // DisplayP3,
    // Rec2020,
}

#[allow(dead_code)]
pub struct Color4 {
    pub space: ColorSpace,
    pub components: Vec<f64>,
    pub alpha: f64,
}
```

## 启用条件

- sass-spec 主流用例 100% 通过后
- `unstable_css4_colors` feature flag
- 独立分支开发

## 跳过清单位置

- `css4_color_skip.rs`：集中管理跳过文件路径
- spec/ 中的子目录可批量跳过

## 运行时行为

遇到 CSS 4.0 颜色语法时：
1. 解析为 `Value::Error("CSS 4.0 colors not supported")`
2. 跳过该测试用例
3. 记录统计信息
