# Sass-Spec 测试结构

## 概述

sass-sass Spec（约 1306 个测试用例）以 HRX 格式组织，每个用例包含输入 SCSS 和期望输出 CSS。

## 目录结构

```
sass-spec-main/
├── spec/
│   ├── core_functions/
│   │   ├── color/
│   │   ├── list/
│   │   ├── map/
│   │   ├── math/
│   │   ├── meta/
│   │   ├── selector/
│   │   ├── string/
│   │   └── ...
│   ├── css/
│   ├── directives/
│   ├── expressions/
│   ├── interpolation/
│   ├── media/
│   ├── nesting/
│   ├── number/
│   ├── operations/
│   ├── output_style/
│   ├── parent_selector/
│   ├── variables/
│   └── ...
└── spec Lista
```

## 测试用例格式（HRX）

```
<===== input.scss
.foo {
  color: red;
}

<===== output.css
.foo {
  color: red;
}
```

## 命令规范化

```scss
// 输入使用
$var: 1 + 2 * 3;  // Sass 运算

// 输出
$var: 7;          // 预计算结果
```

## 跳过策略

创建 `css4_color_skip.rs` 集中的 skip 清单：

```rust
const CSS4_COLOR_SKIP: &[&str] = &[
    "spec/core_functions/color/oklch/",
    "spec/core_functions/color/oklab/",
    "spec/css/color/",
    // ... 462 个文件路径
];
```

## 测试运行器

```rust
pub struct SpecRunner {
    skip_list: HashSet<PathBuf>,
}

impl SpecRunner {
    pub async fn run_case(&self, path: &Path) -> Result<TestResult> { ... }
    pub async fn run_all(&self) -> TestReport { ... }
}
```

## 覆盖率统计

| 类别 | 用例数 | 状态 |
|------|--------|------|
| Core Functions | ~500 | 待测试 |
| CSS | ~150 | 待测试 |
| Directives | ~200 | 待测试 |
| Expressions | ~100 | 待测试 |
| Variables | ~80 | 待测试 |
| CSS4 Colors | ~462 | 跳过 |
| **总计** | **~1306** | - |
