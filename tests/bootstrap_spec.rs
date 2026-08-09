//! Bootstrap 5.3.8 验证测试。
//!
/// 测试 Bootstrap 使用的核心 SCSS 特性。
/// 注意：完整 Bootstrap 编译需要 @import、@mixin、父选择器 & 等高级特性支持。
/// 当前版本会将 hex 颜色转换为 rgba() 格式。

/// 检查 CSS 是否包含颜色（支持 hex 和 rgba 格式）。
fn assert_contains_color(css: &str, hex: &str) {
    // 将 hex 转换为 rgba 进行比较
    let rgba = hex_to_rgba(hex);
    assert!(
        css.contains(hex) || css.contains(&rgba),
        "CSS 中未找到颜色 {hex} 或 {rgba}:\n{css}"
    );
}

/// 将 hex 颜色转换为 rgba 字符串。
fn hex_to_rgba(hex: &str) -> String {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
        format!("rgba({r}, {g}, {b}, 1)")
    } else {
        hex.to_string()
    }
}

#[test]
fn test_bootstrap_variables() {
    // Bootstrap 风格的变量定义
    let scss = r##"
$primary: #0d6efd;
$secondary: #6c757d;
$success: #198754;

.btn-primary {
    background-color: $primary;
}

.btn-secondary {
    background-color: $secondary;
}
"##;

    let css = sasspile::compile_expanded(scss).unwrap();
    assert_contains_color(&css, "#0d6efd");
    assert_contains_color(&css, "#6c757d");
}

#[test]
fn test_bootstrap_nesting() {
    // Bootstrap 风格的选择器嵌套
    let scss = r##"
.navbar {
    display: flex;
    align-items: center;

    .navbar-brand {
        font-size: 1.25rem;
        font-weight: 600;
    }

    .navbar-nav {
        display: flex;
        gap: 1rem;

        .nav-link {
            color: rgba(0, 0, 0, 0.55);
        }
    }
}
"##;

    let css = sasspile::compile_expanded(scss).unwrap();
    assert!(css.contains(".navbar .navbar-brand"));
    assert!(css.contains(".navbar .navbar-nav"));
    assert!(css.contains(".navbar .navbar-nav .nav-link"));
}

#[test]
fn test_bootstrap_math_operations() {
    // Bootstrap 使用的数学运算
    let scss = r##"
$spacer: 1rem;

.mt-1 { margin-top: $spacer * 0.25; }
.mt-2 { margin-top: $spacer * 0.5; }
.mt-3 { margin-top: $spacer; }
.p-1 { padding: $spacer * 0.25; }
.p-2 { padding: $spacer * 0.5; }
.p-3 { padding: $spacer; }
"##;

    let css = sasspile::compile_expanded(scss).unwrap();
    assert!(css.contains("margin-top: 0.25rem"));
    assert!(css.contains("margin-top: 0.5rem"));
    assert!(css.contains("margin-top: 1rem"));
    assert!(css.contains("padding: 0.25rem"));
}

#[test]
fn test_bootstrap_media_queries() {
    // Bootstrap 使用的媒体查询
    let scss = r##"
.container {
    width: 100%;
    padding-right: 0.75rem;
    padding-left: 0.75rem;
    margin-right: auto;
    margin-left: auto;
}

@media (min-width: 576px) {
    .container { max-width: 540px; }
}

@media (min-width: 768px) {
    .container { max-width: 720px; }
}

@media (min-width: 992px) {
    .container { max-width: 960px; }
}
"##;

    let css = sasspile::compile_expanded(scss).unwrap();
    assert!(css.contains("@media"));
    assert!(css.contains("min-width: 576px"));
    assert!(css.contains("max-width: 540px"));
}

#[test]
fn test_bootstrap_grid_system() {
    // Bootstrap 网格系统基础
    let scss = r##"
.row {
    display: flex;
    flex-wrap: wrap;
    margin-top: -0.5rem;
    margin-right: -0.75rem;
    margin-left: -0.75rem;
}

.col { flex: 1 0 0%; }
.col-auto { flex: 0 0 auto; width: auto; }
"##;

    let css = sasspile::compile_expanded(scss).unwrap();
    assert!(css.contains("display: flex"));
    assert!(css.contains("flex-wrap: wrap"));
    assert!(css.contains("flex: 1 0 0%"));
}

#[test]
fn test_bootstrap_utility_api() {
    // Bootstrap 工具类 API 风格
    let scss = r##"
.w-25 { width: 25%; }
.w-50 { width: 50%; }
.w-75 { width: 75%; }
.w-100 { width: 100%; }
.w-auto { width: auto; }
"##;

    let css = sasspile::compile_expanded(scss).unwrap();
    assert!(css.contains("width: 25%"));
    assert!(css.contains("width: 50%"));
    assert!(css.contains("width: 75%"));
    assert!(css.contains("width: 100%"));
    assert!(css.contains("width: auto"));
}

#[test]
fn test_bootstrap_core_css_output() {
    // 模拟 Bootstrap 的 _reboot.scss 部分功能
    let scss = r##"
*,
*::before,
*::after {
    box-sizing: border-box;
}

body {
    margin: 0;
    font-family: system-ui, -apple-system, "Segoe UI", Roboto, sans-serif;
    font-size: 1rem;
    font-weight: 400;
    line-height: 1.5;
    color: #212529;
    background-color: #fff;
}

h1, h2, h3, h4, h5, h6 {
    margin-top: 0;
    margin-bottom: 0.5rem;
    font-weight: 500;
    line-height: 1.2;
}

h1 { font-size: 2.5rem; }
h2 { font-size: 2rem; }
h3 { font-size: 1.75rem; }
h4 { font-size: 1.5rem; }
h5 { font-size: 1.25rem; }
h6 { font-size: 1rem; }

p {
    margin-top: 0;
    margin-bottom: 1rem;
}

a {
    color: #0d6efd;
    text-decoration: underline;
}

img {
    vertical-align: middle;
}

table {
    caption-side: bottom;
    border-collapse: collapse;
}
"##;

    let css = sasspile::compile_expanded(scss).unwrap();
    assert!(css.contains("box-sizing: border-box"));
    assert!(css.contains("font-family:"));
    assert!(css.contains("font-size: 2.5rem"));
    assert!(css.contains("margin-top: 0"));
    assert_contains_color(&css, "#212529");
    assert_contains_color(&css, "#0d6efd");
}

#[test]
fn test_bootstrap_color_values() {
    // Bootstrap 颜色值格式
    let scss = r##"
.text-primary { color: #0d6efd; }
.text-secondary { color: #6c757d; }
.text-success { color: #198754; }
.text-danger { color: #dc3545; }
.text-warning { color: #ffc107; }
.text-info { color: #0dcaf0; }
.text-light { color: #f8f9fa; }
.text-dark { color: #212529; }

.bg-primary { background-color: #0d6efd; }
.bg-secondary { background-color: #6c757d; }
.bg-success { background-color: #198754; }
"##;

    let css = sasspile::compile_expanded(scss).unwrap();
    assert_contains_color(&css, "#0d6efd");
    assert_contains_color(&css, "#6c757d");
    assert_contains_color(&css, "#198754");
    assert_contains_color(&css, "#dc3545");
    assert_contains_color(&css, "#ffc107");
    assert_contains_color(&css, "#0dcaf0");
    assert_contains_color(&css, "#f8f9fa");
    assert_contains_color(&css, "#212529");
}

#[test]
fn test_bootstrap_spacing_system() {
    // Bootstrap 间距系统
    let scss = r##"
.m-0 { margin: 0; }
.m-1 { margin: 0.25rem; }
.m-2 { margin: 0.5rem; }
.m-3 { margin: 1rem; }
.m-4 { margin: 1.5rem; }
.m-5 { margin: 3rem; }

.p-0 { padding: 0; }
.p-1 { padding: 0.25rem; }
.p-2 { padding: 0.5rem; }
.p-3 { padding: 1rem; }
.p-4 { padding: 1.5rem; }
.p-5 { padding: 3rem; }

.gap-0 { gap: 0; }
.gap-1 { gap: 0.25rem; }
.gap-2 { gap: 0.5rem; }
.gap-3 { gap: 1rem; }
"##;

    let css = sasspile::compile_expanded(scss).unwrap();
    assert!(css.contains("margin: 0"));
    assert!(css.contains("margin: 0.25rem"));
    assert!(css.contains("padding: 1rem"));
    assert!(css.contains("gap: 0.5rem"));
}

#[test]
fn test_bootstrap_border_utilities() {
    // Bootstrap 边框工具类
    let scss = r##"
.border { border: 1px solid #dee2e6; }
.border-0 { border: 0; }
.border-top { border-top: 1px solid #dee2e6; }
.border-end { border-right: 1px solid #dee2e6; }
.border-bottom { border-bottom: 1px solid #dee2e6; }
.border-start { border-left: 1px solid #dee2e6; }

.rounded { border-radius: 0.375rem; }
.rounded-0 { border-radius: 0; }
.rounded-1 { border-radius: 0.25rem; }
.rounded-2 { border-radius: 0.375rem; }
.rounded-3 { border-radius: 0.5rem; }
.rounded-circle { border-radius: 50%; }
.rounded-pill { border-radius: 50rem; }
"##;

    let css = sasspile::compile_expanded(scss).unwrap();
    assert!(css.contains("border: 1px solid"));
    assert!(css.contains("border: 0"));
    assert!(css.contains("border-radius: 0.375rem"));
    assert!(css.contains("border-radius: 50%"));
    assert!(css.contains("border-radius: 50rem"));
}

#[test]
fn test_bootstrap_display_utilities() {
    // Bootstrap 显示工具类
    let scss = r##"
.d-none { display: none; }
.d-inline { display: inline; }
.d-inline-block { display: inline-block; }
.d-block { display: block; }
.d-grid { display: grid; }
.d-table { display: table; }
.d-table-row { display: table-row; }
.d-table-cell { display: table-cell; }
.d-flex { display: flex; }
.d-inline-flex { display: inline-flex; }
"##;

    let css = sasspile::compile_expanded(scss).unwrap();
    assert!(css.contains("display: none"));
    assert!(css.contains("display: inline"));
    assert!(css.contains("display: block"));
    assert!(css.contains("display: flex"));
}

#[test]
fn test_bootstrap_flex_utilities() {
    // Bootstrap Flex 工具类
    let scss = r##"
.flex-row { flex-direction: row; }
.flex-column { flex-direction: column; }
.flex-row-reverse { flex-direction: row-reverse; }
.flex-column-reverse { flex-direction: column-reverse; }

.justify-content-start { justify-content: flex-start; }
.justify-content-end { justify-content: flex-end; }
.justify-content-center { justify-content: center; }
.justify-content-between { justify-content: space-between; }
.justify-content-around { justify-content: space-around; }
.justify-content-evenly { justify-content: space-evenly; }

.align-items-start { align-items: flex-start; }
.align-items-end { align-items: flex-end; }
.align-items-center { align-items: center; }
.align-items-baseline { align-items: baseline; }
.align-items-stretch { align-items: stretch; }
"##;

    let css = sasspile::compile_expanded(scss).unwrap();
    assert!(css.contains("flex-direction: row"));
    assert!(css.contains("flex-direction: column"));
    assert!(css.contains("justify-content: center"));
    assert!(css.contains("justify-content: space-between"));
    assert!(css.contains("align-items: center"));
}

/// Bootstrap 兼容性总结测试。
#[test]
fn test_bootstrap_compatibility_summary() {
    let scss = r##"
// Bootstrap 5.3.8 核心语法测试
$primary: #0d6efd;

// 变量
.text-primary { color: $primary; }

// 嵌套
.nav {
    display: flex;
    .nav-item { margin-right: 1rem; }
    .nav-link { color: $primary; }
}

// 媒体查询
@media (min-width: 768px) {
    .container { max-width: 720px; }
}

// 工具类
.d-flex { display: flex; }
.p-3 { padding: 1rem; }
.m-2 { margin: 0.5rem; }
.bg-primary { background-color: $primary; }
.border { border: 1px solid #dee2e6; }
.rounded { border-radius: 0.375rem; }
"##;

    let css = sasspile::compile_expanded(scss).unwrap();
    assert_contains_color(&css, "#0d6efd");
    assert!(css.contains(".nav .nav-item"));
    assert!(css.contains(".nav .nav-link"));
    assert!(css.contains("@media"));
    assert!(css.contains("display: flex"));
    assert!(css.contains("padding: 1rem"));
    assert!(css.contains("margin: 0.5rem"));
    assert!(css.contains("border: 1px solid"));
    assert!(css.contains("border-radius: 0.375rem"));
}
