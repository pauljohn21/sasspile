// ── Vendor Prefix Auto-Injection ───────────────────────────────────────
//
// 当声明使用特定 CSS 属性时，自动注入 vendor prefix 变体。
// Bootstrap dist 期望某些属性（如 transition、appearance）具有 -webkit-、-moz- 前缀。

/// 属性到其 vendor prefix 变体的映射。
/// 每个条目: (属性名, Vec<前缀属性名>)
/// 输出顺序: prefix 变体在前，原始属性在后（匹配 Bootstrap dist 约定）。
const PREFIX_MAP: &[(&str, &[&str])] = &[
    ("file-upload-button", &["-webkit-file-upload-button"]),
    ("column-gap", &["-moz-column-gap"]),
    ("object-fit", &["-o-object-fit"]),
    ("mask-position", &["-webkit-mask-position"]),
    ("transition", &["-webkit-transition", "-moz-transition"]),
    ("appearance", &["-webkit-appearance", "-moz-appearance"]),
];

/// 检查属性是否需要 vendor prefix 注入。
/// 返回需要生成的 prefix 变体列表（不含原始属性）。
pub fn get_vendor_prefixes(property: &str) -> Option<Vec<String>> {
    PREFIX_MAP
        .iter()
        .find(|(prop, _)| *prop == property)
        .map(|(_, prefixes)| prefixes.iter().map(|p| p.to_string()).collect())
}

/// 判断属性是否已带 vendor prefix。
pub fn is_prefixed(property: &str) -> bool {
    property.starts_with("-webkit-")
        || property.starts_with("-moz-")
        || property.starts_with("-o-")
        || property.starts_with("-ms-")
}
