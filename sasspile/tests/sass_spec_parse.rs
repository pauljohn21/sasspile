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
    #[allow(dead_code)]
    path: String,
    test_cases: Vec<HrxCase>,
}

/// HRX 中的单个 test case
#[derive(Debug)]
struct HrxCase {
    input_scss: String,
    #[allow(dead_code)]
    expected_output: Option<String>,
    expected_error: Option<String>,
    #[allow(dead_code)]
    options: Option<String>,
}

/// 解析 HRX 格式：提取所有 case（用 case 名分组，含 input.scss + 可选 error/output）
fn parse_hrx(content: &str, path: &str) -> HrxFile {
    let mut cases = Vec::new();
    // Split by file markers "<===> " (must be at start of line)
    let parts: Vec<&str> = content.split("\n<===> ").collect();

    // 先解析每个 file section
    let mut sections: Vec<(&str, &str)> = Vec::new();
    for (i, part) in parts.iter().enumerate() {
        let part = if i == 0 {
            part.strip_prefix("<===> ").unwrap_or(part)
        } else {
            part
        };
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let (file_path, file_content) = match part.split_once('\n') {
            Some((p, c)) => (p.trim(), c.trim()),
            None => (part.trim(), ""),
        };
        sections.push((file_path, file_content));
    }

    // 收集每个 case 的状态：input / error / output
    let mut case_map: std::collections::HashMap<String, (Option<String>, bool, bool)> = std::collections::HashMap::new();
    for (fpath, fcontent) in &sections {
        // case path 形如 "case_name/input.scss" 或 "case_name/error" 或 "case_name/output.css"
        let parts_split: Vec<&str> = fpath.split('/').collect();
        if parts_split.len() >= 2 {
            let case_name = parts_split[0..parts_split.len()-1].join("/");
            let file_type = parts_split[parts_split.len()-1];
            let entry = case_map.entry(case_name).or_insert((None, false, false));
            match file_type {
                "input.scss" => {
                    // Note: we only collect input.scss (SCSS syntax), not .sass (indented)
                    // 仅当内容非空时才作为输入
                    if !fcontent.is_empty() {
                        entry.0 = Some(fcontent.to_string());
                    }
                }
                "error" => { entry.1 = true; }
                "output.css" => { entry.2 = true; }
                _ => {}
            }
        }
    }

    for (_case_name, (input, has_error, _has_output)) in case_map {
        if let Some(input_scss) = input {
            cases.push(HrxCase {
                input_scss,
                expected_output: None,
                expected_error: if has_error { Some(String::new()) } else { None },
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
/// `expect_error` = true: parser 应该报错（error case）
fn validate_input(source: &str, expect_error: bool) -> Result<(), String> {
    let (_tokens, lex_diags) = tokenize(source);
    let lex_e = lex_diags.errors().len();
    let (_stylesheet, parse_diags) = parse(source);
    let p_e = parse_diags.errors().len();
    let total_errors = lex_e + p_e;

    if expect_error {
        // Error case: parser 应该报错
        if total_errors > 0 {
            Ok(())
        } else {
            Err("expected parser error but parsing succeeded".to_string())
        }
    } else {
        // Valid case: 不报错才 PASS
        if lex_e > 0 {
            let detail: Vec<String> = lex_diags.errors().iter().take(3).map(|d| d.message.clone()).collect();
            return Err(format!("lexer: {lex_e} errors — {}", detail.join("; ")));
        }
        if p_e > 0 {
            let detail: Vec<String> = parse_diags.errors().iter().take(3).map(|d| d.message.clone()).collect();
            return Err(format!("parser: {p_e} errors — {}", detail.join("; ")));
        }
        Ok(())
    }
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
    let mut _parse_fail = 0;
    let mut failed_paths: Vec<(String, String)> = Vec::new();
    let mut error_case_count = 0;  // error HRX cases parser 宽容通过了（不强求）

    for file_path in &all_files {
        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let hrx = parse_hrx(&content, &file_path.to_string_lossy());

        // 只关注 valid case：真正的语法兼容性
        // error case: parser 宽容通过 ≠ 失败（不追求精确错误消息）
        let valid_cases: Vec<_> = hrx.test_cases.iter()
            .filter(|c| c.expected_error.is_none())
            .collect();
        let error_cases: Vec<_> = hrx.test_cases.iter()
            .filter(|c| c.expected_error.is_some())
            .collect();
        error_case_count += error_cases.len();

        let file_ok = valid_cases.iter().all(|case| {
            validate_input(&case.input_scss, false).is_ok()
        });

        if file_ok {
            parse_success += 1;
        } else {
            _parse_fail += 1;
            let err_msg = valid_cases.iter().find_map(|case| {
                validate_input(&case.input_scss, false).err()
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

    // Group by top-level directory
    let mut dir_counts: std::collections::HashMap<String, (usize, usize)> = std::collections::HashMap::new();
    let spec_root = std::path::Path::new("/Users/pauljohn/rust/sass-spec-main/spec/");
    for file_path in &all_files {
        let top_dir = file_path.strip_prefix(spec_root).unwrap_or(file_path).components().next().map(|c| c.as_os_str().to_string_lossy().to_string()).unwrap_or_default();
        let entry = dir_counts.entry(top_dir).or_insert((0, 0));
        entry.0 += 1;
    }
    for (path, _) in &failed_paths {
        let p = std::path::Path::new(path);
        let top_dir = p.strip_prefix(spec_root).unwrap_or(p).components().next().map(|c| c.as_os_str().to_string_lossy().to_string()).unwrap_or_default();
        if let Some(e) = dir_counts.get_mut(&top_dir) {
            e.1 += 1;
        }
    }

    panic!(
        "sass-spec parse: {}/{} passed ({:.1}%) | {} error cases (宽容通过)\n\nBy directory (total/fail):\n{}\n\nTop error patterns:\n{}\n\nSample failures:\n{}",
        parse_success,
        total_files,
        pass_rate,
        error_case_count,
        {
            let mut v: Vec<_> = dir_counts.iter().collect();
            v.sort_by(|a, b| (b.1).1.cmp(&(a.1).1));
            v.iter().map(|(d, (t, f))| format!("  {:20} {}/{}", d, f, t)).collect::<Vec<_>>().join("\n")
        },
        sorted_errors.iter().take(15).map(|(k, v)| format!("  [{:3}x] {}", v, k)).collect::<Vec<_>>().join("\n"),
        failed_paths.iter().take(20).map(|(p, e)| format!("  {}\n    -> {}", p, e)).collect::<Vec<_>>().join("\n"),
    );
}
