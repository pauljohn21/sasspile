//! Debug test for mixin parsing

use sasspile::directive::member_parse::parse_module_members;

#[test]
fn test_parse_multiline_mixin_sass() {
    let content = "@mixin foo\n  c\n    d: e\n";
    println!("Input lines: {:?}", content.lines().collect::<Vec<_>>());
    let members = parse_module_members(content);
    println!("=== parse multiline mixin sass ===");
    println!("mixins: {:?}", members.mixins);
    println!("raw_rules: {:?}", members.raw_rules);
}

#[test]
fn test_parse_multiline_mixin_scss() {
    let content = "@mixin foo {\n  c {\n    d: e\n  }\n}\n";
    println!("Input lines: {:?}", content.lines().collect::<Vec<_>>());
    let members = parse_module_members(content);
    println!("=== parse multiline mixin scss ===");
    println!("mixins: {:?}", members.mixins);
    println!("raw_rules: {:?}", members.raw_rules);
}
