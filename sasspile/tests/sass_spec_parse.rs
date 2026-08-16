//! sass-spec 解析兼容性测试运行器。
//!
//! 遍历 sass-spec-main/spec 下的所有 .hrx 文件，
//! 提取 input.scss，仅验证 tokenize + parse 是否成功（无错误）。
//! 不验证 CSS 输出，聚焦 Parser 兼容性。

use std::path::PathBuf;
use sasspile::{tokenize, parse};

/// HRX 文件解析结果
#[derive(Debug)]
struct HrxFile {
    path: String,
    test_cases: Vec<HrxCase>,
}

/// HRX 中的单个 test case
#[derive(Debug)]
struct HrxCase {
    input_scss: String,
    expected_output: Option<String>,
    expected_error: Option<String>,
    options: Option<String>,
}

/// 解析 HRX 格式：仅提取所有 input.scss 文件内容
/// HRX 格式：以 "<===> path" 开头，紧接文件内容，直到下一个 "<===>"
fn parse_hrx(content: &str, path: &str) -> HrxFile {
    let mut cases = Vec::new();
    // Split by file markers "<===> " (must be at start of line)
    let parts: Vec<&str> = content.split("\n<===> ").collect();

    for (i, part) in parts.iter().enumerate() {
        let part = if i == 0 {
            // First part may start with "<===> "
            part.strip_prefix("<===> ").unwrap_or(part)
        } else {
            part
        };
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        // First line is the file path
        let (file_path, file_content) = match part.split_once('\n') {
            Some((p, c)) => (p.trim(), c.trim()),
            None => (part.trim(), ""),
        };
        // Only collect SCSS inputs
        if file_path.contains("input.scss") && !file_content.is_empty() {
            cases.push(HrxCase {
                input_scss: file_content.to_string(),
                expected_output: None,
                expected_error: None,
                options: None,
            });
        }
    }

    HrxFile {
        path: path.to_string(),
        test_cases: cases,
    }
}

/// 递归收集目录下所有 .hrx 文件
fn collect_hrx(dir: &str) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(collect_hrx(path.to_str().unwrap_or("")));
            } else if path.extension().and_then(|e| e.to_str()) == Some("hrx") {
                files.push(path);
            }
        }
    }
    files
}

/// 验证单个 input.scss 能否成功 tokenize + parse
fn validate_input(source: &str) -> Result<(), String> {
    let (_tokens, lex_diags) = tokenize(source);
    let lex_e = lex_diags.errors().len();
    if lex_e > 0 {
        let detail: Vec<String> = lex_diags.errors().iter().take(3).map(|d| d.message.clone()).collect();
        return Err(format!("lexer: {lex_e} errors — {}", detail.join("; ")));
    }

    let (_stylesheet, parse_diags) = parse(source);
    let p_e = parse_diags.errors().len();
    if p_e > 0 {
        let detail: Vec<String> = parse_diags.errors().iter().take(3).map(|d| d.message.clone()).collect();
        return Err(format!("parser: {p_e} errors — {}", detail.join("; ")));
    }

    Ok(())
}

#[test]
fn debug_while_syntax() {
    // Class with .#{...} pattern
    let cases = vec![
        (".#{$namespace}-foo { color: red; }", "dot-interp-class"),
        (".#{$x} { color: red; }", "dot-interp"),
        ("#{$x}-foo { color: red; }", "interp-class"),
        ("@mixin foo {\n  .#{$x}-bar { color: red; }\n}", "mixin with dot-interp"),
        ("a { .#{$x}-bar { color: red; } }", "nested dot-interp"),
        (".#{$namespace}-virtual-scrollbar {\n  opacity: 0;\n}", "exact table-v2"),
    ];
    for (src, desc) in &cases {
        let (sheet, diag) = parse(src);
        let errs: Vec<String> = diag.errors().iter().map(|d| d.message.clone()).collect();
        println!("[{}]: {} (nodes={})", desc, if errs.is_empty() { "OK".to_string() } else { format!("FAIL {:?}", errs) }, sheet.nodes.len());
    }
}

#[test]
fn sass_spec_parse_compat() {
    let spec_dirs = [
        "/Users/pauljohn/rust/sass-spec-main/spec",
    ];

    let mut all_files = Vec::new();
    for dir in &spec_dirs {
        all_files.extend(collect_hrx(dir));
    }

    let total_files = all_files.len();
    let mut parse_success = 0;
    let mut parse_fail = 0;
    let mut failed_paths: Vec<(String, String)> = Vec::new();

    for file_path in &all_files {
        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let hrx = parse_hrx(&content, &file_path.to_string_lossy());

        let file_ok = hrx.test_cases.iter().all(|case| {
            validate_input(&case.input_scss).is_ok()
        });

        if file_ok {
            parse_success += 1;
        } else {
            parse_fail += 1;
            // Collect first error for reporting
            let err_msg = hrx.test_cases.iter().find_map(|case| {
                validate_input(&case.input_scss).err()
            }).unwrap_or_default();
            failed_paths.push((
                file_path.to_string_lossy().to_string(),
                err_msg,
            ));
        }
    }

    let pass_rate = if total_files > 0 {
        (parse_success as f64 / total_files as f64) * 100.0
    } else {
        0.0
    };

    // Collect distinct error messages
    let mut error_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for (_, err) in &failed_paths {
        let key = err.split(" — ").next().unwrap_or(err).to_string();
        *error_counts.entry(key).or_insert(0) += 1;
    }
    let mut sorted_errors: Vec<_> = error_counts.iter().collect();
    sorted_errors.sort_by(|a, b| b.1.cmp(a.1));

    panic!(
        "sass-spec parse: {}/{} passed ({:.1}%)\n\nTop error patterns:\n{}\n\nSample failures:\n{}",
        parse_success,
        total_files,
        pass_rate,
        sorted_errors.iter().take(15).map(|(k, v)| format!("  [{:3}x] {}", v, k)).collect::<Vec<_>>().join("\n"),
        failed_paths.iter().take(15).map(|(p, e)| format!("  {}\n    -> {}", p, e)).collect::<Vec<_>>().join("\n"),
    );

    // 最终断言：解析成功率应 ≥ 90%
    assert!(
        pass_rate >= 90.0,
        "Parse pass rate too low: {:.1}% ({}/{})\nSample failures: {:?}",
        pass_rate, parse_success, total_files,
        failed_paths.iter().take(10).collect::<Vec<_>>()
    );
}
