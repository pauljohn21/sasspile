//! @at-root 提升 + @media 合并测试
//!
//! 验证:
//!   - @at-root 将嵌套规则提升到顶层
//!   - 相同 @media query 合并为单个 AtRule

use sasspile::compile;

#[test]
fn at_root_hoists_to_top_level() {
    let input = ".parent {\n  @at-root .child {\n    color: red;\n  }\n}";
    let output = compile(input);
    // @at-root 应该让 .child 脱离 .parent, 直接输出为顶层 .child
    assert!(
        output.contains(".child"),
        "@at-root 应输出 .child, got: {output}"
    );
    // 不应有 .parent .child 嵌套
    assert!(
        !output.contains(".parent .child"),
        "@at-root 不应保留嵌套关系, got: {output}"
    );
}

#[test]
fn at_root_with_ampersand() {
    let input = ".btn {\n  @at-root &--primary {\n    color: blue;\n  }\n}";
    let output = compile(input);
    assert!(
        output.contains(".btn--primary"),
        "@at-root & 展开, got: {output}"
    );
}

#[test]
fn media_query_merge() {
    let input = "@media (max-width: 768px) {\n  .a {\n    color: red;\n  }\n}\n@media (max-width: 768px) {\n  .b {\n    color: blue;\n  }\n}";
    let output = compile(input);
    // 两个相同 @media 合并为一个
    let media_count = output.matches("@media (max-width: 768px)").count();
    assert_eq!(
        media_count, 1,
        "相同 @media 应合并为一个, got {media_count} occurrences: {output}"
    );
    // 两个内容都应存在
    assert!(output.contains(".a"), "应包含 .a, got: {output}");
    assert!(output.contains(".b"), "应包含 .b, got: {output}");
}

#[test]
fn media_different_queries_no_merge() {
    let input = "@media (max-width: 768px) {\n  .a {\n    color: red;\n  }\n}\n@media (min-width: 1024px) {\n  .b {\n    color: blue;\n  }\n}";
    let output = compile(input);
    // 不同 query 不合并
    assert!(
        output.contains("@media (max-width: 768px)"),
        "应保留 max-width query, got: {output}"
    );
    assert!(
        output.contains("@media (min-width: 1024px)"),
        "应保留 min-width query, got: {output}"
    );
}
