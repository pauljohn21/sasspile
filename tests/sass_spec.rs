//! sass-spec 合规测试框架。
//!
//! 读取 HRX 格式的测试用例，编译并验证输出。

use std::path::Path;

/// 解析 HRX 文件内容，提取输入和期望输出。
fn parse_hrx(content: &str) -> Vec<(String, String, String)> {
    let mut cases = Vec::new();
    let mut current_name = String::new();
    let mut current_input = String::new();
    let mut current_output = String::new();
    let mut section = "";

    for line in content.lines() {
        if line.starts_with("<===>") {
            // 保存上一个 case
            if !current_name.is_empty() && !current_input.is_empty() {
                cases.push((
                    current_name.clone(),
                    current_input.clone(),
                    current_output.clone(),
                ));
            }
            // 新 case
            section = line.trim_start_matches("<===>").trim();
            match section {
                "input.scss" => {
                    current_input = String::new();
                }
                "output.css" => {
                    current_output = String::new();
                }
                _ => {
                    // case 名称或其他
                    if !section.contains('.') {
                        current_name = section.to_string();
                    }
                }
            }
        } else {
            match section {
                "input.scss" => {
                    current_input.push_str(line);
                    current_input.push('\n');
                }
                "output.css" => {
                    current_output.push_str(line);
                    current_output.push('\n');
                }
                _ => {}
            }
        }
    }

    // 保存最后一个 case
    if !current_name.is_empty() && !current_input.is_empty() {
        cases.push((
            current_name,
            current_input,
            current_output,
        ));
    }

    cases
}

/// 运行单个 sass-spec 测试用例。
fn run_spec_case(name: &str, input: &str, expected_output: &str) -> Result<(), String> {
    let actual = sasspile::compile_expanded(input)
        .map_err(|e| format!("编译失败 [{name}]: {e}"))?;

    // 标准化空白后比较
    let actual_trimmed = actual.trim();
    let expected_trimmed = expected_output.trim();

    if actual_trimmed != expected_trimmed {
        return Err(format!(
            "输出不匹配 [{name}]:\n期望:\n{expected_trimmed}\n实际:\n{actual_trimmed}"
        ));
    }

    Ok(())
}

/// 运行目录中的所有 HRX 测试文件。
fn run_spec_dir(dir: &Path) -> (usize, usize, Vec<String>) {
    let mut passed = 0;
    let mut failed = 0;
    let mut failures = Vec::new();

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("hrx") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    let cases = parse_hrx(&content);
                    for (name, input, output) in cases {
                        let case_name = format!(
                            "{}/{}",
                            path.file_stem().unwrap().to_string_lossy(),
                            name
                        );
                        match run_spec_case(&case_name, &input, &output) {
                            Ok(()) => passed += 1,
                            Err(e) => {
                                failed += 1;
                                failures.push(e);
                            }
                        }
                    }
                }
            }
        }
    }

    (passed, failed, failures)
}

#[test]
fn test_variables_basic() {
    let spec_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../sass-spec-main/spec/variables");
    let (passed, failed, failures) = run_spec_dir(&spec_dir);

    if failed > 0 {
        eprintln!("变量测试: {passed} 通过, {failed} 失败");
        for f in &failures {
            eprintln!("{f}");
        }
    }
    assert_eq!(failed, 0, "{failed} 个变量测试失败");
}

#[test]
fn test_values_numbers() {
    let spec_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../sass-spec-main/spec/values/numbers");
    let (passed, failed, failures) = run_spec_dir(&spec_dir);

    if failed > 0 {
        eprintln!("数值测试: {passed} 通过, {failed} 失败");
        for f in &failures {
            eprintln!("{f}");
        }
    }
    assert_eq!(failed, 0, "{failed} 个数值测试失败");
}

#[test]
fn test_values_colors() {
    let spec_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../sass-spec-main/spec/values/colors");
    let (passed, failed, failures) = run_spec_dir(&spec_dir);

    if failed > 0 {
        eprintln!("颜色测试: {passed} 通过, {failed} 失败");
        for f in &failures {
            eprintln!("{f}");
        }
    }
    assert_eq!(failed, 0, "{failed} 个颜色测试失败");
}

#[test]
fn test_values_strings() {
    let spec_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../sass-spec-main/spec/values/strings.hrx");
    let (passed, failed, failures) = run_spec_dir(&spec_dir);

    if failed > 0 {
        eprintln!("字符串测试: {passed} 通过, {failed} 失败");
        for f in &failures {
            eprintln!("{f}");
        }
    }
    assert_eq!(failed, 0, "{failed} 个字符串测试失败");
}

#[test]
fn test_values_lists() {
    let spec_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../sass-spec-main/spec/values/lists");
    let (passed, failed, failures) = run_spec_dir(&spec_dir);

    if failed > 0 {
        eprintln!("列表测试: {passed} 通过, {failed} 失败");
        for f in &failures {
            eprintln!("{f}");
        }
    }
    assert_eq!(failed, 0, "{failed} 个列表测试失败");
}

#[test]
fn test_values_maps() {
    let spec_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../sass-spec-main/spec/values/maps");
    let (passed, failed, failures) = run_spec_dir(&spec_dir);

    if failed > 0 {
        eprintln!("Map 测试: {passed} 通过, {failed} 失败");
        for f in &failures {
            eprintln!("{f}");
        }
    }
    assert_eq!(failed, 0, "{failed} 个 Map 测试失败");
}

#[test]
fn test_css_basic() {
    let spec_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../sass-spec-main/spec/css");
    let (passed, failed, failures) = run_spec_dir(&spec_dir);

    if failed > 0 {
        eprintln!("CSS 测试: {passed} 通过, {failed} 失败");
        for f in &failures {
            eprintln!("{f}");
        }
    }
    assert_eq!(failed, 0, "{failed} 个 CSS 测试失败");
}

#[test]
fn test_core_math() {
    let spec_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../sass-spec-main/spec/core_functions/math");
    let (passed, failed, failures) = run_spec_dir(&spec_dir);

    if failed > 0 {
        eprintln!("Math 测试: {passed} 通过, {failed} 失败");
        for f in &failures {
            eprintln!("{f}");
        }
    }
    assert_eq!(failed, 0, "{failed} 个 Math 测试失败");
}

#[test]
fn test_core_color() {
    let spec_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../sass-spec-main/spec/core_functions/color");
    let (passed, failed, failures) = run_spec_dir(&spec_dir);

    if failed > 0 {
        eprintln!("Color 测试: {passed} 通过, {failed} 失败");
        for f in &failures {
            eprintln!("{f}");
        }
    }
    assert_eq!(failed, 0, "{failed} 个 Color 测试失败");
}

#[test]
fn test_core_string() {
    let spec_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../sass-spec-main/spec/core_functions/string");
    let (passed, failed, failures) = run_spec_dir(&spec_dir);

    if failed > 0 {
        eprintln!("String 测试: {passed} 通过, {failed} 失败");
        for f in &failures {
            eprintln!("{f}");
        }
    }
    assert_eq!(failed, 0, "{failed} 个 String 测试失败");
}

#[test]
fn test_core_list() {
    let spec_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../sass-spec-main/spec/core_functions/list");
    let (passed, failed, failures) = run_spec_dir(&spec_dir);

    if failed > 0 {
        eprintln!("List 测试: {passed} 通过, {failed} 失败");
        for f in &failures {
            eprintln!("{f}");
        }
    }
    assert_eq!(failed, 0, "{failed} 个 List 测试失败");
}

#[test]
fn test_core_map() {
    let spec_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../sass-spec-main/spec/core_functions/map");
    let (passed, failed, failures) = run_spec_dir(&spec_dir);

    if failed > 0 {
        eprintln!("Map 测试: {passed} 通过, {failed} 失败");
        for f in &failures {
            eprintln!("{f}");
        }
    }
    assert_eq!(failed, 0, "{failed} 个 Map 测试失败");
}

#[test]
fn test_directives_if() {
    let spec_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../sass-spec-main/spec/directives/if");
    let (passed, failed, failures) = run_spec_dir(&spec_dir);

    if failed > 0 {
        eprintln!("@if 测试: {passed} 通过, {failed} 失败");
        for f in &failures {
            eprintln!("{f}");
        }
    }
    assert_eq!(failed, 0, "{failed} 个 @if 测试失败");
}

#[test]
fn test_directives_for() {
    let spec_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../sass-spec-main/spec/directives/for");
    let (passed, failed, failures) = run_spec_dir(&spec_dir);

    if failed > 0 {
        eprintln!("@for 测试: {passed} 通过, {failed} 失败");
        for f in &failures {
            eprintln!("{f}");
        }
    }
    assert_eq!(failed, 0, "{failed} 个 @for 测试失败");
}

#[test]
fn test_directives_each() {
    let spec_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../sass-spec-main/spec/directives/each");
    let (passed, failed, failures) = run_spec_dir(&spec_dir);

    if failed > 0 {
        eprintln!("@each 测试: {passed} 通过, {failed} 失败");
        for f in &failures {
            eprintln!("{f}");
        }
    }
    assert_eq!(failed, 0, "{failed} 个 @each 测试失败");
}

#[test]
fn test_directives_while() {
    let spec_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../sass-spec-main/spec/directives/while");
    let (passed, failed, failures) = run_spec_dir(&spec_dir);

    if failed > 0 {
        eprintln!("@while 测试: {passed} 通过, {failed} 失败");
        for f in &failures {
            eprintln!("{f}");
        }
    }
    assert_eq!(failed, 0, "{failed} 个 @while 测试失败");
}

#[test]
fn test_css_nesting() {
    let spec_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../sass-spec-main/spec/css/plain/style_rule/nesting");
    let (passed, failed, failures) = run_spec_dir(&spec_dir);

    if failed > 0 {
        eprintln!("嵌套测试: {passed} 通过, {failed} 失败");
        for f in &failures {
            eprintln!("{f}");
        }
    }
    assert_eq!(failed, 0, "{failed} 个嵌套测试失败");
}

#[test]
fn test_operators() {
    let spec_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../sass-spec-main/spec/operators");
    let (passed, failed, failures) = run_spec_dir(&spec_dir);

    if failed > 0 {
        eprintln!("运算符测试: {passed} 通过, {failed} 失败");
        for f in &failures {
            eprintln!("{f}");
        }
    }
    assert_eq!(failed, 0, "{failed} 个运算符测试失败");
}

#[test]
fn test_parser_expressions() {
    let spec_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../sass-spec-main/spec/parser");
    let (passed, failed, failures) = run_spec_dir(&spec_dir);

    if failed > 0 {
        eprintln!("解析器测试: {passed} 通过, {failed} 失败");
        for f in &failures {
            eprintln!("{f}");
        }
    }
    assert_eq!(failed, 0, "{failed} 个解析器测试失败");
}

#[test]
fn test_expressions_if() {
    let spec_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../sass-spec-main/spec/expressions/if");
    let (passed, failed, failures) = run_spec_dir(&spec_dir);

    if failed > 0 {
        eprintln!("@if 表达式测试: {passed} 通过, {failed} 失败");
        for f in &failures {
            eprintln!("{f}");
        }
    }
    assert_eq!(failed, 0, "{failed} 个 @if 表达式测试失败");
}

/// 运行所有 sass-spec 测试并统计合规率。
#[test]
fn test_sass_spec_comprehensive() {
    let spec_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../sass-spec-main/spec");
    let mut total_passed = 0;
    let mut total_failed = 0;
    let mut all_failures = Vec::new();

    // 递归遍历所有目录
    if let Ok(entries) = std::fs::read_dir(&spec_root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let (p, f, failures) = run_spec_dir(&path);
                total_passed += p;
                total_failed += f;
                all_failures.extend(failures);
            }
        }
    }

    let total = total_passed + total_failed;
    let compliance = if total > 0 {
        (total_passed as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    eprintln!(
        "\n=== sass-spec 合规统计 ===\n总计: {total}\n通过: {total_passed}\n失败: {total_failed}\n合规率: {:.1}%",
        compliance
    );

    if !all_failures.is_empty() {
        eprintln!("\n失败的测试:");
        for f in all_failures.iter().take(20) {
            eprintln!("  - {f}");
        }
        if all_failures.len() > 20 {
            eprintln!("  ... 还有 {} 个失败", all_failures.len() - 20);
        }
    }
}
