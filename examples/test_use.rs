use sasspile::compile_with_files;
use std::collections::HashMap;

fn test(name: &str, input: &str, files: &HashMap<String, String>, expected: &str) -> bool {
    println!("=== {} ===", name);
    let result = compile_with_files(input, files);
    let result_t = result.trim();
    let expected_t = expected.trim();
    let passed = result_t == expected_t;
    println!("[{}]", if passed { "PASS" } else { "FAIL" });
    if !passed {
        println!("expected:\n{}", expected_t);
        println!("actual:\n{}", result_t);
    }
    println!();
    passed
}

fn main() {
    let mut all_pass = true;

    // Test 1: forward_only
    {
        let mut f = HashMap::new();
        f.insert("_other.scss".to_string(), "a {b: c}".to_string());
        all_pass &= test("forward_only", "@forward \"other\";", &f, "a {\n  b: c;\n}");
    }

    // Test 2: with/single
    {
        let mut f = HashMap::new();
        f.insert(
            "_other.scss".to_string(),
            "$a: original !default;\nb {c: $a}".to_string(),
        );
        all_pass &= test(
            "use/with/single",
            "@use \"other\" with ($a: configured);",
            &f,
            "b {\n  c: configured;\n}",
        );
    }

    // Test 3: variable_use namespaced
    {
        let mut f = HashMap::new();
        f.insert("other.scss".to_string(), "$member: value;".to_string());
        all_pass &= test(
            "variable_use",
            "@use \"other\";\na {b: other.$member}",
            &f,
            "a {\n  b: value;\n}",
        );
    }

    // Test 4: function namespaced
    {
        let mut f = HashMap::new();
        f.insert(
            "other.scss".to_string(),
            "@function member() {@return value}".to_string(),
        );
        all_pass &= test(
            "function_use",
            "@use \"other\";\na {b: other.member()}",
            &f,
            "a {\n  b: value;\n}",
        );
    }

    // Test 5: mixin namespaced
    {
        let mut f = HashMap::new();
        f.insert(
            "other.scss".to_string(),
            "@mixin member {a {b: c}}".to_string(),
        );
        all_pass &= test(
            "mixin_use",
            "@use \"other\";\n@include other.member;",
            &f,
            "a {\n  b: c;\n}",
        );
    }

    // Test 6: indirect forward
    {
        let mut f = HashMap::new();
        f.insert("_upstream.scss".to_string(), "$c: d;".to_string());
        f.insert(
            "_midstream.scss".to_string(),
            "@forward \"upstream\";".to_string(),
        );
        all_pass &= test(
            "indirect/use",
            "@use \"midstream\";\na {b: midstream.$c}",
            &f,
            "a {\n  b: d;\n}",
        );
    }

    println!("\n===ALL {}===", if all_pass { "PASS" } else { "FAIL" });
}
