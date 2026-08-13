//! HRX 子进程运行器——每个测试用例在独立进程中编译，完全内存隔离。
//!
//! 用法：`cargo test --test hrx_runner -- --nocapture`
//! 该测试遍历 core_functions/selector 下所有 HRX 文件，用 sasspile CLI 子进程编译。

use std::path::{Path, PathBuf};
use std::process::Command;

/// HRX 测试用例。
struct HrxCase {
    files: Vec<(String, String)>,
    input_path: String,
    expected_output: String,
    expect_error: bool,
}

/// 解析 HRX 内容，返回所有 test cases。
fn parse_hrx(content: &str) -> Vec<HrxCase> {
    let mut files: Vec<(String, String)> = Vec::new();
    let mut current_path = String::new();
    let mut current_content = String::new();

    for line in content.lines() {
        if line.starts_with("<===>") {
            if !current_path.is_empty() {
                files.push((current_path.clone(), current_content.clone()));
            }
            current_path = line.trim_start_matches("<===>").trim().to_string();
            current_content = String::new();
        } else {
            current_content.push_str(line);
            current_content.push('\n');
        }
    }
    if !current_path.is_empty() {
        files.push((current_path, current_content));
    }

    let mut cases = Vec::new();
    for (path, _input) in &files {
        if path.ends_with("input.scss") {
            let base = path.strip_suffix("input.scss").unwrap_or(path).to_string();
            let output_path = format!("{base}output.css");
            let error_path = format!("{base}error");

            let expected_output = files
                .iter()
                .find(|(p, _)| p == &output_path)
                .map(|(_, c)| c.clone())
                .unwrap_or_default();
            let expect_error = files.iter().any(|(p, _)| p == &error_path);

            let case_files: Vec<(String, String)> = files
                .iter()
                .filter(|(p, _)| p.ends_with(".scss") || p.ends_with(".css"))
                .map(|(p, c)| (p.clone(), c.clone()))
                .collect();

            cases.push(HrxCase {
                files: case_files,
                input_path: path.clone(),
                expected_output,
                expect_error,
            });
        }
    }
    cases
}

/// 用 sasspile CLI 子进程运行单个测试用例。
fn run_case_subprocess(case: &HrxCase, sasspile_bin: &Path) -> bool {
    if case.expected_output.is_empty() && !case.expect_error {
        return true;
    }

    let total_size: usize = case.files.iter().map(|(_, c)| c.len()).sum();
    if total_size > 20_000 {
        return false;
    }

    let tmp_dir = std::env::temp_dir().join(format!("sass-spec-hrx-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp_dir);
    std::fs::create_dir_all(&tmp_dir).ok();

    for (path, content) in &case.files {
        let file_path = tmp_dir.join(path);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        std::fs::write(&file_path, content).ok();
    }

    let input_file = tmp_dir.join(&case.input_path);
    let Ok(input_content) = std::fs::read_to_string(&input_file) else {
        return false;
    };
    let child = Command::new(sasspile_bin)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn();

    let mut child = match child {
        Ok(c) => c,
        Err(_) => return false,
    };
    // 通过 stdin 传入 SCSS 源码
    if let Some(ref mut stdin) = child.stdin {
        use std::io::Write;
        let _ = stdin.write_all(input_content.as_bytes());
    }
    let output = match child.wait_with_output() {
        Ok(o) => o,
        Err(_) => return false,
    };

    let _ = std::fs::remove_dir_all(&tmp_dir);

    if case.expect_error {
        !output.status.success()
    } else if output.status.success() {
        let actual = String::from_utf8_lossy(&output.stdout);
        actual.trim() == case.expected_output.trim()
    } else {
        false
    }
}

/// 处理单个 HRX 文件——子进程隔离。
fn process_hrx_file(file_path: &Path, sasspile_bin: &Path) -> (usize, usize, usize) {
    let Ok(content) = std::fs::read_to_string(file_path) else {
        return (0, 0, 0);
    };
    let cases = parse_hrx(&content);
    drop(content);

    let (mut pass, mut fail, mut skip) = (0, 0, 0);
    for case in &cases {
        if case.expected_output.is_empty() && !case.expect_error {
            skip += 1;
            continue;
        }
        if run_case_subprocess(case, sasspile_bin) {
            pass += 1;
        } else {
            fail += 1;
        }
    }
    (pass, fail, skip)
}

/// 用子进程模式运行指定目录下的所有 HRX 文件。
fn run_dir_subprocess(spec_root: &Path, subdir: &str, sasspile_bin: &Path) {
    let dir = spec_root.join(subdir);
    if !dir.exists() {
        eprintln!("目录不存在: {subdir}");
        return;
    }

    let (mut total_pass, mut total_fail, mut total_skip) = (0, 0, 0);
    let mut file_count = 0;

    let mut hrx_files = Vec::new();
    collect_hrx_files(&dir, &mut hrx_files);

    for file in &hrx_files {
        file_count += 1;
        let (pass, fail, skip) = process_hrx_file(file, sasspile_bin);
        total_pass += pass;
        total_fail += fail;
        total_skip += skip;
    }

    let evaluated = total_pass + total_fail;
    let pct = total_pass * 100 / evaluated.max(1);
    eprintln!(
        "{subdir}: {} 文件, pass={} fail={} skip={} pct={}%",
        file_count, total_pass, total_fail, total_skip, pct
    );
}

#[test]
#[ignore]
fn test_all_subprocess() {
    let Ok(sasspile_bin) = std::env::var("SASSPILE_BIN") else {
        eprintln!("需要设置 SASSPILE_BIN 环境变量指向 sasspile 二进制");
        return;
    };
    let sasspile_bin = PathBuf::from(sasspile_bin);

    let spec_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../sass-spec-main/spec");

    // 所有目录用子进程模式运行（完全内存隔离）
    let dirs = [
        "variables",
        "values",
        "css",
        "operators",
        "expressions",
        "directives",
        "core_functions",
        "parser",
        "callable",
    ];

    for dir in &dirs {
        run_dir_subprocess(&spec_root, dir, &sasspile_bin);
    }
}

fn collect_hrx_files(dir: &Path, files: &mut Vec<PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_hrx_files(&path, files);
            } else if path.extension().and_then(|s| s.to_str()) == Some("hrx") {
                files.push(path);
            }
        }
    }
}
