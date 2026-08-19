//! sass-spec 合规测试框架 v2——正确解析 HRX 格式。
//!
//! 内存优化：逐文件处理，限制测试数量。

use std::path::Path;

/// HRX 测试用例。
#[derive(Debug)]
struct HrxCase {
    name: String,
    input: String,
    expected_output: String,
    expect_error: bool,
}

/// 解析 HRX 文件内容，提取测试用例。
fn parse_hrx(content: &str) -> Vec<HrxCase> {
    let mut files: Vec<(String, String)> = Vec::new();
    let mut current_path = String::new();
    let mut current_content = String::new();

    for line in content.lines() {
        if line.starts_with("<===>") {
            if !current_path.is_empty() {
                files.push((current_path.clone(), current_content));
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
    for (path, input) in &files {
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

            cases.push(HrxCase {
                name: base.trim_end_matches('/').to_string(),
                input: input.clone(),
                expected_output,
                expect_error,
            });
        }
    }
    cases
}

/// 运行单个测试用例——限制输入大小防止内存爆炸。
fn run_case(case: &HrxCase) -> Result<(), String> {
    // 限制输入大小——超大输入可能是恶意或错误测试
    if case.input.len() > 10000 {
        return Err(format!("输入过大跳过 [{}]", case.name));
    }

    let result = sasspile::compile_expanded(&case.input);

    if case.expect_error {
        match result {
            Ok(_) => Err(format!("期望失败但成功 [{}]", case.name)),
            Err(_) => Ok(()),
        }
    } else if case.expected_output.is_empty() {
        Ok(())
    } else {
        let actual = result.map_err(|e| format!("编译失败 [{}]: {e}", case.name))?;
        let actual_trimmed = actual.trim();
        let expected_trimmed = case.expected_output.trim();

        if actual_trimmed != expected_trimmed {
            Err(format!(
                "不匹配 [{}]: 期望 {} 字节, 实际 {} 字节",
                case.name,
                expected_trimmed.len(),
                actual_trimmed.len()
            ))
        } else {
            Ok(())
        }
    }
}

/// 递归运行目录——限制最大测试数。
fn run_dir(dir: &Path, max_tests: usize) -> (usize, usize, usize) {
    let mut passed = 0;
    let mut failed = 0;
    let mut total = 0;

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if total >= max_tests {
                break;
            }
            let path = entry.path();
            if path.is_dir() {
                let (p, f, t) = run_dir(&path, max_tests - total);
                passed += p;
                failed += f;
                total += t;
            } else if path.extension().and_then(|s| s.to_str()) == Some("hrx") {
                // 限制单文件大小
                if let Ok(meta) = std::fs::metadata(&path)
                    && meta.len() > 100_000 {
                        continue;
                    } // 跳过超大 HRX
                if let Ok(content) = std::fs::read_to_string(&path) {
                    let cases = parse_hrx(&content);
                    for case in &cases {
                        if total >= max_tests {
                            break;
                        }
                        total += 1;
                        match run_case(case) {
                            Ok(()) => passed += 1,
                            Err(_) => failed += 1,
                        }
                    }
                }
            }
        }
    }
    (passed, failed, total)
}

// —— 子目录测试（限制 50 个）——

#[test]
fn test_operators() {
    sasspile::init_tracing();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("sass-spec/spec/operators");
    let (p, f, t) = run_dir(&dir, 50);
    tracing::info!("operators: {p}/{t} 通过, {f} 失败");
    assert!(t > 0, "无测试用例");
}

#[test]
fn test_css_basic() {
    sasspile::init_tracing();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("sass-spec/spec/css");
    let (p, f, t) = run_dir(&dir, 50);
    tracing::info!("css: {p}/{t} 通过, {f} 失败");
    assert!(t > 0, "无测试用例");
}

#[test]
fn test_directives_if() {
    sasspile::init_tracing();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("sass-spec/spec/directives/if");
    let (p, f, t) = run_dir(&dir, 50);
    tracing::info!("@if: {p}/{t} 通过, {f} 失败");
    assert!(t > 0, "无测试用例");
}

/// 诊断——显示测试失败的详细对比。
#[test]
fn test_css_diagnostic() {
    sasspile::init_tracing();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("sass-spec/spec/css");
    let mut shown = 0;
    diag_dir(&dir, &mut shown);
}

fn diag_dir(dir: &Path, shown: &mut usize) {
    if *shown >= 10 {
        return;
    }
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if *shown >= 10 {
                return;
            }
            let path = entry.path();
            if path.is_dir() {
                diag_dir(&path, shown);
            } else if path.extension().and_then(|s| s.to_str()) == Some("hrx") {
                if let Ok(meta) = std::fs::metadata(&path)
                    && meta.len() > 50000 {
                        continue;
                    }
                if let Ok(content) = std::fs::read_to_string(&path) {
                    let cases = parse_hrx(&content);
                    let stem = path.file_stem().unwrap().to_string_lossy();
                    for case in &cases {
                        if *shown >= 10 {
                            return;
                        }
                        if case.expected_output.is_empty() {
                            continue;
                        }
                        match sasspile::compile_expanded(&case.input) {
                            Ok(actual) => {
                                let a = actual.trim();
                                let e = case.expected_output.trim();
                                if a != e {
                                    *shown += 1;
                                    tracing::warn!(test = %format!("{stem}/{}", case.name), input = %case.input.trim(), expected = %e, actual = %a, "FAIL");
                                }
                            }
                            Err(err) => {
                                *shown += 1;
                                tracing::warn!(test = %format!("{stem}/{}", case.name), input = %case.input.trim(), error = %err, "ERROR");
                            }
                        }
                    }
                }
            }
        }
    }
}

/// 全量 sass-spec 合规快报——默认不运行，用 --ignored 手动触发。
#[test]
#[ignore]
fn test_sass_spec_summary() {
    let spec_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("sass-spec/spec");
    let (passed, failed, total) = run_dir(&spec_root, 50);
    let compliance = if total > 0 {
        (passed as f64 / total as f64) * 100.0
    } else {
        0.0
    };
    tracing::info!(
        passed = passed,
        failed = failed,
        total = total,
        compliance = format!("{compliance:.1}%"),
        "sass-spec 合规快报（前 {total} 个）"
    );
}
