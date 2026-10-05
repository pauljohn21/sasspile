//! —— selector_combine 纯函数单元测试 ——

use sasspile::eval::selector_combine::{
    combine_selectors, has_descendant_prefix, split_selectors_respecting_parens,
};

// —— split_selectors_respecting_parens ——

#[test]
fn test_split_basic_comma() {
    let result = split_selectors_respecting_parens(".a, .b, .c");
    assert_eq!(result, vec![".a", ".b", ".c"]);
}

#[test]
fn test_split_parens_internal_comma() {
    // :not(.a, .b) 内部的逗号不应分割
    let result = split_selectors_respecting_parens(":not(.a, .b)");
    assert_eq!(result, vec![":not(.a, .b)"]);
}

#[test]
fn test_split_nested_parens() {
    // :is(:not(.a, .b), .c) — 双层嵌套
    let result = split_selectors_respecting_parens(":is(:not(.a, .b), .c)");
    assert_eq!(result, vec![":is(:not(.a, .b), .c)"]);
}

#[test]
fn test_split_empty() {
    let result = split_selectors_respecting_parens("");
    assert!(result.is_empty());
}

#[test]
fn test_split_whitespace_only() {
    let result = split_selectors_respecting_parens("  ");
    assert!(result.is_empty());
}

#[test]
fn test_split_trailing_comma() {
    let result = split_selectors_respecting_parens(".a, .b,");
    assert_eq!(result, vec![".a", ".b"]);
}

// —— combine_selectors ——

#[test]
fn test_combine_basic_descendant() {
    assert_eq!(combine_selectors(".parent", ".child"), ".parent .child");
}

#[test]
fn test_combine_ampersand_replacement() {
    assert_eq!(combine_selectors(".parent", "&.child"), ".parent.child");
}

#[test]
fn test_combine_multiple_parents() {
    let result = combine_selectors(".a, .b", ".x");
    assert_eq!(result, ".a .x, .b .x");
}

#[test]
fn test_combine_multiple_children() {
    let result = combine_selectors(".p", ".x, .y");
    assert_eq!(result, ".p .x, .p .y");
}

#[test]
fn test_combine_cartesian_product() {
    let result = combine_selectors(".a, .b", ".x, .y");
    assert_eq!(result, ".a .x, .a .y, .b .x, .b .y");
}

#[test]
fn test_combine_ampersand_cartesian_product() {
    let result = combine_selectors(".a, .b", "&-x, &-y");
    assert_eq!(result, ".a-x, .a-y, .b-x, .b-y");
}

#[test]
fn test_combine_empty_parent() {
    assert_eq!(combine_selectors("", ".child"), ".child");
}

#[test]
fn test_combine_empty_child() {
    assert_eq!(combine_selectors(".parent", ""), ".parent");
}

#[test]
fn test_combine_both_empty() {
    assert_eq!(combine_selectors("", ""), "");
}

#[test]
fn test_combine_child_with_ampersand_empty_parent() {
    assert_eq!(combine_selectors("", "&.x"), "&.x");
}

// —— has_descendant_prefix ——

#[test]
fn test_has_prefix_compound_dot() {
    // .el-button.el-button--large — . 是分隔符，compound 前缀
    assert!(has_descendant_prefix(".el-button", ".el-button.el-button--large"));
}

#[test]
fn test_has_prefix_compound_hash() {
    // .el-button#id — # 是分隔符
    assert!(has_descendant_prefix(".el-button", ".el-button#id"));
}

#[test]
fn test_has_prefix_compound_pseudo() {
    // .el-button:hover — : 是分隔符
    assert!(has_descendant_prefix(".el-button", ".el-button:hover"));
}

#[test]
fn test_has_prefix_descendant() {
    // .el-button--large .el-button__inner — descendant 前缀
    assert!(has_descendant_prefix(".el-button--large", ".el-button--large .el-button__inner"));
}

#[test]
fn test_has_prefix_no_match() {
    assert!(!has_descendant_prefix(".el-button", ".el-input--large"));
}

#[test]
fn test_has_prefix_exact_match() {
    // parent == child 时，starts_with_compound_prefix 返回 true（child IS parent，无需再组合）
    // 这是正确的语义：child 已等于 parent，不需要 descendant combine
    assert!(has_descendant_prefix(".el-button", ".el-button"));
}

#[test]
fn test_has_prefix_empty_parent() {
    assert!(!has_descendant_prefix("", ".anything"));
}

#[test]
fn test_has_prefix_empty_child() {
    assert!(!has_descendant_prefix(".parent", ""));
}

#[test]
fn test_has_prefix_partial_classname_no_match() {
    // .el-butto 不等于 .el-button（class 名部分匹配不应算前缀）
    assert!(!has_descendant_prefix(".el-button", ".el-buttons"));
}
